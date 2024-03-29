use core::time::Duration;

use bitfield_struct::bitfield;
use macro_bits::bit;

use crate::mgmt_frame::body::ManagementFrameSubtype;

use self::subtypes::{ControlFrameSubtype, DataFrameSubtype};

/// This modules contains the enum for the individual subtypes.
pub mod subtypes;

pub mod read_iterator;

pub mod reason;

/// This is one **T**ime **U**nit, which equalls 1024µs.
pub const TU: Duration = Duration::from_micros(1024);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// The frame type of an IEEE 802.11 frame.
pub enum FrameType {
    Management(ManagementFrameSubtype),
    Control(ControlFrameSubtype),
    Data(DataFrameSubtype),
    Unknown(u8),
}
impl FrameType {
    /// Constructs the frame type from it's representation.
    pub const fn from_bits(value: u8) -> Self {
        let frame_type = value & bit!(0, 1);
        let frame_subtype = (value & bit!(2, 3, 4, 5)) >> 2;
        match frame_type {
            0b00 => Self::Management(ManagementFrameSubtype::from_bits(frame_subtype)),
            0b01 => Self::Control(ControlFrameSubtype::from_bits(frame_subtype)),
            0b10 => Self::Data(DataFrameSubtype::from_bits(frame_subtype)),
            _ => Self::Unknown(frame_subtype),
        }
    }
    /// Turns the frame type into it's representation.
    pub const fn into_bits(self) -> u8 {
        match self {
            FrameType::Management(subtype) => subtype.into_bits() << 2,
            FrameType::Control(subtype) => 0b01 | (subtype.into_bits() << 2),
            FrameType::Data(subtype) => 0b10 | (subtype.into_bits() << 2),
            FrameType::Unknown(subtype) => 0b11 | (subtype << 2),
        }
    }
}
impl From<u16> for FrameType {
    fn from(value: u16) -> Self {
        Self::from_bits(value as u8)
    }
}
impl From<FrameType> for u16 {
    fn from(value: FrameType) -> Self {
        value.into_bits() as u16
    }
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash)]
/// These are the flags included in the frame control field.
pub struct FCFFlags {
    /// This frame is going to the distribution system.
    pub to_ds: bool,
    /// This frame is coming from the distribution system.
    pub from_ds: bool,
    /// This frame was fragmented and more are following.
    pub more_fragments: bool,
    /// This frame is a retransmission.
    pub retry: bool,
    // TODO: Docs
    pub pwr_mgmt: bool,
    // TODO: Docs
    pub more_data: bool,
    /// This frames contents are encrypted.
    pub protected: bool,
    // TODO: Docs
    pub order: bool,
}
#[bitfield(u16)]
#[derive(PartialEq, Eq, Hash)]
/// This is the frame control field, which is at the beginning of every frame.
pub struct FrameControlField {
    #[bits(2)]
    pub version: u8,
    #[bits(6)]
    pub frame_type: FrameType,
    #[bits(8)]
    pub flags: FCFFlags,
}
#[bitfield(u16)]
#[derive(PartialEq, Eq, Hash)]
/// This is information about the sequence number and the potential fragment number.
pub struct SequenceControl {
    #[bits(4)]
    pub fragment_number: u8,
    #[bits(12)]
    pub sequence_number: u16,
}
