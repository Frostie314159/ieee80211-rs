use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::{
    common::{Empty, FCFFlags, FrameControlField, FrameType},
    elements::Elements,
    IEEE80211Frame, ToFrame,
};

use self::{
    body::{ManagementFrameBody, ManagementFrameSubtype},
    header::ManagementFrameHeader,
};

pub mod body;
pub mod header;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// An IEEE 802.11 Management Frame.
pub struct ManagementFrame<'a, ElementContainer = Elements<'a>, ActionFramePayload = &'a [u8]>
where
    ElementContainer: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
    ActionFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
{
    pub header: ManagementFrameHeader,
    pub body: ManagementFrameBody<'a, ElementContainer, ActionFramePayload>,
}
impl<
        'a,
        ElementContainer: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
        ActionFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
    > ManagementFrame<'a, ElementContainer, ActionFramePayload>
{
    pub const fn get_fcf(&self) -> FrameControlField {
        FrameControlField::new()
            .with_frame_type(FrameType::Management(self.body.get_subtype()))
            .with_flags(self.header.fcf_flags)
    }
}
impl ManagementFrame<'_> {
    pub const fn length_in_bytes(&self) -> usize {
        self.header.length_in_bytes() + self.body.length_in_bytes()
    }
}
impl<
        'a,
        ElementContainer: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
        ActionFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
    > MeasureWith<()> for ManagementFrame<'a, ElementContainer, ActionFramePayload>
{
    fn measure_with(&self, ctx: &()) -> usize {
        self.header.length_in_bytes() + self.body.measure_with(ctx)
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
impl<
        'a,
        ElementContainer: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
        ActionFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
    > TryIntoCtx for ManagementFrame<'a, ElementContainer, ActionFramePayload>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.header, &mut offset)?;
        buf.gwrite(self.body, &mut offset)?;
        Ok(offset)
    }
}
impl<
        'a,
        ElementContainer: TryIntoCtx<Error = scroll::Error> + MeasureWith<()> + 'a,
        ActionFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()> + 'a,
    > ToFrame<'a, ElementContainer, ActionFramePayload, Empty>
    for ManagementFrame<'a, ElementContainer, ActionFramePayload>
{
    fn to_frame(self) -> IEEE80211Frame<'a, ElementContainer, ActionFramePayload, Empty> {
        IEEE80211Frame::Management(self)
    }
}
