use core::{fmt::Display, marker::PhantomData};

use scroll::{
    ctx::{MeasureWith, StrCtx, TryFromCtx, TryIntoCtx},
    Pwrite,
};

use super::{Element, ElementID};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// The SSID element holds the human-readable identifier of a BSS.
///
/// The SSID isn't public, since if we check the length at initialization, we won't have to do checks while serializing.
pub struct SSIDElement<'a, SSID = &'a str> {
    ssid: SSID,
    _phantom: PhantomData<&'a ()>,
}
impl<'a> SSIDElement<'a> {
    /// Create a new SSID element.
    ///
    /// This returns [None] if `ssid` is longer than 32 bytes.
    pub const fn const_new(ssid: &'a str) -> Option<Self> {
        if ssid.len() <= 32 {
            Some(Self {
                ssid,
                _phantom: PhantomData,
            })
        } else {
            None
        }
    }
}
impl<SSID: AsRef<str>> SSIDElement<'_, SSID> {
    /// Create a new SSID element.
    ///
    /// This returns [None] if `ssid` is longer than 32 bytes.
    pub fn new(ssid: SSID) -> Option<Self> {
        if ssid.as_ref().len() <= 32 {
            Some(Self {
                ssid,
                _phantom: PhantomData,
            })
        } else {
            None
        }
    }
    #[doc(hidden)]
    #[inline]
    // Only for internal use, by macros.
    pub const fn new_unchecked(ssid: SSID) -> Self {
        Self {
            ssid,
            _phantom: PhantomData,
        }
    }

    #[inline]
    /// Get the ssid as a [str] reference.
    pub fn ssid(&self) -> &str {
        self.ssid.as_ref()
    }

    #[inline]
    /// Take the SSID.
    pub fn take_ssid(self) -> SSID {
        self.ssid
    }

    /// Check if the SSID is hidden.
    ///
    /// # Returns
    /// - [`true`] If the SSID is empty.
    /// - [`false`] If the SSID isn't empty.
    pub fn is_hidden(&self) -> bool {
        self.ssid().is_empty()
    }
    /// Return the length in bytes.
    ///
    /// This is useful for hardcoded SSIDs, since it's `const`.
    pub fn length_in_bytes(&self) -> usize {
        self.ssid().len()
    }
}
impl<SSID: AsRef<str>> Element for SSIDElement<'_, SSID> {
    const ELEMENT_ID: ElementID = ElementID::Id(0x00);
    type ReadType<'a> = SSIDElement<'a>;
}
impl<SSID: AsRef<str>> AsRef<str> for SSIDElement<'_, SSID> {
    fn as_ref(&self) -> &str {
        self.ssid()
    }
}
impl<SSID: AsRef<str>> Display for SSIDElement<'_, SSID> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.ssid.as_ref())
    }
}
#[cfg(feature = "defmt")]
impl<SSID: AsRef<str>> defmt::Format for SSIDElement<'_, SSID> {
    fn format(&self, fmt: defmt::Formatter) {
        self.ssid.as_ref().format(fmt)
    }
}
impl<'a> TryFromCtx<'a> for SSIDElement<'a> {
    type Error = scroll::Error;
    #[inline]
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        if from.len() > 32 {
            return Err(scroll::Error::TooBig {
                size: 32,
                len: from.len(),
            });
        }
        <&'a str as TryFromCtx<'a, StrCtx>>::try_from_ctx(from, StrCtx::Length(from.len()))
            .map(|(ssid, len)| (Self::new_unchecked(ssid), len))
    }
}
impl<SSID: AsRef<str>> MeasureWith<()> for SSIDElement<'_, SSID> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.length_in_bytes()
    }
}
impl<SSID: AsRef<str>> TryIntoCtx for SSIDElement<'_, SSID> {
    type Error = scroll::Error;
    #[inline]
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        buf.pwrite(self.ssid(), 0)
    }
}
#[macro_export]
/// Generate an [SSIDElement], while performing all validation at compile time.
///
/// This macro requires, that the passed parameter is either a literal or a const and must be a `&str`.
/// ```
/// use ieee80211::ssid;
///
/// let ssid_element = ssid!("OpenRF");
/// assert_eq!(ssid_element.ssid(), "OpenRF");
/// ```
/// If the provided literal is longer than 32 bytes, the macro will panic at compile time.
///
/// ```compile_fail
/// use ieee80211::ssid;
///
/// let _ssid_element = ssid!("Some unreasonably long SSID, for whatever reason.");
/// ```
macro_rules! ssid {
    ($ssid:expr) => {{
        use ::ieee80211::elements::SSIDElement;
        const RESULT: SSIDElement<'static> = {
            assert!(
                $ssid.as_bytes().len() <= 32,
                "SSIDs must not exceed a length of more than 32 bytes."
            );
            SSIDElement::new_unchecked($ssid)
        };
        RESULT
    }};
}
