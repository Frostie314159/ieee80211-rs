use macro_bits::{bit, bitfield};
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pwrite,
};

use crate::tlvs::{ToTLV, IEEE80211TLV};

use super::{rate_iter::RateIterator, EncodedExtendedRate};

bitfield! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// Data rate encoded as specified in IEEE 802.11.
    pub struct EncodedRate: u8 {
        /// The value of the data rate.
        ///
        /// The formular is `rate * 500` to get kbps. Use [EncodedRate::rate_in_kbps] to calculate this.
        pub rate: u8 => bit!(0, 1, 2, 3, 4, 5, 6),
        /// Is the data rate IEEE 802.11b.
        pub is_b: bool => bit!(7)
    }
}
impl EncodedRate {
    /// Returns the data rate in kbps.
    pub const fn rate_in_kbps(&self) -> usize {
        self.rate as usize * 500
    }
    /// Creates a rate from it's speed in kbps.
    pub const fn from_rate_in_kbps(rate: usize, is_b: bool) -> Self {
        Self {
            rate: (rate / 500) as u8,
            is_b,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// TLV containing the rates supported by the AP.
///
/// The `supported_rates` field is an [Iterator] over [EncodedRate]. This allows passing rates, agnostic of the collection.
/// When deserializing this struct, the Iterator is [SupportedRatesTLVReadRateIterator].
/// There must be no more than 8 rates present, since anything after that gets truncated.
pub struct SupportedRatesTLV<I> {
    pub supported_rates: I,
}
impl<I: IntoIterator<Item = EncodedRate> + Clone> MeasureWith<()> for SupportedRatesTLV<I> {
    fn measure_with(&self, _ctx: &()) -> usize {
        // Each rate is exactly one byte.
        self.supported_rates.clone().into_iter().count()
    }
}
impl<'a> TryFromCtx<'a> for SupportedRatesTLV<SupportedRatesTLVReadRateIterator<'a>> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        if from.len() > 8 {
            Err(scroll::Error::TooBig {
                size: 8,
                len: from.len(),
            })
        } else {
            Ok((
                SupportedRatesTLV {
                    supported_rates: SupportedRatesTLVReadRateIterator::new(from),
                },
                from.len(),
            ))
        }
    }
}
impl<I: IntoIterator<Item = EncodedRate> + Clone> TryIntoCtx for SupportedRatesTLV<I> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        // No more than 8 data rates.
        for rate in self.supported_rates.into_iter().take(8) {
            buf.gwrite(rate.to_representation(), &mut offset)?;
        }

        Ok(offset)
    }
}
impl<
        'a,
        RateIterator: IntoIterator<Item = EncodedRate> + Clone,
        ExtendedRateIterator: IntoIterator<Item = EncodedExtendedRate> + Clone,
    > ToTLV<'a, RateIterator, ExtendedRateIterator> for SupportedRatesTLV<RateIterator>
{
    fn to_tlv(self) -> IEEE80211TLV<'a, RateIterator, ExtendedRateIterator> {
        IEEE80211TLV::SupportedRates(self)
    }
}
/// The Iterator returned, when reading the [SupportedRatesTLV].
pub type SupportedRatesTLVReadRateIterator<'a> = RateIterator<'a, EncodedRate>;
