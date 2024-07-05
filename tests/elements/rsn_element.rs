use std::marker::PhantomData;

use ieee80211::elements::rsn::{
    IEEE80211AKMType, IEEE80211CipherSuiteSelector, RSNCapabilities, RSNElement, IEEE80211PMKID,
};

use crate::gen_element_rw_test;

const EXPECTED_RSN_ELEMENT: RSNElement<
    [IEEE80211CipherSuiteSelector; 1],
    [IEEE80211AKMType; 1],
    [IEEE80211PMKID; 0],
> = RSNElement {
    group_data_cipher_suite: Some(IEEE80211CipherSuiteSelector::Ccmp128),
    pairwise_cipher_suite_list: Some([IEEE80211CipherSuiteSelector::Ccmp128]),
    akm_list: Some([IEEE80211AKMType::Psk]),
    rsn_capbilities: Some(RSNCapabilities::new().with_ptksa_replay_counter(3)),
    pmkid_list: None,
    group_management_cipher_suite: None,
    _phantom: PhantomData,
};
const EXPECTED_RSN_ELEMENT_BYTES: &[u8] = include_bytes!("../../bins/elements/rsn.bin");

gen_element_rw_test!(
    test_rsn_element_rw,
    RSNElement,
    EXPECTED_RSN_ELEMENT,
    EXPECTED_RSN_ELEMENT_BYTES
);
