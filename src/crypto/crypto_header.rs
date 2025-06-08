use macro_bits::{bit, check_bit};
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// The header used by CCMP and GCMP cryptographic encapsulation.
///
/// This currently does not support WEP and TKIP.
pub struct CryptoHeader {
    packet_number: [u8; 6],
    key_id: u8,
}
impl CryptoHeader {
    // NOTE: We calculate the values here, so it's more obvious where they come from.

    /// The largest representable packet number.
    pub const MAX_PN: u64 = 2u64.pow(48) - 1;
    /// The largest representable key ID.
    pub const MAX_KEY_ID: u8 = 2u8.pow(2) - 1;

    /// Create a new [CryptoHeader].
    ///
    /// Returns [Option::None] if `packet_number` is larger than [Self::MAX_PN] or `key_id` is
    /// larger than [Self::MAX_KEY_ID].
    pub fn new(packet_number: u64, key_id: u8) -> Option<Self> {
        Self::pn_and_key_id_valid(packet_number, key_id).then_some(Self {
            packet_number: packet_number.to_le_bytes()[..6].try_into().unwrap(),
            key_id,
        })
    }
    /// Check if the packet number and key ID are in range.
    const fn pn_and_key_id_valid(packet_number: u64, key_id: u8) -> bool {
        packet_number <= Self::MAX_PN || key_id <= Self::MAX_KEY_ID
    }
    /// Get the packet number as a [u64].
    ///
    /// This will return a number between 0 and including [Self::MAX_PN].
    pub fn packet_number(&self) -> u64 {
        let mut extended_packet_number = [0u8; 8];
        extended_packet_number[..6].copy_from_slice(self.packet_number.as_slice());
        u64::from_le_bytes(extended_packet_number)
    }
    /// Get the key ID.
    ///
    /// This will return a number between 0 and including [Self::MAX_KEY_ID].
    pub fn key_id(&self) -> u8 {
        self.key_id
    }
}
impl<'a> TryFromCtx<'a> for CryptoHeader {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let header = from.gread::<[u8; 8]>(&mut offset)?;

        let mut packet_number = [0u8; 6];
        packet_number[..2].copy_from_slice(&header[..2]);
        packet_number[2..].copy_from_slice(&header[4..]);

        if !check_bit!(header[3], bit!(5)) {
            return Err(scroll::Error::BadInput {
                size: offset,
                msg: "Ext IV bit not set.",
            });
        }
        let key_id = header[3] << 6;

        Ok((
            Self {
                packet_number,
                key_id,
            },
            offset,
        ))
    }
}
impl TryIntoCtx<()> for CryptoHeader {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(&self.packet_number[..2], &mut offset)?;
        buf.gwrite(0u8, &mut offset)?;
        buf.gwrite(bit!(5) | (self.key_id << 6), &mut offset)?;
        buf.gwrite(&self.packet_number[2..], &mut offset)?;

        Ok(offset)
    }
}
impl MeasureWith<()> for CryptoHeader {
    fn measure_with(&self, _ctx: &()) -> usize {
        8
    }
}
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// The state of the MIC in a frame.
pub enum MicState {
    /// No MIC is present.
    NotPresent,
    /// An 8 byte MIC is present.
    Short,
    // A 16 byte MIC is present.
    Long,
}
impl MicState {
    /// The length of the MIC.
    pub const fn mic_length(&self) -> usize {
        match self {
            Self::NotPresent => 0,
            Self::Short => 8,
            Self::Long => 16,
        }
    }
}
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Wrapper around a payload, which adds fields required for cryptographic algorithms.
///
/// This currently does not do any encryption or MIC calculation on it's own, but merely generates
/// the correctly layouted data and adds the CCMP/GCMP header.
pub struct CryptoWrapper<P> {
    /// The cryptographic header prepended to the payload.
    pub crypto_header: CryptoHeader,
    /// The actual payload.
    pub payload: P,
    /// The state of the MIC.
    pub mic_state: MicState,
}
impl<'a, P: TryFromCtx<'a, PayloadCtx, Error = scroll::Error>, PayloadCtx: Copy>
    TryFromCtx<'a, (MicState, PayloadCtx)> for CryptoWrapper<P>
{
    type Error = scroll::Error;
    fn try_from_ctx(
        from: &'a [u8],
        (mic_state, payload_ctx): (MicState, PayloadCtx),
    ) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let crypto_header = from.gread(&mut offset)?;
        let mic_length = mic_state.mic_length();
        let payload =
            from[offset..][..from.len() - offset - mic_length].pread_with(0, payload_ctx)?;

        Ok((
            Self {
                crypto_header,
                payload,
                mic_state,
            },
            from.len(),
        ))
    }
}
impl<P: TryIntoCtx<(), Error = scroll::Error>> TryIntoCtx<()> for CryptoWrapper<P> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        let mic_length = self.mic_state.mic_length();

        buf.gwrite(self.crypto_header, &mut offset)?;
        buf.gwrite(self.payload, &mut offset)?;
        buf[offset..][..mic_length].fill(0);
        offset += mic_length;

        Ok(offset)
    }
}
impl<P: MeasureWith<()>> MeasureWith<()> for CryptoWrapper<P> {
    fn measure_with(&self, ctx: &()) -> usize {
        self.crypto_header.measure_with(ctx)
            + self.payload.measure_with(ctx)
            + self.mic_state.mic_length()
    }
}
