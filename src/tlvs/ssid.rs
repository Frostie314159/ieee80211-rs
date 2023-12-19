use scroll::{
    ctx::{MeasureWith, StrCtx, TryFromCtx, TryIntoCtx},
    Pwrite,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
/// A SSID tlv.
///
/// The SSID isn't public, since if we check the length at initialization, we won't have to do checks while serializing.
pub struct SSIDTLV<'a>(&'a str);
impl<'a> SSIDTLV<'a> {
    /// Create a new SSID tlv.
    ///
    /// This returns [None] if `ssid` is longer than 32 bytes.
    pub const fn new(ssid: &'a str) -> Option<SSIDTLV<'a>> {
        if ssid.as_bytes().len() <= 32 {
            Some(Self(ssid))
        } else {
            None
        }
    }

    /// Returns a refrence to the SSID.
    pub const fn ssid(&'a self) -> &'a str {
        self.0
    }

    /// Check if the SSID is hidden.
    ///
    /// # Returns
    /// - [`true`] If the SSID is empty.
    /// - [`false`] If the SSID isn't empty.
    pub const fn is_hidden(&self) -> bool {
        self.0.is_empty()
    }
    /// Return the length in bytes.
    ///
    /// This is useful for hardcoded SSIDs, since it's `const`.
    pub const fn length_in_bytes(&self) -> usize {
        self.0.len()
    }
}
impl MeasureWith<()> for SSIDTLV<'_> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.length_in_bytes()
    }
}
impl<'a> TryFromCtx<'a> for SSIDTLV<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        if from.len() > 32 {
            return Err(scroll::Error::TooBig {
                size: 32,
                len: from.len(),
            });
        }
        <&'a str as TryFromCtx<'a, StrCtx>>::try_from_ctx(from, StrCtx::Length(from.len()))
            .map(|(ssid, len)| (SSIDTLV(ssid), len))
    }
}
impl<'a> TryFromCtx<'a, usize> for SSIDTLV<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: usize) -> Result<(Self, usize), Self::Error> {
        <Self as TryFromCtx<'a>>::try_from_ctx(from, ())
    }
}
impl TryIntoCtx for SSIDTLV<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        buf.pwrite(self.0, 0)
    }
}
