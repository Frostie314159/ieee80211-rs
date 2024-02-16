use core::iter::Empty;

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::{
    common::subtypes::ManagementFrameSubtype,
    elements::{
        rates::{EncodedRate, RatesReadIterator},
        ElementReadIterator, IEEE80211Element,
    },
};

use self::{action::ActionFrameBody, beacon::BeaconFrameBody, probe_request::ProbeRequestBody};

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
    ElementIterator = ElementReadIterator<'a>,
    ActionFramePayload = &'a [u8],
> where
    RateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ElementIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>>,
{
    Action(ActionFrameBody<ActionFramePayload>),
    ActionNoAck(ActionFrameBody<ActionFramePayload>),
    ProbeRequest(ProbeRequestBody<'a, RateIterator, ExtendedRateIterator, ElementIterator>),
    Beacon(BeaconFrameBody<'a, RateIterator, ExtendedRateIterator, ElementIterator>),
    ATIM,
    Unknown {
        sub_type: ManagementFrameSubtype,
        body: &'a [u8],
    },
}
impl<'a, RateIterator, ExtendedRateIterator, ElementIterator, ActionFramePayload>
    ManagementFrameBody<'a, RateIterator, ExtendedRateIterator, ElementIterator, ActionFramePayload>
where
    RateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ElementIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>>,
{
    /// This returns the subtype of the body.
    /// It's mostly for internal use, but since it might be useful it's `pub`.
    pub const fn get_subtype(&self) -> ManagementFrameSubtype {
        match self {
            Self::Action(_) => ManagementFrameSubtype::Action,
            Self::ActionNoAck(_) => ManagementFrameSubtype::ActionNoAck,
            Self::ProbeRequest(_) => ManagementFrameSubtype::ProbeRequest,
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
            Self::ProbeRequest(probe_request) => probe_request.length_in_bytes(),
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
        ElementIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>> + Clone + 'a,
        ActionFramePayload: MeasureWith<()>,
    > MeasureWith<()>
    for ManagementFrameBody<
        'a,
        RateIterator,
        ExtendedRateIterator,
        ElementIterator,
        ActionFramePayload,
    >
{
    fn measure_with(&self, ctx: &()) -> usize {
        match self {
            Self::Action(action) | Self::ActionNoAck(action) => action.measure_with(ctx),
            Self::Beacon(beacon) => beacon.measure_with(ctx),
            Self::ProbeRequest(probe_request) => probe_request.measure_with(ctx),
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
                ManagementFrameSubtype::ProbeRequest => {
                    Self::ProbeRequest(from.gread(&mut offset)?)
                }
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
        ElementIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>> + Clone + 'a,
        ActionFramePayload: TryIntoCtx<Error = scroll::Error>,
    > TryIntoCtx
    for ManagementFrameBody<
        'a,
        RateIterator,
        ExtendedRateIterator,
        ElementIterator,
        ActionFramePayload,
    >
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        match self {
            Self::Action(action_frame_body) | Self::ActionNoAck(action_frame_body) => {
                buf.pwrite(action_frame_body, 0)
            }
            Self::ProbeRequest(probe_request) => buf.pwrite(probe_request, 0),
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
    ElementIterator = Empty<IEEE80211Element<'a, RateIterator, ExtendedRateIterator>>,
    ActionFramePayload = &'a [u8],
> where
    RateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ElementIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>>,
{
    fn to_management_frame_body(
        self,
    ) -> ManagementFrameBody<
        'a,
        RateIterator,
        ExtendedRateIterator,
        ElementIterator,
        ActionFramePayload,
    >;
}
