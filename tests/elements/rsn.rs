use std::marker::PhantomData;

use ieee80211::elements::rsn::{
    IEEE80211AkmType, IEEE80211CipherSuiteSelector, IEEE80211Pmkid, RsnCapabilities, RsnElement
};

use crate::roundtrip_test;

const EXPECTED_RSN_ELEMENT: RsnElement<
    [IEEE80211CipherSuiteSelector; 1],
    [IEEE80211AkmType; 1],
    [IEEE80211Pmkid; 0],
> = RsnElement {
    group_data_cipher_suite: Some(IEEE80211CipherSuiteSelector::Ccmp128),
    pairwise_cipher_suite_list: Some([IEEE80211CipherSuiteSelector::Ccmp128]),
    akm_list: Some([IEEE80211AkmType::Psk]),
    rsn_capbilities: Some(RsnCapabilities::new().with_ptksa_replay_counter(3)),
    pmkid_list: None,
    group_management_cipher_suite: None,
    _phantom: PhantomData,
};
const EXPECTED_RSN_ELEMENT_BYTES: &[u8] = include_bytes!("../../bins/elements/rsn.bin");

roundtrip_test!(
    test_rsn_element_rw,
    RsnElement,
    EXPECTED_RSN_ELEMENT,
    EXPECTED_RSN_ELEMENT_BYTES
);
#[test]
fn test_rsn_element_builder() {
    assert_eq!(
        RsnElement::WPA2_PERSONAL,
        RsnElement::new()
            .with_group_data_cipher_suite(IEEE80211CipherSuiteSelector::Ccmp128)
            .with_pairwise_cipher_suite_list([IEEE80211CipherSuiteSelector::Ccmp128])
            .with_akm_list([IEEE80211AkmType::Psk])
            .with_rsn_capabilities(RsnCapabilities::new())
    )
}
#[test]
fn test_akm_parameters() {
    assert_eq!(IEEE80211AkmType::Psk.key_mic_len().unwrap(), 16);
}
