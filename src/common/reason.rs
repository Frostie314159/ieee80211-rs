use macro_bits::serializable_enum;

serializable_enum! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub enum IEEE80211Reason: u16 {
        #[default]
        Unspecified => 0x01,
        InvalidAuthentication => 0x02,
        LeavingNetworkDeauth => 0x03,
        Inactivity => 0x04,
        NoMoreSTAs => 0x05,
        InvalidClass2Frame => 0x06,
        InvalidClass3Frame => 0x07,
        LeavingNetworkDisassoc => 0x08
    }
}
