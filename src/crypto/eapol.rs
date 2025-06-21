use core::{
    marker::PhantomData,
    ops::{Range, RangeFrom},
};

use bitfield_struct::bitfield;
use llc_rs::SnapLlcFrame;
use macro_bits::serializable_enum;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{
    data_frame::DataFrame,
    elements::{rsn::IEEE80211AkmType, ReadElements},
};
serializable_enum! {
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    /// Cipher and MIC combination.
    pub enum KeyDescriptorVersion: u8 {
        /// RC4 for encryption and HMAC-MD5 for integrity
        Rc4HmacMd5 => 1,
        /// AES Key Wrap for encryption and HMAC-SHA1 for integrity
        AesHmacSha1 => 2,
        /// AES Key Wrap for encryption and AES-CMAC for integrity
        AesCmac => 3
    }
}

#[bitfield(u16, order = Lsb, defmt = cfg(feature = "defmt"))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Information about the EAPOL Key Frame.
pub struct KeyInformation {
    #[bits(3)]
    pub key_descriptor_version: KeyDescriptorVersion,
    /// Is this message part of a PTK derivation.
    ///
    /// NOTE: Officially this is called the "Key Type", however it effectively indicates if this is
    /// part of a PTK derivation.
    pub is_pairwise: bool,
    #[bits(2)]
    pub __: u8,
    /// Shall the key be installed.
    pub install: bool,
    /// Is an acknowledgement to this message required.
    ///
    /// If this is `true`, the recipient of this frame shall transmit an EAPOL Key Frame as an
    /// acknowledgement to this frame.
    pub key_ack: bool,
    /// Is a MIC present.
    pub key_mic: bool,
    /// Indicates if the frame contains the last key required for initialization.
    pub secure: bool,
    /// Indicates that a TKIP MIC failure occured.
    pub error: bool,
    /// Indicates a request to initiate a handshake.
    pub request: bool,
    /// Is the the key data encrypted.
    pub encrypted_key_data: bool,
    #[bits(3)]
    pub __: u8,
}

pub type EapolDataFrame<'a, KeyMic = &'a [u8], ElementContainer = ReadElements<'a>> =
    DataFrame<'a, SnapLlcFrame<'a, EapolKeyFrame<'a, KeyMic, ElementContainer>>>;

const EAPOL_2004_PROTOCOL_VERSION: u8 = 2;
const EAPOL_KEY_MESSAGE_TYPE: u8 = 3;
const EAPOL_802_11_KEY_DESCRIPTOR_TYPE: u8 = 2;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// An EAPOL key frame.
pub struct EapolKeyFrame<'a, KeyMic: AsRef<[u8]> = &'a [u8], ElementContainer = ReadElements<'a>> {
    /// Information about the Key Frame.
    pub key_information: KeyInformation,
    /// The length of the Temporal Key (TK).
    pub key_length: u16,
    /// The key replacy counter.
    pub key_replay_counter: u64,
    /// The nonce of the transmitting party.
    pub key_nonce: [u8; 32],
    /// The EAPOL Key IV.
    pub key_iv: u128,
    /// The current packet number of the transmitting party.
    pub key_rsc: u64,
    /// The key Message Integrity Check (MIC).
    pub key_mic: KeyMic,
    /// The key data.
    pub key_data: ElementContainer,
    pub _phantom: PhantomData<&'a ()>,
}
impl<'a> TryFromCtx<'a, IEEE80211AkmType> for EapolKeyFrame<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(
        from: &'a [u8],
        akm_suite: IEEE80211AkmType,
    ) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;
        let _protocol_version: u8 = from.gread(&mut offset)?;

        let packet_type: u8 = from.gread(&mut offset)?;
        if packet_type != EAPOL_KEY_MESSAGE_TYPE {
            return Err(scroll::Error::BadInput {
                size: offset,
                msg: "EAPOL frame type wasn't EAPOL-Key.",
            });
        }
        let packet_body_length: u16 = from.gread_with(&mut offset, Endian::Big)?;
        let packet_body = from.pread_with::<&[u8]>(0, packet_body_length as usize + 4)?;
        let descriptor_type: u8 = packet_body.gread_with(&mut offset, Endian::Big)?;
        if descriptor_type != EAPOL_802_11_KEY_DESCRIPTOR_TYPE {
            return Err(scroll::Error::BadInput {
                size: offset,
                msg: "EAPOL Key frame descriptor type wasn't two.",
            });
        }

        let key_information =
            KeyInformation::from_bits(packet_body.gread_with(&mut offset, Endian::Big)?);
        let key_length = packet_body.gread_with(&mut offset, Endian::Big)?;
        let key_replay_counter = packet_body.gread_with(&mut offset, Endian::Big)?;
        let key_nonce = packet_body.gread_with(&mut offset, Endian::Big)?;
        let key_iv = packet_body.gread_with(&mut offset, Endian::Big)?;
        let key_rsc = packet_body.gread_with(&mut offset, Endian::Big)?;
        offset += 8;
        let key_mic_len = akm_suite.key_mic_len().ok_or(scroll::Error::BadInput {
            size: offset,
            msg: "No MIC length available for AKM suite.",
        })?;
        let key_mic = packet_body.gread_with(&mut offset, key_mic_len)?;
        let key_data_length: u16 = packet_body.gread_with(&mut offset, Endian::Big)?;
        let key_data = packet_body.gread_with(&mut offset, key_data_length as usize)?;
        let key_data = ReadElements { bytes: key_data };

        Ok((
            Self {
                key_information,
                key_length,
                key_replay_counter,
                key_nonce,
                key_iv,
                key_rsc,
                key_mic,
                key_data,
                _phantom: PhantomData,
            },
            offset,
        ))
    }
}
impl<'a, KeyMic: AsRef<[u8]>, ElementContainer: MeasureWith<()>> MeasureWith<()>
    for EapolKeyFrame<'a, KeyMic, ElementContainer>
{
    fn measure_with(&self, ctx: &()) -> usize {
        1 + 1
            + 2
            + 1
            + 2
            + 2
            + 8
            + 32
            + 16
            + 8
            + 8
            + self.key_mic.as_ref().len()
            + 2
            + self.key_data.measure_with(ctx)
    }
}
impl<'a, KeyMic: AsRef<[u8]>, ElementContainer: TryIntoCtx<(), Error = scroll::Error>> TryIntoCtx<()>
    for EapolKeyFrame<'a, KeyMic, ElementContainer>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;
        buf.gwrite_with(EAPOL_2004_PROTOCOL_VERSION, &mut offset, Endian::Big)?;
        buf.gwrite_with(EAPOL_KEY_MESSAGE_TYPE, &mut offset, Endian::Big)?;

        let packet_body_length_offset = offset;
        offset += 2;
        buf.gwrite_with(EAPOL_802_11_KEY_DESCRIPTOR_TYPE, &mut offset, Endian::Big)?;
        buf.gwrite_with(self.key_information.into_bits(), &mut offset, Endian::Big)?;

        buf.gwrite_with(self.key_length, &mut offset, Endian::Big)?;
        buf.gwrite_with(self.key_replay_counter, &mut offset, Endian::Big)?;
        buf.gwrite_with(self.key_nonce, &mut offset, Endian::Big)?;
        buf.gwrite_with(self.key_iv, &mut offset, Endian::Big)?;
        buf.gwrite_with(self.key_rsc, &mut offset, Endian::Big)?;
        buf.gwrite_with(0u64, &mut offset, Endian::Big)?;
        buf.gwrite(self.key_mic.as_ref(), &mut offset)?;

        let key_data_length_offset = offset;
        offset += 2;
        buf.gwrite(self.key_data, &mut offset)?;

        let key_data_length = offset - key_data_length_offset - 2;
        buf.pwrite_with(key_data_length as u16, key_data_length_offset, Endian::Big)?;

        let packet_body_length = offset - packet_body_length_offset - 2;
        buf.pwrite_with(
            packet_body_length as u16,
            packet_body_length_offset,
            Endian::Big,
        )?;

        Ok(offset)
    }
}
impl<'a, KeyMic: AsRef<[u8]>, ElementContainer> EapolDataFrame<'a, KeyMic, ElementContainer> {
    /// Get the range in which the EAPOL MIC field is in the serialized data frame.
    pub fn eapol_mic_range(&self) -> Option<Range<usize>> {
        let mic_length = self.payload.as_ref()?.payload.key_mic.as_ref().len();
        let mic_start = self.header.length_in_bytes() + 8 + 1 + 1 + 2 + 1 + 2 + 2 + 8 + 32 + 16 + 8 + 8;
        Some(mic_start..mic_start + mic_length)
    }
    /// Get the range in which the EAPOL Key Data Length field is in the serialized data frame.
    pub fn eapol_key_data_length_range(&self) -> Option<Range<usize>> {
        let key_data_length_start = self.header.length_in_bytes() + 8
            + 1
            + 1
            + 2
            + 1
            + 2
            + 2
            + 8
            + 32
            + 16
            + 8
            + 8
            + self.payload.as_ref()?.payload.key_mic.as_ref().len();
        Some(key_data_length_start..key_data_length_start + 2)
    }
    /// Get the range in which the EAPOL Key Data field is in the serialized data frame.
    pub fn eapol_key_data_range(&self) -> Option<RangeFrom<usize>> {
        let key_data_start = self.header.length_in_bytes() + 8
            + 1
            + 1
            + 2
            + 1
            + 2
            + 2
            + 8
            + 32
            + 16
            + 8
            + 8
            + self.payload.as_ref()?.payload.key_mic.as_ref().len()
            + 2;
        Some(key_data_start..)
    }
}
