use core::iter::Empty;

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::{
    common::subtypes::ManagementFrameSubtype,
    elements::{
        rates::{EncodedRate, RatesReadIterator},
        IEEE80211Element, TLVReadIterator,
    },
};

use self::{action::ActionFrameBody, beacon::BeaconFrameBody};

/// This module contains structs related to action frames
pub mod action;
/// This module contains the body for the beacon frame.
pub mod beacon;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// This is the body of a management frame.
/// The rest of the frame can be found in [crate::frames::ManagementFrame].
pub enum ManagementFrameBody<
    'a,
    RateIterator = RatesReadIterator<'a>,
    ExtendedRateIterator = RatesReadIterator<'a>,
    TLVIterator = TLVReadIterator<'a>,
    ActionFramePayload = &'a [u8],
> where
    TLVIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>>,
{
    Action(ActionFrameBody<ActionFramePayload>),
    ActionNoAck(ActionFrameBody<ActionFramePayload>),
    Beacon(BeaconFrameBody<'a, RateIterator, ExtendedRateIterator, TLVIterator>),
    ATIM,
    Unknown {
        sub_type: ManagementFrameSubtype,
        body: &'a [u8],
    },
}
impl<
        'a,
        RateIterator,
        ExtendedRateIterator,
        TLVIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>> + 'a,
        ActionFramePayload,
    > ManagementFrameBody<'a, RateIterator, ExtendedRateIterator, TLVIterator, ActionFramePayload>
{
    /// This returns the subtype of the body.
    /// It's mostly for internal use, but since it might be useful it's `pub`.
    pub const fn get_subtype(&self) -> ManagementFrameSubtype {
        match self {
            Self::Action(_) => ManagementFrameSubtype::Action,
            Self::ActionNoAck(_) => ManagementFrameSubtype::ActionNoAck,
            Self::Beacon(_) => ManagementFrameSubtype::Beacon,
            Self::ATIM => ManagementFrameSubtype::ATIM,
            Self::Unknown { sub_type, .. } => *sub_type,
        }
    }
}
impl ManagementFrameBody<'_> {
    /// The total length in bytes.
    pub const fn length_in_bytes(&self) -> usize {
        match self {
            Self::Action(action) | Self::ActionNoAck(action) => action.length_in_bytes(),
            Self::Beacon(beacon) => beacon.length_in_bytes(),
            Self::ATIM => 0,
            Self::Unknown { body, .. } => body.len(),
        }
    }
}
impl<
        'a,
        RateIterator: IntoIterator<Item = EncodedRate> + Clone,
        ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
        TLVIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>> + Clone,
        ActionFramePayload: MeasureWith<()>,
    > MeasureWith<()>
    for ManagementFrameBody<'a, RateIterator, ExtendedRateIterator, TLVIterator, ActionFramePayload>
{
    fn measure_with(&self, ctx: &()) -> usize {
        match self {
            Self::Action(action) | Self::ActionNoAck(action) => action.measure_with(ctx),
            Self::Beacon(beacon) => beacon.measure_with(ctx),
            Self::ATIM => 0,
            Self::Unknown { body, .. } => body.len(),
        }
    }
}
impl<'a> TryFromCtx<'a, ManagementFrameSubtype> for ManagementFrameBody<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(
        from: &'a [u8],
        sub_type: ManagementFrameSubtype,
    ) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;
        Ok((
            match sub_type {
                ManagementFrameSubtype::Action => Self::Action(from.gread(&mut offset)?),
                ManagementFrameSubtype::ActionNoAck => Self::ActionNoAck(from.gread(&mut offset)?),
                ManagementFrameSubtype::Beacon => Self::Beacon(from.gread(&mut offset)?),
                ManagementFrameSubtype::ATIM => Self::ATIM,
                _ => {
                    return Err(scroll::Error::BadInput {
                        size: offset,
                        msg: "Management frame subtype not implemented.",
                    })
                }
            },
            offset,
        ))
    }
}
impl<
        'a,
        RateIterator: IntoIterator<Item = EncodedRate> + Clone,
        ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
        TLVIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>> + Clone,
        ActionFramePayload: TryIntoCtx<Error = scroll::Error>,
    > TryIntoCtx
    for ManagementFrameBody<'a, RateIterator, ExtendedRateIterator, TLVIterator, ActionFramePayload>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        match self {
            Self::Action(action_frame_body) | Self::ActionNoAck(action_frame_body) => {
                buf.pwrite(action_frame_body, 0)
            }
            Self::Beacon(beacon_frame_body) => buf.pwrite(beacon_frame_body, 0),
            Self::ATIM => Ok(0),
            Self::Unknown { body, .. } => buf.pwrite(body, 0),
        }
    }
}
pub trait ToManagementFrameBody<
    'a,
    RateIterator = Empty<EncodedRate>,
    ExtendedRateIterator = Empty<EncodedRate>,
    TLVIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>> = Empty<
        IEEE80211Element<'a, RateIterator, ExtendedRateIterator>,
    >,
>
{
    fn to_management_frame_body(self) -> ManagementFrameBody<'a, RateIterator, ExtendedRateIterator, TLVIterator>;
}
