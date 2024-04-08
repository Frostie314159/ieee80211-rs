use macro_bits::serializable_enum;

use crate::common::IEEE_OUI;

serializable_enum! {
    pub enum IEEE80211AuthenticationAlgorithmNumber: u16 {
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
macro_rules! cipher_suite_selectors {
    (
        $(
            #[$enum_meta:meta]
        )*
        $enum_vis:vis enum $enum_name:ident {
            $(
                $(
                    #[$cipher_suite_meta:meta]
                )*
                $cipher_suite_name:ident => ($oui:tt, $cipher_suite_type:expr)
            ),*
        }
    ) => {
        $(
            #[$enum_meta]
        )*
        $enum_vis enum $enum_name {
            Unknown {
                oui: [u8; 3],
                suite_type: u8
            },
            $(
                $(
                    #[$cipher_suite_meta]
                )*
                $cipher_suite_name
            ),*
        }
        impl $enum_name {
            pub const fn with_oui_and_suite_type(oui: [u8; 3], suite_type: u8) -> Self {
                match (oui, suite_type) {
                    $(
                        ($oui, $cipher_suite_type) => Self::$cipher_suite_name,
                    )*
                    (oui, suite_type) => Self::Unknown {
                        oui,
                        suite_type
                    }
                }
            }
            pub const fn oui(&self) -> [u8; 3] {
                match self {
                    $(
                        Self::$cipher_suite_name => $oui,
                    )*
                    Self::Unknown { oui, .. } => *oui
                }
            }
            pub const fn suite_type(&self) -> u8 {
                match self {
                    $(
                        Self::$cipher_suite_name => $cipher_suite_type,
                    )*
                    Self::Unknown { suite_type, .. } => *suite_type
                }
            }
        }
    };
}
cipher_suite_selectors! {
    pub enum IEEE80211CipherSuiteSelector {
        UseGroupCipherSuite => (IEEE_OUI, 0),
        Wep40 => (IEEE_OUI, 1),
        Tkip => (IEEE_OUI, 2),
        Ccmp128 => (IEEE_OUI, 4),
        Wep104 => (IEEE_OUI, 5),
        BipCmac128 => (IEEE_OUI, 6),
        GroupAddessedTrafficNotAllowed => (IEEE_OUI, 7),
        Gcmp128 => (IEEE_OUI, 8),
        Gcmp256 => (IEEE_OUI, 9),
        Ccmp256 => (IEEE_OUI, 10),
        BipGcmp128 => (IEEE_OUI, 11),
        BipGcmp256 => (IEEE_OUI, 12),
        BipCcmp256 => (IEEE_OUI, 13)
    }
}
cipher_suite_selectors! {
    pub enum IEEE80211KeyManagementType {
        None => (IEEE_OUI, 0),
        Wpa => (IEEE_OUI, 1),
        Psk => (IEEE_OUI, 2),
        FTOverIEEE8021X => (IEEE_OUI, 3),
        FTUsingPsk => (IEEE_OUI, 4),
        WpaSha256 => (IEEE_OUI, 5),
        PskSha256 => (IEEE_OUI, 6),
        Tdls => (IEEE_OUI, 7),
        Sae => (IEEE_OUI, 8),
        FTUsingSae => (IEEE_OUI, 9),
        APPeerKey => (IEEE_OUI, 10),
        WpaSha256SuiteB => (IEEE_OUI, 11),
        WpaSha384SuiteB => (IEEE_OUI, 12),
        FTOverIEEE8021XSha384 => (IEEE_OUI, 13),
        FilsSha256Aes256 => (IEEE_OUI, 14),
        FilsSha384Aes512 => (IEEE_OUI, 15),
        FTOverFilsSha256Aes256 => (IEEE_OUI, 16),
        FTOverFilsSha384Aes512 => (IEEE_OUI, 17),
        /// See [RFC 8110](https://datatracker.ietf.org/doc/html/rfc8110).
        OpportunisticWirelessEncryption => (IEEE_OUI, 18),
        FTUsingPskSha384 => (IEEE_OUI, 19),
        PskSha384 => (IEEE_OUI, 20),
        Pasn => (IEEE_OUI, 21),
        SaeGroupDefend => (IEEE_OUI, 22),
        FTUsingSaeGroupDefend => (IEEE_OUI, 23)
    }
}

pub struct RSNElement {
    pub version: u16,
    pub group_data_cipher_suite: Option<IEEE80211CipherSuiteSelector>,
}
