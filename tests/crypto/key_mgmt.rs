// use std::array::from_fn;

use ieee80211::crypto::{derive_ptk, map_passphrase_to_psk, prf, prf_iter};

fn run_psk_test_vector(passphrase: &str, ssid: &str, psk: &str) {
    let mut buf = [0x00; 32];
    map_passphrase_to_psk(passphrase, ssid, &mut buf);
    assert_eq!(&buf, &hex::decode(psk).unwrap().as_slice());
}

#[test]
fn test_passphrase_to_psk_mapping() {
    [
        (
            "password",
            "IEEE",
            "f42c6fc52df0ebef9ebb4b90b38a5f902e83fe1b135a70e23aed762e9710a12e",
        ),
        (
            "ThisIsAPassword",
            "ThisIsASSID",
            "0dc0d6eb90555ed6419756b9a15ec3e3209b63df707dd508d14581f8982721af",
        ),
        (
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ",
            "becb93866bb8c3832cb777c2f559807c8c59afcb6eae734885001300a981cc62",
        ),
    ]
    .iter()
    .for_each(|(passphrase, ssid, psk)| run_psk_test_vector(passphrase, ssid, psk));
}
fn run_prf_test_vector(key: &[u8], label: &str, data: &[u8], output: &str) {
    let output = hex::decode(output).unwrap();
    let mut buf = vec![0x00u8; output.len()];
    prf(key, label, data, &mut buf);
    assert_eq!(&buf, &output);
}
#[test]
fn test_prf() {
    [
        (
            [0x0bu8; 20].as_slice(),
            "prefix",
            "Hi There".as_bytes(),
            "bcd4c650b30b9684951829e0d75f9d54b862175ed9f00606",
        ),
        (
            "Jefe".as_bytes(),
            "prefix-2",
            "what do ya want for nothing?".as_bytes(),
            "47c4908e30c947521ad20be9053450ecbea23d3aa604b77326d8b3825ff7475c",
        ),
        (
            [0xaau8; 80].as_slice(),
            "prefix-3",
            "Test Using Larger Than Block-Size Key - Hash Key First".as_bytes(),
            "0ab6c33ccf70d0d736f4b04c8a7373255511abc5073713163bd0b8c9eeb7e1956fa066820a73ddee3f6d3bd407e0682a"
        ),
        (
            [0x0bu8; 20].as_slice(),
            "prefix-4",
            "Hi There Again".as_bytes(),
            "248cfbc532ab38ffa483c8a2e40bf170eb542a2e0916d7bf6d97da2c4c5ca877736c53a65b03fa4b3745ce7613f6ad68e0e4a798b7cf691c96176fd634a59a49"
    )
    ]
    .iter()
    .for_each(|(key, label, data, output)| run_prf_test_vector(key, label, data, output));

    let key = [0xff; 16];
    let data = "Test String".as_bytes();
    let label = "Test Label";

    let mut contiguous_output = [0x00u8; 32];
    prf(key.as_slice(), label, data, &mut contiguous_output);

    let mut non_contiguous_output = [0x00u8; 32];
    prf_iter(
        &key,
        label,
        &[&data[..5], &data[5..6], &data[6..]],
        &mut non_contiguous_output,
    );
}
#[test]
fn test_ptk_derivation() {
    // These test vectors are taken from Aircrack-NG's test suite, since those in IEEE 802.11 Annex
    // J are missing some quite crucial details, like AKM and such.
    // https://github.com/aircrack-ng/aircrack-ng/blob/master/test/cryptounittest/test-calc-ptk.c
    // Aircrack-NG is under GPL, while this library is under Apache/MIT, so I'm unsure about the
    // legality of using the test vectors. If they have anything against us using them, I'll
    // happily remove them.

    let pmk = b"\xee\x51\x88\x37\x93\xa6\xf6\x8e\x96\x15\xfe\x73\xc8\x0a\x3a\xa6\xf2\xdd\x0e\xa5\x37\xbc\xe6\x27\xb9\x29\x18\x3c\xc6\xe5\x79\x25";
    let authenticator_address = [0x00, 0x14, 0x6c, 0x7e, 0x40, 0x80];
    let supplicant_address = [0x00, 0x13, 0x46, 0xfe, 0x32, 0x0c];
    let authenticator_nonce = b"\x22\x58\x54\xb0\x44\x4d\xe3\xaf\x06\xd1\x49\x2b\x85\x29\x84\xf0\x4c\xf6\x27\x4c\x0e\x32\x18\xb8\x68\x17\x56\x86\x4d\xb7\xa0\x55";
    let supplicant_nonce = b"\x59\x16\x8b\xc3\xa5\xdf\x18\xd7\x1e\xfb\x64\x23\xf3\x40\x08\x8d\xab\x9e\x1b\xa2\xbb\xc5\x86\x59\xe0\x7b\x37\x64\xb0\xde\x85\x70";
    let expected_tk = b"\xea\x0e\x40\x46\x33\xc8\x02\x45\x03\x02\x86\x8c\xca\xa7\x49\xde\x5c\xba\x5a\xbc\xb2\x67\xe2\xde\x1d\x5e\x21\xe5\x7a\xcc\xd5\x07\x9b\x31\xe9\xff\x22\x0e\x13\x2a\xe4\xf6\xed\x9e\xf1\xac\xc8\x85\x45\x82\x5f\xc3\x2e\xe5\x59\x61\x39\x5a\xe4\x37\x34\xd6\xc1\x07\x98\xef\x5a\xfe\x42\xc0\x74\x26\x47\x18\x68\xa5\x77\xd4\xd1\x7e";

    let mut ptk = [0x00u8; 80];

    derive_ptk(
        pmk.as_slice(),
        &authenticator_address,
        &supplicant_address,
        authenticator_nonce,
        supplicant_nonce,
        ptk.as_mut_slice(),
    );
    assert_eq!(ptk.as_slice(), expected_tk);
}
