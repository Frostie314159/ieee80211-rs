use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::{
    common::{subtypes::ManagementFrameSubtype, FCFFlags, FrameControlField, FrameType},
    data_frame::DataFrameReadPayload,
    tlvs::{
        rates::{
            EncodedExtendedRate, EncodedRate, ExtendedSupportedRatesTLVReadRateIterator,
            SupportedRatesTLVReadRateIterator,
        },
        TLVReadIterator, IEEE80211TLV,
    },
    IEEE80211Frame, ToFrame,
};

use self::{body::ManagementFrameBody, header::ManagementFrameHeader};

pub mod body;
pub mod header;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManagementFrame<
    'a,
    RateIterator = SupportedRatesTLVReadRateIterator<'a>,
    ExtendedRateIterator = ExtendedSupportedRatesTLVReadRateIterator<'a>,
    TLVIterator = TLVReadIterator<'a>,
    ActionFramePayload = &'a [u8],
> where
    TLVIterator: IntoIterator<Item = IEEE80211TLV<'a, RateIterator, ExtendedRateIterator>>,
{
    pub header: ManagementFrameHeader,
    pub body: ManagementFrameBody<
        'a,
        RateIterator,
        ExtendedRateIterator,
        TLVIterator,
        ActionFramePayload,
    >,
}
impl<
        'a,
        RateIterator,
        ExtendedRateIterator,
        TLVIterator: IntoIterator<Item = IEEE80211TLV<'a, RateIterator, ExtendedRateIterator>>,
        ActionFramePayload,
    > ManagementFrame<'a, RateIterator, ExtendedRateIterator, TLVIterator, ActionFramePayload>
{
    pub const fn get_fcf(&self) -> FrameControlField {
        FrameControlField {
            version: 0,
            frame_type: FrameType::Management(self.body.get_subtype()),
            flags: self.header.fcf_flags,
        }
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
        ExtendedRateIterator: IntoIterator<Item = EncodedExtendedRate> + Clone,
        TLVIterator: IntoIterator<Item = IEEE80211TLV<'a, RateIterator, ExtendedRateIterator>> + Clone,
        ActionFramePayload: MeasureWith<()>,
    > MeasureWith<()>
    for ManagementFrame<'a, RateIterator, ExtendedRateIterator, TLVIterator, ActionFramePayload>
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
        ExtendedRateIterator: IntoIterator<Item = EncodedExtendedRate> + Clone,
        TLVIterator: IntoIterator<Item = IEEE80211TLV<'a, RateIterator, ExtendedRateIterator>> + Clone,
        ActionFramePayload: TryIntoCtx<Error = scroll::Error>,
    > TryIntoCtx
    for ManagementFrame<'a, RateIterator, ExtendedRateIterator, TLVIterator, ActionFramePayload>
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
        RateIterator: 'a,
        ExtendedRateIterator: 'a,
        TLVIterator: IntoIterator<Item = IEEE80211TLV<'a, RateIterator, ExtendedRateIterator>> + 'a,
        ActionFramePayload: 'a,
    >
    ToFrame<
        'a,
        RateIterator,
        ExtendedRateIterator,
        TLVIterator,
        ActionFramePayload,
        DataFrameReadPayload<'a>,
    > for ManagementFrame<'a, RateIterator, ExtendedRateIterator, TLVIterator, ActionFramePayload>
{
    fn to_frame(
        self,
    ) -> IEEE80211Frame<
        'a,
        RateIterator,
        ExtendedRateIterator,
        TLVIterator,
        ActionFramePayload,
        DataFrameReadPayload<'a>,
    > {
        IEEE80211Frame::Management(self)
    }
}
