use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::{
    common::{subtypes::ManagementFrameSubtype, FCFFlags, FrameControlField, FrameType},
    data_frame::DataFrameReadPayload,
    elements::{
        rates::{EncodedRate, RatesReadIterator},
        ElementReadIterator, IEEE80211Element,
    },
    IEEE80211Frame, ToFrame,
};

use self::{body::ManagementFrameBody, header::ManagementFrameHeader};

pub mod body;
pub mod header;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// An IEEE 802.11 Management Frame.
pub struct ManagementFrame<
    'a,
    RateIterator = RatesReadIterator<'a>,
    ExtendedRateIterator = RatesReadIterator<'a>,
    ElementIterator = ElementReadIterator<'a>,
    ActionFramePayload = &'a [u8],
> where
    RateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ElementIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>>,
{
    pub header: ManagementFrameHeader,
    pub body: ManagementFrameBody<
        'a,
        RateIterator,
        ExtendedRateIterator,
        ElementIterator,
        ActionFramePayload,
    >,
}
impl<'a, RateIterator, ExtendedRateIterator, ElementIterator, ActionFramePayload>
    ManagementFrame<'a, RateIterator, ExtendedRateIterator, ElementIterator, ActionFramePayload>
where
    RateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ElementIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>>,
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
        RateIterator: IntoIterator<Item = EncodedRate> + Clone,
        ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
        ElementIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>> + Clone + 'a,
        ActionFramePayload: MeasureWith<()>,
    > MeasureWith<()>
    for ManagementFrame<'a, RateIterator, ExtendedRateIterator, ElementIterator, ActionFramePayload>
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
        RateIterator: IntoIterator<Item = EncodedRate> + Clone,
        ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
        ElementIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>> + Clone + 'a,
        ActionFramePayload: TryIntoCtx<Error = scroll::Error>,
    > TryIntoCtx
    for ManagementFrame<'a, RateIterator, ExtendedRateIterator, ElementIterator, ActionFramePayload>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.header, &mut offset)?;
        buf.gwrite(self.body, &mut offset)?;
        Ok(offset)
    }
}
impl<'a, RateIterator, ExtendedRateIterator, ElementIterator, ActionFramePayload>
    ToFrame<
        'a,
        RateIterator,
        ExtendedRateIterator,
        ElementIterator,
        ActionFramePayload,
        DataFrameReadPayload<'a>,
    >
    for ManagementFrame<'a, RateIterator, ExtendedRateIterator, ElementIterator, ActionFramePayload>
where
    RateIterator: IntoIterator<Item = EncodedRate> + Clone + 'a,
    ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone + 'a,
    ElementIterator:
        IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>> + 'a,
    ActionFramePayload: 'a,
{
    fn to_frame(
        self,
    ) -> IEEE80211Frame<
        'a,
        RateIterator,
        ExtendedRateIterator,
        ElementIterator,
        ActionFramePayload,
        DataFrameReadPayload<'a>,
    > {
        IEEE80211Frame::Management(self)
    }
}
