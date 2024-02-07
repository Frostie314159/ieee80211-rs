use macro_bits::{bit, bitfield, check_bit, serializable_enum};

serializable_enum! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// Spatial multiplexing power save mode.
    pub enum SmPwSave: u8 {
        #[default]
        Static => 0,
        Dynamic => 1,
        Reserved => 2,
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
bitfield! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// The MCS's supported by the transmitter.
    ///
    /// Condition | Tx MCS Set Defined | Tx Rx MCS Set Not Equal | Tx Maximum Number Spatial Streams Supported | Tx Unequal Modulation Supported
    /// -- | -- | -- | -- | --
    /// No Tx MCS set is defined | 0 | 0 | 0 | 0
    /// The Tx MCS set is defined to be equal to the Rx MCS set | 1 | 0 | 0 | 0
    /// The Tx MCS set may differ from the Rx MCS set | 1 | 1 | * | *
    pub struct SupportedMCSSet: u64 {
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
        pub implicit_transmit_beamforming_receiving_capable: bool => bit!(0),
        pub receive_staggered_sounding_capable: bool => bit!(1),
        pub transmit_staggered_sounding_capable: bool => bit!(2),
        pub receive_ndp_capable: bool => bit!(3),
        pub transmit_ndp_capable: bool => bit!(4),
        pub implicit_transmit_beamforming_capable: bool => bit!(5),
        pub calibration: BeamformingCalibration => bit!(6, 7),
        pub explicit_csi_transmit_beamforming_capable: bool => bit!(8),
        pub explicit_noncompressed_steering_capable: bool => bit!(9),
        pub explicit_compressed_steering_capable: bool => bit!(10),
        pub explicit_transmit_beamforming_csi_feedback: BeamformingFeedback => bit!(11, 12),
        pub explicit_noncompressed_beamforming_feedback_capable: BeamformingFeedback => bit!(13, 14),
        pub explicit_compressed_beamforming_feedback_capable: BeamformingFeedback => bit!(15, 16),
        pub minimal_grouping: GroupingCapability => bit!(17, 18),
        pub csi_number_of_beamformer_antennas_supported: u8 => bit!(19, 20),
        pub noncompresssed_steering_number_of_beamformer_antennas_supported: u8 => bit!(21, 22),
        pub compresssed_steering_number_of_beamformer_antennas_supported: u8 => bit!(23, 24),
        pub csi_max_number_of_rows_beamformer_supported: u8 => bit!(25, 26),
        pub channel_estimation_capability: u8 => bit!(27, 28),
        pub reserved: u8 => bit!(29, 30, 31)
    }
}
bitfield! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct ASELCapability: u8 {
        pub antenna_selection_capable: bool => bit!(0),
        pub explicit_csi_feedback_based_transmit_asel_capable: bool => bit!(1),
        pub antenna_indices_feedback_based_transmit_asel_capable: bool => bit!(2),
        pub explicit_csi_feedback_capable: bool => bit!(3),
        pub antenna_indices_feedback_capable: bool => bit!(4),
        pub receive_asel_capable: bool => bit!(5),
        pub transmit_sounding_ppdus_capable: bool => bit!(6),
        pub reserved: bool => bit!(7)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct HTCapabilitiesTLV {
    pub ht_capabilities_info: HTCapabilitiesInfo,
    pub ampdu_parameters: AMpduParameters,
    pub supported_rx_mcs_set: [u8; 10],
    pub supported_mcs_set: SupportedMCSSet,
    pub extended_capabilities: HTExtendedCapabilities,
    pub transmit_beamforming_capabilities: TransmitBeamformingCapabilities,
    pub asel_capabilities: ASELCapability,
}
impl HTCapabilitiesTLV {
    pub fn get_rx_mcs_iterator(&self) -> impl Iterator<Item = usize> + '_ {
        (0..80).filter(|index| check_bit!(self.supported_rx_mcs_set[index / 8], bit!(index % 8)))
    }
}
