use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};
use tlv_rs::{raw_tlv::RawTLV, TLV};

use crate::common::read_iterator::ReadIterator;

/// This module contains the elements, which are found in the body of some frames.
/// If an element only consists of one struct, like the [ssid::SSIDTLV], they are re-exported, otherwise they get their own module.
mod dsss_parameter_set;
pub use dsss_parameter_set::DSSSParameterSetElement;
pub mod rates;
mod ssid;
pub use ssid::SSIDElement;
mod bss_load;
pub mod ht;
pub use bss_load::*;
mod ibss_parameter_set;
pub use ibss_parameter_set::IBSSParameterSetElement;

pub mod rsn;
mod vendor_specific_element;
pub use vendor_specific_element::VendorSpecificElement;

pub mod element_chain;

/// A raw TLV.
pub type RawIEEE80211Element<'a> = RawTLV<'a, u8, u8>;
type TypedIEEE80211Element<Payload> = TLV<u8, u8, u8, Payload>;

/// An element identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ElementID {
    /// A normal ID.
    Id(u8),
    /// An extension ID.
    /// This implies, that the normal ID is 255.
    ExtId(u8),
}
impl ElementID {
    /// Checks if this element ID is an extended element ID.
    pub const fn is_ext(&self) -> bool {
        matches!(self, Self::ExtId(_))
    }
    /// Returns the ID used for matching.
    ///
    /// If [Self::is_ext] is true, this returns 255.
    pub const fn id(&self) -> u8 {
        match self {
            Self::Id(id) => *id,
            Self::ExtId(_) => 0xff,
        }
    }
    /// Returns the extended ID.
    ///
    /// If [Self::is_ext] is false, this returns None.
    pub const fn ext_id(&self) -> Option<u8> {
        match self {
            Self::Id(_) => None,
            Self::ExtId(ext_id) => Some(*ext_id),
        }
    }
}

/// A trait representing shared behaviour between elements.
pub trait Element: Sized + MeasureWith<()> + TryIntoCtx<Error = scroll::Error> {
    /// The ID of this element.
    const ELEMENT_ID: ElementID;
    /// The type returned, by reading this element.bi
    type ReadType<'a>: TryFromCtx<'a, Error = scroll::Error>;
}

/// A raw extension element containing just a slice.
///
/// This is mostly for internal use, while reading.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub struct RawIEEE80211ExtElement<'a> {
    pub ext_id: u8,
    pub slice: &'a [u8],
}
impl<'a> TryFromCtx<'a> for RawIEEE80211ExtElement<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let ext_id = from.gread(&mut offset)?;
        let slice = &from[offset..];

        Ok((Self { ext_id, slice }, offset))
    }
}
impl MeasureWith<()> for RawIEEE80211ExtElement<'_> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.slice.len() + 1
    }
}
impl TryIntoCtx for RawIEEE80211ExtElement<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;
        buf.gwrite(self.ext_id, &mut offset)?;
        buf.gwrite(self.slice, &mut offset)?;

        Ok(offset)
    }
}

/// A typed extension element.
///
/// This is mainly used for writing.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub struct TypedIEEE80211ExtElement<Payload> {
    pub ext_id: u8,
    pub payload: Payload,
}
impl<'a, Payload: TryFromCtx<'a, Error = scroll::Error> + 'a> TryFromCtx<'a>
    for TypedIEEE80211ExtElement<Payload>
{
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let ext_id = from.gread(&mut offset)?;
        let payload = from.gread(&mut offset)?;

        Ok((Self { ext_id, payload }, offset))
    }
}
impl<Payload: MeasureWith<()>> MeasureWith<()> for TypedIEEE80211ExtElement<Payload> {
    fn measure_with(&self, ctx: &()) -> usize {
        self.payload.measure_with(ctx) + 1
    }
}
impl<Payload: TryIntoCtx<Error = scroll::Error>> TryIntoCtx for TypedIEEE80211ExtElement<Payload> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.ext_id, &mut offset)?;
        buf.gwrite(self.payload, &mut offset)?;

        Ok(offset)
    }
}

/// A container, which contains the elements of a frame.
///
/// It can be used to extract different elements from the body of a frame.
/// If the element type you're looking for, is already implemented, you can use the [Self::get_first_element] and [Self::get_matching_elements] functions to extract them.
/// These functions will automatically parse the elements and take the type of the element as a generic parameter.
/// If the element type isn't implemented yet, you can still extract it using the [Self::get_first_element_raw] and [Self::get_matching_elements_raw] functions, which return [RawIEEE80211Elements](RawIEEE80211Element).
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub struct ReadElements<'a> {
    pub bytes: &'a [u8],
}
impl<'a> ReadElements<'a> {
    /// Check if the provided [RawIEEE80211Element] matches with the [ElementID].
    fn element_id_matches(raw_element: &RawIEEE80211Element<'a>, element_id: ElementID) -> bool {
        match element_id {
            ElementID::Id(id) => id == raw_element.tlv_type,
            ElementID::ExtId(ext_id) if raw_element.tlv_type == 0xff => {
                let Ok(ext_element) = raw_element.slice.pread::<RawIEEE80211ExtElement>(0) else {
                    return false;
                };
                ext_id == ext_element.ext_id
            }
            _ => false,
        }
    }
    /// Parse a [RawIEEE80211Element] into the specified type.
    fn parse_raw_element<ElementType: Element>(
        raw_element: RawIEEE80211Element<'a>,
    ) -> Option<ElementType::ReadType<'a>> {
        raw_element.slice.pread(0).ok()
    }
    /// Returns an iterator over [RawIEEE80211Elements](RawIEEE80211Element).
    pub fn raw_element_iterator(&self) -> ReadIterator<'a, Endian, RawIEEE80211Element> {
        ReadIterator::<Endian, RawIEEE80211Element<'a>>::new(self.bytes)
    }
    /// Returns an iterator over [RawIEEE80211Elements](RawIEEE80211Element), which match the specified [ElementID].
    pub fn get_matching_elements_raw(
        &'a self,
        element_id: ElementID,
    ) -> impl Iterator<Item = RawIEEE80211Element<'a>> + 'a {
        self.raw_element_iterator()
            .filter(move |raw_element| Self::element_id_matches(raw_element, element_id))
    }
    /// Returns an [Iterator] over a specific type of element, which is specified over the generic parameter.
    pub fn get_matching_elements<ElementType: Element + 'a>(
        &'a self,
    ) -> impl Iterator<Item = ElementType::ReadType<'a>> + 'a {
        self.raw_element_iterator().filter_map(|raw_element| {
            if Self::element_id_matches(&raw_element, ElementType::ELEMENT_ID) {
                Self::parse_raw_element::<ElementType>(raw_element)
            } else {
                None
            }
        })
    }
    /// Returns the first [RawIEEE80211Element], which matches the specified [ElementID].
    pub fn get_first_element_raw(
        &'a self,
        element_id: ElementID,
    ) -> Option<RawIEEE80211Element<'a>> {
        self.get_matching_elements_raw(element_id).next()
    }
    /// This returns the first element, matching the specified element type.
    pub fn get_first_element<ElementType: Element + 'a>(
        &'a self,
    ) -> Option<ElementType::ReadType<'a>> {
        self.get_matching_elements::<ElementType>().next()
    }
}
impl<'a> TryFromCtx<'a> for ReadElements<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        Ok((ReadElements { bytes: from }, from.len()))
    }
}
impl TryIntoCtx for ReadElements<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        buf.pwrite(self.bytes, 0)
    }
}
impl MeasureWith<()> for ReadElements<'_> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.bytes.len()
    }
}
