use macro_bits::serializable_enum;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pwrite,
};
use tlv_rs::TLV;

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

pub type RawIEEE80211TLV<'a, Payload> = TLV<'a, u8, TLVType, u8, Payload>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IEEE80211TLV<'a> {
    SSID(SSIDTLV<'a>),
    Unknown(RawIEEE80211TLV<'a, &'a [u8]>),
}
impl MeasureWith<()> for IEEE80211TLV<'_> {
    fn measure_with(&self, ctx: &()) -> usize {
        2 + match self {
            Self::SSID(tlv) => tlv.measure_with(ctx),
            Self::Unknown(tlv) => tlv.data.len(),
        }
    }
}
impl<'a> TryFromCtx<'a> for IEEE80211TLV<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let (tlv, len) = <RawIEEE80211TLV<'a, &'a [u8]> as TryFromCtx<'a, Endian>>::try_from_ctx(
            from,
            Endian::Little,
        )?;
        Ok((
            match tlv.tlv_type {
                TLVType::SSID => Self::SSID(SSIDTLV::try_from_ctx(from, ()).map(|(ssid, _)| ssid)?),
                TLVType::Unknown(_) => Self::Unknown(tlv),
            },
            len,
        ))
    }
}
impl TryIntoCtx for IEEE80211TLV<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        match self {
            IEEE80211TLV::SSID(ssid_tlv) => buf.pwrite(
                RawIEEE80211TLV {
                    tlv_type: TLVType::SSID,
                    data: ssid_tlv,
                    ..Default::default()
                },
                0,
            ),
            Self::Unknown(tlv) => buf.pwrite(tlv, 0),
        }
    }
}
