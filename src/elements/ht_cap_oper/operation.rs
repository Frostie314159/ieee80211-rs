use macro_bits::{bit, bitfield, serializable_enum};
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use super::SupportedMCSSet;

serializable_enum! {
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

bitfield! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// Information about the operation of an HT-STA.
    pub struct HTOperationInformation: u64 {
        /// Offset of the secondary channel from the primary channel.
        pub secondary_channel_offset: SecondaryChannelOffset => bit!(0, 1),
        /// Indicates if channel widths other than 20MHz are allowed.
        pub any_channel_width: bool => bit!(2),
        /// Indicates wether the use of reduced interframe space is permitted within the BSS.
        pub rifs_permitted: bool => bit!(3),
        /// Indicates the protection requirements of HT transmissions.
        pub ht_protection_mode: HTProtectionMode => bit!(8, 9),
        /// Indicates if any HT-STAs, which are not HT-greenfield capable, are associated with the BSS.
        pub nongreenfield_ht_sta_present: bool => bit!(10),
        /// Indicates if the use of protection for non-HT STAs by overlapping BSS is determined to be desirable.
        pub obss_non_ht_sta_present: bool => bit!(12),
        /// Defines the channel center frequency for a 160 or 80+80MHz BSS bandwidth with NSS support less than Max VHT NSS.
        pub channel_center_frequency_segment_2: u8 => bit!(13, 14, 15, 16, 17, 18, 19, 20),
        /// Indicates wether the AP transmits an STBC beacon.
        pub dual_beacon: bool => bit!(30),
        /// Indicates if dual CTS protection is required.
        pub dual_cts_protection: bool => bit!(31),
        /// Indicates wether the beacon containing this element is a primary or an STBC beacon.
        pub stbc_beacon: bool => bit!(32)
    }
}

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
        let ht_operation_information = HTOperationInformation::from_representation(
            u64::from_le_bytes(ht_operation_information),
        );
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
            &self
                .ht_operation_information
                .to_representation()
                .to_be_bytes()[..5],
            &mut offset,
        )?;
        buf.gwrite(self.basic_ht_mcs_set, &mut offset)?;

        Ok(offset)
    }
}
