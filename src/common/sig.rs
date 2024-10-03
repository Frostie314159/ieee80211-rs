use bitfield_struct::bitfield;

#[bitfield(u32, defmt = cfg(feature = "defmt"))]
#[derive(PartialEq, Eq, Hash)]
/// The HT-SIG field, contained in the HT preamble.
///
/// NOTE: The N_ESS, CRC and tail bits fields are currently missing, since array backed bitfields
/// aren't supported yet.
pub struct HtSig {
    #[bits(7)]
    /// An index into the HT-MCS table, used by the current transmission.
    pub mcs: u8,
    /// Indicates, wether this is a 40MHz transmission or not.
    pub is_40mhz: bool,
    /// The length of the PPDU.
    pub ht_length: u16,
    /// Indicates, wether channel estimate smoothing is recommended.
    pub smoothing_recommended: bool,
    /// Indicates, wether this is a sounding PPDU or not.
    pub not_sounding: bool,
    #[bits(1, default = 1)]
    reserved: u8,
    /// Indicates, wether this is an A-MPDU or not.
    pub is_ampdu: bool,
    #[bits(2)]
    /// Indicates the amount of spatial streams used.
    pub spatial_stream_count: u8,
    /// If true indicates LDPC coding, otherwise BCC coding.
    pub is_ldpc: bool,
    /// Indicates, wether this transmission uses short or long GI.
    pub short_gi: bool,
}
