use ieee80211::elements::IBSSParameterSetElement;

use crate::roundtrip_test;

const EXPECTED_IBSS_PARAMETER_SET_ELEMENT: IBSSParameterSetElement = IBSSParameterSetElement {
    atim_window: 42, // See: Hitchhikers guide to the galaxy.
};
const EXPECTED_IBSS_PARAMETER_SET_ELEMENT_BYTES: &[u8] = 42u16.to_le_bytes().as_slice();

roundtrip_test!(
    test_ibss_paremeter_set,
    IBSSParameterSetElement,
    EXPECTED_IBSS_PARAMETER_SET_ELEMENT,
    EXPECTED_IBSS_PARAMETER_SET_ELEMENT_BYTES
);
