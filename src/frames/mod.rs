use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{
    common::{FrameControlField, FrameType},
    tlvs::{TLVReadIterator, IEEE80211TLV},
};

mod data_frame;
mod mgmt_frame;

pub use data_frame::*;
pub use mgmt_frame::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// An IEEE 802.11 frame.
/// The variants of this enum corespond to the type specified in the FCF.
/// The [TryIntoCtx] implementation for this takes a [bool] as `Ctx`, which specifies if the fcs is at the end.
pub enum IEEE80211Frame<
    'a,
    TLVIterator = TLVReadIterator<'a>,
    DataFramePayload = DataFrameReadPayload<'a>,
> {
    Management(ManagementFrame<'a, TLVIterator>),
    Data(DataFrame<DataFramePayload>),
}
impl<TLVIterator, DataFramePayload> IEEE80211Frame<'_, TLVIterator, DataFramePayload> {
    /// This returns the frame control field.
    pub const fn get_fcf(&self) -> FrameControlField {
        match self {
            Self::Management(management_frame) => management_frame.get_fcf(),
            Self::Data(data_frame) => data_frame.header.get_fcf(),
        }
    }
}
impl<'a> IEEE80211Frame<'a> {
    /// Total length in bytes.
    pub const fn length_in_bytes(&self, fcs_at_end: bool) -> usize {
        2 + // Type/Subtype and Flags
        match self {
            Self::Management(management_frame) => management_frame.length_in_bytes(),
            Self::Data(data_frame) => data_frame.length_in_bytes()
        } +
        if fcs_at_end {
            4
        } else {
            0
        }
    }
}
impl<
        'a,
        TLVIterator: IntoIterator<Item = IEEE80211TLV<'a>> + Clone,
        DataFramePayload: MeasureWith<()>,
    > MeasureWith<bool> for IEEE80211Frame<'a, TLVIterator, DataFramePayload>
{
    fn measure_with(&self, fcs_at_end: &bool) -> usize {
        2 + match self {
            Self::Management(management_frame) => management_frame.measure_with(&()),
            Self::Data(data_frame) => data_frame.measure_with(&()),
        } + if *fcs_at_end { 4 } else { 0 }
    }
}
impl<'a> TryFromCtx<'a, bool> for IEEE80211Frame<'a, TLVReadIterator<'a>> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], fcs_at_end: bool) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let fcf =
            FrameControlField::from_representation(from.gread_with(&mut offset, Endian::Little)?);

        // This prevents subsequent parsers from reading the FCS.
        let body_slice = if fcs_at_end {
            from.pread_with::<&[u8]>(0, from.len() - offset - 4)?
        } else {
            from
        };
        let frame = match fcf.frame_type {
            FrameType::Management(subtype) => {
                Self::Management(body_slice.gread_with(&mut offset, (subtype, fcf.flags))?)
            }
            FrameType::Data(subtype) => {
                Self::Data(body_slice.gread_with(&mut offset, (subtype, fcf.flags))?)
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
                != from.gread_with(&mut offset, Endian::Little)?
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
        TLVIterator: IntoIterator<Item = IEEE80211TLV<'a>>,
        DataFramePayload: TryIntoCtx<Error = scroll::Error>,
    > TryIntoCtx<bool> for IEEE80211Frame<'a, TLVIterator, DataFramePayload>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], fcs_at_end: bool) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.get_fcf().to_representation(), &mut offset)?;

        match self {
            Self::Management(management_frame) => buf.gwrite(management_frame, &mut offset)?,
            Self::Data(data_frame) => buf.gwrite(data_frame, &mut offset)?,
        };
        if fcs_at_end {
            buf.gwrite_with(crc32fast::hash(&buf[..offset]), &mut offset, Endian::Little)?;
        }

        Ok(offset)
    }
}
pub trait ToFrame<'a, TLVIterator, DataFramePayload>: 'a {
    fn to_frame(self) -> IEEE80211Frame<'a, TLVIterator, DataFramePayload>;
}
