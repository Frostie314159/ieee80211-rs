use core::marker::PhantomData;

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};
use tlv_rs::{raw_tlv::RawTLV, TLV};

use crate::common::ReadIterator;

mod dsss_parameter_set;
pub use dsss_parameter_set::DSSSParameterSetElement;
pub mod rates;
mod ssid;
pub use ssid::SSIDElement;
mod bss_load;
pub use bss_load::BSSLoadElement;
pub mod ht;
mod ibss_parameter_set;
pub use ibss_parameter_set::IBSSParameterSetElement;
pub mod rsn;
mod vendor_specific_element;
pub use vendor_specific_element::VendorSpecificElement;
mod owe_transition;
pub mod vht;
pub use owe_transition::OWETransitionModeElement;
pub mod tim;

pub mod element_chain;

/// A raw TLV.
pub type RawIEEE80211Element<'a> = RawTLV<'a, u8, u8>;
type TypedIEEE80211Element<Payload> = TLV<u8, u8, u8, Payload>;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// An element identifier.
pub enum ElementID {
    /// A normal ID.
    Id(u8),
    /// An extension ID.
    /// This implies, that the normal ID is 255.
    ExtId(u8),
    VendorSpecific {
        prefix: &'static [u8],
    },
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
            Self::VendorSpecific { .. } => 0xdd,
        }
    }
    /// Returns the extended ID.
    ///
    /// If [Self::is_ext] is false, this returns None.
    pub const fn ext_id(&self) -> Option<u8> {
        match self {
            Self::ExtId(ext_id) => Some(*ext_id),
            _ => None,
        }
    }
    pub const fn vendor_prefix(&self) -> Option<&'static [u8]> {
        match *self {
            Self::VendorSpecific { prefix } => Some(prefix),
            _ => None,
        }
    }
    pub const fn element_header_length(&self) -> usize {
        match self {
            // One byte ID and one byte length.
            ElementID::Id(_) => 2,
            // One byte ID, one byte length and one byte extended ID.
            ElementID::ExtId(_) => 3,
            // Two bytes for the regular element header and the length of the vendor specific prefix.
            ElementID::VendorSpecific { prefix } => 2 + prefix.len(),
        }
    }
}

/// A trait representing shared behaviour between elements.
pub trait Element: Sized + MeasureWith<()> + TryIntoCtx<Error = scroll::Error> {
    /// The ID of this element.
    const ELEMENT_ID: ElementID;
    /// Is the element fragmentable.
    const FRAGMENTABLE: bool = false;
    /// The type returned, by reading this element.
    type ReadType<'a>: TryFromCtx<'a, Error = scroll::Error>;
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
/// A raw extension element containing just a slice.
///
/// This is mostly for internal use, while reading.
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

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
/// A typed extension element.
///
/// This is mainly used for writing.
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

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
/// A container, which contains the elements of a frame.
///
/// It can be used to extract different elements from the body of a frame.
/// If the element type you're looking for, is already implemented, you can use the [Self::get_first_element] and [Self::get_matching_elements] functions to extract them.
/// These functions will automatically parse the elements and take the type of the element as a generic parameter.
/// If the element type isn't implemented yet, you can still extract it using the [Self::get_first_element_raw] and [Self::get_matching_elements_raw] functions, which return [RawIEEE80211Elements](RawIEEE80211Element).
pub struct ReadElements<'bytes> {
    pub bytes: &'bytes [u8],
}
impl<'bytes> ReadElements<'bytes> {
    /// Check if the provided [RawIEEE80211Element] matches with the [ElementID].
    pub fn element_id_matches(
        raw_element: &RawIEEE80211Element<'bytes>,
        element_id: ElementID,
    ) -> bool {
        match element_id {
            ElementID::Id(id) => id == raw_element.tlv_type,
            ElementID::ExtId(ext_id) if raw_element.tlv_type == 0xff => {
                let Ok(ext_element) = raw_element.slice.pread::<RawIEEE80211ExtElement>(0) else {
                    return false;
                };
                ext_id == ext_element.ext_id
            }
            ElementID::VendorSpecific { prefix } if raw_element.tlv_type == 0xdd => {
                let Ok(vendor_specific_element) =
                    raw_element.slice.pread::<VendorSpecificElement>(0)
                else {
                    return false;
                };
                vendor_specific_element.get_payload().starts_with(prefix)
            }
            _ => false,
        }
    }
    /// Parse a [RawIEEE80211Element] into the specified type.
    ///
    /// This doesn't validate that the types match, and will most likely cause the parser for the element to return an error.
    pub fn parse_raw_element<ElementType: Element>(
        raw_element: RawIEEE80211Element<'bytes>,
    ) -> Option<ElementType::ReadType<'bytes>> {
        match ElementType::ELEMENT_ID {
            ElementID::Id(_) => raw_element.slice,
            ElementID::ExtId(_) => {
                let ext_element: RawIEEE80211ExtElement = raw_element.slice.pread(0).ok()?;
                ext_element.slice
            }
            ElementID::VendorSpecific { prefix } => {
                let vendor_specific_element: VendorSpecificElement =
                    raw_element.slice.pread(0).ok()?;
                &vendor_specific_element.get_payload()[prefix.len()..]
            }
        }
        .pread(0)
        .ok()
    }
    /// Returns an iterator over [RawIEEE80211Elements](RawIEEE80211Element).
    pub fn raw_element_iterator(self) -> ReadIterator<'bytes, Endian, RawIEEE80211Element<'bytes>> {
        ReadIterator::<Endian, RawIEEE80211Element<'bytes>>::new(self.bytes)
    }

    /// Returns an iterator over [RawIEEE80211Elements](RawIEEE80211Element), which match the specified [ElementID].
    pub fn get_matching_elements_raw(
        self,
        element_id: ElementID,
    ) -> impl Iterator<Item = RawIEEE80211Element<'bytes>> + 'bytes {
        self.raw_element_iterator()
            .filter(move |raw_element| Self::element_id_matches(raw_element, element_id))
    }
    /// Returns an [Iterator] over a specific type of element, which is specified over the generic parameter.
    pub fn get_matching_elements<ElementType: Element>(
        self,
    ) -> impl Iterator<Item = ElementType::ReadType<'bytes>> + 'bytes {
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
        self,
        element_id: ElementID,
    ) -> Option<RawIEEE80211Element<'bytes>> {
        self.get_matching_elements_raw(element_id).next()
    }
    /// This returns the first element, matching the specified element type.
    pub fn get_first_element<ElementType: Element>(self) -> Option<ElementType::ReadType<'bytes>> {
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

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
/// A wrapper for any type implementing the [Element] trait.
///
/// This handles all the quirks of writing an element, like extended ID or vendor prefix.
pub(crate) struct WrappedIEEE80211Element<Inner>(pub Inner);
impl<Inner: Element> MeasureWith<()> for WrappedIEEE80211Element<Inner> {
    fn measure_with(&self, ctx: &()) -> usize {
        // Get the size of the element header and add the length of the body.
        Inner::ELEMENT_ID.element_header_length() + self.0.measure_with(ctx)
    }
}
impl<Inner: Element> TryIntoCtx<()> for WrappedIEEE80211Element<Inner> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        match Inner::ELEMENT_ID {
            ElementID::Id(id) => buf.pwrite(
                TypedIEEE80211Element {
                    tlv_type: id,
                    payload: self.0,
                    _phantom: PhantomData,
                },
                0,
            ),
            ElementID::ExtId(ext_id) => buf.pwrite(
                TypedIEEE80211Element {
                    tlv_type: 0xff,
                    payload: TypedIEEE80211ExtElement {
                        ext_id,
                        payload: self.0,
                    },
                    _phantom: PhantomData,
                },
                0,
            ),
            ElementID::VendorSpecific { prefix } => buf.pwrite(
                TypedIEEE80211Element {
                    // Vendor specific elements always have the ID 221 (or 0xdd in hex).
                    // No idea, how they got that number.
                    tlv_type: 0xdd,
                    payload: VendorSpecificElement::new_prefixed(prefix, self.0),
                    _phantom: PhantomData,
                },
                0,
            ),
        }
    }
}
