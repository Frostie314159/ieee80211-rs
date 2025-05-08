use core::{fmt::Display, marker::PhantomData};

use scroll::{
    ctx::{MeasureWith, StrCtx, TryFromCtx, TryIntoCtx},
    Pwrite,
};

use super::{Element, ElementID};

#[doc(hidden)]
/// The actual type of an SSID element.
///
/// This is used to differentiate between SSID and MeshID, which share the exact same element
/// encoding.
pub trait SSIDLikeElementType {
    /// The ID of the element.
    const ELEMENT_ID: ElementID;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// Common functionality for [SSIDElement] and [MeshIDElement].
pub struct SSIDLikeElement<'a, Type: SSIDLikeElementType, SSID = &'a str> {
    ssid: SSID,
    _phantom: PhantomData<(&'a (), Type)>,
}
impl<'a, Type: SSIDLikeElementType> SSIDLikeElement<'a, Type> {
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
impl<Type: SSIDLikeElementType, SSID: AsRef<str>> SSIDLikeElement<'_, Type, SSID> {
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
impl<Type: SSIDLikeElementType + 'static, SSID: AsRef<str>> Element
    for SSIDLikeElement<'_, Type, SSID>
{
    const ELEMENT_ID: ElementID = Type::ELEMENT_ID;
    type ReadType<'a> = SSIDLikeElement<'a, Type>;
}
impl<Type: SSIDLikeElementType, SSID: AsRef<str>> AsRef<str> for SSIDLikeElement<'_, Type, SSID> {
    fn as_ref(&self) -> &str {
        self.ssid()
    }
}
impl<Type: SSIDLikeElementType, SSID: AsRef<str>> Display for SSIDLikeElement<'_, Type, SSID> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.ssid.as_ref())
    }
}
#[cfg(feature = "defmt")]
impl<Type: SSIDLikeElementType, SSID: AsRef<str>> defmt::Format
    for SSIDLikeElement<'_, Type, SSID>
{
    fn format(&self, fmt: defmt::Formatter) {
        self.ssid.as_ref().format(fmt)
    }
}
impl<'a, Type: SSIDLikeElementType + 'static> TryFromCtx<'a> for SSIDLikeElement<'a, Type> {
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
impl<Type: SSIDLikeElementType, SSID: AsRef<str>> MeasureWith<()>
    for SSIDLikeElement<'_, Type, SSID>
{
    fn measure_with(&self, _ctx: &()) -> usize {
        self.length_in_bytes()
    }
}
impl<Type: SSIDLikeElementType, SSID: AsRef<str>> TryIntoCtx for SSIDLikeElement<'_, Type, SSID> {
    type Error = scroll::Error;
    #[inline]
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        buf.pwrite(self.ssid(), 0)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[doc(hidden)]
pub struct SSIDElementType;
impl SSIDLikeElementType for SSIDElementType {
    const ELEMENT_ID: ElementID = ElementID::Id(0x00);
}
/// The SSID element holds the human-readable identifier of a BSS.
///
/// The SSID isn't public, since if we check the length at initialization, we won't have to do checks while serializing.
pub type SSIDElement<'a, SSID = &'a str> = SSIDLikeElement<'a, SSIDElementType, SSID>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[doc(hidden)]
pub struct MeshIDElementType;
impl SSIDLikeElementType for MeshIDElementType {
    const ELEMENT_ID: ElementID = ElementID::Id(0x72);
}
/// The MeshID element holds the human-readable identifier of a MBSS.
///
/// The MeshID isn't public, since if we check the length at initialization, we won't have to do checks while serializing.
pub type MeshIDElement<'a, SSID = &'a str> = SSIDLikeElement<'a, MeshIDElementType, SSID>;

#[macro_export]
#[doc(hidden)]
macro_rules! ssid_internal {
    ($ssid:expr, $ssid_type:ident) => {{
        use ::ieee80211::elements::$ssid_type;
        const RESULT: $ssid_type<'static> = {
            ::core::assert!(
                $ssid.as_bytes().len() <= 32,
                "SSIDs or MeshIDs must not exceed a length of more than 32 bytes."
            );
            $ssid_type::new_unchecked($ssid)
        };
        RESULT
    }};
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
    ($ssid:expr) => {
        ::ieee80211::ssid_internal!($ssid, SSIDElement)
    };
}
#[macro_export]
macro_rules! mesh_id {
    ($mesh_id:expr) => {
        ::ieee80211::ssid_internal!($mesh_id, MeshIDElement);
    };
}
