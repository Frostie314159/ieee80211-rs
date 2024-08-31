use crate::common::FrameType;

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
}
#[macro_export]
/// This macro allows matching a strongly typed frame from a byte slice.
/// 
/// # Note
/// When using control flow operators inside this macro, you'll have to rely on named blocks, due to the internal implementation.
/// If anyone knows a better way of doing this efficiently, please let me know.
macro_rules! match_frames {
    (
        $bytes:expr,
        $(
            $binding:pat = $frame_type:ty => $block:block
        )*
    ) => {
        {
            use ieee80211::scroll::Pread;
            use ieee80211::{common::{FrameControlField, FrameType}, IEEE80211Frame};
            use core::mem::discriminant;

            let fcf = $bytes.pread(0).map(FrameControlField::from_bits);
            if let Ok(fcf) = fcf {
                'matched: {
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
                            break 'matched match $bytes.pread::<$frame_type>(0) {
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
