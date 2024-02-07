use core::time::Duration;

use macro_bits::{bit, bitfield};

use self::subtypes::{ControlFrameSubtype, DataFrameSubtype, ManagementFrameSubtype};

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
    pub const fn from_representation(value: u8) -> Self {
        let frame_type = value & bit!(0, 1);
        let frame_subtype = (value & bit!(2, 3, 4, 5)) >> 2;
        match frame_type {
            0b00 => Self::Management(ManagementFrameSubtype::from_representation(frame_subtype)),
            0b01 => Self::Control(ControlFrameSubtype::from_representation(frame_subtype)),
            0b10 => Self::Data(DataFrameSubtype::from_representation(frame_subtype)),
            _ => Self::Unknown(frame_subtype),
        }
    }
    /// Turns the frame type into it's representation.
    pub const fn to_representation(self) -> u8 {
        match self {
            FrameType::Management(subtype) => subtype.to_representation() << 2,
            FrameType::Control(subtype) => 0b01 | (subtype.to_representation() << 2),
            FrameType::Data(subtype) => 0b10 | (subtype.to_representation() << 2),
            FrameType::Unknown(subtype) => 0b11 | (subtype << 2),
        }
    }
}
impl From<u16> for FrameType {
    fn from(value: u16) -> Self {
        Self::from_representation(value as u8)
    }
}
impl From<FrameType> for u16 {
    fn from(value: FrameType) -> Self {
        value.to_representation() as u16
    }
}
bitfield! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// These are the flags included in the frame control field.
    pub struct FCFFlags: u8 {
        /// This frame is going to the distribution system.
        pub to_ds: bool => bit!(0),
        /// This frame is coming from the distribution system.
        pub from_ds: bool => bit!(1),
        /// This frame was fragmented and more are following.
        pub more_fragments: bool => bit!(2),
        /// This frame is a retransmission.
        pub retry: bool => bit!(3),
        // TODO: Docs
        pub pwr_mgt: bool => bit!(4),
        // TODO: Docs
        pub more_data: bool => bit!(5),
        /// This frames contents are encrypted.
        pub protected: bool => bit!(6),
        // TODO: Docs.
        pub order: bool => bit!(7)
    }
}
bitfield! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    /// This is the frame control field, which is at the beginning of every frame.
    pub struct FrameControlField: u16 {
        pub version: u8 => bit!(0,1),
        pub frame_type: FrameType => bit!(2,3,4,5,6,7),
        pub flags: FCFFlags => 0xff00
    }
}
bitfield! {
    #[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
    /// This is information about the sequence number and the potential fragment number.
    pub struct FragSeqInfo: u16 {
        pub fragment_number: u8 => bit!(0,1,2,3),
        pub sequence_number: u16 => bit!(4,5,6,7,8,9,10,11,12,13,14,15)
    }
}
