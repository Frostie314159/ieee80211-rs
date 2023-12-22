#![no_std]
#![forbid(unsafe_code)]

use core::time::Duration;

pub mod common;
pub mod frames;
pub mod tlvs;
pub mod type_state;

pub const TU: Duration = Duration::from_micros(1024);
