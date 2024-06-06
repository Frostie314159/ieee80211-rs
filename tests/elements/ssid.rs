use ieee80211::{elements::SSIDElement, ssid};

use crate::gen_element_rw_test;

// We can't test the [ssid] macro, since rust doesn't support expected build failures.
// This isn't doesn't really matter, since it's short enough to audit by hand.

const EXPECTED_SSID_STRING: &str = "OpenRF";
const EXPECTED_SSID_ELEMENT: SSIDElement = ssid!(EXPECTED_SSID_STRING);
const EXPECTED_SSID_ELEMENT_BYTES: &[u8] = EXPECTED_SSID_STRING.as_bytes();
const WILDCARD_SSID_ELEMENT: SSIDElement = ssid!("");
// One byte too long.
const INVALID_SSID_STRING: &str = "Lorem ipsum dolor sit amet augue.";

gen_element_rw_test!(
    test_ssid_element_rw,
    SSIDElement,
    EXPECTED_SSID_ELEMENT,
    EXPECTED_SSID_ELEMENT_BYTES
);

#[test]
fn test_ssid_element_misc() {
    assert!(
        WILDCARD_SSID_ELEMENT.is_hidden(),
        "Wildcard SSID wasn't marked as hidden. How did this happen?"
    );
    // Not so fun fact: This test technically already caught an error, since I screwed up when writing the original function...
    assert_eq!(
        EXPECTED_SSID_STRING.as_bytes().len(),
        EXPECTED_SSID_ELEMENT.length_in_bytes(),
        "Length in bytes returned didn't match what was expected."
    );
    assert_eq!(
        SSIDElement::new(EXPECTED_SSID_STRING),
        Some(SSIDElement::new_unchecked(EXPECTED_SSID_STRING)),
        "Creating a SSID element, with a valid SSID failed."
    );
    assert!(
        SSIDElement::new(INVALID_SSID_STRING).is_none(),
        "Creating a SSID element, with an invalid SSID succeeded."
    );
}
