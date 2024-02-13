use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pwrite,
};

use super::{EncodedRate, RatesReadIterator};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// TLV containing the rates supported by the AP.
///
/// The `supported_rates` field is an [Iterator] over [EncodedRate]. This allows passing rates, agnostic of the collection.
/// When deserializing this struct, the Iterator is [RatesReadIterator].
/// There must be no more than 8 rates present, since anything after that gets truncated.
pub struct SupportedRatesElement<I> {
    pub supported_rates: I,
}
impl<I: IntoIterator<Item = EncodedRate> + Clone> MeasureWith<()> for SupportedRatesElement<I> {
    fn measure_with(&self, _ctx: &()) -> usize {
        // Each rate is exactly one byte.
        self.supported_rates.clone().into_iter().count()
    }
}
impl<'a> TryFromCtx<'a> for SupportedRatesElement<RatesReadIterator<'a>> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        if from.len() > 8 {
            Err(scroll::Error::TooBig {
                size: 8,
                len: from.len(),
            })
        } else {
            Ok((
                SupportedRatesElement {
                    supported_rates: RatesReadIterator::new(from),
                },
                from.len(),
            ))
        }
    }
}
impl<I: IntoIterator<Item = EncodedRate> + Clone> TryIntoCtx for SupportedRatesElement<I> {
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

#[macro_export]
/// This macro generates an [EncodedRate].
///
/// ```
/// use ieee80211::{rate, elements::rates::EncodedRate};
///
/// let normal_rate = rate!(1.5); // 1.5Mbit/s
/// assert_eq!(normal_rate, EncodedRate {
///     rate: 3,
///     is_b: false
/// });
/// let b_rate = rate!(1.5 B); // 1.5Mbit/s IEEE 802.11b
/// assert_eq!(b_rate, EncodedRate {
///     rate: 3,
///     is_b: true
/// });
/// ```
macro_rules! rate {
    ($rate:literal) => {
        ::ieee80211::elements::rates::EncodedRate::from_rate_in_kbps(
            ::ieee80211::const_soft_float::soft_f32::SoftF32($rate as f32)
                .mul(::ieee80211::const_soft_float::soft_f32::SoftF32(1000f32))
                .0 as usize,
            false,
        )
    };
    ($rate:literal B) => {
        ::ieee80211::elements::rates::EncodedRate::from_rate_in_kbps(
            ::ieee80211::const_soft_float::soft_f32::SoftF32($rate as f32)
                .mul(::ieee80211::const_soft_float::soft_f32::SoftF32(1000f32))
                .0 as usize,
            true,
        )
    };
}

#[macro_export]
/// Generate a [SupportedRatesElement].
///
/// ```
/// use ieee80211::{supported_rates, rate, elements::rates::SupportedRatesElement};
///
/// let supported_rates_element = supported_rates![
///     1.5 B,
///     2
/// ];
/// assert_eq!(supported_rates_element, SupportedRatesElement {
///     supported_rates: [
///         rate!(1.5 B),
///         rate!(2)
///     ]
/// });
/// ```
macro_rules! supported_rates {
    ($(
        $rate:literal $($is_b:ident)?
    ),*) => {
        {
            use ::ieee80211::{elements::rates::{SupportedRatesElement, EncodedRate}, rate, const_soft_float::soft_f32::SoftF32, macro_bits::{check_bit, set_bit, bit}};
            const RATE_COUNT: usize = $(
                {
                    let _ = $rate;
                    1
                } +
            )* 0;
            const RATES: [EncodedRate; RATE_COUNT] = [
                $(
                    ::ieee80211::rate!($rate $($is_b)?)
                ),*
            ];
            const _: () = {
                assert!(RATES.len() <= 8, "More than eight rates are invalid.");
            };
            const RESULT: SupportedRatesElement<[EncodedRate; RATE_COUNT]> = SupportedRatesElement {
                supported_rates: RATES
            };
            RESULT
        }
    };
}
