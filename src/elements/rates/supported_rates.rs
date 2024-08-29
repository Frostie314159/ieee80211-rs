use core::marker::PhantomData;

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pwrite,
};

use crate::elements::{Element, ElementID};

use super::{EncodedRate, RatesReadIterator};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, Hash)]
/// An element containing the rates supported by the AP.
///
/// The `supported_rates` field is an [Iterator] over [EncodedRate]. This allows passing rates, agnostic of the collection.
/// When deserializing this struct, the Iterator is [RatesReadIterator].
/// There must be no more than 8 rates present, since anything after that gets truncated.
pub struct SupportedRatesElement<'a, I = RatesReadIterator<'a>>
where
    I: IntoIterator<Item = EncodedRate>,
{
    pub supported_rates: I,
    pub _phantom: PhantomData<&'a ()>,
}
impl<I: IntoIterator<Item = EncodedRate>> SupportedRatesElement<'_, I> {
    #[doc(hidden)]
    // For internal use only.
    pub const fn new_unchecked(supported_rates: I) -> Self {
        Self {
            supported_rates,
            _phantom: PhantomData,
        }
    }
}
impl<I> SupportedRatesElement<'_, I>
where
    I: IntoIterator<Item = EncodedRate> + Clone,
    I::IntoIter: ExactSizeIterator,
{
    /// Create a new supported rates element.
    ///
    /// This returns [None], if more than eight rates are supplied.
    pub fn new(supported_rates: I) -> Option<Self> {
        if supported_rates.clone().into_iter().len() <= 8 {
            Some(Self::new_unchecked(supported_rates))
        } else {
            None
        }
    }
}
impl<LhsIterator, RhsIterator> PartialEq<SupportedRatesElement<'_, RhsIterator>>
    for SupportedRatesElement<'_, LhsIterator>
where
    LhsIterator: IntoIterator<Item = EncodedRate> + Clone,
    RhsIterator: IntoIterator<Item = EncodedRate> + Clone,
{
    fn eq(&self, other: &SupportedRatesElement<RhsIterator>) -> bool {
        self.supported_rates
            .clone()
            .into_iter()
            .eq(other.supported_rates.clone())
    }
}
impl<I> Eq for SupportedRatesElement<'_, I> where I: IntoIterator<Item = EncodedRate> + Clone {}
impl<I: IntoIterator<Item = EncodedRate> + Clone> MeasureWith<()> for SupportedRatesElement<'_, I> {
    fn measure_with(&self, _ctx: &()) -> usize {
        // Each rate is exactly one byte.
        self.supported_rates.clone().into_iter().count()
    }
}
impl<'a> TryFromCtx<'a> for SupportedRatesElement<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        if from.len() > 8 {
            Err(scroll::Error::TooBig {
                size: 8,
                len: from.len(),
            })
        } else {
            Ok((
                Self::new_unchecked(RatesReadIterator::new(from)),
                from.len(),
            ))
        }
    }
}
impl<I: IntoIterator<Item = EncodedRate>> TryIntoCtx for SupportedRatesElement<'_, I> {
    type Error = scroll::Error;
    #[inline]
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        // No more than 8 data rates.
        for rate in self.supported_rates.into_iter().take(8) {
            buf.gwrite(rate.0, &mut offset)?;
        }

        Ok(offset)
    }
}

impl<I: IntoIterator<Item = EncodedRate> + Clone> Element for SupportedRatesElement<'_, I> {
    const ELEMENT_ID: ElementID = ElementID::Id(0x01);
    type ReadType<'a> = SupportedRatesElement<'a>;
}

#[macro_export]
/// Generate an [EncodedRate].
///
/// ```
/// use ieee80211::{rate, elements::rates::EncodedRate};
///
/// let normal_rate = rate!(1.5); // 1.5Mbit/s
/// assert_eq!(normal_rate, EncodedRate::new()
///     .with_rate(3)
///     .with_is_b(false)
/// );
/// let b_rate = rate!(1.5 B); // 1.5Mbit/s IEEE 802.11b
/// assert_eq!(b_rate, EncodedRate::new()
///     .with_rate(3)
///     .with_is_b(true)
/// );
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
/// assert_eq!(supported_rates_element, SupportedRatesElement::new_unchecked(
///     [
///         rate!(1.5 B),
///         rate!(2)
///     ])
/// );
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
            const RESULT: SupportedRatesElement<[EncodedRate; RATE_COUNT]> = SupportedRatesElement::new_unchecked(RATES);
            RESULT
        }
    };
}
