use alloc::borrow::Cow;
use macro_bits::serializable_enum;
use scroll::{ctx::MeasureWith, Pread};
use tlv_rs::TLV;

use crate::util::write_to_vec;

use self::ssid::SSIDTLV;

pub mod ssid;
pub mod supported_rates;

serializable_enum! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum TLVType: u8 {
        #[default]
        SSID => 0x00
    }
}

pub type RawIEEE80211TLV<'a> = TLV<'a, u8, TLVType, u8>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IEEE80211TLV<'a> {
    SSID(SSIDTLV),
    Unknown(RawIEEE80211TLV<'a>),
}
impl<'a> MeasureWith<()> for IEEE80211TLV<'a> {
    fn measure_with(&self, ctx: &()) -> usize {
        (match self {
            Self::SSID(tlv) => tlv.measure_with(ctx),
            Self::Unknown(tlv) => tlv.measure_with(ctx) - 2, // header is already included
        }) + 2
    }
}
impl<'a> IEEE80211TLV<'a> {
    pub fn from_raw_tlv(raw_tlv: RawIEEE80211TLV<'a>) -> Result<Self, scroll::Error> {
        Ok(match raw_tlv.tlv_type {
            TLVType::SSID => Self::SSID(raw_tlv.data.pread(0)?),
            _ => Self::Unknown(raw_tlv),
        })
    }
    pub fn to_raw_tlv(self) -> Result<RawIEEE80211TLV<'a>, scroll::Error> {
        let (tlv_type, data) = match self {
            Self::SSID(tlv) => (TLVType::SSID, Cow::Owned(write_to_vec(tlv, &())?)),
            Self::Unknown(tlv) => (tlv.tlv_type, tlv.data.into()),
        };
        Ok(RawIEEE80211TLV {
            tlv_type,
            data: data.into(),
            ..Default::default()
        })
    }
}
