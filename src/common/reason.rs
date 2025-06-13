use macro_bits::serializable_enum;

serializable_enum! {
    #[non_exhaustive]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// An IEEE 802.11 reason code used in certain management frames.
    pub enum IEEE80211Reason: u16 {
        #[default]
        Unspecified => 1,
        InvalidAuthentication => 2,
        LeavingNetworkDeauth => 3,
        Inactivity => 4,
        NoMoreSTAs => 5,
        InvalidClass2Frame => 6,
        InvalidClass3Frame => 7,
        LeavingNetworkDisassoc => 8,

        MeshPeeringCancelled => 52,
        MeshMaxPeers => 53,
        MeshConfigurationPolicyViolation => 54,
        MeshCloseRcvd => 55,
        MeshMaxRetries => 56,
        MeshConfirmTimeout => 57,
        MeshInvalidGTK => 58,
        MeshInconsistentParameters => 59,
        MeshInvalidSecurityCapability => 60,
        MeshPathErrorNoProxyInformation => 61,
        MeshPathErrorNoForwardingInformation => 62,
        MeshPathErrorDestinationUnreachable => 63,
        MacAddressAlreadyExistsInMBSS => 64,
        MeshChannelSwitchRegulatoryRequirements => 65,
        MeshChannelSwitchUnspecified => 66,
        TransmissionLinkEstablishmentFailed => 67,
        AlternativeChannelOccupied => 68
    }
}
