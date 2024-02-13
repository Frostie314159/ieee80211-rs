use scroll::{
    ctx::{MeasureWith, StrCtx, TryFromCtx, TryIntoCtx},
    Pwrite,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// A SSID tlv.
///
/// The SSID isn't public, since if we check the length at initialization, we won't have to do checks while serializing.
pub struct SSIDElement<'a>(&'a str);
impl<'a> SSIDElement<'a> {
    /// Create a new SSID element.
    ///
    /// This returns [None] if `ssid` is longer than 32 bytes.
    pub const fn new(ssid: &'a str) -> Option<SSIDElement<'a>> {
        if ssid.as_bytes().len() <= 32 {
            Some(Self(ssid))
        } else {
            None
        }
    }

    /// Create a new SSID element without asserting, that the length is no more than 32 bytes.
    ///
    /// If you are passing a literal directly use the .. macro, which does the assertion at compile time.
    pub const fn new_unchecked(ssid: &'a str) -> SSIDElement<'a> {
        Self(ssid)
    }

    /// Returns a refrence to the SSID.
    pub const fn ssid(&'a self) -> &'a str {
        self.take_ssid()
    }

    /// Take the SSID.
    pub const fn take_ssid(self) -> &'a str {
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
impl MeasureWith<()> for SSIDElement<'_> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.length_in_bytes()
    }
}
impl<'a> TryFromCtx<'a> for SSIDElement<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        if from.len() > 32 {
            return Err(scroll::Error::TooBig {
                size: 32,
                len: from.len(),
            });
        }
        <&'a str as TryFromCtx<'a, StrCtx>>::try_from_ctx(from, StrCtx::Length(from.len()))
            .map(|(ssid, len)| (SSIDElement(ssid), len))
    }
}
impl TryIntoCtx for SSIDElement<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        buf.pwrite(self.0, 0)
    }
}
#[macro_export]
/// Generate an SSID element, while performing all validation at compile time.
///
/// ```
/// use ieee80211::ssid;
///
/// let ssid_element = ssid!("OpenRF");
/// assert_eq!(ssid_element.ssid(), "OpenRF");
/// ```
macro_rules! ssid {
    ($ssid:literal) => {{
        use ::ieee80211::elements::SSIDElement;
        const RESULT: SSIDElement = {
            assert!(
                $ssid.as_bytes().len() <= 32,
                "SSIDs must not exceed a length of more than 32 bytes."
            );
            SSIDElement::new_unchecked($ssid)
        };
        RESULT
    }};
}
