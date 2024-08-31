use core::{mem::discriminant, time::Duration};

use bitfield_struct::bitfield;
use macro_bits::bit;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

mod subtypes;
pub use subtypes::*;
mod read_iterator;
pub use read_iterator::*;
mod capabilities;
pub use capabilities::*;
mod reason;
pub use reason::*;
mod status_code;
pub use status_code::*;
mod type_state;
pub use type_state::*;
mod auth_algo_num;
pub use auth_algo_num::*;
mod aid;
pub use aid::*;

/// This is one **T**ime **U**nit, which equalls 1024µs.
pub const TU: Duration = Duration::from_micros(1024);

pub const IEEE_OUI: [u8; 3] = [0x00, 0x0f, 0xac];
pub const WIFI_ALLIANCE_OUI: [u8; 3] = [0x50, 0x6f, 0x9a];

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
    pub fn type_matches(&self, other: Self) -> bool {
        discriminant(self) == discriminant(&other)
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

/// These are the flags included in the frame control field.
#[bitfield(u8, defmt = cfg(feature = "defmt"))]
#[derive(PartialEq, Eq, Hash)]
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
#[bitfield(u16, defmt = cfg(feature = "defmt"))]
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
#[bitfield(u16, defmt = cfg(feature = "defmt"))]
#[derive(PartialEq, Eq, Hash)]
/// This is information about the sequence number and the potential fragment number.
pub struct SequenceControl {
    #[bits(4)]
    pub fragment_number: u8,
    #[bits(12)]
    pub sequence_number: u16,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
/// An empty type, used for filling empty generics.
pub struct Empty;
impl<'a> TryFromCtx<'a> for Empty {
    type Error = scroll::Error;
    fn try_from_ctx(_: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        Ok((Self, 0))
    }
}
impl MeasureWith<()> for Empty {
    fn measure_with(&self, _: &()) -> usize {
        0
    }
}
impl TryIntoCtx for Empty {
    type Error = scroll::Error;
    fn try_into_ctx(self, _: &mut [u8], _: ()) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

pub(crate) fn strip_and_validate_fcs(bytes: &[u8]) -> Result<&[u8], scroll::Error> {
    let (slice_without_fcs, fcs) = bytes.split_at(bytes.len() - 4);
    if fcs.pread_with::<u32>(0, Endian::Little)? == crc32fast::hash(slice_without_fcs) {
        Ok(slice_without_fcs)
    } else {
        Err(scroll::Error::BadInput {
            size: 0,
            msg: "FCS check failed.",
        })
    }
}

pub(crate) fn attach_fcs(buf: &mut [u8], offset: &mut usize) -> Result<usize, scroll::Error> {
    let fcs = crc32fast::hash(&buf[..*offset]);
    buf.gwrite_with(fcs, offset, Endian::Little)
}
