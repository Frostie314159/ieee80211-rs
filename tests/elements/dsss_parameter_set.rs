use ieee80211::elements::DSSSParameterSetElement;

use crate::roundtrip_test;

const EXPECTED_DSSS_PARAMETER_SET_ELEMENT: DSSSParameterSetElement = DSSSParameterSetElement {
    current_channel: 13,
};
const EXPECTED_DSSS_PARAMETER_SET_BYTES: &[u8] = &[13];

roundtrip_test!(
    test_dsss_parameter_set_rw,
    DSSSParameterSetElement,
    EXPECTED_DSSS_PARAMETER_SET_ELEMENT,
    EXPECTED_DSSS_PARAMETER_SET_BYTES
);
