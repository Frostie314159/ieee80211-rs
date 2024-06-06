use ieee80211::elements::DSSSParameterSetElement;

use crate::gen_element_rw_test;

const EXPECTED_DSSS_PARAMETER_SET_ELEMENT: DSSSParameterSetElement = DSSSParameterSetElement {
    current_channel: 13
};
const EXPECTED_DSSS_PARAMETER_SET_BYTES: &[u8] = &[13];

gen_element_rw_test!(test_dsss_parameter_set_rw, DSSSParameterSetElement, EXPECTED_DSSS_PARAMETER_SET_ELEMENT, EXPECTED_DSSS_PARAMETER_SET_BYTES);

