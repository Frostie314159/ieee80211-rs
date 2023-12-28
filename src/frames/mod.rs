use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{
    common::{FrameControlField, FrameType},
    tlvs::{TLVReadIterator, IEEE80211TLV},
};

use self::{data_frame::DataFrame, mgmt_frame::ManagementFrame};

pub mod data_frame;
pub mod mgmt_frame;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frame<'a, TLVIterator> {
    Management(ManagementFrame<'a, TLVIterator>),
    Data(DataFrame<'a>),
}
impl<TLVIterator> Frame<'_, TLVIterator> {
    pub const fn get_fcf(&self) -> FrameControlField {
        match self {
            Self::Management(management_frame) => management_frame.get_fcf(),
            Self::Data(data_frame) => data_frame.header.get_fcf(),
        }
    }
}
impl<'a> Frame<'a, TLVReadIterator<'a>> {
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
impl<'a, TLVIterator: Iterator<Item = IEEE80211TLV<'a>> + Clone> MeasureWith<bool>
    for Frame<'a, TLVIterator>
{
    fn measure_with(&self, fcs_at_end: &bool) -> usize {
        2 + match self {
            Self::Management(management_frame) => management_frame.measure_with(&()),
            Self::Data(data_frame) => data_frame.measure_with(&()),
        } + if *fcs_at_end { 4 } else { 0 }
    }
}
impl<'a> TryFromCtx<'a, bool> for Frame<'a, TLVReadIterator<'a>> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], fcs_at_end: bool) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let fcf =
            FrameControlField::from_representation(from.gread_with(&mut offset, Endian::Little)?);

        // This prevents subsequent parsers from reading the FCS.
        let body_slice = from.pread_with::<&[u8]>(0, from.len() - 4)?;
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
impl<'a, TLVIterator: Iterator<Item = IEEE80211TLV<'a>>> TryIntoCtx<bool>
    for Frame<'a, TLVIterator>
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
