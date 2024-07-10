use control_frame::ControlFrame;
use mac_parser::MACAddress;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{
    common::{FrameControlField, FrameType},
    elements::ReadElements,
};

use self::{
    data_frame::{DataFrame, DataFrameReadPayload},
    mgmt_frame::ManagementFrame,
};

/// Support for control frames.
pub mod control_frame;
/// This module contains structs around data frames.
pub mod data_frame;
pub mod mgmt_frame;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// An IEEE 802.11 frame.
/// The variants of this enum corespond to the type specified in the FCF.
/// The [TryIntoCtx] implementation for this takes a [bool] as `Ctx`, which specifies if the fcs is at the end.
pub enum IEEE80211Frame<
    'a,
    ElementContainer = ReadElements<'a>,
    ActionFramePayload = &'a [u8],
    DataFramePayload = DataFrameReadPayload<'a>,
> where
    ElementContainer: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
    ActionFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
    DataFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
{
    Management(ManagementFrame<'a, ElementContainer, ActionFramePayload>),
    Control(ControlFrame<'a>),
    Data(DataFrame<'a, DataFramePayload>),
}
impl<
        'a,
        ElementContainer: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
        ActionFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
        DataFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
    > IEEE80211Frame<'a, ElementContainer, ActionFramePayload, DataFramePayload>
{
    /// This returns the frame control field.
    pub const fn get_fcf(&self) -> FrameControlField {
        match self {
            Self::Management(management_frame) => management_frame.get_fcf(),
            Self::Control(control_frame) => control_frame.get_fcf(),
            Self::Data(data_frame) => data_frame.header.get_fcf(),
        }
    }
}
impl IEEE80211Frame<'_> {
    /// Total length in bytes.
    pub const fn length_in_bytes(&self, fcs_at_end: bool) -> usize {
        2 + // Type/Subtype and Flags
        match self {
            Self::Management(management_frame) => management_frame.length_in_bytes(),
            Self::Control(control_frame) => control_frame.length_in_bytes(),
            Self::Data(data_frame) => data_frame.length_in_bytes()
        } +
        if fcs_at_end {
            4
        } else {
            0
        }
    }
    /// Query the receiver address of the frame.
    pub fn receiver_address(&self) -> Option<MACAddress> {
        match self {
            Self::Management(ManagementFrame { header, .. }) => Some(header.receiver_address),
            Self::Control(control_frame) => Some(control_frame.receiver_address()),
            Self::Data(DataFrame { header, .. }) => Some(*header.receiver_address()),
        }
    }
    /// Query the transmitter address of the frame.
    pub fn transmitter_address(&self) -> Option<MACAddress> {
        match self {
            Self::Management(ManagementFrame { header, .. }) => Some(header.transmitter_address),
            Self::Control(control_frame) => control_frame.transmitter_address(),
            Self::Data(DataFrame { header, .. }) => Some(*header.transmitter_address()),
        }
    }
}
impl<
        'a,
        ElementContainer: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
        ActionFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
        DataFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
    > MeasureWith<bool>
    for IEEE80211Frame<'a, ElementContainer, ActionFramePayload, DataFramePayload>
{
    fn measure_with(&self, fcs_at_end: &bool) -> usize {
        2 + match self {
            Self::Management(management_frame) => management_frame.measure_with(&()),
            Self::Control(control_frame) => control_frame.measure_with(&()),
            Self::Data(data_frame) => data_frame.measure_with(&()),
        } + if *fcs_at_end { 4 } else { 0 }
    }
}
impl<'a> TryFromCtx<'a, bool> for IEEE80211Frame<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], fcs_at_end: bool) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let fcf = FrameControlField::from_bits(from.gread_with(&mut offset, Endian::Little)?);

        if fcf.flags().protected() {
            return Err(scroll::Error::BadInput {
                size: offset,
                msg: "Protected frames aren't supported yet.",
            });
        }

        // This prevents subsequent parsers from reading the FCS.
        let body_slice = if fcs_at_end {
            from.pread_with::<&[u8]>(0, from.len() - offset - 4)?
        } else {
            from
        };
        let frame = match fcf.frame_type() {
            FrameType::Management(subtype) => {
                Self::Management(body_slice.gread_with(&mut offset, (subtype, fcf.flags()))?)
            }
            FrameType::Data(subtype) => {
                Self::Data(body_slice.gread_with(&mut offset, (subtype, fcf.flags()))?)
            }
            FrameType::Control(subtype) => {
                Self::Control(body_slice.gread_with(&mut offset, (subtype, fcf.flags()))?)
            }
            _ => {
                return Err(scroll::Error::BadInput {
                    size: offset,
                    msg: "Frame type not yet implemented.",
                })
            }
        };
        if fcs_at_end
            && crc32fast::hash(&from[..(from.len() - 4)])
                != from.gread_with::<u32>(&mut offset, Endian::Little)?
        {
            Err(scroll::Error::BadInput {
                size: offset,
                msg: "FCS failure.",
            })
        } else {
            Ok((frame, offset))
        }
    }
}
impl<
        'a,
        ElementContainer: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
        ActionFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
        DataFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
    > TryIntoCtx<bool>
    for IEEE80211Frame<'a, ElementContainer, ActionFramePayload, DataFramePayload>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], fcs_at_end: bool) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.get_fcf().into_bits(), &mut offset)?;

        match self {
            Self::Management(management_frame) => buf.gwrite(management_frame, &mut offset)?,
            Self::Control(control_frame) => buf.gwrite(control_frame, &mut offset)?,
            Self::Data(data_frame) => buf.gwrite(data_frame, &mut offset)?,
        };
        if fcs_at_end {
            buf.gwrite_with(crc32fast::hash(&buf[..offset]), &mut offset, Endian::Little)?;
        }

        Ok(offset)
    }
}
pub trait ToFrame<
    'a,
    ElementContainer: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
    ActionFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
    DataFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
>: 'a
{
    fn to_frame(self)
        -> IEEE80211Frame<'a, ElementContainer, ActionFramePayload, DataFramePayload>;
}
