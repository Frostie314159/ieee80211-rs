use macro_bits::{bit, bitfield};
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pwrite,
};

use super::RateReadIterator;

bitfield! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// Extended data rate encoded as specified in IEEE 802.11.
    pub struct EncodedExtendedRate: u8 {
        /// The value of the data rate.
        ///
        /// The formular is `rate * 500` to get kbps. Use [EncodedRate::rate_in_kbps] to calculate this.
        pub rate: u8 => bit!(0, 1, 2, 3, 4, 5, 6)
    }
}
impl EncodedExtendedRate {
    /// Returns the data rate in kbps.
    pub const fn rate_in_kbps(&self) -> usize {
        self.rate as usize * 500
    }
    /// Creates a rate from it's speed in kbps.
    pub const fn from_rate_in_kbps(rate: usize) -> Self {
        Self {
            rate: (rate / 500) as u8,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// TLV containing rates supported by the peer.
///
/// The `supported_rates` field is an [Iterator] over [EncodedRate]. This allows passing rates, agnostic of the collection.
/// When deserializing this struct, the Iterator is [ExtendedSupportedRatesTLVReadRateIterator].
pub struct ExtendedSupportedRatesTLV<I> {
    pub supported_rates: I,
}
impl<I: ExactSizeIterator> MeasureWith<()> for ExtendedSupportedRatesTLV<I> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.supported_rates.len()
    }
}
impl<'a> TryFromCtx<'a>
    for ExtendedSupportedRatesTLV<ExtendedSupportedRatesTLVReadRateIterator<'a>>
{
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
                    supported_rates: from
                        .iter()
                        .copied()
                        .map(EncodedExtendedRate::from_representation),
                },
                from.len(),
            ))
        }
    }
}
impl<I: IntoIterator<Item = EncodedExtendedRate>> TryIntoCtx for ExtendedSupportedRatesTLV<I> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;
        for supported_rate in self.supported_rates {
            buf.gwrite(supported_rate.to_representation(), &mut offset)?;
        }
        Ok(offset)
    }
}

/// Iterator over the rates supported by the sender.
pub type ExtendedSupportedRatesTLVReadRateIterator<'a> = RateReadIterator<'a, EncodedExtendedRate>;
