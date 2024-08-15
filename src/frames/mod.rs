use crate::common::FrameType;

/// Support for control frames.
pub mod control_frame;
/// This module contains structs around data frames.
pub mod data_frame;
/// This module contains the data frames.
pub mod mgmt_frame;

pub trait IEEE80211Frame {
    const TYPE: FrameType;
    const MATCH_ONLY_TYPE: bool;
}
#[macro_export]
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
                            break 'matched $bytes.pread::<$frame_type>(0).map(|$binding| $block);
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
