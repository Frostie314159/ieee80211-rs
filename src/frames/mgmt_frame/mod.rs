use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::common::{
    subtypes::ManagementFrameSubtype, FCFFlags, FrameControlField, FrameType,
};

use self::{header::ManagementFrameHeader, body::ManagementFrameBody};

pub mod header;
pub mod body;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManagementFrame<'a> {
    pub header: ManagementFrameHeader,
    pub body: ManagementFrameBody<'a>,
}
impl ManagementFrame<'_> {
    pub const fn get_fcf(&self) -> FrameControlField {
        FrameControlField {
            version: 0,
            frame_type: FrameType::Management(self.body.get_sub_type()),
            flags: self.header.fcf_flags,
        }
    }
    pub const fn length_in_bytes(&self) -> usize {
        self.header.length_in_bytes() + self.body.length_in_bytes()
    }
}
impl MeasureWith<()> for ManagementFrame<'_> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.length_in_bytes()
    }
}
impl<'a> TryFromCtx<'a, (ManagementFrameSubtype, FCFFlags)> for ManagementFrame<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(
        from: &'a [u8],
        (subtype, fcf_flags): (ManagementFrameSubtype, FCFFlags),
    ) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let header = from.gread_with(&mut offset, fcf_flags)?;
        let body = from.gread_with(&mut offset, subtype)?;

        Ok((Self { header, body }, offset))
    }
}
impl TryIntoCtx for ManagementFrame<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.header, &mut offset)?;
        buf.gwrite(self.body, &mut offset)?;
        Ok(offset)
    }
}
