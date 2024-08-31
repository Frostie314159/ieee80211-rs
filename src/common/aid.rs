use core::{fmt::Debug, ops::RangeInclusive};

use bitfield_struct::bitfield;

#[bitfield(u16, conversion = false, debug = false)]
#[derive(PartialEq, Eq, Hash)]
/// An association ID.
///
/// This can **only** be constructed through [Self::new_checked], to make it impossible to create invalid AID's.
/// # Note
/// This currently only valid for a non-S1G and non-DMG STA, due to the bounds imposed on the AID.
pub struct AssociationID {
    #[bits(14)]
    internal_aid: u16,
    #[bits(2, default = 0b11)]
    padding: u8,
}
impl AssociationID {
    /// The lowest valid AID.
    pub const MIN_AID: u16 = 1;
    /// The highest valid AID.
    pub const MAX_AID: u16 = 2007;
    /// This is the range of all valid AIDs.
    pub const VALID_AID_RANGE: RangeInclusive<u16> = Self::MIN_AID..=Self::MAX_AID;

    /// Creates a new [AssociationID] and performs bounds checks.
    pub const fn new_checked(aid: u16) -> Option<Self> {
        if aid >= Self::MIN_AID && aid <= Self::MAX_AID {
            Some(Self::new().with_internal_aid(aid))
        } else {
            None
        }
    }
    #[doc(hidden)]
    #[inline]
    pub const fn new_unchecked(aid: u16) -> Self {
        Self::new().with_internal_aid(aid)
    }
    /// Get the AID.
    pub const fn aid(&self) -> u16 {
        self.internal_aid()
    }
    /// Convert into bits.
    pub const fn into_bits(self) -> u16 {
        self.0
    }
}
impl Debug for AssociationID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{}", self.aid()))
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for AssociationID {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "{}", self.aid())
    }
}
#[macro_export]
/// Generate a new [AssociationID], while performing all checks at compile-time.
///
/// If the [AssociationID] is within [AssociationID::VALID_AID_RANGE], the macro will never fail to evaluate.
/// ```
/// use ieee80211::aid;
///
/// let _aid = aid!(1);
/// ```
///
/// If the [AssociationID] is some invalid value, like 0 or 2008, the macro will panic at compile-time.
/// ```compile_fail
/// use ieee80211::aid;
///
/// let _aid = aid!(2008);
/// ```
macro_rules! aid {
    ($aid:expr) => {
        {
            use ::ieee80211::common::AssociationID;
            // We could use inline const, but that would mean an MSRV of 1.79.0, which may be too recent.
            const AID: AssociationID = {
                assert!($aid != 0, "An AssociationID of zero is invalid.");
                assert!($aid <= AssociationID::MAX_AID, "An AssociationID greater than 2007 is invalid");
                AssociationID::new_unchecked($aid)
            };
            AID
        }
    };
}
