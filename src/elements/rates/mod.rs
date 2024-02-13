mod supported_rates;

use macro_bits::{bit, bitfield};
pub use supported_rates::*;

mod extended_supported_rates;
pub use extended_supported_rates::*;

mod rate_iter;
pub use rate_iter::RatesReadIterator;

bitfield! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// Data rate encoded as specified in IEEE 802.11.
    pub struct EncodedRate: u8 {
        /// The value of the data rate.
        ///
        /// The formular is `rate * 500` to get kbps. Use [EncodedRate::rate_in_kbps] to calculate this.
        pub rate: u8 => bit!(0, 1, 2, 3, 4, 5, 6),
        /// Is the data rate IEEE 802.11b.
        pub is_b: bool => bit!(7)
    }
}
impl EncodedRate {
    /// Returns the data rate in kbps.
    pub const fn rate_in_kbps(&self) -> usize {
        self.rate as usize * 500
    }
    /// Creates a rate from it's speed in kbps.
    pub const fn from_rate_in_kbps(rate: usize, is_b: bool) -> Self {
        Self {
            rate: (rate / 500) as u8,
            is_b,
        }
    }
}
#[cfg(feature = "std")]
impl ::std::fmt::Display for EncodedRate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "{}Mbit/s {}",
            self.rate as f32 / 2f32,
            if self.is_b { " (B)" } else { "" }
        ))
    }
}
