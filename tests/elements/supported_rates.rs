use ieee80211::{
    elements::rates::{EncodedRate, ExtendedSupportedRatesElement, SupportedRatesElement},
    extended_supported_rates, rate, supported_rates,
};

use crate::gen_element_rw_test;

const EXPECTED_RATE: EncodedRate = rate!(1.5 B);
const EXPECTED_ENCODED_RATE: u8 = 0x83;
const EXPECTED_SUPPORTED_RATES: SupportedRatesElement<[EncodedRate; 1]> = supported_rates![
    1.5 B
];
const EXPECTED_SUPPORTED_RATES_BYTES: &[u8] = &[EXPECTED_ENCODED_RATE];
const EXPECTED_EXTENDED_SUPPORTED_RATES: ExtendedSupportedRatesElement<[EncodedRate; 2]> = extended_supported_rates![
    1.5 B,
    2
];
const EXPECTED_EXTENDED_SUPPORTED_RATES_BYTES: &[u8] = &[0x83, 0x04];

#[test]
fn test_encoded_rate() {
    let rate = EncodedRate::from_bits(EXPECTED_ENCODED_RATE);
    assert_eq!(
        rate, EXPECTED_RATE,
        "Decoded rate didn't match what was expected."
    );
    let encoded_rate = rate.into_bits();
    assert_eq!(
        encoded_rate, EXPECTED_ENCODED_RATE,
        "Encoded rate didn't match what was expected."
    );
    assert!(
        EXPECTED_RATE.is_b(),
        "Rate wasn't indicated to be an 802.11b rate."
    );
    assert_eq!(
        EXPECTED_RATE.rate_in_kbps(),
        1500,
        "Rate wasn't indicated to be 1.5Mb/s"
    );
}
#[test]
fn test_supported_rates_misc() {
    assert!(
        SupportedRatesElement::new([rate!(1.5 B)]).is_some(),
        "Creating a supported rates element, with valid rates, failed."
    );
    assert!(
        SupportedRatesElement::new([rate!(1.5 B); 9]).is_none(),
        "Creating a supported rates element, with invalid rates, succeeded."
    );
    assert!(
        ExtendedSupportedRatesElement::new([rate!(1.5 B)]).is_some(),
        "Creating an extended supported rates element, with valid rates, failed."
    );
    assert!(
        ExtendedSupportedRatesElement::new([rate!(1.5 B); 252]).is_none(),
        "Creating an extended supported rates element, with invalid rates, succeeded."
    );
}
gen_element_rw_test!(
    test_supported_rates_rw,
    SupportedRatesElement,
    EXPECTED_SUPPORTED_RATES,
    EXPECTED_SUPPORTED_RATES_BYTES
);
gen_element_rw_test!(
    test_extended_supported_rates_rw,
    ExtendedSupportedRatesElement,
    EXPECTED_EXTENDED_SUPPORTED_RATES,
    EXPECTED_EXTENDED_SUPPORTED_RATES_BYTES
);
