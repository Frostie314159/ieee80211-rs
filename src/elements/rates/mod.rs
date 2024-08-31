//! This module contains support for the Supported- and Extended Supported Rates elements.

mod supported_rates;

use bitfield_struct::bitfield;
pub use supported_rates::*;

mod extended_supported_rates;
pub use extended_supported_rates::*;

mod rate_iter;
pub use rate_iter::RatesReadIterator;

#[bitfield(u8, defmt = cfg(feature = "defmt"))]
#[derive(PartialEq, Eq, Hash)]
/// Data rate encoded as specified in IEEE 802.11.
pub struct EncodedRate {
    #[bits(7)]
    /// The value of the data rate.
    ///
    /// The formular is `rate * 500` to get kbps. Use [EncodedRate::rate_in_kbps] to calculate this.
    pub rate: u8,

    /// Is the data rate IEEE 802.11b.
    pub is_b: bool,
}

impl EncodedRate {
    #[inline]
    /// Returns the data rate in kbps.
    pub const fn rate_in_kbps(&self) -> usize {
        self.rate() as usize * 500
    }
    #[inline]
    /// Creates a rate from it's speed in kbps.
    pub const fn from_rate_in_kbps(rate: usize, is_b: bool) -> Self {
        Self::new().with_rate((rate / 500) as u8).with_is_b(is_b)
    }
}
#[cfg(feature = "alloc")]
impl ::alloc::fmt::Display for EncodedRate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "{}Mbit/s {}",
            self.rate() as f32 / 2f32,
            if self.is_b() { " (B)" } else { "" }
        ))
    }
}
