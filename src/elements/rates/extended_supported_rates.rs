use core::marker::PhantomData;

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pwrite,
};

use crate::elements::{Element, ElementID};

use super::{EncodedRate, RatesReadIterator};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, Hash)]
/// An element containing rates supported by the peer.
///
/// The `supported_rates` field is an [Iterator] over [super::EncodedRate]. This allows passing rates, agnostic of the collection.
/// When deserializing this struct, the Iterator is [RatesReadIterator].
pub struct ExtendedSupportedRatesElement<'a, I = RatesReadIterator<'a>>
where
    I: IntoIterator<Item = EncodedRate>,
{
    pub supported_rates: I,
    pub _phantom: PhantomData<&'a ()>,
}
impl<I: IntoIterator<Item = EncodedRate>> ExtendedSupportedRatesElement<'_, I> {
    #[doc(hidden)]
    // For internal use only.
    pub const fn new_unchecked(supported_rates: I) -> Self {
        Self {
            supported_rates,
            _phantom: PhantomData,
        }
    }
}
impl<I> ExtendedSupportedRatesElement<'_, I>
where
    I: IntoIterator<Item = EncodedRate> + Clone,
    I::IntoIter: ExactSizeIterator,
{
    /// Create a new supported rates element.
    ///
    /// This returns [None], if more than 251 rates are supplied.
    pub fn new(supported_rates: I) -> Option<Self> {
        if supported_rates.clone().into_iter().len() <= 251 {
            Some(Self::new_unchecked(supported_rates))
        } else {
            None
        }
    }
}
impl<LhsIterator, RhsIterator> PartialEq<ExtendedSupportedRatesElement<'_, RhsIterator>>
    for ExtendedSupportedRatesElement<'_, LhsIterator>
where
    LhsIterator: IntoIterator<Item = EncodedRate> + Clone,
    RhsIterator: IntoIterator<Item = EncodedRate> + Clone,
{
    fn eq(&self, other: &ExtendedSupportedRatesElement<RhsIterator>) -> bool {
        self.supported_rates
            .clone()
            .into_iter()
            .eq(other.supported_rates.clone())
    }
}
impl<I> Eq for ExtendedSupportedRatesElement<'_, I> where I: IntoIterator<Item = EncodedRate> + Clone
{}
impl<I: IntoIterator<Item = EncodedRate> + Clone> MeasureWith<()>
    for ExtendedSupportedRatesElement<'_, I>
{
    fn measure_with(&self, _ctx: &()) -> usize {
        self.supported_rates.clone().into_iter().count()
    }
}
impl<'a> TryFromCtx<'a> for ExtendedSupportedRatesElement<'a> {
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
                    _phantom: PhantomData,
                },
                from.len(),
            ))
        }
    }
}
impl<I: IntoIterator<Item = EncodedRate>> TryIntoCtx for ExtendedSupportedRatesElement<'_, I> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;
        for supported_rate in self.supported_rates {
            buf.gwrite(supported_rate.0, &mut offset)?;
        }
        Ok(offset)
    }
}
impl<I: IntoIterator<Item = EncodedRate> + Clone> Element for ExtendedSupportedRatesElement<'_, I> {
    const ELEMENT_ID: ElementID = ElementID::Id(0x32);
    type ReadType<'a> = ExtendedSupportedRatesElement<'a>;
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
/// assert_eq!(extended_supported_rates_element, ExtendedSupportedRatesElement::new_unchecked([
///     rate!(1.5),
///     rate!(2)
/// ]));
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
            const RESULT: ExtendedSupportedRatesElement<[EncodedRate; RATE_COUNT]> = ExtendedSupportedRatesElement::new_unchecked(RATES);
            RESULT
        }
    };
}
