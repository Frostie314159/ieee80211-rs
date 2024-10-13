//! This module provides support for action frame bodies.
//!
//! Action frames can be matched, like any other frame, through [match_frames](crate::match_frames).
//!
//! IEEE 802.11-2020 defines 32 categories of action frames, with even more sub categories.
//! This implementation allows a specific action frame, like a Block Ack Request (BAR) to be treated, like any other management frame.
//! Vendor specific action frames are supported, since it's irrelevant to the [ActionBody::matches] function, if a sub-category is checked or an OUI.
//! It is also possible, to implement the [ActionBody] trait outside this crate, and have it be matched.
//!
//! ## Implementation note
//! All action frame bodies must implement the [ActionBody] trait and, when serialized, write out the category code themselves.
//! Creating a wrapper type around this would've just created another level of indirection, which would've worsened the UX.

use macro_bits::serializable_enum;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

mod vendor;
pub use vendor::{
    append_vendor_action_header, strip_and_check_vendor_action_header, RawVendorSpecificActionBody,
    RawVendorSpecificActionFrame, VENDOR_SPECIFIC_ACTION_HEADER_LENGTH,
};

serializable_enum! {
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    /// This enum contains the category code specified in the body of an [Action Frame](ActionBody).
    pub enum CategoryCode: u8 {
        #[default]
        VendorSpecific => 127
    }
}

/// A trait implemented by all bodies of an action frame.
pub trait ActionBody {
    /// The category code of the action frame body.
    const CATEGORY_CODE: CategoryCode;
    /// Check if the supplied [RawActionBody] is of the same type, as this body.
    fn matches(action_body: RawActionBody<'_>) -> bool;
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// An unparsed action frame body.
///
/// This is mostly used for [match_frames](crate::match_frames).
pub struct RawActionBody<'a> {
    pub category_code: CategoryCode,
    pub payload: &'a [u8],
}
impl RawActionBody<'_> {
    /// Check if the action frame is vendor specific and oui match.
    pub fn is_vendor_and_matches(&self, oui: [u8; 3]) -> bool {
        self.category_code == CategoryCode::VendorSpecific
            && self
                .payload
                .pread::<[u8; 3]>(0)
                .map(|read_oui| read_oui == oui)
                .unwrap_or_default()
    }
}
impl<'a> TryFromCtx<'a> for RawActionBody<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let category_code = CategoryCode::from_bits(from.gread(&mut offset)?);
        let payload = &from[offset..];
        Ok((
            Self {
                category_code,
                payload,
            },
            offset,
        ))
    }
}
impl MeasureWith<()> for RawActionBody<'_> {
    fn measure_with(&self, ctx: &()) -> usize {
        1 + self.payload.measure_with(ctx)
    }
}
impl TryIntoCtx for RawActionBody<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.category_code.into_bits(), &mut offset)?;
        buf.gwrite(self.payload, &mut offset)?;

        Ok(offset)
    }
}
