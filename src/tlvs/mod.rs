use core::marker::PhantomData;

use macro_bits::serializable_enum;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pwrite,
};
use tlv_rs::{TLV, raw_tlv::RawTLV};

use self::{ssid::SSIDTLV, supported_rates::{SupportedRatesTLV, ReadIterator, EncodedRate}};

pub mod ssid;
pub mod supported_rates;

serializable_enum! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    /// Type of an IEEE 802.11 TLV.
    pub enum TLVType: u8 {
        #[default]
        SSID => 0x00,
        SupportedRates => 0x01
    }
}

/// A raw TLV.
pub type RawIEEE80211TLV<'a> = RawTLV<'a, u8, u8>;
type TypedIEEE80211TLV<Payload> = TLV<u8, u8, TLVType, Payload>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IEEE80211TLV<'a, RateIterator = ReadIterator<'a>> {
    SSID(SSIDTLV<'a>),
    SupportedRates(SupportedRatesTLV<RateIterator>),
    Unknown(RawIEEE80211TLV<'a>),
}
impl MeasureWith<()> for IEEE80211TLV<'_> {
    fn measure_with(&self, ctx: &()) -> usize {
        2 + match self {
            Self::SSID(tlv) => tlv.measure_with(ctx),
            Self::SupportedRates(tlv) => tlv.measure_with(ctx),
            Self::Unknown(tlv) => tlv.slice.len(),
        }
    }
}
impl<'a> TryFromCtx<'a> for IEEE80211TLV<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let (tlv, len) = <RawIEEE80211TLV<'a> as TryFromCtx<'a, Endian>>::try_from_ctx(
            from,
            Endian::Little,
        )?;
        Ok((
            match TLVType::from_representation(tlv.tlv_type) {
                TLVType::SSID => Self::SSID(SSIDTLV::try_from_ctx(from, ()).map(|(tlv, _)| tlv)?),
                TLVType::SupportedRates => Self::SupportedRates(
                    SupportedRatesTLV::<ReadIterator>::try_from_ctx(from, ()).map(|(tlv, _)| tlv)?,
                ),
                TLVType::Unknown(_) => Self::Unknown(tlv),
            },
            len,
        ))
    }
}
impl<RateIterator: Iterator<Item = EncodedRate> + Clone> TryIntoCtx for IEEE80211TLV<'_, RateIterator> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        match self {
            Self::SSID(payload) => buf.pwrite(
                TypedIEEE80211TLV {
                    tlv_type: TLVType::SSID,
                    payload,
                    _phantom: PhantomData
                },
                0,
            ),
            Self::SupportedRates(payload) => buf.pwrite(
                TypedIEEE80211TLV {
                    tlv_type: TLVType::SupportedRates,
                    payload,
                    _phantom: PhantomData
                },
                0,
            ),
            Self::Unknown(tlv) => buf.pwrite(tlv, 0),
        }
    }
}
