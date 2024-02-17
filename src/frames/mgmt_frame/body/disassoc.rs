use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{
    common::reason::IEEE80211Reason,
    elements::{
        rates::{EncodedRate, RatesReadIterator},
        ElementReadIterator, IEEE80211Element,
    },
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// A frame sent to disassociate from a BSS.
pub struct DisassociationFrameBody<
    'a,
    RateIterator = RatesReadIterator<'a>,
    ExtendedRateIterator = RatesReadIterator<'a>,
    ElementIterator = ElementReadIterator<'a>,
> where
    RateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ElementIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>>,
{
    /// The reason for the disassociation.
    pub reason: IEEE80211Reason,
    pub tagged_payload: ElementIterator,
}
impl DisassociationFrameBody<'_> {
    pub const fn length_in_bytes(&self) -> usize {
        2 + match self.tagged_payload.bytes {
            Some(bytes) => bytes.len(),
            None => 0
        }
    }
}
impl<'a, RateIterator, ExtendedRateIterator, ElementIterator> MeasureWith<()>
    for DisassociationFrameBody<'a, RateIterator, ExtendedRateIterator, ElementIterator>
where
    RateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ElementIterator:
        IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>> + Clone + 'a,
    IEEE80211Element<'a, RateIterator, ExtendedRateIterator>: MeasureWith<()>,
{
    fn measure_with(&self, ctx: &()) -> usize {
        2 + self
            .tagged_payload
            .clone()
            .into_iter()
            .map(|element| element.measure_with(ctx))
            .sum::<usize>()
    }
}
impl<'a> TryFromCtx<'a> for DisassociationFrameBody<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let reason = IEEE80211Reason::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        let tagged_payload = ElementReadIterator::new(&from[offset..]);

        Ok((
            Self {
                reason,
                tagged_payload,
            },
            offset,
        ))
    }
}
impl<'a, RateIterator, ExtendedRateIterator, ElementIterator> TryIntoCtx
    for DisassociationFrameBody<'a, RateIterator, ExtendedRateIterator, ElementIterator>
where
    RateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ElementIterator:
        IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>> + Clone + 'a,
    IEEE80211Element<'a, RateIterator, ExtendedRateIterator>: TryIntoCtx<Error = scroll::Error>,
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(self.reason.into_bits(), &mut offset, Endian::Little)?;
        for element in self.tagged_payload {
            buf.gwrite(element, &mut offset)?;
        }

        Ok(offset)
    }
}
