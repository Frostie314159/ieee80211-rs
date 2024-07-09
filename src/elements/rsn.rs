use core::marker::PhantomData;

use bitfield_struct::bitfield;
use macro_bits::serializable_enum;
use scroll::{
    ctx::{MeasureWith, SizeWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::common::{
    ieee80211_list::{IEEE80211List, IEEE80211ReadList},
    IEEE_OUI,
};

use super::{Element, ElementID};

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
        impl<'a> TryFromCtx<'a> for $enum_name {
            type Error = scroll::Error;
            fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
                let mut offset = 0;

                let oui = from.gread_with(&mut offset, Endian::Little)?;
                let suite_type = from.gread_with(&mut offset, Endian::Little)?;

                Ok((
                    Self::with_oui_and_suite_type(oui, suite_type),
                    offset
                ))
            }
        }
        impl SizeWith for $enum_name {
            fn size_with(_ctx: &()) -> usize {
                4
            }
        }
        impl TryIntoCtx for $enum_name {
            type Error = scroll::Error;
            fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
                let mut offset = 0;

                buf.gwrite_with(self.oui(), &mut offset, Endian::Little)?;
                buf.gwrite_with(self.suite_type(), &mut offset, Endian::Little)?;

                Ok(offset)
            }
        }
    };
}
cipher_suite_selectors! {
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
cipher_suite_selectors! {
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

#[bitfield(u16)]
#[derive(PartialEq, Eq, Hash)]
/// The specific capabilities of the transmitting STA.
pub struct RSNCapabilities {
    pub supports_preauthentication: bool,
    pub no_pairwise_key: bool,
    #[bits(2)]
    pub ptksa_replay_counter: u8,
    #[bits(2)]
    pub gtksa_replay_counter: u8,
    /// Protection of management frames is required.
    pub mfp_required: bool,
    /// Protection of management frames is optionally supported.
    pub mfp_enabled: bool,
    pub supports_joint_multi_band_rsna: bool,
    pub supports_peer_key_enabled_handshake: bool,
    pub spp_amsdu_capable: bool,
    pub spp_amsdu_required: bool,
    pub pbac_capable: bool,
    pub ext_key_id_for_individually_addressed_frames: bool,
    pub ocvc: bool,
    pub __: bool,
}
impl RSNCapabilities {
    /// Check if the specified management frame protection (MFP) policy is valid.
    ///
    /// This returns false, if MFP is required but not enabled.
    pub const fn is_mfp_valid(&self) -> bool {
        !self.mfp_required() || self.mfp_enabled()
    }
    /// Check if the own management frame protection (MFP) policy is compatible with the other provided one.
    pub const fn is_mfp_compatible(&self, other: RSNCapabilities) -> bool {
        // Associations aren't allowed with invalid MFP policies, so rule them out directly.
        if !self.is_mfp_valid() || !other.is_mfp_valid() {
            return false;
        }
        if !self.mfp_enabled() & other.mfp_required() || self.mfp_required() & !other.mfp_enabled()
        {
            return false;
        }
        true
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// This is a temporary fix until <https://github.com/m4b/scroll/pull/99> and <https://github.com/m4b/scroll/pull/100> get merged.
pub struct IEEE80211PMKID(pub [u8; 16]);

impl<'a> TryFromCtx<'a> for IEEE80211PMKID {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        <[u8; 16]>::try_from_ctx(from, Endian::Little)
            .map(|(pmkid, offset)| (IEEE80211PMKID(pmkid), offset))
    }
}
impl TryIntoCtx for IEEE80211PMKID {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        self.0.try_into_ctx(buf, Endian::Little)
    }
}
impl SizeWith for IEEE80211PMKID {
    fn size_with(_ctx: &()) -> usize {
        16
    }
}

macro_rules! compare_list_option {
    ($lhs:expr, $rhs:expr, $field_name:ident) => {
        match ($lhs.$field_name.as_ref(), $rhs.$field_name.as_ref()) {
            (Some(lhs), Some(rhs)) => lhs.eq(rhs),
            (None, None) => true,
            _ => false,
        }
    };
}

#[derive(Clone, Copy, Debug, Hash)]
/// The RSN element contains information about the security characteristics of a BSS.
///
/// # Note
/// The reason, that all fields after the `version` are wrapped in an [Option] is, that they only appear if there are enough bytes left for them.
/// This means, that if you want to use the `rsn_capabilities` field, all prior fields need to be [Option::Some] even if they are just default values.
/// This is not validated while writing, due to the performance hit, and can cause invalid outputs.
pub struct RSNElement<
    'a,
    PairwiseCipherSuiteList = IEEE80211ReadList<'a, IEEE80211CipherSuiteSelector, u16, 4>,
    AKMList = IEEE80211ReadList<'a, IEEE80211AKMType, u16, 4>,
    PMKIDList = IEEE80211ReadList<'a, IEEE80211PMKID, u16, 4>,
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
impl<
        'a,
        LPairwiseCipherSuiteList: IEEE80211List<IEEE80211CipherSuiteSelector, u16> + Clone,
        LAKMList: IEEE80211List<IEEE80211AKMType, u16> + Clone,
        LPMKIDList: IEEE80211List<IEEE80211PMKID, u16> + Clone,
        RPairwiseCipherSuiteList: IEEE80211List<IEEE80211CipherSuiteSelector, u16> + Clone,
        RAKMList: IEEE80211List<IEEE80211AKMType, u16> + Clone,
        RPMKIDList: IEEE80211List<IEEE80211PMKID, u16> + Clone,
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
impl<'a, PairwiseCipherSuiteList, AKMList, PMKIDList> Default
    for RSNElement<'a, PairwiseCipherSuiteList, AKMList, PMKIDList>
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
        if let Ok(pairwise_cipher_suite_list) = from.gread(&mut offset) {
            rsn_element.pairwise_cipher_suite_list = Some(pairwise_cipher_suite_list);
        } else {
            return Ok((rsn_element, offset));
        }
        if let Ok(akm_list) = from.gread(&mut offset) {
            rsn_element.akm_list = Some(akm_list);
        } else {
            return Ok((rsn_element, offset));
        }
        if let Ok(rsn_capabilities) = from.gread(&mut offset) {
            rsn_element.rsn_capbilities = Some(RSNCapabilities::from_bits(rsn_capabilities));
        } else {
            return Ok((rsn_element, offset));
        }
        if let Ok(pmkid_list) = from.gread(&mut offset) {
            rsn_element.pmkid_list = Some(pmkid_list);
        } else {
            return Ok((rsn_element, offset));
        }
        if let Ok(group_management_cipher_suite) = from.gread(&mut offset) {
            rsn_element.group_management_cipher_suite = Some(group_management_cipher_suite);
        } else {
            return Ok((rsn_element, offset));
        }

        Ok((rsn_element, offset))
    }
}
impl<
        PairwiseCipherSuiteList: IEEE80211List<IEEE80211CipherSuiteSelector, u16>,
        AKMList: IEEE80211List<IEEE80211AKMType, u16>,
        PMKIDList: IEEE80211List<IEEE80211PMKID, u16>,
    > MeasureWith<()> for RSNElement<'_, PairwiseCipherSuiteList, AKMList, PMKIDList>
{
    fn measure_with(&self, _ctx: &()) -> usize {
        2 + if self.group_data_cipher_suite.is_some() {
            4
        } else {
            0
        } + if let Some(pairwise_cipher_suite_list) = &self.pairwise_cipher_suite_list {
            pairwise_cipher_suite_list.size_in_bytes()
        } else {
            0
        } + if let Some(akm_list) = &self.akm_list {
            akm_list.size_in_bytes()
        } else {
            0
        } + if self.rsn_capbilities.is_some() { 2 } else { 0 }
            + if let Some(pmkid_list) = &self.pmkid_list {
                pmkid_list.size_in_bytes()
            } else {
                0
            }
            + if let Some(pmkid_list) = &self.pmkid_list {
                pmkid_list.size_in_bytes()
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
        $buf.gwrite($list.element_count(), &mut $offset)?;
        for element in $list.iter() {
            $buf.gwrite(element, &mut $offset)?;
        }
    };
}
// The additional `TryIntoCtx` bounds are present, because doing this using an iterator is horribly inefficent.
impl<
        PairwiseCipherSuiteList: IEEE80211List<IEEE80211CipherSuiteSelector, u16>,
        AKMList: IEEE80211List<IEEE80211AKMType, u16>,
        PMKIDList: IEEE80211List<IEEE80211PMKID, u16>,
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
        }
        if let Some(pairwise_cipher_suite_list) = self.pairwise_cipher_suite_list {
            write_list!(buf, offset, pairwise_cipher_suite_list);
        }
        if let Some(akm_list) = self.akm_list {
            write_list!(buf, offset, akm_list);
        }
        if let Some(rsn_capabilities) = self.rsn_capbilities {
            buf.gwrite_with(rsn_capabilities.into_bits(), &mut offset, Endian::Little)?;
        }
        if let Some(pmkid_list) = self.pmkid_list {
            write_list!(buf, offset, pmkid_list);
        }
        if let Some(group_management_cipher_suite) = self.group_management_cipher_suite {
            buf.gwrite(group_management_cipher_suite, &mut offset)?;
        }

        Ok(offset)
    }
}
impl<PairwiseCipherSuiteList, AKMList, PMKIDList> Element
    for RSNElement<'_, PairwiseCipherSuiteList, AKMList, PMKIDList>
where
    PairwiseCipherSuiteList: Clone
        + IEEE80211List<IEEE80211CipherSuiteSelector, u16>
        + TryIntoCtx<Error = scroll::Error>,
    AKMList: Clone + IEEE80211List<IEEE80211AKMType, u16> + TryIntoCtx<Error = scroll::Error>,
    PMKIDList: Clone + IEEE80211List<IEEE80211PMKID, u16> + TryIntoCtx<Error = scroll::Error>,
    Self: MeasureWith<()>,
{
    const ELEMENT_ID: ElementID = ElementID::Id(0x30);
    type ReadType<'a> = RSNElement<'a>;
}
