use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::{
    common::subtypes::ManagementFrameSubtype,
    tlvs::{TLVReadIterator, IEEE80211TLV},
};

use self::{action::ActionFrameBody, beacon::BeaconFrameBody};

pub mod action;
pub mod beacon;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManagementFrameBody<'a, TLVIterator = TLVReadIterator<'a>> {
    Action(ActionFrameBody<'a>),
    ActionNoAck(ActionFrameBody<'a>),
    Beacon(BeaconFrameBody<'a, TLVIterator>),
    ATIM,
}
impl<TLVIterator> ManagementFrameBody<'_, TLVIterator> {
    pub const fn get_sub_type(&self) -> ManagementFrameSubtype {
        match self {
            Self::Action(_) => ManagementFrameSubtype::Action,
            Self::ActionNoAck(_) => ManagementFrameSubtype::ActionNoAck,
            Self::Beacon(_) => ManagementFrameSubtype::Beacon,
            Self::ATIM => ManagementFrameSubtype::ATIM,
        }
    }
}
impl ManagementFrameBody<'_> {
    pub const fn length_in_bytes(&self) -> usize {
        match self {
            Self::Action(action) | Self::ActionNoAck(action) => action.length_in_bytes(),
            Self::Beacon(beacon) => beacon.length_in_bytes(),
            Self::ATIM => 0,
        }
    }
}
impl<'a, TLVIterator: Iterator<Item = IEEE80211TLV<'a>> + Clone> MeasureWith<()>
    for ManagementFrameBody<'a, TLVIterator>
{
    fn measure_with(&self, ctx: &()) -> usize {
        match self {
            Self::Action(action) | Self::ActionNoAck(action) => action.measure_with(ctx),
            Self::Beacon(beacon) => beacon.measure_with(ctx),
            Self::ATIM => 0,
        }
    }
}
impl<'a> TryFromCtx<'a, ManagementFrameSubtype> for ManagementFrameBody<'a, TLVReadIterator<'a>> {
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
impl<'a, TLVIterator: Iterator<Item = IEEE80211TLV<'a>>> TryIntoCtx
    for ManagementFrameBody<'a, TLVIterator>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        match self {
            Self::Action(action_frame_body) | Self::ActionNoAck(action_frame_body) => {
                buf.pwrite(action_frame_body, 0)
            }
            Self::Beacon(beacon_frame_body) => buf.pwrite(beacon_frame_body, 0),
            Self::ATIM => Ok(0),
        }
    }
}
