use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pwrite,
};

use crate::elements::{
    rates::{EncodedRate, RatesReadIterator},
    ElementReadIterator, IEEE80211Element,
};

use super::{ManagementFrameBody, ToManagementFrameBody};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// The body of a probe request.
pub struct ProbeRequestBody<
    'a,
    RateIterator = RatesReadIterator<'a>,
    ExtendedRateIterator = RatesReadIterator<'a>,
    ElementIterator = ElementReadIterator<'a>,
> where
    RateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ElementIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>>,
{
    pub tagged_payload: ElementIterator,
}
impl ProbeRequestBody<'_> {
    /// The entire length in bytes.
    pub const fn length_in_bytes(&self) -> usize {
        match self.tagged_payload.bytes {
            Some(bytes) => bytes.len(),
            None => 0,
        }
    }
}
impl<'a, RateIterator, ExtendedRateIterator, ElementIterator>
    ProbeRequestBody<'a, RateIterator, ExtendedRateIterator, ElementIterator>
where
    RateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ElementIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>> + Clone,
{
    /// Extract the SSID from the tlvs.
    pub fn ssid(&self) -> Option<&str> {
        // SSID should be the first TLV.
        self.tagged_payload.clone().into_iter().find_map(|tlv| {
            if let IEEE80211Element::SSID(ssid_tlv) = tlv {
                Some(ssid_tlv.take_ssid())
            } else {
                None
            }
        })
    }
}
impl<'a, RateIterator, ExtendedRateIterator, ElementIterator> MeasureWith<()>
    for ProbeRequestBody<'a, RateIterator, ExtendedRateIterator, ElementIterator>
where
    RateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ElementIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>> + Clone,
{
    fn measure_with(&self, ctx: &()) -> usize {
        self.tagged_payload
            .clone()
            .into_iter()
            .map(|tlv| tlv.measure_with(ctx))
            .sum::<usize>()
    }
}
impl<'a> TryFromCtx<'a> for ProbeRequestBody<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        Ok((
            Self {
                tagged_payload: ElementReadIterator::new(from),
            },
            from.len(),
        ))
    }
}
impl<
        'a,
        RateIterator: IntoIterator<Item = EncodedRate> + Clone,
        ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
        ElementIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>> + Clone + 'a,
    > TryIntoCtx for ProbeRequestBody<'a, RateIterator, ExtendedRateIterator, ElementIterator>
where
    IEEE80211Element<'a, RateIterator, ExtendedRateIterator>: MeasureWith<()>,
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        for element in self.tagged_payload {
            buf.gwrite(element, &mut offset)?;
        }

        Ok(offset)
    }
}
impl<'a, RateIterator, ExtendedRateIterator, ElementIterator>
    ToManagementFrameBody<'a, RateIterator, ExtendedRateIterator, ElementIterator>
    for ProbeRequestBody<'a, RateIterator, ExtendedRateIterator, ElementIterator>
where
    RateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ElementIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>>,
{
    fn to_management_frame_body(
        self,
    ) -> ManagementFrameBody<'a, RateIterator, ExtendedRateIterator, ElementIterator> {
        ManagementFrameBody::ProbeRequest(self)
    }
}
