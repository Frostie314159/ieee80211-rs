use macro_bits::serializable_enum;

serializable_enum! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// An IEEE 802.11 status code used in certain management frames.
    pub enum IEEE80211StatusCode: u16 {
        #[default]
        Success => 0,
        RefusedReasonUnspec => 1,
        TDLSRejectedAlternativeProvided => 2,
        TDLSRejected => 3,
        SecurityDisabled => 5,
        UnacceptableLifetime => 6,
        NotInSameBSS => 7,
        RefusedCapabilitiesMismatch => 10,
        DeniedNoAssociationExists => 11,
        DeniedOtherReason => 12
    }
}
