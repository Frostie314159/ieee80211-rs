#![no_std]
#![feature(iter_next_chunk, iterator_try_collect)]
#![forbid(unsafe_code)]

extern crate alloc;

use core::time::Duration;

use macro_bits::{bit, bitfield};
pub mod frame_control_field;
pub mod frames;
pub mod mgmt_frame;
pub mod tlvs;
pub(crate) mod util;

pub const TU: Duration = Duration::from_micros(1024);

bitfield! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct FragSeqInfo: u16 {
        pub fragment_number: u8 => bit!(0,1,2,3),
        pub sequence_number: u16 => bit!(4,5,6,7,8,9,10,11,12,13,14,15)
    }
}
