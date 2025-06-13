use core::cmp::Ordering;

use hmac::{
    digest::{FixedOutput, KeyInit},
    Hmac, Mac,
};
use pbkdf2::pbkdf2_hmac;
use sha1::Sha1;
use sha2::{Sha256, Sha384};

/// HMAC-SHA-1 function
pub type HSha1 = Hmac<Sha1>;
/// HMAC-SHA-256 function
pub type HSha256 = Hmac<Sha256>;
/// HMAC-SHA-384 function
pub type HSha384 = Hmac<Sha384>;

/// Generate the Pairwise Master Key Identifier (PMKID)
///
/// This is generic over the used HMAC function. The default is [HSha1].
pub fn generate_pmkid<H: Mac + KeyInit>(
    key: &[u8],
    authenticator_address: &[u8; 6],
    supplicant_address: &[u8; 6],
    output: &mut [u8; 16],
) {
    let mut hmac = <H as Mac>::new_from_slice(key).unwrap();
    [
        "PMK Name".as_bytes(),
        authenticator_address,
        supplicant_address,
    ]
    .iter()
    .for_each(|chunk| hmac.update(chunk));
    output.copy_from_slice(&hmac.finalize().into_bytes()[..16]);
}

/// Maps a passphrase to a PSK, as specified in Annex J of IEEE 802.11-2020.
///
/// The length of `output` is the length of the PSK.
/// This is used by WPA2.
pub fn map_passphrase_to_psk(passphrase: &str, ssid: &str, output: &mut [u8]) {
    pbkdf2_hmac::<Sha1>(passphrase.as_bytes(), ssid.as_bytes(), 4096, output);
}
/// Pseudo Random Function (PRF) with data iterator
///
/// This is exactly the same as [prf], but instead of taking in a single data slice, it takes in a
/// reference to some kind of collection of data slices. This is useful, because in IEEE 802.11 the
/// PRF is almost always used with multiple chunks of data concatenated together, which would be
/// hard to do with [prf].
pub fn prf_iter<'a, D>(key: &[u8], label: &str, data: &'a D, output: &mut [u8])
where
    &'a D: IntoIterator<Item = &'a &'a [u8]>,
    <&'a D as IntoIterator>::IntoIter: Clone,
{
    // The output size of SHA1.
    const SHA1_OUTPUT_SIZE: usize = 160 / 8;

    let data_iter = data.into_iter();
    // chunks_mut handles the clamping for us
    for (i, output_chunk) in output.chunks_mut(SHA1_OUTPUT_SIZE).enumerate() {
        // We initialize HSHA1 with the key and update it with every piece of data.
        let mut h_sha_1 = <HSha1 as Mac>::new_from_slice(key).unwrap();
        h_sha_1.update(label.as_bytes());
        h_sha_1.update(&[0x00u8]);

        // Here we update it with the data chunks
        data_iter
            .clone()
            .for_each(|data_chunk| h_sha_1.update(data_chunk));

        h_sha_1.update(&[i as u8]);
        // If the chunk is as big as SHA1's output, we can output the data directly into the output
        // buffer. Otherwise we have to put it in a variable first.
        if output_chunk.len() == SHA1_OUTPUT_SIZE {
            h_sha_1.finalize_into(output_chunk.into());
        } else {
            let output = h_sha_1.finalize().into_bytes();
            output_chunk.copy_from_slice(&output[..output_chunk.len()]);
        }
    }
}
/// Pseudo Random Function (PRF)
///
/// According to 12.7.1.2 IEEE 802.11-2020
pub fn prf(key: &[u8], label: &str, data: &[u8], output: &mut [u8]) {
    prf_iter(key, label, &[data], output)
}

/// Sort two byte slices lexicographically.
///
/// The first slice in the returned tuple is lexicographically smaller than the second one, unless
/// both are equal, in which case it's `b`.
pub fn sort_lexicographically<'a>(a: &'a [u8], b: &'a [u8]) -> (&'a [u8], &'a [u8]) {
    if a.iter().partial_cmp(b.iter()) == Some(Ordering::Less) {
        (a, b)
    } else {
        (b, a)
    }
}
/// Derive a Pairwise Transient Key (PTK)
///
/// This derives the PTK from a PMK and the authenticator and supplicant address and nonce.
pub fn derive_ptk(
    pmk: &[u8],
    authenticator_address: &[u8; 6],
    supplicant_address: &[u8; 6],
    authenticator_nonce: &[u8; 32],
    supplicant_nonce: &[u8; 32],
    ptk: &mut [u8],
) {
    // This combines the min max stuff together.
    // NOTE: Who the hell came up with this?
    let (min_address, max_address) =
        sort_lexicographically(authenticator_address, supplicant_address);
    let (min_nonce, max_nonce) = sort_lexicographically(authenticator_nonce, supplicant_nonce);
    prf_iter(
        pmk,
        "Pairwise key expansion",
        &[min_address, max_address, min_nonce, max_nonce],
        ptk,
    );
}
