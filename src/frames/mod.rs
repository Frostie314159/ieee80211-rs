use mac_parser::MACAddress;
use mgmt_frame::{body::action::RawActionBody, RawActionFrame};
use scroll::{ctx::TryFromCtx, Endian, Pread};

use crate::common::{
    strip_and_validate_fcs, FrameControlField, FrameType, ManagementFrameSubtype, SequenceControl,
};

/// Support for control frames.
pub mod control_frame;
/// This module contains structs around data frames.
pub mod data_frame;
pub mod mgmt_frame;

/// A trait implemented by all frames in this crate.
///
/// It is used for providing information about a frame.
pub trait IEEE80211Frame {
    const TYPE: FrameType;
    #[doc(hidden)]
    /// If the frame is an action frame, this will check, wether the supplied [ReadActionBody] matches itself.
    ///
    /// This has to be implemented for all frame types, due to the [match_frames] macro, and is meant for internal use.
    /// For all non-action management frames, this will always return false.
    fn read_action_body_matches(_action_body: RawActionBody<'_>) -> bool {
        false
    }
}
/// A generic IEEE 802.11 frame.
///
/// This allows extraction of certain fields, without knowing the actual type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GenericFrame<'a> {
    bytes: &'a [u8],
}
impl<'a> GenericFrame<'a> {
    /// Create a new [GenericFrame].
    ///
    /// If `with_fcs` is true, the fcs will be validated and internally stripped from the bytes
    /// slice.
    pub fn new(bytes: &'a [u8], with_fcs: bool) -> Result<Self, scroll::Error> {
        let bytes = if with_fcs {
            strip_and_validate_fcs(bytes)?
        } else {
            bytes
        };
        if bytes.len() < 10 {
            return Err(scroll::Error::BadInput {
                size: 0,
                msg: "Byte slice for generic frame was shorter than 10.",
            });
        }
        Ok(Self { bytes })
    }
    /// Get the frame control field.
    ///
    /// This can't fail, since all frames have this and we validate it's presence when creating a
    /// [GenericFrame].
    pub fn frame_control_field(&self) -> FrameControlField {
        FrameControlField::from_bits(self.bytes.pread_with(0, Endian::Little).unwrap())
    }
    /// Get the duration.
    ///
    /// This can't fail, since all frames have this and we validate it's presence when creating a
    /// [GenericFrame].
    pub fn duration(&self) -> u16 {
        self.bytes.pread_with(2, Endian::Little).unwrap()
    }
    /// Get the first address.
    ///
    /// This can't fail, since all frames have this and we validate it's presence when creating a
    /// [GenericFrame].
    pub fn address_1(&self) -> MACAddress {
        self.bytes.pread(4).unwrap()
    }
    /// Get the second address.
    ///
    /// This may return [None], if the frame type doesn't have a second address, or the byte slice
    /// ends early.
    pub fn address_2(&self) -> Option<MACAddress> {
        if self.frame_control_field().frame_type().has_address_2() {
            self.bytes.pread(10).ok()
        } else {
            None
        }
    }
    /// Get the second address.
    ///
    /// This may return [None], if the frame type doesn't have a third address, or the byte slice
    /// ends early.
    pub fn address_3(&self) -> Option<MACAddress> {
        if self.frame_control_field().frame_type().has_address_3() {
            self.bytes.pread(16).ok()
        } else {
            None
        }
    }
    /// Get the second address.
    ///
    /// This may return [None], if the frame type doesn't have a sequence control field, or the byte slice
    /// ends early.
    pub fn sequence_control(&self) -> Option<SequenceControl> {
        if self
            .frame_control_field()
            .frame_type()
            .has_sequence_control()
        {
            self.bytes.pread(22).map(SequenceControl::from_bits).ok()
        } else {
            None
        }
    }
    /// Check if the frame type matches.
    pub fn matches<Frame: IEEE80211Frame>(self) -> bool {
        let fcf = self.frame_control_field();
        match (fcf.frame_type(), Frame::TYPE) {
            (FrameType::Control(_), FrameType::Control(_))
            | (FrameType::Data(_), FrameType::Data(_)) => true,
            _ if fcf.frame_type() == Frame::TYPE => match Frame::TYPE {
                FrameType::Management(ManagementFrameSubtype::Action)
                | FrameType::Management(ManagementFrameSubtype::ActionNoACK) => {
                    let Ok(raw_action_frame) = self.bytes.pread_with::<RawActionFrame>(0, false)
                    else {
                        return false;
                    };
                    Frame::read_action_body_matches(raw_action_frame.body)
                }
                _ => true,
            },
            _ => false,
        }
    }
    /// Parse this generic frame to a typed one.
    pub fn parse_to_typed<Frame: IEEE80211Frame + TryFromCtx<'a, bool, Error = scroll::Error>>(
        &self,
    ) -> Option<Result<Frame, scroll::Error>> {
        if self.matches::<Frame>() {
            Some(self.bytes.pread_with(0, false))
        } else {
            None
        }
    }
}
#[macro_export]
/// This macro allows matching a strongly typed frame from a byte slice.
///
/// # Notes
/// If you match for action frames in this macro and include a [RawActionFrame](crate::mgmt_frame::RawActionFrame), it will always be matched for an action frame, as long as there are no strongly typed action frames before it.
macro_rules! match_frames {
    (
        $bytes:expr,
        $(with_fcs: $ctx:expr,)?
        $(
            $binding:pat = $frame_type:ty => $block:block
        )+
    ) => {
        {
            use ieee80211::{GenericFrame, scroll};
            const WITH_FCS: bool = {
                let mut with_fcs = false;

                $(
                    let _ = $ctx;
                    with_fcs = true;
                )?

                with_fcs
            };
            let generic_frame = GenericFrame::new($bytes, WITH_FCS);
            assert!(generic_frame.is_ok());
            match generic_frame {
                Ok(generic_frame) => {
                    if false {
                        unreachable!()
                    }
                    $(
                        else if let Some(frame_res) = generic_frame.parse_to_typed::<$frame_type>() {
                            match frame_res {
                                Ok($binding) => Ok($block),
                                Err(err) => Err(err)
                            }
                        }
                    )*
                    else {
                        Err(scroll::Error::BadInput { size: 0, msg: "Frame type not matched." })
                    }
                }
                Err(err) => Err(err)
            }
        }
    };
}
