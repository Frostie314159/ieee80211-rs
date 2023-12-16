use core::str::FromStr;
use heapless::String;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pwrite,
};

pub type SSID = String<32>;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct SSIDTLV {
    pub ssid: SSID,
}
impl SSIDTLV {
    #[inline]
    pub fn is_hidden(&self) -> bool {
        self.ssid.is_empty()
    }
}
impl MeasureWith<()> for SSIDTLV {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.ssid.len()
    }
}
impl<'a> TryFromCtx<'a> for SSIDTLV {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        if from.len() > 32 {
            return Err(scroll::Error::TooBig {
                size: 32,
                len: from.len(),
            });
        }
        let ssid =
            SSID::from_str(
                core::str::from_utf8(from).map_err(|_| scroll::Error::BadInput {
                    size: from.len(),
                    msg: "Invalid UTF-8",
                })?,
            )
            .unwrap();
        let len = ssid.len();
        Ok((Self { ssid }, len))
    }
}
impl TryIntoCtx for SSIDTLV {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        buf.pwrite(self.ssid.as_bytes(), 0)
    }
}
