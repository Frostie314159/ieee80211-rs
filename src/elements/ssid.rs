use scroll::{
    ctx::{MeasureWith, StrCtx, TryFromCtx, TryIntoCtx},
    Pwrite,
};

use super::{Element, ElementID};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// A SSID tlv.
///
/// The SSID isn't public, since if we check the length at initialization, we won't have to do checks while serializing.
pub struct SSIDElement<S>(S);
impl<'a> SSIDElement<&'a str> {
    /// Create a new SSID element.
    ///
    /// This returns [None] if `ssid` is longer than 32 bytes.
    pub const fn new(ssid: &'a str) -> Option<Self> {
        if ssid.as_bytes().len() <= 32 {
            Some(Self(ssid))
        } else {
            None
        }
    }
}
impl<S: AsRef<str>> SSIDElement<S> {
    #[doc(hidden)]
    // Only for internal use, by macros.
    pub const fn new_unchecked(ssid: S) -> Self {
        Self(ssid)
    }

    /// Take the SSID.
    pub fn ssid(&self) -> &str {
        self.0.as_ref()
    }

    pub fn take_ssid(self) -> S {
        self.0
    }

    /// Check if the SSID is hidden.
    ///
    /// # Returns
    /// - [`true`] If the SSID is empty.
    /// - [`false`] If the SSID isn't empty.
    pub fn is_hidden(&self) -> bool {
        self.0.as_ref().is_empty()
    }
    /// Return the length in bytes.
    ///
    /// This is useful for hardcoded SSIDs, since it's `const`.
    pub fn length_in_bytes(&self) -> usize {
        self.0.as_ref().len()
    }
}
impl<S: AsRef<str>> MeasureWith<()> for SSIDElement<S> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.length_in_bytes()
    }
}
impl<'a> TryFromCtx<'a> for SSIDElement<&'a str> {
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
impl<S: AsRef<str>> TryIntoCtx for SSIDElement<S> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        buf.pwrite(self.0.as_ref(), 0)
    }
}
impl<S: AsRef<str>> Element for SSIDElement<S> {
    const ELEMENT_ID: ElementID = ElementID::Id(0x00);
    type ReadType<'b> = SSIDElement<&'b str>;
}
impl<S: AsRef<str>> AsRef<str> for SSIDElement<S> {
    fn as_ref(&self) -> &str {
        self.ssid()
    }
}
#[macro_export]
/// Generate an SSID element, while performing all validation at compile time.
///
/// This macro requires, that the passed parameter is either a literal or a const and must be a `&str`.
///
/// ```
/// use ieee80211::ssid;
///
/// let ssid_element = ssid!("OpenRF");
/// assert_eq!(ssid_element.ssid(), "OpenRF");
/// ```
macro_rules! ssid {
    ($ssid:expr) => {{
        use ::ieee80211::elements::SSIDElement;
        const RESULT: SSIDElement<&str> = {
            assert!(
                $ssid.as_bytes().len() <= 32,
                "SSIDs must not exceed a length of more than 32 bytes."
            );
            SSIDElement::new_unchecked($ssid)
        };
        RESULT
    }};
}
