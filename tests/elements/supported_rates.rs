use ieee80211::{
    elements::rates::{EncodedRate, RatesReadIterator, SupportedRatesElement},
    rate, supported_rates,
};

use crate::gen_element_rw_test;

const EXPECTED_RATE: EncodedRate = rate!(1.5 B);
const EXPECTED_ENCODED_RATE: u8 = 0x83;
const EXPECTED_SUPPORTED_RATES: SupportedRatesElement<[EncodedRate; 1]> = supported_rates![
    1.5 B
];
const EXPECTED_SUPPORTED_RATES_BYTES: &[u8] = &[EXPECTED_ENCODED_RATE];

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
gen_element_rw_test!(
    test_supported_rates_rw,
    SupportedRatesElement<RatesReadIterator<'_>>,
    EXPECTED_SUPPORTED_RATES,
    EXPECTED_SUPPORTED_RATES_BYTES
);
