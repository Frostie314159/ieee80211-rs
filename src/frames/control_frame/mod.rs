use mac_parser::MACAddress;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{
    common::{subtypes::ControlFrameSubtype, Empty, FCFFlags, FrameControlField, FrameType},
    IEEE80211Frame, ToFrame,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// This is the body of a control frame.
pub enum ControlFrame<'a> {
    RTS {
        fcf_flags: FCFFlags,
        duration: u16,
        receiver_address: MACAddress,
        transmitter_address: MACAddress,
    },
    CTS {
        fcf_flags: FCFFlags,
        duration: u16,
        receiver_address: MACAddress,
    },
    Ack {
        fcf_flags: FCFFlags,
        duration: u16,
        receiver_address: MACAddress,
    },
    Unknown {
        subtype: ControlFrameSubtype,
        fcf_flags: FCFFlags,
        body: &'a [u8],
    },
}
impl ControlFrame<'_> {
    pub const fn length_in_bytes(&self) -> usize {
        match self {
            ControlFrame::RTS { .. } => 14,
            ControlFrame::CTS { .. } => 8,
            ControlFrame::Ack { .. } => 14,
            ControlFrame::Unknown { body, .. } => body.len(),
        }
    }
    pub const fn get_subtype(&self) -> ControlFrameSubtype {
        match self {
            ControlFrame::RTS { .. } => ControlFrameSubtype::RTS,
            ControlFrame::CTS { .. } => ControlFrameSubtype::CTS,
            ControlFrame::Ack { .. } => ControlFrameSubtype::Ack,
            ControlFrame::Unknown { subtype, .. } => *subtype,
        }
    }
    pub const fn get_fcf_flags(&self) -> FCFFlags {
        match self {
            ControlFrame::RTS { fcf_flags, .. }
            | ControlFrame::CTS { fcf_flags, .. }
            | ControlFrame::Ack { fcf_flags, .. }
            | ControlFrame::Unknown { fcf_flags, .. } => *fcf_flags,
        }
    }
    pub const fn get_fcf(&self) -> FrameControlField {
        FrameControlField::new()
            .with_frame_type(FrameType::Control(self.get_subtype()))
            .with_flags(self.get_fcf_flags())
    }
    pub fn receiver_address(&self) -> MACAddress {
        match self {
            Self::RTS {
                receiver_address, ..
            }
            | Self::CTS {
                receiver_address, ..
            }
            | Self::Ack {
                receiver_address, ..
            } => *receiver_address,
            Self::Unknown { body, .. } => body.pread(2).unwrap_or_default(),
        }
    }
    pub const fn transmitter_address(&self) -> Option<MACAddress> {
        match self {
            Self::RTS {
                transmitter_address,
                ..
            } => Some(*transmitter_address),
            _ => None,
        }
    }
}
impl<'a> TryFromCtx<'a, (ControlFrameSubtype, FCFFlags)> for ControlFrame<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(
        from: &'a [u8],
        (subtype, fcf_flags): (ControlFrameSubtype, FCFFlags),
    ) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;
        let body = match subtype {
            ControlFrameSubtype::RTS => Self::RTS {
                fcf_flags,
                duration: from.gread_with(&mut offset, Endian::Little)?,
                receiver_address: from.gread(&mut offset)?,
                transmitter_address: from.gread(&mut offset)?,
            },
            ControlFrameSubtype::CTS => Self::CTS {
                fcf_flags,
                duration: from.gread_with(&mut offset, Endian::Little)?,
                receiver_address: from.gread(&mut offset)?,
            },
            ControlFrameSubtype::Ack => Self::Ack {
                fcf_flags,
                duration: from.gread_with(&mut offset, Endian::Little)?,
                receiver_address: from.gread(&mut offset)?,
            },
            _ => {
                offset = from.len();
                Self::Unknown {
                    subtype,
                    fcf_flags,
                    body: from,
                }
            }
        };
        Ok((body, offset))
    }
}
impl MeasureWith<()> for ControlFrame<'_> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.length_in_bytes()
    }
}
impl TryIntoCtx for ControlFrame<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;
        match self {
            ControlFrame::RTS {
                duration,
                receiver_address,
                transmitter_address,
                ..
            } => {
                buf.gwrite_with(duration, &mut offset, Endian::Little)?;
                buf.gwrite(receiver_address, &mut offset)?;
                buf.gwrite(transmitter_address, &mut offset)?;
            }
            ControlFrame::CTS {
                duration,
                receiver_address,
                ..
            } => {
                buf.gwrite_with(duration, &mut offset, Endian::Little)?;
                buf.gwrite(receiver_address, &mut offset)?;
            }
            ControlFrame::Ack {
                duration,
                receiver_address,
                ..
            } => {
                buf.gwrite_with(duration, &mut offset, Endian::Little)?;
                buf.gwrite(receiver_address, &mut offset)?;
            }
            ControlFrame::Unknown { body, .. } => {
                buf.gwrite(body, &mut offset)?;
            }
        }
        Ok(offset)
    }
}
impl<'a> ToFrame<'a, Empty, Empty, Empty> for ControlFrame<'a> {
    fn to_frame(self) -> IEEE80211Frame<'a, Empty, Empty, Empty> {
        IEEE80211Frame::Control(self)
    }
}
