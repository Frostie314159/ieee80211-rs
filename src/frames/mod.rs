use mac_parser::MACAddress;
use mgmt_frame::body::action::RawActionBody;
use scroll::{Endian, Pread};

use crate::common::{strip_and_validate_fcs, FrameControlField, FrameType, SequenceControl};

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
}
#[macro_export]
/// This macro allows matching a strongly typed frame from a byte slice.
///
/// # Notes
/// When using control flow operators inside this macro, you'll have to rely on named blocks, due to the internal implementation.
/// If anyone knows a better way of doing this efficiently and without named blocks, please let me know.
///
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
            use ieee80211::scroll::Pread;
            use ieee80211::{common::{FrameControlField, FrameType, ManagementFrameSubtype}, IEEE80211Frame, mgmt_frame::RawActionFrame};
            use core::mem::discriminant;

            const WITH_FCS: bool = {
                let mut with_fcs = false;

                $(
                    let _ = $ctx;
                    with_fcs = true;
                )?

                with_fcs
            };

            const ACTION_FRAME_MATCHED: bool = {
                let mut action_frame_matched = false;

                $(
                    action_frame_matched |= matches!(
                        <$frame_type as IEEE80211Frame>::TYPE,
                        FrameType::Management(ManagementFrameSubtype::Action)
                    );
                )*

                action_frame_matched
            };
            let fcf = $bytes.pread(0).map(FrameControlField::from_bits);
            if let Ok(fcf) = fcf {

                'matched: {
                    let parsed_action_frame =
                        if ACTION_FRAME_MATCHED && matches!(fcf.frame_type(), FrameType::Management(ManagementFrameSubtype::Action)) {
                            match $bytes.pread_with::<RawActionFrame>(0, WITH_FCS) {
                                Ok(action_frame) => Some(action_frame),
                                Err(err) => break 'matched Err(err)
                            }
                        } else {
                            None
                        };
                    $(
                        'matched_inner: {
                            match (<$frame_type as IEEE80211Frame>::TYPE, fcf.frame_type()) {
                                (FrameType::Management(lhs), FrameType::Management(rhs)) if lhs == rhs => {}
                                (FrameType::Control(_), FrameType::Control(_)) => {}
                                (FrameType::Data(_), FrameType::Data(_)) => {}
                                _ => {
                                    break 'matched_inner;
                                }
                            }
                            break 'matched match $bytes.pread_with::<$frame_type>(0, WITH_FCS) {
                                Ok($binding) => Ok($block),
                                Err(err) => Err(err)
                            };
                        }
                    )*
                    Err(ieee80211::scroll::Error::BadInput {
                        size: 0,
                        msg: "Frame type not matched."
                    })
                }
            } else {
                Err(fcf.unwrap_err())
            }
        }
    };
}
