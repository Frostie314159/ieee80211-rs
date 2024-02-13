use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pwrite,
};

use super::{EncodedRate, RatesReadIterator};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// TLV containing rates supported by the peer.
///
/// The `supported_rates` field is an [Iterator] over [super::EncodedRate]. This allows passing rates, agnostic of the collection.
/// When deserializing this struct, the Iterator is [RatesReadIterator].
pub struct ExtendedSupportedRatesElement<I> {
    pub supported_rates: I,
}
impl<I: IntoIterator<Item = EncodedRate> + Clone> MeasureWith<()>
    for ExtendedSupportedRatesElement<I>
{
    fn measure_with(&self, _ctx: &()) -> usize {
        self.supported_rates.clone().into_iter().count()
    }
}
impl<'a> TryFromCtx<'a> for ExtendedSupportedRatesElement<RatesReadIterator<'a>> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        if from.len() > 255 {
            Err(scroll::Error::TooBig {
                size: 8,
                len: from.len(),
            })
        } else {
            Ok((
                Self {
                    supported_rates: RatesReadIterator::new(from),
                },
                from.len(),
            ))
        }
    }
}
impl<I: IntoIterator<Item = EncodedRate>> TryIntoCtx for ExtendedSupportedRatesElement<I> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;
        for supported_rate in self.supported_rates {
            buf.gwrite(supported_rate.to_representation(), &mut offset)?;
        }
        Ok(offset)
    }
}

#[macro_export]
/// Generate an [ExtendedSupportedRatesElement].
///
/// ```
/// use ieee80211::{extended_supported_rates, rate, elements::rates::ExtendedSupportedRatesElement};
///
/// let extended_supported_rates_element = extended_supported_rates![
///     1.5,
///     2
/// ];
/// assert_eq!(extended_supported_rates_element, ExtendedSupportedRatesElement {
///     supported_rates: [
///         rate!(1.5),
///         rate!(2)
///     ]
/// });
/// ```
macro_rules! extended_supported_rates {
    ($(
        $rate:literal $($is_b:ident)?
    ),*) => {
        {
            use ::ieee80211::{elements::rates::{ExtendedSupportedRatesElement, EncodedRate}, rate, const_soft_float::soft_f32::SoftF32, macro_bits::{check_bit, set_bit, bit}};
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
                assert!(RATES.len() <= 251, "More than 251 rates are invalid.");
            };
            const RESULT: ExtendedSupportedRatesElement<[EncodedRate; RATE_COUNT]> = ExtendedSupportedRatesElement {
                supported_rates: RATES
            };
            RESULT
        }
    };
}
