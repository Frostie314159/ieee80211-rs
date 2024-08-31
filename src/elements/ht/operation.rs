use bitfield_struct::bitfield;
use macro_bits::serializable_enum;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::elements::{Element, ElementID};

use super::SupportedMCSSet;

serializable_enum! {
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// The offset of the secondary channel from the primary.
    ///
    /// When using the `iw` command the mapping is as follows.
    ///
    /// Variant | iw channel spec
    /// -- | --
    /// [NotPresent](SecondaryChannelOffset::NotPresent) | HT20
    /// [Above](SecondaryChannelOffset::Above) | HT40+
    /// [Below](SecondaryChannelOffset::Below) | HT40-
    pub enum SecondaryChannelOffset: u8 {
        #[default]
        /// No secondary channel is present.
        NotPresent => 0x00,
        /// Secondary channel is above.
        Above => 0x01,
        Reserved => 0x02,
        /// Secondary channel is below.
        Below => 0x03
    }
}

serializable_enum! {
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub enum HTProtectionMode: u8 {
        #[default]
        /// No protection mode.
        None => 0x00,
        /// Nonmember protection mode.
        NonMember => 0x01,
        /// 20MHz protection mode.
        TwentyMHz => 0x02,
        /// Non-HT mixed mode.
        NonHTMixed => 0x03
    }
}

#[bitfield(u64, defmt = cfg(feature = "defmt"))]
#[derive(PartialEq, Eq, Hash)]
/// Information about the operation of an HT-STA.
pub struct HTOperationInformation {
    #[bits(2)]
    /// Offset of the secondary channel from the primary channel.
    pub secondary_channel_offset: SecondaryChannelOffset,
    /// Indicates if channel widths other than 20MHz are allowed.
    pub any_channel_width: bool,
    /// Indicates wether the use of reduced interframe space is permitted within the BSS.
    pub rifs_permitted: bool,
    #[bits(4)]
    __: u8,
    #[bits(2)]
    /// Indicates the protection requirements of HT transmissions.
    pub ht_protection_mode: HTProtectionMode,
    /// Indicates if any HT-STAs, which are not HT-greenfield capable, are associated with the BSS.
    pub nongreenfield_ht_sta_present: bool,
    __: bool,
    /// Indicates if the use of protection for non-HT STAs by overlapping BSS is determined to be desirable.
    pub obss_non_ht_sta_present: bool,
    #[bits(8)]
    /// Defines the channel center frequency for a 160 or 80+80MHz BSS bandwidth with NSS support less than Max VHT NSS.
    pub channel_center_frequency_segment_2: u8,
    #[bits(9)]
    __: u16,
    /// Indicates wether the AP transmits an STBC beacon.
    pub dual_beacon: bool,
    /// Indicates if dual CTS protection is required.
    pub dual_cts_protection: bool,
    /// Indicates wether the beacon containing this element is a primary or an STBC beacon.
    pub stbc_beacon: bool,

    #[bits(31)]
    __: u64,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// The operation of HT STAs in the BSS is controlled by the HT Operation element.
pub struct HTOperationElement {
    /// Indicates the channel number of the primary channel.
    pub primary_channel: u8,
    /// Information about the operation of the HT-STA.
    pub ht_operation_information: HTOperationInformation,
    /// Indicates the HT-MCS values that are supported by the HT-STA.
    pub basic_ht_mcs_set: SupportedMCSSet,
}
impl MeasureWith<()> for HTOperationElement {
    fn measure_with(&self, _ctx: &()) -> usize {
        22
    }
}
impl TryFromCtx<'_> for HTOperationElement {
    type Error = scroll::Error;
    fn try_from_ctx(from: &[u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let primary_channel = from.gread(&mut offset)?;
        let mut ht_operation_information = [0u8; 8];
        ht_operation_information[..5].copy_from_slice(from.gread_with(&mut offset, 5)?);
        let ht_operation_information =
            HTOperationInformation::from_bits(u64::from_le_bytes(ht_operation_information));
        let basic_ht_mcs_set = from.gread(&mut offset)?;

        Ok((
            Self {
                primary_channel,
                ht_operation_information,
                basic_ht_mcs_set,
            },
            offset,
        ))
    }
}
impl TryIntoCtx for HTOperationElement {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.primary_channel, &mut offset)?;
        buf.gwrite(
            &self.ht_operation_information.into_bits().to_be_bytes()[..5],
            &mut offset,
        )?;
        buf.gwrite(self.basic_ht_mcs_set, &mut offset)?;

        Ok(offset)
    }
}
impl Element for HTOperationElement {
    const ELEMENT_ID: ElementID = ElementID::Id(0x3d);
    type ReadType<'a> = Self;
}
