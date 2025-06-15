use std::marker::PhantomData;

use ieee80211::elements::rsn::{
    IEEE80211AKMType, IEEE80211CipherSuiteSelector, RSNCapabilities, RSNElement, IEEE80211PMKID,
};

use crate::roundtrip_test;

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

roundtrip_test!(
    test_rsn_element_rw,
    RSNElement,
    EXPECTED_RSN_ELEMENT,
    EXPECTED_RSN_ELEMENT_BYTES
);
#[test]
fn test_rsn_element_builder() {
    assert_eq!(
        RSNElement::WPA2_PERSONAL,
        RSNElement::new()
            .with_group_data_cipher_suite(IEEE80211CipherSuiteSelector::Ccmp128)
            .with_pairwise_cipher_suite_list([IEEE80211CipherSuiteSelector::Ccmp128])
            .with_akm_list([IEEE80211AKMType::Psk])
    )
}
#[test]
fn test_akm_parameters() {
    assert_eq!(IEEE80211AKMType::Psk.kek_len().unwrap(), 16);
}
