use macro_bits::serializable_enum;

serializable_enum! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub enum IEEE80211AuthenticationAlgorithmNumber: u16 {
        #[default]
        OpenSystem => 0,
        SharedKey => 1,
        FastBSSTransition => 2,
        SimultaneousAuthenticationOfEquals => 3,
        FILSSharedKeyAuthenticationWithout => 4,
        FILSSharedKeyAuthenticationWith => 5,
        FILSPublicKeyAuthentication => 6,
        VendorSpecificUse => 0xffff
    }
}
