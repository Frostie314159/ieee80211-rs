use macro_bits::serializable_enum;

serializable_enum! {
    #[non_exhaustive]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// An IEEE 802.11 status code used in certain management frames.
    pub enum IEEE80211StatusCode: u16 {
        #[default]
        Success => 0,
        RefusedReasonUnspec => 1,
        TdlsRejectedAlternativeProvided => 2,
        TdlsRejected => 3,
        SecurityDisabled => 5,
        UnacceptableLifetime => 6,
        NotInSameBss => 7,
        RefusedCapabilitiesMismatch => 10,
        DeniedNoAssociationExists => 11,
        DeniedOtherReason => 12,
        UnsupportedAuthAlgorithm => 13,
        TransactionSequenceError => 14,
        ChallengeFailure => 15,
        RejectedSequenceTimeout => 16,
        DeniedNoMoreStas => 17,
        RefusedBasicRatesMismatch => 18,
        DeniedNoShortPreambleSupport => 19,
        RejectedSpectrumManagementRequired => 22,
        RejectedBadPowerCapability => 23,
        RejectedBadSupportedChannels => 24,
        DeniedNoShortSlotTimeSupport => 25,
        DeniedNoHtSupport => 27,
        R0khUnreachable => 28,
        RefusedTemporarily => 30,
        RobustManagementPolicyViolation => 31,
        UnspecifiedQosFailure => 32,
        DeniedInsufficientBandwidth => 33,
        DeniedPoorChannelConditions => 34,
        DeniedQosNotSupported => 35,
        RequestDeclined => 37,
        InvalidParameters => 38,
        RejectedWithSuggestedChanges => 39,
        StatusInvalidElement => 40,
        StatusInvalidGroupCipher => 41,
        StatusInvalidPairwiseCipher => 42,
        StatusInvalidAkmp => 43,
        UnsupportedRsneVersion => 44,
        InvalidRsneCapabilities => 45,
        StatusCipherOutOfPolicy => 46,
        RejectedForDelayPeriod => 47,
        NotPresent => 49,
        NotQosSta => 50,
        DeniedListenIntervalTooLarge => 51,
        StatusInvalidFtActionFrameCount => 52,
        StatusInvalidPmkid => 53,
        StatusInvalidMde => 54,
        StatusInvalidFte => 55,
        RequestedTclasNotSupported => 56,
        InsufficientTclasProcessingResources => 57,
        TryAnotherBss => 58,
        GasAdvertisementProtocolNotSupported => 59,
        NoOutstandingGasRequest => 60,
        GasResponseNotReceivedFrom => 61,
        GasQueryTimeout => 62,
        GasQueryResponseToo => 63,
        RejectedHomeWithSuggestedChanges => 64,
        ServerUnreachable => 65,
        RejectedForSspPermissions => 67,
        RefusedUnauthenticatedAccessNotSupported => 68,
        InvalidRsne => 72,
        UApsdCoexistenceNotSupported => 73,
        UApsdCoexModeNotSupported => 74,
        BadIntervalWithUApsdCoex => 75,
        AntiCloggingTokenRequired => 76,
        UnsupportedFiniteCyclicGroup => 77,
        CannotFindAlternativeTbtt => 78,
        TransmissionFailure => 79,
        RequestedTclasNotSupported2 => 80,
        TclasResourcesExhausted => 81,
        RejectedWithSuggestedBssTransition => 82,
        RejectWithSchedule => 83,
        RejectNoWakeupSpecified => 84,
        SuccessPowerSaveMode => 85,
        PendingAdmittingFstSession => 86,
        PerformingFstNow => 87,
        PendingGapInBaWindow => 88,
        RejectUpidSetting => 89,
        RefusedExternalReason => 92,
        RefusedApOutOfMemory => 93,
        RejectedEmergencyServicesNotSupported => 94,
        QueryResponseOutstanding => 95,
        RejectDseBand => 96,
        TclasProcessingTerminated => 97,
        TsScheduleConflict => 98,
        DeniedWithSuggestedBandAndChannel => 99,
        MccaopReservationConflict => 100,
        MafLimitExceeded => 101,
        MccaTrackLimitExceeded => 102,
        DeniedDueToSpectrumManagement => 103,
        DeniedVhtNotSupported => 104,
        Enablement => 105,
        Restriction => 106,
        Authorization => 107,
        EnergyLimitedOperationNotSupported => 108,
        RejectedNdpBlockAckSuggested => 109,
        RejectedMaxAwayDurationUnacceptable => 110,
        FlowControlOperationSupported => 111,
        FilsAuthenticationFailure => 112,
        UnknownAuthenticationServer => 113,
        DeniedNotificationPeriodAllocation => 116,
        DeniedChannelSplitting => 117,
        DeniedAllocation => 118,
        CmmgFeaturesNotSupported => 119,
        GasFragmentNotAvailable => 120,
        SuccessCagVersionsMatch => 121,
        GlkNotAuthorized => 122,
        UnknownPasswordIdentifier => 123,
        DeniedLocalMacAddressPolicyViolation => 125,
        SaeHashToElement => 126,
        TclasProcessingTerminatedInsufficientQos => 128,
        TclasProcessingTerminatedPolicyConflict => 129
    }
}
