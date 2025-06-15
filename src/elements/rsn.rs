//! This module contains support for the RSN element.

use core::{fmt::Display, marker::PhantomData};

use bitfield_struct::bitfield;
use scroll::{
    ctx::{MeasureWith, SizeWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::common::{ReadIterator, IEEE_OUI};

use super::{Element, ElementID};

/// Combine the OUI and suite type into the cipher suite selector.
const fn merge_oui_and_suite_type(oui: [u8; 3], suite_type: u8) -> u32 {
    let mut cipher_suite_selector = [0x00u8; 4];

    // We can't use copy_from_slice here, since it's not const, due to the mutable reference.
    cipher_suite_selector[0] = oui[0];
    cipher_suite_selector[1] = oui[1];
    cipher_suite_selector[2] = oui[2];
    cipher_suite_selector[3] = suite_type;

    u32::from_le_bytes(cipher_suite_selector)
}
/// Split the cipher suite selector into OUI and suite type.
const fn split_cipher_suite_selector(cipher_suite_selector: u32) -> ([u8; 3], u8) {
    let suite_type = cipher_suite_selector.to_le_bytes();
    let mut oui = [0x00u8; 3];
    oui[0] = suite_type[0];
    oui[1] = suite_type[1];
    oui[2] = suite_type[2];
    (oui, suite_type[3])
}
/// This is a const method to extract the suite type, if the OUI matches that of the IEEE.
const fn get_suite_type_if_oui_is_ieee(cipher_suite_selector: u32) -> Option<u8> {
    if cipher_suite_selector & 0xff_ffff == 0xac0f00u32 {
        Some((cipher_suite_selector >> 24) as u8)
    } else {
        None
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
        #[non_exhaustive]
        $enum_vis enum $enum_name {
            Unknown {
                cipher_suite_selector: u32
            },
            $(
                $(
                    #[$cipher_suite_meta]
                )*
                $cipher_suite_name
            ),*
        }
        impl $enum_name {

            #[inline]
            pub const fn with_cipher_suite_selector(cipher_suite_selector: u32) -> Self {
                match split_cipher_suite_selector(cipher_suite_selector) {
                    $(
                        ($oui, $cipher_suite_type) => Self::$cipher_suite_name,
                    )*
                    (oui, suite_type) => Self::Unknown {
                        cipher_suite_selector: merge_oui_and_suite_type(oui, suite_type)
                    }
                }
            }
            #[inline]
            pub const fn cipher_suite_selector(&self) -> u32 {
                match *self {
                    $(
                        Self::$cipher_suite_name => merge_oui_and_suite_type($oui, $cipher_suite_type),
                    )*
                    Self::Unknown { cipher_suite_selector } => cipher_suite_selector
                }
            }
        }
        impl<'a> TryFromCtx<'a> for $enum_name {
            type Error = scroll::Error;
            #[inline]
            fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
                let mut offset = 0;

                let cipher_suite_selector = from.gread_with(&mut offset, Endian::Little)?;

                Ok((
                    Self::with_cipher_suite_selector(cipher_suite_selector),
                    offset
                ))
            }
        }
        impl SizeWith for $enum_name {
            #[inline]
            fn size_with(_ctx: &()) -> usize {
                4
            }
        }
        impl TryIntoCtx for $enum_name {
            type Error = scroll::Error;
            #[inline]
            fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
                let mut offset = 0;

                buf.gwrite_with(self.cipher_suite_selector(), &mut offset, Endian::Little)?;

                Ok(offset)
            }
        }
    };
}
cipher_suite_selectors! {
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// The cipher suites.
    pub enum IEEE80211CipherSuiteSelector {
        #[default]
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
        BIPGcmp128 => (IEEE_OUI, 11),
        BIPGcmp256 => (IEEE_OUI, 12),
        BIPCcmp256 => (IEEE_OUI, 13)
    }
}
impl IEEE80211CipherSuiteSelector {
    /// Get the length of the temporal key (TK).
    ///
    /// Length is in bytes.
    pub const fn tk_len(&self) -> Option<usize> {
        if let Some(suite_type) = get_suite_type_if_oui_is_ieee(self.cipher_suite_selector()) {
            Some(match suite_type {
                1 => 5,
                2 | 9..=10 | 12..=13 => 32,
                4 | 6 | 8 | 11 => 16,
                5 => 13,
                _ => return None,
            })
        } else {
            None
        }
    }
    /// Get the length of the Message Integrity Check (MIC)
    ///
    /// Length is in bytes.
    pub const fn mic_len(&self) -> Option<usize> {
        if let Some(suite_type) = get_suite_type_if_oui_is_ieee(self.cipher_suite_selector()) {
            Some(match suite_type {
                2 | 4 | 6 => 8,
                8..=13 => 16,
                _ => return None,
            })
        } else {
            None
        }
    }
}
cipher_suite_selectors! {
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// The authentication and key-management type.
    pub enum IEEE80211AKMType {
        #[default]
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
impl IEEE80211AKMType {
    /// Get the length of the EAPOL-Key confirmation key (KCK)
    ///
    /// Length is in bytes.
    pub const fn kck_len(&self) -> Option<usize> {
        if let Some(suite_type) = get_suite_type_if_oui_is_ieee(self.cipher_suite_selector()) {
            Some(match suite_type {
                1..=11 => 16,
                12..=13 => 24,
                _ => return None,
            })
        } else {
            None
        }
    }
    /// Get the length of the EAPOL-Key encryption key (KEK)
    ///
    /// Length is in bytes.
    pub const fn kek_len(&self) -> Option<usize> {
        if let Some(suite_type) = get_suite_type_if_oui_is_ieee(self.cipher_suite_selector()) {
            Some(match suite_type {
                1..=11 => 16,
                12..=14 | 16 => 32,
                15 | 17 => 64,
                _ => return None,
            })
        } else {
            None
        }
    }
    /// Get the length of the EAPOL-Key confirmation key 2 (KCK2)
    ///
    /// Length is in bytes.
    /// Used in Fast Transistion.
    pub const fn kck2_len(&self) -> Option<usize> {
        if let Some(suite_type) = get_suite_type_if_oui_is_ieee(self.cipher_suite_selector()) {
            Some(match suite_type {
                16 => 16,
                17 => 24,
                _ => return None,
            })
        } else {
            None
        }
    }
    /// Get the length of the EAPOL-Key encryption key 2 (KEK2)
    ///
    /// Length is in bytes.
    /// Used in Fast Transistion.
    pub const fn kek2_len(&self) -> Option<usize> {
        if let Some(suite_type) = get_suite_type_if_oui_is_ieee(self.cipher_suite_selector()) {
            Some(match suite_type {
                16 => 16,
                17 => 32,
                _ => return None,
            })
        } else {
            None
        }
    }
    /// Get the length of the key message integrity check (MIC)
    ///
    /// Length is in bytes.
    pub const fn key_mic_len(&self) -> Option<usize> {
        if let Some(suite_type) = get_suite_type_if_oui_is_ieee(self.cipher_suite_selector()) {
            Some(match suite_type {
                1..=11 | 16 => 16,
                12..=13 | 17 => 24,
                14..=15 => 0,
                _ => return None,
            })
        } else {
            None
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// Configuration of an optional feature
pub enum OptionalFeatureConfig {
    #[default]
    /// The feature is disabled.
    Disabled,
    /// The feature is enabled but not required.
    Capable,
    /// The feature is enabled and required.
    Required,
    /// The configuration is invalid.
    Invalid,
}
impl OptionalFeatureConfig {
    pub const fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0b00 => Self::Disabled,
            0b01 => Self::Capable,
            0b10 => Self::Required,
            _ => Self::Invalid,
        }
    }
    pub const fn into_bits(self) -> u8 {
        match self {
            Self::Disabled => 0b00,
            Self::Capable => 0b01,
            Self::Required => 0b11,
            Self::Invalid => 0b10,
        }
    }
    /// Check if the transmitting station is capable of the feature.
    pub const fn is_capable(&self) -> bool {
        matches!(self, Self::Required | Self::Capable)
    }
    /// Check if the feature is required.
    pub const fn is_required(&self) -> bool {
        matches!(self, Self::Required)
    }
    /// Check if the configurations are compatible.
    pub const fn is_compatible(&self, rhs: &Self) -> bool {
        self.is_required() == rhs.is_capable() && self.is_capable() == rhs.is_required()
    }
}

#[bitfield(u16, defmt = cfg(feature = "defmt"))]
#[derive(PartialEq, Eq, Hash)]
/// The specific capabilities of the transmitting STA.
pub struct RSNCapabilities {
    /// Is preauthentication supported.
    pub supports_preauthentication: bool,
    pub no_pairwise_key: bool,
    #[bits(2)]
    pub ptksa_replay_counter: u8,
    #[bits(2)]
    pub gtksa_replay_counter: u8,
    #[bits(2)]
    /// Management Frame Protection (MFP) configuration
    pub mfp_config: OptionalFeatureConfig,
    /// Joint Multi Band RSNA capable
    pub joint_multi_band_rsna_capable: bool,
    /// Peer Key Enabled Handshake capable
    pub peer_key_enabled_handshake_capable: bool,
    #[bits(2)]
    /// A-MPDU Signalling and Payload Protection (SPP) configuration
    pub spp_ampdu_config: OptionalFeatureConfig,
    /// Protected Block ACK Agreement Capable
    pub pbac_capable: bool,
    pub ext_key_id_for_individually_addressed_frames: bool,
    /// Is Operating Channel Validation (OCV).
    pub ocv_capable: bool,
    pub __: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// This is a temporary fix until <https://github.com/m4b/scroll/pull/99> and <https://github.com/m4b/scroll/pull/100> get merged.
pub struct IEEE80211PMKID(pub [u8; 16]);
impl Display for IEEE80211PMKID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:16x}", u128::from_be_bytes(self.0)))
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for IEEE80211PMKID {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "{=u128:x}", u128::from_be_bytes(self.0))
    }
}
impl<'a> TryFromCtx<'a> for IEEE80211PMKID {
    type Error = scroll::Error;
    #[inline]
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        <[u8; 16]>::try_from_ctx(from, Endian::Little)
            .map(|(pmkid, offset)| (IEEE80211PMKID(pmkid), offset))
    }
}
impl TryIntoCtx for IEEE80211PMKID {
    type Error = scroll::Error;
    #[inline]
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        buf.pwrite(self.0.as_slice(), 0)
    }
}
impl SizeWith for IEEE80211PMKID {
    #[inline]
    fn size_with(_ctx: &()) -> usize {
        16
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Hash)]
/// The RSN element contains information about the security characteristics of a BSS.
///
/// # Note
/// The reason, that all fields after the `version` are wrapped in an [Option] is, that they only appear if there are enough bytes left for them.
/// This means, that if you want to use the `rsn_capabilities` field, all prior fields need to be [Option::Some] even if they are just default values.
/// Due to this, it is highly recommended, that you use the `with_` methods, to construct the element.
/// This is not validated while writing, due to the performance hit, and can cause invalid outputs.
pub struct RSNElement<
    'a,
    PairwiseCipherSuiteList = ReadIterator<'a, (), IEEE80211CipherSuiteSelector>,
    AKMList = ReadIterator<'a, (), IEEE80211AKMType>,
    PMKIDList = ReadIterator<'a, (), IEEE80211PMKID>,
> {
    /// The cipher suite used for group addressed data traffic.
    pub group_data_cipher_suite: Option<IEEE80211CipherSuiteSelector>,
    /// The list of cipher suites supported for individually addressed traffic.
    pub pairwise_cipher_suite_list: Option<PairwiseCipherSuiteList>,
    /// The list of supported authentication and key-management suites.
    pub akm_list: Option<AKMList>,
    /// The RSN capabilities of the transmitting STA.
    pub rsn_capbilities: Option<RSNCapabilities>,
    /// The list of primary master key IDs.
    pub pmkid_list: Option<PMKIDList>,
    /// The cipher suite used for group addressed management frames.
    pub group_management_cipher_suite: Option<IEEE80211CipherSuiteSelector>,
    pub _phantom: PhantomData<&'a ()>,
}
impl RSNElement<'_> {
    /// Create a new empty [RSNElement].
    pub const fn new() -> RSNElement<
        'static,
        [IEEE80211CipherSuiteSelector; 0],
        [IEEE80211AKMType; 0],
        [IEEE80211PMKID; 0],
    > {
        RSNElement {
            group_data_cipher_suite: None,
            pairwise_cipher_suite_list: None,
            akm_list: None,
            rsn_capbilities: None,
            pmkid_list: None,
            group_management_cipher_suite: None,
            _phantom: PhantomData,
        }
    }
    /// An [RSNElement] equivalent to WPA-Personal.
    pub const WPA_PERSONAL: RSNElement<
        'static,
        [IEEE80211CipherSuiteSelector; 1],
        [IEEE80211AKMType; 1],
        [IEEE80211PMKID; 0],
    > = RSNElement {
        group_data_cipher_suite: Some(IEEE80211CipherSuiteSelector::Tkip),
        pairwise_cipher_suite_list: Some([IEEE80211CipherSuiteSelector::Tkip]),
        akm_list: Some([IEEE80211AKMType::Psk]),
        rsn_capbilities: None,
        pmkid_list: None,
        group_management_cipher_suite: None,
        _phantom: PhantomData,
    };
    /// An [RSNElement] equivalent to WPA/WPA2-Personal.
    pub const WPA_WPA2_PERSONAL: RSNElement<
        'static,
        [IEEE80211CipherSuiteSelector; 2],
        [IEEE80211AKMType; 1],
        [IEEE80211PMKID; 0],
    > = RSNElement {
        group_data_cipher_suite: Some(IEEE80211CipherSuiteSelector::Tkip),
        pairwise_cipher_suite_list: Some([
            IEEE80211CipherSuiteSelector::Tkip,
            IEEE80211CipherSuiteSelector::Ccmp128,
        ]),
        akm_list: Some([IEEE80211AKMType::Psk]),
        rsn_capbilities: None,
        pmkid_list: None,
        group_management_cipher_suite: None,
        _phantom: PhantomData,
    };
    /// An [RSNElement] equivalent to WPA2-Personal.
    pub const WPA2_PERSONAL: RSNElement<
        'static,
        [IEEE80211CipherSuiteSelector; 1],
        [IEEE80211AKMType; 1],
        [IEEE80211PMKID; 0],
    > = RSNElement {
        group_data_cipher_suite: Some(IEEE80211CipherSuiteSelector::Ccmp128),
        pairwise_cipher_suite_list: Some([IEEE80211CipherSuiteSelector::Ccmp128]),
        akm_list: Some([IEEE80211AKMType::Psk]),
        rsn_capbilities: None,
        pmkid_list: None,
        group_management_cipher_suite: None,
        _phantom: PhantomData,
    };
    /// An [RSNElement] equivalent to WPA2/WPA3-Personal.
    pub const WPA2_WPA3_PERSONAL: RSNElement<
        'static,
        [IEEE80211CipherSuiteSelector; 1],
        [IEEE80211AKMType; 2],
        [IEEE80211PMKID; 0],
    > = RSNElement {
        group_data_cipher_suite: Some(IEEE80211CipherSuiteSelector::Ccmp128),
        pairwise_cipher_suite_list: Some([IEEE80211CipherSuiteSelector::Ccmp128]),
        akm_list: Some([IEEE80211AKMType::Psk, IEEE80211AKMType::Sae]),
        rsn_capbilities: None,
        pmkid_list: None,
        group_management_cipher_suite: None,
        _phantom: PhantomData,
    };
    /// An [RSNElement] equivalent to WPA3-Personal.
    pub const WPA3_PERSONAL: RSNElement<
        'static,
        [IEEE80211CipherSuiteSelector; 1],
        [IEEE80211AKMType; 1],
        [IEEE80211PMKID; 0],
    > = RSNElement {
        group_data_cipher_suite: Some(IEEE80211CipherSuiteSelector::Ccmp128),
        pairwise_cipher_suite_list: Some([IEEE80211CipherSuiteSelector::Ccmp128]),
        akm_list: Some([IEEE80211AKMType::Sae]),
        rsn_capbilities: Some(
            RSNCapabilities::new().with_mfp_config(OptionalFeatureConfig::Required),
        ),
        pmkid_list: Some([]),
        group_management_cipher_suite: Some(IEEE80211CipherSuiteSelector::BipCmac128),
        _phantom: PhantomData,
    };
    /// An [RSNElement] equivalent to OWE.
    pub const OWE: RSNElement<
        'static,
        [IEEE80211CipherSuiteSelector; 1],
        [IEEE80211AKMType; 1],
        [IEEE80211PMKID; 0],
    > = RSNElement {
        group_data_cipher_suite: Some(IEEE80211CipherSuiteSelector::Ccmp128),
        pairwise_cipher_suite_list: Some([IEEE80211CipherSuiteSelector::Ccmp128]),
        akm_list: Some([IEEE80211AKMType::OpportunisticWirelessEncryption]),
        rsn_capbilities: Some(
            RSNCapabilities::new().with_mfp_config(OptionalFeatureConfig::Required),
        ),
        pmkid_list: None,
        group_management_cipher_suite: Some(IEEE80211CipherSuiteSelector::Ccmp128),
        _phantom: PhantomData,
    };
}
impl<PairwiseCipherSuiteList: Default, AKMList: Default, PMKIDList: Default>
    RSNElement<'static, PairwiseCipherSuiteList, AKMList, PMKIDList>
{
    const DEFAULT_CIPHER_SUITE: IEEE80211CipherSuiteSelector =
        IEEE80211CipherSuiteSelector::Ccmp128;
    const DEFAULT_RSN_CAPABILITIES: RSNCapabilities = RSNCapabilities::new();
    /// Add a group data cipher suite to the [RSNElement].
    pub fn with_group_data_cipher_suite(
        mut self,
        group_data_cipher_suite: IEEE80211CipherSuiteSelector,
    ) -> Self {
        self.group_data_cipher_suite = Some(group_data_cipher_suite);
        self
    }
    /// Add a pairwise cipher suite to the [RSNElement].
    ///
    /// This overrides all previous fields with a default value, if they are [None].
    pub fn with_pairwise_cipher_suite_list<InnerPairwiseCipherSuiteList>(
        self,
        pairwise_cipher_suite_list: InnerPairwiseCipherSuiteList,
    ) -> RSNElement<'static, InnerPairwiseCipherSuiteList, AKMList, PMKIDList> {
        RSNElement {
            group_data_cipher_suite: self
                .group_data_cipher_suite
                .or(Some(Self::DEFAULT_CIPHER_SUITE)),
            pairwise_cipher_suite_list: Some(pairwise_cipher_suite_list),
            akm_list: self.akm_list,
            rsn_capbilities: self.rsn_capbilities,
            pmkid_list: self.pmkid_list,
            group_management_cipher_suite: self.group_management_cipher_suite,
            _phantom: PhantomData,
        }
    }
    /// Add an AKM list to the [RSNElement].
    ///
    /// This overrides all previous fields with a default value, if they are [None].
    pub fn with_akm_list<InnerAKMList>(
        self,
        akm_list: InnerAKMList,
    ) -> RSNElement<'static, PairwiseCipherSuiteList, InnerAKMList, PMKIDList> {
        RSNElement {
            group_data_cipher_suite: self
                .group_data_cipher_suite
                .or(Some(Self::DEFAULT_CIPHER_SUITE)),
            pairwise_cipher_suite_list: self
                .pairwise_cipher_suite_list
                .or(Some(PairwiseCipherSuiteList::default())),
            akm_list: Some(akm_list),
            rsn_capbilities: self.rsn_capbilities,
            pmkid_list: self.pmkid_list,
            group_management_cipher_suite: self.group_management_cipher_suite,
            _phantom: PhantomData,
        }
    }
    /// Add [RSNCapabilities] to the [RSNElement].
    ///
    /// This overrides all previous fields with a default value, if they are [None].
    pub fn with_rsn_capabilities(
        self,
        rsn_capabilities: RSNCapabilities,
    ) -> RSNElement<'static, PairwiseCipherSuiteList, AKMList, PMKIDList> {
        RSNElement {
            group_data_cipher_suite: self
                .group_data_cipher_suite
                .or(Some(Self::DEFAULT_CIPHER_SUITE)),
            pairwise_cipher_suite_list: self
                .pairwise_cipher_suite_list
                .or(Some(PairwiseCipherSuiteList::default())),
            akm_list: self.akm_list.or(Some(AKMList::default())),
            rsn_capbilities: Some(rsn_capabilities),
            pmkid_list: self.pmkid_list,
            group_management_cipher_suite: self.group_management_cipher_suite,
            _phantom: PhantomData,
        }
    }
    /// Add a PMKID list to the [RSNElement].
    ///
    /// This overrides all previous fields with a default value, if they are [None].
    pub fn with_pmkid_list<InnerPMKIDList>(
        self,
        pmkid_list: InnerPMKIDList,
    ) -> RSNElement<'static, PairwiseCipherSuiteList, AKMList, InnerPMKIDList> {
        RSNElement {
            group_data_cipher_suite: self
                .group_data_cipher_suite
                .or(Some(Self::DEFAULT_CIPHER_SUITE)),
            pairwise_cipher_suite_list: self
                .pairwise_cipher_suite_list
                .or(Some(PairwiseCipherSuiteList::default())),
            akm_list: self.akm_list.or(Some(AKMList::default())),
            rsn_capbilities: self
                .rsn_capbilities
                .or(Some(Self::DEFAULT_RSN_CAPABILITIES)),
            pmkid_list: Some(pmkid_list),
            group_management_cipher_suite: self.group_management_cipher_suite,
            _phantom: PhantomData,
        }
    }
    /// Add a group management cipher suite to the [RSNElement].
    ///
    /// This overrides all previous fields with a default value, if they are [None].
    pub fn with_group_management_cipher_suite(
        self,
        group_management_cipher_suite: IEEE80211CipherSuiteSelector,
    ) -> RSNElement<'static, PairwiseCipherSuiteList, AKMList, PMKIDList> {
        RSNElement {
            group_data_cipher_suite: self
                .group_data_cipher_suite
                .or(Some(Self::DEFAULT_CIPHER_SUITE)),
            pairwise_cipher_suite_list: self
                .pairwise_cipher_suite_list
                .or(Some(PairwiseCipherSuiteList::default())),
            akm_list: self.akm_list.or(Some(AKMList::default())),
            rsn_capbilities: self
                .rsn_capbilities
                .or(Some(Self::DEFAULT_RSN_CAPABILITIES)),
            pmkid_list: self.pmkid_list.or(Some(PMKIDList::default())),
            group_management_cipher_suite: Some(group_management_cipher_suite),
            _phantom: PhantomData,
        }
    }
}
macro_rules! compare_list_option {
    ($lhs:expr, $rhs:expr, $field_name:ident) => {
        match ($lhs.$field_name.clone(), $rhs.$field_name.clone()) {
            (Some(lhs), Some(rhs)) => lhs.into_iter().eq(rhs.into_iter()),
            (None, None) => true,
            _ => false,
        }
    };
}
impl<
        'a,
        LPairwiseCipherSuiteList: IntoIterator<Item = IEEE80211CipherSuiteSelector> + Clone,
        LAKMList: IntoIterator<Item = IEEE80211AKMType> + Clone,
        LPMKIDList: IntoIterator<Item = IEEE80211PMKID> + Clone,
        RPairwiseCipherSuiteList: IntoIterator<Item = IEEE80211CipherSuiteSelector> + Clone,
        RAKMList: IntoIterator<Item = IEEE80211AKMType> + Clone,
        RPMKIDList: IntoIterator<Item = IEEE80211PMKID> + Clone,
    > PartialEq<RSNElement<'a, RPairwiseCipherSuiteList, RAKMList, RPMKIDList>>
    for RSNElement<'a, LPairwiseCipherSuiteList, LAKMList, LPMKIDList>
{
    fn eq(&self, other: &RSNElement<RPairwiseCipherSuiteList, RAKMList, RPMKIDList>) -> bool {
        self.group_data_cipher_suite == other.group_data_cipher_suite
            && compare_list_option!(self, other, pairwise_cipher_suite_list)
            && compare_list_option!(self, other, akm_list)
            && self.rsn_capbilities == other.rsn_capbilities
            && compare_list_option!(self, other, pmkid_list)
            && self.group_management_cipher_suite == other.group_management_cipher_suite
    }
}
impl<PairwiseCipherSuiteList, AKMList, PMKIDList> Default
    for RSNElement<'_, PairwiseCipherSuiteList, AKMList, PMKIDList>
{
    fn default() -> Self {
        Self {
            group_data_cipher_suite: None,
            pairwise_cipher_suite_list: None,
            akm_list: None,
            rsn_capbilities: None,
            pmkid_list: None,
            group_management_cipher_suite: None,
            _phantom: PhantomData,
        }
    }
}
macro_rules! read_list {
    ($rsn_element:expr, $from:expr, $offset:expr, $list_name:ident) => {
        let Ok(list_length) = $from.gread_with::<u16>(&mut $offset, Endian::Little) else {
            return Ok(($rsn_element, $offset));
        };
        if let Ok(list_bytes) = $from.gread_with(&mut $offset, list_length as usize * 4) {
            $rsn_element.$list_name = Some(ReadIterator::new(list_bytes));
        } else {
            return Ok(($rsn_element, $offset));
        }
    };
}
impl<'a> TryFromCtx<'a> for RSNElement<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let mut rsn_element = RSNElement::default();
        if from.gread_with::<u16>(&mut offset, Endian::Little)? != 1 {
            return Err(scroll::Error::BadInput {
                size: offset,
                msg: "RSN versions other than one are unsupported.",
            });
        }
        if let Ok(group_data_cipher_suite) = from.gread(&mut offset) {
            rsn_element.group_data_cipher_suite = Some(group_data_cipher_suite);
        } else {
            return Ok((rsn_element, offset));
        }
        read_list!(rsn_element, from, offset, pairwise_cipher_suite_list);
        read_list!(rsn_element, from, offset, akm_list);
        if let Ok(rsn_capabilities) = from.gread(&mut offset) {
            rsn_element.rsn_capbilities = Some(RSNCapabilities::from_bits(rsn_capabilities));
        } else {
            return Ok((rsn_element, offset));
        }
        read_list!(rsn_element, from, offset, pmkid_list);
        if let Ok(group_management_cipher_suite) = from.gread(&mut offset) {
            rsn_element.group_management_cipher_suite = Some(group_management_cipher_suite);
        } else {
            return Ok((rsn_element, offset));
        }

        Ok((rsn_element, offset))
    }
}
impl<
        PairwiseCipherSuiteList: IntoIterator<Item = IEEE80211CipherSuiteSelector> + Clone,
        AKMList: IntoIterator<Item = IEEE80211AKMType> + Clone,
        PMKIDList: IntoIterator<Item = IEEE80211PMKID> + Clone,
    > MeasureWith<()> for RSNElement<'_, PairwiseCipherSuiteList, AKMList, PMKIDList>
{
    fn measure_with(&self, _ctx: &()) -> usize {
        2 + if self.group_data_cipher_suite.is_some() {
            4
        } else {
            0
        } + if let Some(pairwise_cipher_suite_list) = &self.pairwise_cipher_suite_list {
            2 + pairwise_cipher_suite_list.clone().into_iter().count() * 4
        } else {
            0
        } + if let Some(akm_list) = &self.akm_list {
            2 + akm_list.clone().into_iter().count() * 4
        } else {
            0
        } + if self.rsn_capbilities.is_some() { 2 } else { 0 }
            + if let Some(pmkid_list) = &self.pmkid_list {
                2 + pmkid_list.clone().into_iter().count() * 4
            } else {
                0
            }
            + if self.group_management_cipher_suite.is_some() {
                4
            } else {
                0
            }
    }
}
macro_rules! write_list {
    ($buf:expr, $offset:expr, $list:expr) => {
        $offset += 2;
        let list_length = $buf.gwrite($list, &mut $offset)?;
        $buf.pwrite_with(
            list_length as u16 / 4,
            $offset - list_length - 2,
            Endian::Little,
        )?;
    };
}
// The additional `TryIntoCtx` bounds are present, because doing this using an iterator is horribly inefficent.
impl<
        PairwiseCipherSuiteList: TryIntoCtx<(), Error = scroll::Error>,
        AKMList: TryIntoCtx<(), Error = scroll::Error>,
        PMKIDList: TryIntoCtx<(), Error = scroll::Error>,
    > TryIntoCtx for RSNElement<'_, PairwiseCipherSuiteList, AKMList, PMKIDList>
where
    Self: MeasureWith<()>,
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(1u16, &mut offset, Endian::Little)?;
        if let Some(group_data_cipher_suite) = self.group_data_cipher_suite {
            buf.gwrite(group_data_cipher_suite, &mut offset)?;
        } else {
            return Ok(offset);
        }
        if let Some(pairwise_cipher_suite_list) = self.pairwise_cipher_suite_list {
            write_list!(buf, offset, pairwise_cipher_suite_list);
        } else {
            return Ok(offset);
        }
        if let Some(akm_list) = self.akm_list {
            write_list!(buf, offset, akm_list);
        } else {
            return Ok(offset);
        }
        if let Some(rsn_capabilities) = self.rsn_capbilities {
            buf.gwrite_with(rsn_capabilities.into_bits(), &mut offset, Endian::Little)?;
        } else {
            return Ok(offset);
        }
        if let Some(pmkid_list) = self.pmkid_list {
            write_list!(buf, offset, pmkid_list);
        } else {
            return Ok(offset);
        }
        if let Some(group_management_cipher_suite) = self.group_management_cipher_suite {
            buf.gwrite(group_management_cipher_suite, &mut offset)?;
        } else {
            return Ok(offset);
        }

        Ok(offset)
    }
}
impl<PairwiseCipherSuiteList, AKMList, PMKIDList> Element
    for RSNElement<'_, PairwiseCipherSuiteList, AKMList, PMKIDList>
where
    Self: MeasureWith<()> + TryIntoCtx<Error = scroll::Error>,
{
    const ELEMENT_ID: ElementID = ElementID::Id(0x30);
    type ReadType<'a> = RSNElement<'a>;
}
