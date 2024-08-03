use hex::decode_to_slice;
use ieee80211::crypto::map_passphrase_to_psk;

fn run_test_vector(passphrase: &str, ssid: &str, psk: &str) {
    let mut buf = [0x00; 32];
    decode_to_slice(psk.as_bytes(), buf.as_mut_slice()).unwrap();
    assert_eq!(map_passphrase_to_psk(passphrase, ssid), buf);
}

#[test]
fn test_passphrase_to_psk_mapping() {
    run_test_vector(
        "password",
        "IEEE",
        "f42c6fc52df0ebef9ebb4b90b38a5f902e83fe1b135a70e23aed762e9710a12e",
    );
    run_test_vector(
        "ThisIsAPassword",
        "ThisIsASSID",
        "0dc0d6eb90555ed6419756b9a15ec3e3209b63df707dd508d14581f8982721af",
    );
    run_test_vector(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ",
        "becb93866bb8c3832cb777c2f559807c8c59afcb6eae734885001300a981cc62",
    );
}
