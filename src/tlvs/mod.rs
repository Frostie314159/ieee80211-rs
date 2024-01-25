use core::{fmt::Debug, iter::repeat, marker::PhantomData};

use macro_bits::serializable_enum;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};
use tlv_rs::{raw_tlv::RawTLV, TLV};

use self::rates::{
    EncodedRate, ExtendedSupportedRatesTLV, ExtendedSupportedRatesTLVReadRateIterator,
    SupportedRatesTLV, SupportedRatesTLVReadRateIterator,
};
/// This module contains the elements, which are found in the body of some frames.
/// If an element only consists of one struct, like the [ssid::SSIDTLV], they are re-exported, otherwise they get their own module.
mod dsss_parameter_set;
pub use dsss_parameter_set::DSSSParameterSet;
pub mod rates;
mod ssid;
pub use ssid::SSIDTLV;

serializable_enum! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    /// Type of an IEEE 802.11 TLV.
    pub enum TLVType: u8 {
        #[default]
        SSID => 0x00,
        SupportedRates => 0x01,
        DSSSParameterSet => 0x03,
        ExtendedSupportedRates => 0x14
    }
}

/// A raw TLV.
pub type RawIEEE80211TLV<'a> = RawTLV<'a, u8, u8>;
type TypedIEEE80211TLV<Payload> = TLV<u8, u8, TLVType, Payload>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// This enum contains all possible elements.
pub enum IEEE80211TLV<
    'a,
    RateIterator = SupportedRatesTLVReadRateIterator<'a>,
    ExtendedRateIterator = ExtendedSupportedRatesTLVReadRateIterator<'a>,
> {
    SSID(SSIDTLV<'a>),
    SupportedRates(SupportedRatesTLV<RateIterator>),
    DSSSParameterSet(DSSSParameterSet),
    ExtendedSupportedRates(ExtendedSupportedRatesTLV<ExtendedRateIterator>),
    Unknown(RawIEEE80211TLV<'a>),
}
impl MeasureWith<()> for IEEE80211TLV<'_> {
    fn measure_with(&self, ctx: &()) -> usize {
        2 + match self {
            Self::SSID(tlv) => tlv.measure_with(ctx),
            Self::SupportedRates(tlv) => tlv.measure_with(ctx),
            Self::DSSSParameterSet(_) => 1,
            Self::ExtendedSupportedRates(tlv) => tlv.measure_with(ctx),
            Self::Unknown(tlv) => tlv.slice.len(),
        }
    }
}
impl<'a> TryFromCtx<'a> for IEEE80211TLV<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let (tlv, len) =
            <RawIEEE80211TLV<'a> as TryFromCtx<'a, Endian>>::try_from_ctx(from, Endian::Little)?;
        let from = tlv.slice;
        Ok((
            match TLVType::from_representation(tlv.tlv_type) {
                TLVType::SSID => Self::SSID(SSIDTLV::try_from_ctx(from, ()).map(|(tlv, _)| tlv)?),
                TLVType::SupportedRates => Self::SupportedRates(
                    SupportedRatesTLV::try_from_ctx(from, ()).map(|(tlv, _)| tlv)?,
                ),
                TLVType::DSSSParameterSet => Self::DSSSParameterSet(
                    DSSSParameterSet::try_from_ctx(from, ()).map(|(tlv, _)| tlv)?,
                ),
                TLVType::ExtendedSupportedRates => Self::ExtendedSupportedRates(
                    ExtendedSupportedRatesTLV::try_from_ctx(from, ()).map(|(tlv, _)| tlv)?,
                ),
                TLVType::Unknown(_) => Self::Unknown(tlv),
            },
            len,
        ))
    }
}
impl<RateIterator: Iterator<Item = EncodedRate> + Clone + ExactSizeIterator> TryIntoCtx
    for IEEE80211TLV<'_, RateIterator>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        match self {
            Self::SSID(payload) => buf.pwrite(
                TypedIEEE80211TLV {
                    tlv_type: TLVType::SSID,
                    payload,
                    _phantom: PhantomData,
                },
                0,
            ),
            Self::SupportedRates(payload) => buf.pwrite(
                TypedIEEE80211TLV {
                    tlv_type: TLVType::SupportedRates,
                    payload,
                    _phantom: PhantomData,
                },
                0,
            ),
            Self::DSSSParameterSet(payload) => buf.pwrite(
                TypedIEEE80211TLV {
                    tlv_type: TLVType::SupportedRates,
                    payload,
                    _phantom: PhantomData,
                },
                0,
            ),
            Self::ExtendedSupportedRates(payload) => buf.pwrite(
                TypedIEEE80211TLV {
                    tlv_type: TLVType::ExtendedSupportedRates,
                    payload,
                    _phantom: PhantomData,
                },
                0,
            ),
            Self::Unknown(tlv) => buf.pwrite(tlv, 0),
        }
    }
}
#[derive(Clone, Copy, Eq)]
/// This is an iterator over the elements contained in the body of a frame.
///
/// It's short circuiting.
pub struct TLVReadIterator<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) offset: usize,
}
impl<'a> TLVReadIterator<'a> {
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
}
impl Debug for TLVReadIterator<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(*self).finish()
    }
}
impl PartialEq for TLVReadIterator<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}
impl<'a> Iterator for TLVReadIterator<'a> {
    type Item = IEEE80211TLV<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.bytes.gread(&mut self.offset).ok()
    }
}
impl ExactSizeIterator for TLVReadIterator<'_> {
    fn len(&self) -> usize {
        repeat(())
            .scan(0usize, |offset, _| {
                self.bytes.gread::<RawIEEE80211TLV>(offset).ok()
            })
            .count()
    }
}
