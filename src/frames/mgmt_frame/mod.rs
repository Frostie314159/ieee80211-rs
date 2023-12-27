use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::{
    common::{subtypes::ManagementFrameSubtype, FCFFlags, FrameControlField, FrameType},
    tlvs::{TLVReadIterator, IEEE80211TLV},
};

use self::{body::ManagementFrameBody, header::ManagementFrameHeader};

pub mod body;
pub mod header;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManagementFrame<'a, TLVIterator> {
    pub header: ManagementFrameHeader,
    pub body: ManagementFrameBody<'a, TLVIterator>,
}
impl<TLVIterator> ManagementFrame<'_, TLVIterator> {
    pub const fn get_fcf(&self) -> FrameControlField {
        FrameControlField {
            version: 0,
            frame_type: FrameType::Management(self.body.get_sub_type()),
            flags: self.header.fcf_flags,
        }
    }
}
impl<'a> ManagementFrame<'a, TLVReadIterator<'a>> {
    pub const fn length_in_bytes(&self) -> usize {
        self.header.length_in_bytes() + self.body.length_in_bytes()
    }
}
impl<'a, TLVIterator: Iterator<Item = IEEE80211TLV<'a>> + Clone> MeasureWith<()>
    for ManagementFrame<'a, TLVIterator>
{
    fn measure_with(&self, ctx: &()) -> usize {
        self.header.length_in_bytes() + self.body.measure_with(ctx)
    }
}
impl<'a> TryFromCtx<'a, (ManagementFrameSubtype, FCFFlags)>
    for ManagementFrame<'a, TLVReadIterator<'a>>
{
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
impl<'a, TLVIterator: Iterator<Item = IEEE80211TLV<'a>>> TryIntoCtx
    for ManagementFrame<'a, TLVIterator>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.header, &mut offset)?;
        buf.gwrite(self.body, &mut offset)?;
        Ok(offset)
    }
}
