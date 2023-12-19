use heapless::Vec;
use macro_bits::{bit, bitfield};
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pwrite,
};

bitfield! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct EncodedRate: u8 {
        pub rate: u8 => bit!(0, 1, 2, 3, 4, 5, 6),
        pub is_b: bool => bit!(7)
    }
}
impl EncodedRate {
    pub const fn rate_in_kbps(&self) -> usize {
        self.rate as usize * 500
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SupportedRatesTLV {
    pub supported_rates: Vec<EncodedRate, 8>,
}
impl MeasureWith<()> for SupportedRatesTLV {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.supported_rates.len()
    }
}
impl TryFromCtx<'_> for SupportedRatesTLV {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'_ [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        if from.len() > 8 {
            Err(scroll::Error::TooBig {
                size: 8,
                len: from.len(),
            })
        } else {
            Ok((
                SupportedRatesTLV {
                    supported_rates: Vec::from_iter(
                        from.iter().copied().map(EncodedRate::from_representation),
                    ),
                },
                from.len(),
            ))
        }
    }
}
impl TryFromCtx<'_, usize> for SupportedRatesTLV {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'_ [u8], _ctx: usize) -> Result<(Self, usize), Self::Error> {
        Self::try_from_ctx(from, ())
    }
}
impl TryIntoCtx for SupportedRatesTLV {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;
        for rate in self.supported_rates {
            buf.gwrite(rate.to_representation(), &mut offset)?;
        }
        Ok(offset)
    }
}
