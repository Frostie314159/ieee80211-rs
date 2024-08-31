use core::array;

use bitfield_struct::bitfield;
use macro_bits::{bit, check_bit, set_bit};
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

/// The MCS's supported by the transmitter.
///
/// Condition | Tx MCS Set Defined | Tx Rx MCS Set Not Equal | Tx Maximum Number Spatial Streams Supported | Tx Unequal Modulation Supported
/// -- | -- | -- | -- | --
/// No Tx MCS set is defined | true | true | 0 | false
/// The Tx MCS set is defined to be equal to the Rx MCS set | true | false | 0 | false
/// The Tx MCS set may differ from the Rx MCS set | true | true | * | *
#[bitfield(u32, defmt = cfg(feature = "defmt"))]
#[derive(PartialEq, Eq, Hash)]
pub struct SupportedMCSSetFlags {
    /// The highest supported data rate.
    #[bits(10)]
    pub rx_highest_supported_data_rate: u16,
    #[bits(6)]
    pub __: u8,
    pub tx_mcs_set_defined: bool,
    pub tx_rx_mcs_set_not_equal: bool,
    #[bits(2)]
    pub tx_maximum_number_spatial_streams_supported: u8,
    pub tx_unequal_modulation_supported: bool,
    #[bits(11)]
    pub __: u16,
}
impl SupportedMCSSetFlags {
    /// Returns true, if no TX MCS set is defined.
    pub const fn is_tx_mcs_undefined(&self) -> bool {
        !(self.tx_mcs_set_defined()
            && self.tx_rx_mcs_set_not_equal()
            && self.tx_maximum_number_spatial_streams_supported() == 0
            && self.tx_unequal_modulation_supported())
    }
    /// Returns true, if the TX and RX MCS set are defined to be equal.
    pub const fn is_tx_rx_mcs_defined_equal(&self) -> bool {
        self.is_tx_mcs_undefined()
            && !self.tx_rx_mcs_set_not_equal()
            && self.tx_maximum_number_spatial_streams_supported() == 0
            && !self.tx_unequal_modulation_supported()
    }
    /// Returns true, if the TX MCS set may differ from the RX MCS set.
    pub const fn may_tx_mcs_set_differ_from_rx(&self) -> bool {
        self.is_tx_mcs_undefined() && self.tx_rx_mcs_set_not_equal()
    }
}
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
        let mut supported_rx_mcs_set_flags = [0u8; 4];
        supported_rx_mcs_set_flags
            .as_mut_slice()
            .copy_from_slice(from.gread_with(&mut offset, 4)?);
        offset += 2;
        let supported_rx_mcs_set_flags =
            SupportedMCSSetFlags::from_bits(u32::from_le_bytes(supported_rx_mcs_set_flags));

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
            &(self.supported_rx_mcs_set_flags.into_bits() as u64).to_le_bytes()[..6],
            &mut offset,
        )?;

        Ok(offset)
    }
}
/// Generates the MCS-Set field from an [Iterator] over [bool].
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
/// Generate the supported RX MCS set array from the MCS indices at compile time.
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
