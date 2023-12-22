use core::{
    iter::{Copied, Map},
    slice::Iter,
};
use macro_bits::{bit, bitfield};
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pwrite,
};

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
}

/// The default rate iterator returned, when parsing the [SupportedRatesTLV].
pub type ReadIterator<'a> = Map<Copied<Iter<'a, u8>>, fn(u8) -> EncodedRate>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// TLV containing the rates supported by the AP.
///
/// The `supported_rates` field is an [Iterator] over [EncodedRate]. This allows passing rates, agnostic of the collection.
/// When deserializing this struct, the Iterator is [ReadIterator].
/// There must be no more than 8 rates present, since anything after that gets truncated.
pub struct SupportedRatesTLV<I> {
    pub supported_rates: I,
}
impl<I: Iterator<Item = EncodedRate> + Clone> MeasureWith<()> for SupportedRatesTLV<I> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.supported_rates.clone().count()
    }
}
impl<'a> TryFromCtx<'a> for SupportedRatesTLV<ReadIterator<'a>> {
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
                    supported_rates: from.iter().copied().map(EncodedRate::from_representation),
                },
                from.len(),
            ))
        }
    }
}
impl<I: Iterator<Item = EncodedRate> + Clone> TryIntoCtx for SupportedRatesTLV<I> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        // No more than 8 data rates.
        for rate in self.supported_rates.take(8) {
            buf.gwrite(rate.to_representation(), &mut offset)?;
        }

        Ok(offset)
    }
}
