use hmac::Hmac;
use pbkdf2::pbkdf2_array;
use sha1::Sha1;

/// Maps a passphrase to a PSK, as specified in Annex J of IEEE 802.11-2020.
/// 
/// This is used by WPA2.
pub fn map_passphrase_to_psk(passphrase: &str, ssid: &str) -> [u8; 32] {
    pbkdf2_array::<Hmac<Sha1>, 32>(passphrase.as_bytes(), ssid.as_bytes(), 4096).unwrap()
}
