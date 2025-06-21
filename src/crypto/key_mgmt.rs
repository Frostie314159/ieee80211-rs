use core::cmp::Ordering;

use aes_kw::KekAes128;
use hmac::{
    digest::{FixedOutput, KeyInit},
    Hmac, Mac,
};
use llc_rs::EtherType;
use pbkdf2::pbkdf2_hmac;
use scroll::{ctx::TryIntoCtx, Endian, Pread, Pwrite};
use sha1::Sha1;
use sha2::{Sha256, Sha384};

use crate::{
    crypto::eapol::KeyInformation,
    data_frame::header::DataFrameHeader,
    elements::rsn::{IEEE80211AkmType, IEEE80211CipherSuiteSelector},
};

use super::eapol::{EapolDataFrame, EapolKeyFrame};

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
/// Implemented according to 12.7.1.2 IEEE 802.11-2020. This is the default PRF, with HMAC-SHA-1.
/// For some AKM suites, different PRF's are used.
pub fn prf(key: &[u8], label: &str, data: &[u8], output: &mut [u8]) {
    prf_iter(key, label, &[data], output)
}

/// Sort two byte slices lexicographically.
///
/// The first slice in the returned tuple is lexicographically smaller than the second one, unless
/// both are equal, in which case it's `b`.
fn sort_lexicographically<'a>(a: &'a [u8], b: &'a [u8]) -> (&'a [u8], &'a [u8]) {
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
/// Partition a PTK into KCK, KEK and TK
///
/// This will return [None], if either the AKM or Cipher suite are unknown, or the provided PTK is
/// too short. If the PTK is longer than the KCK, KEK and TK together, the excess data will just be
/// truncated.
pub fn partition_ptk(
    ptk: &[u8],
    akm_suite: IEEE80211AkmType,
    cipher_suite: IEEE80211CipherSuiteSelector,
) -> Option<(&[u8], &[u8], &[u8])> {
    let kck_len = akm_suite.kck_len()?;
    let kek_len = akm_suite.kek_len()?;
    let tk_len = cipher_suite.tk_len()?;
    let (kck, kek_and_tk) = ptk.split_at_checked(kck_len)?;
    let (kek, tk) = kek_and_tk.split_at_checked(kek_len)?;
    let tk = tk.get(..tk_len)?;
    Some((kck, kek, tk))
}
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// An error related to cryptographic key management operation.
pub enum KeyManagementError {
    /// The key was too short.
    InvalidKeyLength,
    /// The output slice length didn't match the output size of the algorithm.
    InvalidOutputLength,
    /// The provided scratch buffer was too short.
    ScratchBufferTooShort,
}
/// Wrap the EAPOL key data using the NIST AES Key-Wrap algorithm.
///
/// The `key_data` slice should contain the entire Key Data segment of the EAPOL Key frame.
/// Currently this assumes a 128 bit KEK is in use, which is true for most AKM suites.
pub fn wrap_eapol_key_data(
    kek: &[u8; 16],
    key_data: &[u8],
    output: &mut [u8],
) -> Result<(), KeyManagementError> {
    let kw = KekAes128::new(kek.into());
    kw.wrap_with_padding(key_data, output)
        .map_err(|_| KeyManagementError::InvalidOutputLength)?;
    Ok(())
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
/// An error related to EAPOL frame serialization.
pub enum EapolSerdeError {
    /// The provided output buffer was too short.
    BufferTooShort,
    /// The provided temporary buffer was too short.
    TemporaryBufferToShort,
    /// The data frame didn't contain a payload.
    NoPayload,
    /// A needed key wasn't provided.
    MissingKey,
    /// Deserializing the key frame failed.
    KeyFrameDeserializationFailure,
    /// The used AKM suite is unknown.
    UnknownAkmSuite,
    /// The MIC in the frame didn't match the calculated MIC.
    InvalidMic,
    /// The ether type wasn't EAPOL.
    EtherTypeNotEapol,
}
/// Serialize the provided EAPOL Key frame.
///
/// The temp_buffer needs to be at least as long as the key data field aligned to eight bytes, plus
/// eight bytes.
pub fn serialize_eapol_data_frame<
    KeyMic: AsRef<[u8]>,
    ElementContainer: TryIntoCtx<(), Error = scroll::Error>,
>(
    kck: Option<&[u8; 16]>,
    kek: Option<&[u8; 16]>,
    eapol_data_frame: EapolDataFrame<'_, KeyMic, ElementContainer>,
    buffer: &mut [u8],
    temp_buffer: &mut [u8],
) -> Result<usize, EapolSerdeError> {
    if temp_buffer.len() < 24 {
        return Err(EapolSerdeError::TemporaryBufferToShort);
    }
    let mic_range = eapol_data_frame
        .eapol_mic_range()
        .ok_or(EapolSerdeError::NoPayload)?;
    let mic_len = eapol_data_frame
        .payload
        .as_ref()
        .ok_or(EapolSerdeError::NoPayload)?
        .payload
        .key_mic
        .as_ref()
        .len();
    let key_data_length_range = eapol_data_frame
        .eapol_key_data_length_range()
        .ok_or(EapolSerdeError::NoPayload)?;
    let key_data_range = eapol_data_frame
        .eapol_key_data_range()
        .ok_or(EapolSerdeError::NoPayload)?;
    let key_information = eapol_data_frame
        .payload
        .as_ref()
        .ok_or(EapolSerdeError::NoPayload)?
        .payload
        .key_information;
    let eapol_frame_start = eapol_data_frame.header.length_in_bytes() + 8;

    let mut written = buffer
        .pwrite(eapol_data_frame, 0)
        .map_err(|_| EapolSerdeError::BufferTooShort)?;

    let key_data_length = buffer[key_data_length_range.clone()]
        .pread_with::<u16>(0, Endian::Big)
        .expect("If the TryIntoCtx impl for EapolKeyFrame is correct, this can't fail.")
        as usize;

    if key_data_length != 0 && key_information.encrypted_key_data() {
        let padded_key_data_length = if key_data_length < 16 {
            16
        } else if key_data_length % 8 != 0 {
            (key_data_length & !(0b111)) + 8
        } else {
            key_data_length
        };

        let padded_key_data = buffer[key_data_range.clone()]
            .get_mut(..padded_key_data_length)
            .ok_or(EapolSerdeError::BufferTooShort)?;
        if padded_key_data_length != key_data_length {
            padded_key_data[key_data_length] = 0xdd;
            padded_key_data[key_data_length + 1..].fill(0x00);
        }
        let padded_and_wrapped_key_data_length = padded_key_data_length + 8;

        let kw = KekAes128::new(kek.ok_or(EapolSerdeError::MissingKey)?.into());
        kw.wrap(
            padded_key_data,
            &mut temp_buffer[..padded_and_wrapped_key_data_length],
        )
        .map_err(|_| EapolSerdeError::TemporaryBufferToShort)?;

        let wrapped_key_data = buffer[key_data_range]
            .get_mut(..padded_and_wrapped_key_data_length)
            .ok_or(EapolSerdeError::BufferTooShort)?;
        wrapped_key_data.copy_from_slice(&temp_buffer[..padded_and_wrapped_key_data_length]);
        written += padded_and_wrapped_key_data_length - key_data_length;

        let _ = buffer.pwrite_with(
            padded_and_wrapped_key_data_length as u16,
            key_data_length_range.start,
            Endian::Big,
        );
        let _ = buffer.pwrite_with(
            (77 + mic_len + 2 + padded_and_wrapped_key_data_length) as u16,
            eapol_frame_start + 2,
            Endian::Big,
        );
    }
    if key_information.key_mic() {
        let mut h_sha_1 =
            <HSha1 as Mac>::new_from_slice(kck.ok_or(EapolSerdeError::MissingKey)?).unwrap();
        h_sha_1.update(&buffer[eapol_frame_start..written]);
        h_sha_1.finalize_into((&mut temp_buffer[..20]).into());
        buffer[mic_range].copy_from_slice(&temp_buffer[..16]);
    }
    Ok(written)
}
/// Decrypt, verify and deserialize an EAPOL data frame.
///
/// The temp buffer needs to be as long as the key data minus eight.
pub fn deserialize_eapol_data_frame<'a>(
    kck: Option<&[u8; 16]>,
    kek: Option<&[u8; 16]>,
    mut buffer: &'a mut [u8],
    temp_buffer: &mut [u8],
    akm_suite: IEEE80211AkmType,
    with_fcs: bool,
) -> Result<EapolKeyFrame<'a>, EapolSerdeError> {
    if with_fcs {
        buffer = buffer
            .get_mut(..buffer.len() - 4)
            .ok_or(EapolSerdeError::BufferTooShort)?;
    }
    let data_frame_header = buffer
        .pread::<DataFrameHeader>(0)
        .map_err(|_| EapolSerdeError::BufferTooShort)?;

    let payload_offset = data_frame_header.length_in_bytes();

    let llc_ether_type_offset = payload_offset + 6;
    let ether_type = buffer
        .get(llc_ether_type_offset..)
        .and_then(|bytes| bytes.pread_with::<u16>(0, Endian::Big).ok())
        .ok_or(EapolSerdeError::BufferTooShort)?;
    if ether_type != EtherType::Eapol.into() {
        return Err(EapolSerdeError::EtherTypeNotEapol);
    }

    let eapol_key_frame_offset = payload_offset + 8;
    let eapol_key_information_offset = eapol_key_frame_offset + 5;
    let eapol_key_information = KeyInformation::from_bits(
        buffer[eapol_key_information_offset..]
            .pread_with(0, Endian::Big)
            .map_err(|_| EapolSerdeError::BufferTooShort)?,
    );
    let mic_len = akm_suite
        .key_mic_len()
        .ok_or(EapolSerdeError::UnknownAkmSuite)?;
    if eapol_key_information.key_mic() {
        let mut h_sha_1 =
            <HSha1 as Mac>::new_from_slice(kck.ok_or(EapolSerdeError::MissingKey)?).unwrap();
        h_sha_1.update(
            buffer
                .get(eapol_key_frame_offset..eapol_key_frame_offset + 81)
                .ok_or(EapolSerdeError::BufferTooShort)?,
        );
        for _ in 0..mic_len / 8 {
            h_sha_1.update(&[0x00u8; 8]);
        }
        h_sha_1.update(
            buffer
                .get(eapol_key_frame_offset + 81 + mic_len..)
                .ok_or(EapolSerdeError::BufferTooShort)?,
        );
        let provided_mic = &buffer[eapol_key_frame_offset + 81..][..mic_len];

        let calculated_mic = h_sha_1.finalize().into_bytes();
        let calculated_mic = &calculated_mic.as_slice()[..mic_len];
        if calculated_mic != provided_mic {
            defmt::info!("Provided MIC: {:02x} Calculated MIC: {:02x}", provided_mic, calculated_mic);
            return Err(EapolSerdeError::InvalidMic);
        }
    }
    if eapol_key_information.encrypted_key_data() {
        let key_data_length_offset = eapol_key_frame_offset + 81 + mic_len;
        let key_data_length: u16 = buffer
            .pread_with(key_data_length_offset, Endian::Big)
            .map_err(|_| EapolSerdeError::BufferTooShort)?;

        let key_data = buffer[key_data_length_offset + 2..]
            .get_mut(..key_data_length as usize)
            .ok_or(EapolSerdeError::BufferTooShort)
            .unwrap();
        let kw = KekAes128::new(kek.ok_or(EapolSerdeError::MissingKey)?.into());
        kw.unwrap(key_data, &mut temp_buffer[..key_data_length as usize - 8])
            .map_err(|_| EapolSerdeError::TemporaryBufferToShort)?;

        buffer
            .pwrite_with(key_data_length - 8, key_data_length_offset, Endian::Big)
            .unwrap();
        buffer
            .pwrite(
                &temp_buffer[..key_data_length as usize - 8],
                key_data_length_offset + 2,
            )
            .unwrap();

        let new_buffer_len = buffer.len() - 8;
        buffer = &mut buffer[..new_buffer_len];
        buffer
            .pwrite_with(
                (new_buffer_len - eapol_key_frame_offset - 4) as u16,
                eapol_key_frame_offset + 2,
                Endian::Big,
            )
            .unwrap();
    }
    buffer
        .pread_with::<EapolKeyFrame>(eapol_key_frame_offset, akm_suite)
        .map_err(|_| EapolSerdeError::KeyFrameDeserializationFailure)
}
