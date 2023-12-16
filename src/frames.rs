use alloc::{borrow::Cow, vec};
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::{
    frame_control_field::{FrameControlField, FrameType},
    mgmt_frame::{action::ActionFrame, beacon::BeaconFrame},
};

pub enum IEEE80211FrameBody<'a> {
    Action(ActionFrame<'a>),
    Beacon(BeaconFrame<'a>),
}

pub struct IEEE80211Frame<'a> {
    pub fcf: FrameControlField,
    pub body: Cow<'a, [u8]>,
}
impl<'a> IEEE80211Frame<'a> {
    pub fn get_body(&'a self) -> Result<IEEE80211FrameBody<'a>, scroll::Error> {
        Ok(match self.fcf.frame_type {
            FrameType::Action => IEEE80211FrameBody::Action(self.body.pread(0)?),
            FrameType::Beacon => IEEE80211FrameBody::Beacon(self.body.pread(0)?),
            _ => {
                return Err(scroll::Error::BadInput {
                    size: 0x00,
                    msg: "Frame type not yet implemented.",
                })
            }
        })
    }
    pub fn from_body(body: IEEE80211FrameBody<'_>) -> Result<Self, scroll::Error> {
        let fcf;
        let mut buf; 
        match body {
            IEEE80211FrameBody::Action(action_frame) => {
                fcf = FrameControlField {
                    frame_type: FrameType::Action,
                    ..Default::default()
                };
                buf = vec![0x00; action_frame.measure_with(&())];
                buf.pwrite(action_frame, 0)?;
            }
            IEEE80211FrameBody::Beacon(beacon_frame) => {
                fcf = FrameControlField {
                    frame_type: FrameType::Beacon,
                    ..Default::default()
                };
                buf = vec![0x00; beacon_frame.measure_with(&())];
                buf.pwrite(beacon_frame, 0)?;
            }
        }
        Ok(
            Self { fcf, body: buf.into() }
        )
    }
}
impl<'a> MeasureWith<()> for IEEE80211Frame<'a> {
    fn measure_with(&self, _ctx: &()) -> usize {
        6 + self.body.len()
    }
}
impl<'a> TryFromCtx<'a> for IEEE80211Frame<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let fcf = FrameControlField::from_representation(from.gread(&mut offset)?);

        let body_len = from.len() - offset - 4;
        let body = Cow::Borrowed(from.gread_with(&mut offset, body_len)?);
        Ok((Self { fcf, body }, offset))
    }
}
impl<'a> TryIntoCtx for IEEE80211Frame<'a> {
    type Error = scroll::Error;
    fn try_into_ctx(self, data: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;
        data.gwrite(self.fcf.to_representation(), &mut offset)?;
        data.gwrite(self.body.as_ref(), &mut offset)?;
        data.gwrite_with(
            crc32fast::hash(&data[..offset]),
            &mut offset,
            scroll::Endian::Little,
        )?;

        Ok(offset)
    }
}
