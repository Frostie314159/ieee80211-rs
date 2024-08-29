//! This module contains support for the VHT Capabilities and Operation elements.

use core::fmt::Debug;

use bitfield_struct::bitfield;
use macro_bits::serializable_enum;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use super::{Element, ElementID};

#[bitfield(u32, defmt = cfg(feature = "defmt"))]
#[derive(PartialEq, Eq, Hash)]
/// General information about the capabilites of the STA's VHT PHY.
pub struct VHTCapabilitiesInfo {
    #[bits(2)]
    pub maximum_mpdu_length: u8,
    #[bits(2)]
    pub supported_channel_width_set: u8,
    pub rx_ldpc: bool,
    pub short_gi_80mhz: bool,
    pub short_gi_160mhz: bool,
    pub tx_stbc: bool,
    #[bits(3)]
    pub rx_stbc: u8,
    pub su_beamformer_capable: bool,
    pub su_beamformee_capable: bool,
    #[bits(3)]
    pub beamformee_sts_capability: u8,
    #[bits(3)]
    pub number_of_sounding_dimensions: u8,
    pub mu_beamformer_capable: bool,
    pub mu_beamformee_capable: bool,
    pub txop_ps: bool,
    pub htc_vht_capable: bool,
    #[bits(3)]
    pub maximum_ampdu_length_exponent: u8,
    #[bits(2)]
    pub vht_link_adaptation_capable: u8,
    pub rx_antenna_pattern_consistency: bool,
    pub tx_antenna_pattern_consistency: bool,
    #[bits(2)]
    pub extended_nss_bw_support: u8,
}
impl VHTCapabilitiesInfo {
    pub const fn maximum_mpdu_length_in_bytes(&self) -> Option<u16> {
        match self.maximum_mpdu_length() {
            0 => Some(3_895),
            1 => Some(7_991),
            2 => Some(11_454),
            _ => None,
        }
    }
    pub const fn with_maximum_mpdu_length_in_bytes(
        self,
        maximum_mpdu_length_in_bytes: u16,
    ) -> Self {
        self.with_maximum_mpdu_length(match maximum_mpdu_length_in_bytes {
            3_895 => 0,
            7_991 => 1,
            11_454 => 2,
            _ => 3,
        })
    }
    pub fn set_maximum_mpdu_length_in_bytes(&mut self, maximum_mpdu_length_in_bytes: u16) {
        *self = self.with_maximum_mpdu_length_in_bytes(maximum_mpdu_length_in_bytes);
    }
}
serializable_enum! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// The supported VHT MCS indices.
    pub enum VHTMCSSupport : u8 {
        ZeroToSeven => 0,
        ZeroToEight => 1,
        ZeroToNine => 2,
        #[default]
        NotSupported => 3
    }
}
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
/// The combinations of VHT-MCSs and spatial streams supported by the STA's VHT PHY.
pub struct VHTMCSMap(u16);
impl VHTMCSMap {
    pub const fn from_bits(bits: u16) -> Self {
        Self(bits)
    }
    pub const fn into_bits(self) -> u16 {
        self.0
    }
    /// Returns the supported VHT MCS range for the given number of spatial streams.
    pub fn vht_mcs_support_for_nss(&self, nss: usize) -> Option<VHTMCSSupport> {
        if (1..9).contains(&nss) {
            Some(VHTMCSSupport::from_bits(
                (self.0 >> ((nss - 1) * 2) & 0b0000_0011) as u8,
            ))
        } else {
            None
        }
    }
    /// Returns an [Iterator] over the VHT MCS ranges.
    pub fn vht_mcs_support_iter(&self) -> impl Iterator<Item = VHTMCSSupport> + '_ {
        (1..9).filter_map(|nss| self.vht_mcs_support_for_nss(nss))
    }
    /// Creates a VHTMCSAndNSSSet field from
    pub fn from_vht_mcs_iter(iter: impl IntoIterator<Item = VHTMCSSupport>) -> Self {
        Self(
            iter.into_iter()
                .take(8)
                .enumerate()
                .fold(0u16, |acc, (i, vht_mcs_support)| {
                    acc | (vht_mcs_support.into_bits() << (i * 2)) as u16
                }),
        )
    }
}
impl Debug for VHTMCSMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.vht_mcs_support_iter()).finish()
    }
}
#[bitfield(u64)]
#[derive(PartialEq, Eq, Hash)]
pub struct SupportedVHTMCSAndNSSSet {
    #[bits(16)]
    pub rx_vht_mcs_map: VHTMCSMap,
    #[bits(13)]
    pub rx_highest_supported_long_gi_data_rate: u16,
    #[bits(3)]
    pub maximum_nsts_total: u8,
    #[bits(16)]
    pub tx_vht_mcs_map: VHTMCSMap,
    #[bits(13)]
    pub tx_highest_supported_long_gi_data_rate: u16,
    pub vht_extended_nss_bw_capable: bool,
    #[bits(2)]
    pub __: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// The capabilities of the STA's VHT PHY.
pub struct VHTCapabilitiesElement {
    pub vht_capabilities_info: VHTCapabilitiesInfo,
    pub supported_vht_mcs_and_nss_set: SupportedVHTMCSAndNSSSet,
}
impl TryFromCtx<'_> for VHTCapabilitiesElement {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'_ [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;
        let vht_capabilities_info =
            VHTCapabilitiesInfo::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        let supported_vht_mcs_and_nss_set =
            SupportedVHTMCSAndNSSSet::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        Ok((
            Self {
                vht_capabilities_info,
                supported_vht_mcs_and_nss_set,
            },
            offset,
        ))
    }
}
impl MeasureWith<()> for VHTCapabilitiesElement {
    fn measure_with(&self, _ctx: &()) -> usize {
        12
    }
}
impl TryIntoCtx for VHTCapabilitiesElement {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;
        buf.gwrite_with(
            self.vht_capabilities_info.into_bits(),
            &mut offset,
            Endian::Little,
        )?;
        buf.gwrite_with(
            self.supported_vht_mcs_and_nss_set.into_bits(),
            &mut offset,
            Endian::Little,
        )?;
        Ok(offset)
    }
}
impl Element for VHTCapabilitiesElement {
    const ELEMENT_ID: ElementID = ElementID::Id(0xbf);
    type ReadType<'a> = VHTCapabilitiesElement;
}
serializable_enum! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum ChannelWidth : u8 {
        TwentyOrFourtyMHz => 0,
        EightyOneSixtyOrEightyPlusEightyMhz => 1,
        OneSixtyMHz => 2,
        NonContiguousEightyPlusEightyMHz => 3
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// The current VHT operation characteristics.
pub struct VHTOperationElement {
    pub channel_bandwidth: ChannelWidth,
    pub channel_center_frequency_segment_0: u8,
    pub channel_center_frequency_segment_1: u8,
    pub basic_vht_mcs_and_nss_set: VHTMCSMap,
}
impl TryFromCtx<'_> for VHTOperationElement {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'_ [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let channel_bandwidth = ChannelWidth::from_bits(from.gread(&mut offset)?);
        let channel_center_frequency_segment_0 = from.gread(&mut offset)?;
        let channel_center_frequency_segment_1 = from.gread(&mut offset)?;
        let basic_vht_mcs_and_nss_set =
            VHTMCSMap::from_bits(from.gread_with(&mut offset, Endian::Little)?);

        Ok((
            Self {
                channel_bandwidth,
                channel_center_frequency_segment_0,
                channel_center_frequency_segment_1,
                basic_vht_mcs_and_nss_set,
            },
            offset,
        ))
    }
}
impl MeasureWith<()> for VHTOperationElement {
    fn measure_with(&self, _ctx: &()) -> usize {
        5
    }
}
impl TryIntoCtx for VHTOperationElement {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.channel_bandwidth.into_bits(), &mut offset)?;
        buf.gwrite(self.channel_center_frequency_segment_0, &mut offset)?;
        buf.gwrite(self.channel_center_frequency_segment_1, &mut offset)?;
        buf.gwrite_with(
            self.basic_vht_mcs_and_nss_set.into_bits(),
            &mut offset,
            Endian::Little,
        )?;

        Ok(offset)
    }
}
impl Element for VHTOperationElement {
    const ELEMENT_ID: ElementID = ElementID::Id(0xc0);
    type ReadType<'a> = VHTOperationElement;
}
