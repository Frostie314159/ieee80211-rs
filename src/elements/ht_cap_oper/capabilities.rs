use macro_bits::{bit, bitfield, serializable_enum};
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use super::SupportedMCSSet;

serializable_enum! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// Spatial multiplexing power save mode.
    pub enum SmPwSave: u8 {
        Static => 0,
        Dynamic => 1,
        Reserved => 2,
        #[default]
        Disabled => 3
    }
}
bitfield! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// HT related capabilities info.
    pub struct HTCapabilitiesInfo: u16 {
        /// Indicates supported for receiving LDPC coded packets.
        pub ldpc_coding_capability: bool => bit!(0),
        /// Indicates the channel widths supported by the STA.
        ///
        /// State | Meaning
        /// -- | --
        /// `false` | Only 20MHz operation supported
        /// `true` | 20MHz and 40MHz operation supported
        pub supported_channel_width_set: bool => bit!(1),
        /// Indicates the spatial multiplexing power save mode.
        pub sm_power_save: SmPwSave => bit!(2,3),
        /// Indicates support for the reception of PPDUs with HT-greenfield format.
        pub green_field: bool => bit!(4),
        /// Indicates short GI support for the reception of packets with 20 MHz bandwidth.
        pub short_gi_20mhz: bool => bit!(5),
        /// Indicates short GI support for the reception of packets with 40 MHz bandwidth.
        pub short_gi_40mhz: bool => bit!(6),
        /// Indicates support for the transmission of PPDUs using STBC.
        pub tx_stbc: bool => bit!(7),
        /// Indicates the amount of spatial streams, with which PPDUs using STBC, can be received.
        /// Valid values are 0-3, with zero indicating lack of support.
        pub rx_stbc: u8 => bit!(8, 9),
        /// Indicates support for delayed block ack operation.
        pub delayed_block_ack: bool => bit!(10),
        /// Indicates support for 7935 octets of maximum A-MSDU length.
        ///
        /// State | Max A-MSDU length
        /// -- | --
        /// `false` | 3839 octets
        /// `true` | 7935 octets
        pub is_max_amsdu_large: bool => bit!(11),
        /// Indicates the use of DSSS/CCK mode in a 20/40MHz BSS.
        pub dsss_40mhz: bool => bit!(12),
        pub reserved: bool => bit!(13),
        /// Indicates wether APs receiving this should prohibit 40MHz operation.
        pub forty_mhz_intolerant: bool => bit!(14),
        /// Indicates support for the L-SIG TXOP protection mechanism.
        pub l_sig_txop_protection_support: bool => bit!(15)
    }
}

serializable_enum! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// The maximum length of an A-MPDU.
    pub enum MAXAMpduLength: u8 {
        /// 8kb
        #[default]
        Small => 0,
        /// 16kb
        Medium => 1,
        /// 32kb
        Large => 2,
        /// 64kb
        VeryLarge => 3
    }
}

serializable_enum! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// Minimum time between the start of adjacent MPDUs in A-MPDU, measured in µs.
    pub enum MpduDensity: u8 {
        #[default]
        NoRestriction => 0,
        Quarter => 1,
        Half => 2,
        One => 3,
        Two => 4,
        Four => 5,
        Eight => 6,
        Sixteen => 7
    }
}

bitfield! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// Parameters for A-MPDU operation.
    pub struct AMpduParameters: u8 {
        /// Indicates the maximum length of an A-MPDU that the STA can receive.
        ///
        /// This is commonly encoded as an exponent.
        pub max_a_mpdu_length: MAXAMpduLength => bit!(0,1),
        /// Determines the minimum time between the start of adjacent MPDUs in an A-MPDU that the STA can receive.
        pub mpdu_density: MpduDensity => bit!(2,3,4),
        pub reserved: u8 => bit!(5, 6, 7)
    }
}
serializable_enum! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// Indicates the time required to switch from 20MHz to 40MHz operation.
    pub enum PCOTransitionTime: u8 {
        #[default]
        NoTransition => 0,
        FourHundredMicroSeconds => 1,
        OnePointFiveMilliSeconds => 2,
        FiveMilliSeconds => 3
    }
}
bitfield! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// Extended HT capabilities
    pub struct HTExtendedCapabilities: u16 {
        /// Indicates support for **P**hased **C**oexistence **O**peration(PCO)
        pub pco_supported: bool => bit!(0),
        /// Indicates the time required to switch from 20MHz to 40MHz operation.
        pub pco_transition_time: PCOTransitionTime => bit!(1, 2),
        pub reserved: u8 => bit!(3, 4, 5, 6, 7),
        /// Indicates wether the STA can provide **M**CS **F**eed**B**ack.
        pub mcs_feedback: u8 => bit!(8, 9),
        /// Indicates support of the HT Control field
        pub plus_htc_support: bool => bit!(10),
        /// Indicates support for the **R**everse **D**irection Protocol responder role.
        pub rd_responder: bool => bit!(11),
        pub reserved_2: u8 => bit!(12, 13, 14, 15)
    }
}
serializable_enum! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// The level of support for beamforming calibration.
    pub enum BeamformingCalibration: u8 {
        #[default]
        NotSupported => 0,
        CanRespondButCantInitiate => 1,
        Reserved => 2,
        CanRespondAndInitiate => 3
    }
}
serializable_enum! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// The level of support for beamforming feedback.
    pub enum BeamformingFeedback: u8 {
        #[default]
        NotSupported => 0,
        DelayedFeedback => 1,
        ImmediateFeedback => 2,
        DelayedAndImmediateFeedback => 3
    }
}
serializable_enum! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// The level of support for grouping.
    pub enum GroupingCapability: u8 {
        #[default]
        NoGrouping => 0,
        GroupsOfTwo => 1,
        GroupsOfFour => 2,
        GroupsOfTwoAndFour => 3
    }
}
bitfield! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// Capabilities related to transmit beamforming.
    pub struct TransmitBeamformingCapabilities: u32 {
        /// Indicates whether this STA can receive Transmit Beamforming steered frames using implicit feedback.
        pub implicit_transmit_beamforming_receiving_capable: bool => bit!(0),
        /// Indicates whether this STA can receive staggered sounding frames.
        pub receive_staggered_sounding_capable: bool => bit!(1),
        /// Indicates whether this STA can transmit staggered sounding frames.
        pub transmit_staggered_sounding_capable: bool => bit!(2),
        /// Indicates whether this receiver can interpret null data PPDUs as sounding frames.
        pub receive_ndp_capable: bool => bit!(3),
        /// Indicates whether this STA can transmit null data PPDUs as sounding frames.
        pub transmit_ndp_capable: bool => bit!(4),
        /// Indicates whether this STA can apply implicit transmit beamforming.
        pub implicit_transmit_beamforming_capable: bool => bit!(5),
        /// Indicates whether the STA can participate in a calibration procedure initiated by another STA
        /// that is capable of generating an immediate response sounding PPDU and can provide a CSI report in response to a sounding PPDU.
        pub calibration: BeamformingCalibration => bit!(6, 7),
        /// Indicates whether this STA can apply transmit beamforming using CSI explicit feedback in its transmission.
        pub explicit_csi_transmit_beamforming_capable: bool => bit!(8),
        /// Indicates whether this STA can apply transmit beamforming
        /// using noncompressed beamforming feedback matrix explicit feedback in its transmission.
        pub explicit_noncompressed_steering_capable: bool => bit!(9),
        /// Indicates whether this STA can apply transmit beamforming
        /// using compressed beamforming feedback matrix explicit feedback in its transmission.
        pub explicit_compressed_steering_capable: bool => bit!(10),
        /// Indicates whether this receiver can return CSI explicit feedback.
        pub explicit_transmit_beamforming_csi_feedback: BeamformingFeedback => bit!(11, 12),
        /// Indicates whether this receiver can return noncompressed beamforming feedback matrix explicit feedback.
        pub explicit_noncompressed_beamforming_feedback_capable: BeamformingFeedback => bit!(13, 14),
        /// Indicates whether this receiver can return compressed beamforming feedback matrix explicit feedback.
        pub explicit_compressed_beamforming_feedback_capable: BeamformingFeedback => bit!(15, 16),
        /// Indicates the minimal grouping used for explicit feedback reports.
        pub minimal_grouping: GroupingCapability => bit!(17, 18),
        /// Indicates the maximum number of beamformer antennas the HT beamformee can support when CSI feedback is required.
        pub csi_number_of_beamformer_antennas_supported: u8 => bit!(19, 20),
        /// Indicates the maximum number of beamformer antennas the HT beamformee can support
        /// when noncompressed beamforming feedback matrix is required.
        pub noncompresssed_steering_number_of_beamformer_antennas_supported: u8 => bit!(21, 22),
        /// Indicates the maximum number of beamformer antennas the HT beamformee can support
        /// when compressed beamforming feedback matrix is required.
        pub compresssed_steering_number_of_beamformer_antennas_supported: u8 => bit!(23, 24),
        /// Indicates the maximum number of rows of CSI explicit feedback from the HT beamformee
        /// or calibration responder or transmit ASEL responder that an HT beamformer or calibration initiator
        /// or transmit ASEL initiator can support when CSI feedback is required.
        pub csi_max_number_of_rows_beamformer_supported: u8 => bit!(25, 26),
        /// Indicates the maximum number of space-time streams (columns of the MIMO channel matrix)
        /// for which channel dimensions can be simultaneously estimated when receiving an NDP sounding PPDU
        /// or the extension portion of the HT Long Training fields (HT-LTFs) in a staggered sounding PPDU.
        pub channel_estimation_capability: u8 => bit!(27, 28),
        pub reserved: u8 => bit!(29, 30, 31)
    }
}
bitfield! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// The Antenna Selection capability of the STA.
    pub struct ASELCapability: u8 {
        /// Indicates whether this STA supports ASEL.
        pub antenna_selection_capable: bool => bit!(0),
        /// Indicates whether this STA supports transmit ASEL based on explicit CSI feedback.
        pub explicit_csi_feedback_based_transmit_asel_capable: bool => bit!(1),
        /// Indicates whether this STA supports transmit ASEL based on antenna indices feedback.
        pub antenna_indices_feedback_based_transmit_asel_capable: bool => bit!(2),
        /// Indicates whether this STA can compute CSI and provide CSI feedback in support of ASEL.
        pub explicit_csi_feedback_capable: bool => bit!(3),
        /// Indicates whether this STA can compute an antenna indices selection and return an antenna indices selection in support of ASEL.
        pub antenna_indices_feedback_capable: bool => bit!(4),
        /// Indicates whether this STA supports receive ASEL.
        pub receive_asel_capable: bool => bit!(5),
        /// Indicates whether this STA can transmit sounding PPDUs for ASEL training on request.
        pub transmit_sounding_ppdus_capable: bool => bit!(6),
        pub reserved: bool => bit!(7)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// The [HTCapabilitiesElement] contains information about the HT capbilities of the STA.
pub struct HTCapabilitiesElement {
    pub ht_capabilities_info: HTCapabilitiesInfo,
    pub ampdu_parameters: AMpduParameters,
    pub supported_mcs_set: SupportedMCSSet,
    pub extended_capabilities: HTExtendedCapabilities,
    pub transmit_beamforming_capabilities: TransmitBeamformingCapabilities,
    pub asel_capability: ASELCapability,
}
impl MeasureWith<()> for HTCapabilitiesElement {
    fn measure_with(&self, _ctx: &()) -> usize {
        26
    }
}
impl TryFromCtx<'_> for HTCapabilitiesElement {
    type Error = scroll::Error;
    fn try_from_ctx(from: &[u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let ht_capabilities_info =
            HTCapabilitiesInfo::from_representation(from.gread_with(&mut offset, Endian::Little)?);
        let ampdu_parameters =
            AMpduParameters::from_representation(from.gread_with(&mut offset, Endian::Little)?);
        let supported_mcs_set = from.gread(&mut offset)?;
        let extended_capabilities = HTExtendedCapabilities::from_representation(
            from.gread_with(&mut offset, Endian::Little)?,
        );
        let transmit_beamforming_capabilities =
            TransmitBeamformingCapabilities::from_representation(
                from.gread_with(&mut offset, Endian::Little)?,
            );
        let asel_capability =
            ASELCapability::from_representation(from.gread_with(&mut offset, Endian::Little)?);

        Ok((
            Self {
                ht_capabilities_info,
                ampdu_parameters,
                supported_mcs_set,
                extended_capabilities,
                transmit_beamforming_capabilities,
                asel_capability,
            },
            offset,
        ))
    }
}
impl TryIntoCtx for HTCapabilitiesElement {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(
            self.ht_capabilities_info.to_representation(),
            &mut offset,
            Endian::Little,
        )?;
        buf.gwrite_with(
            self.ampdu_parameters.to_representation(),
            &mut offset,
            Endian::Little,
        )?;
        buf.gwrite(self.supported_mcs_set, &mut offset)?;
        buf.gwrite_with(
            self.extended_capabilities.to_representation(),
            &mut offset,
            Endian::Little,
        )?;
        buf.gwrite_with(
            self.transmit_beamforming_capabilities.to_representation(),
            &mut offset,
            Endian::Little,
        )?;
        buf.gwrite_with(
            self.asel_capability.to_representation(),
            &mut offset,
            Endian::Little,
        )?;

        Ok(offset)
    }
}
