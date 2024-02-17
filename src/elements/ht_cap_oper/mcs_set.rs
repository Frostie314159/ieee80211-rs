use core::array;

use macro_bits::{bit, bitfield, check_bit, set_bit};
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

bitfield! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// The MCS's supported by the transmitter.
    ///
    /// Condition | Tx MCS Set Defined | Tx Rx MCS Set Not Equal | Tx Maximum Number Spatial Streams Supported | Tx Unequal Modulation Supported
    /// -- | -- | -- | -- | --
    /// No Tx MCS set is defined | 0 | 0 | 0 | 0
    /// The Tx MCS set is defined to be equal to the Rx MCS set | 1 | 0 | 0 | 0
    /// The Tx MCS set may differ from the Rx MCS set | 1 | 1 | * | *
    pub struct SupportedMCSSetFlags: u64 {
        /// The highest supported data rate.
        pub rx_highest_supported_data_rate: u16 => bit!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9),
        pub reserved: u8 => bit!(10, 11, 12, 13, 14, 15),
        pub tx_mcs_set_defined: bool => bit!(16),
        pub tx_rx_mcs_set_not_equal: bool => bit!(17),
        pub tx_maximum_number_spatial_streams_supported: u8 => bit!(18, 19),
        pub tx_unequal_modulation_supported: bool => bit!(20),
        pub reserved_2: u16 => bit!(21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// The MCS Set supported by the transmitter.
///
/// The `supported_rx_mcs_set` field is actually a bitmask, which can be generated either through the [crate::supported_rx_mcs_set] macro
/// or the [generate_supported_rx_mcs_set] function.
pub struct SupportedMCSSet {
    pub supported_rx_mcs_set: [u8; 10],
    pub supported_rx_mcs_set_flags: SupportedMCSSetFlags,
}
impl SupportedMCSSet {
    pub fn supported_rx_mcs_indices(&self) -> impl Iterator<Item = bool> + '_ {
        self.supported_rx_mcs_set
            .into_iter()
            .flat_map(|byte| array::from_fn::<bool, 8, _>(|i| check_bit!(byte, bit!(i))))
    }
}
impl MeasureWith<()> for SupportedMCSSet {
    fn measure_with(&self, _ctx: &()) -> usize {
        16
    }
}
impl TryFromCtx<'_> for SupportedMCSSet {
    type Error = scroll::Error;
    fn try_from_ctx(from: &[u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let supported_rx_mcs_set = from.gread(&mut offset)?;
        let mut supported_rx_mcs_set_flags = [0u8; 8];
        supported_rx_mcs_set_flags[..6].copy_from_slice(from.gread_with(&mut offset, 6)?);
        let supported_rx_mcs_set_flags =
            SupportedMCSSetFlags::from_bits(u64::from_le_bytes(supported_rx_mcs_set_flags));

        Ok((
            Self {
                supported_rx_mcs_set,
                supported_rx_mcs_set_flags,
            },
            offset,
        ))
    }
}
impl TryIntoCtx for SupportedMCSSet {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.supported_rx_mcs_set, &mut offset)?;
        buf.gwrite(
            &self.supported_rx_mcs_set_flags.into_bits().to_le_bytes()[..6],
            &mut offset,
        )?;

        Ok(offset)
    }
}
pub fn generate_supported_rx_mcs_set<I: IntoIterator<Item = bool>>(mcs_indices: I) -> [u8; 10] {
    let mut supported_rx_mcs_set = [0; 10];

    for (mcs_index, supported) in mcs_indices.into_iter().take(76).enumerate() {
        set_bit!(
            supported_rx_mcs_set[mcs_index >> 3],
            bit!(mcs_index & 0b0000_0111),
            supported
        );
    }

    supported_rx_mcs_set
}
#[macro_export]
/// Generate the supported rx MCS set array from the MCS indices at compile time.
///
/// This macro also validates, that there are no duplicates and that the MCS indices are in the range of 0..77.
/// It supports either individual listing of indices or an exclusive range.
/// ```
/// use ieee80211::supported_rx_mcs_set;
///
/// let listing = supported_rx_mcs_set![0,1,2,3,4,5];
/// let range = supported_rx_mcs_set!(0=>6);
/// assert_eq!(listing, range);
/// ```
macro_rules! supported_rx_mcs_set {
    ($(
        $supported_rx_mcs_index:expr
    ),*) => {
        {
            const RESULT: [u8; 10] = {
                let mut buf = [0x00; 10];

                $(
                    {
                        use ::ieee80211::macro_bits::{bit, check_bit, set_bit};
                        assert!($supported_rx_mcs_index >= 0, "MCS indices lower zero are invalid.");
                        assert!($supported_rx_mcs_index < 77, "MCS indices greater than 76 are invalid.");

                        let (byte, bit) = ($supported_rx_mcs_index >> 3, $supported_rx_mcs_index & 0b0000_0111);
                        assert!(!check_bit!(buf[byte], bit!(bit)), concat!("MCS index: ", concat!($supported_rx_mcs_index, " is a duplicate.")));
                        set_bit!(buf[byte], bit!(bit));
                    }
                )*

                buf
            };
            RESULT
        }
    };
    ($start:expr=>$end:expr) => {
        {
            const RESULT: [u8; 10] = {
                use ::ieee80211::macro_bits::{bit, check_bit, set_bit};
                assert!($start >= 0, "MCS indices lower zero are invalid.");
                assert!($end < 77, "MCS indices greater than 76 are invalid.");

                let mut buf = [0x00; 10];

                let mut i = $start;

                while i < $end {
                    set_bit!(buf[i >> 3], bit!(i & 0b0000_0111));
                    i += 1;
                }

                buf
            };
            RESULT
        }
    };
}
