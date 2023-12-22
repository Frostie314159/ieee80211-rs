use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::common::{FrameControlField, FrameType};

use self::{data_frame::DataFrame, mgmt_frame::ManagementFrame};

pub mod data_frame;
pub mod mgmt_frame;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frame<'a> {
    Management(ManagementFrame<'a>),
    Data(DataFrame<'a>),
}
impl Frame<'_> {
    pub const fn length_in_bytes(&self) -> usize {
        2 + // Type/Subtype and Flags
        4 + // FCS
        match self {
            Self::Management(management_frame) => management_frame.length_in_bytes(),
            Self::Data(data_frame) => data_frame.length_in_bytes()
        }
    }
    pub const fn get_fcf(&self) -> FrameControlField {
        match self {
            Self::Management(management_frame) => management_frame.get_fcf(),
            Self::Data(data_frame) => data_frame.header.get_fcf(),
        }
    }
}
impl MeasureWith<()> for Frame<'_> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.length_in_bytes()
    }
}
impl<'a> TryFromCtx<'a> for Frame<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let fcf =
            FrameControlField::from_representation(from.gread_with(&mut offset, Endian::Little)?);

        // This prevents subsequent parsers from reading the FCS.
        let body_slice = &from[..(from.len() - 4)];
        let frame = match fcf.frame_type {
            FrameType::Management(subtype) => Self::Management(
                body_slice.gread_with(&mut offset, (subtype, fcf.flags))?,
            ),
            FrameType::Data(subtype) => {
                Self::Data(body_slice.gread_with(&mut offset, (subtype, fcf.flags))?)
            }
            _ => todo!(),
        };
        if crc32fast::hash(&from[..(from.len() - 4)])
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
impl TryIntoCtx for Frame<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.get_fcf().to_representation(), &mut offset)?;

        match self {
            Self::Management(management_frame) => buf.gwrite(management_frame, &mut offset)?,
            Self::Data(data_frame) => buf.gwrite(data_frame, &mut offset)?,
        };

        buf.gwrite_with(crc32fast::hash(&buf[..offset]), &mut offset, Endian::Little)?;

        Ok(offset)
    }
}
