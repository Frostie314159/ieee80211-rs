use bitfield_struct::bitfield;

#[bitfield(u32)]
/// The HT-SIG field, contained in an HT-PPDU.
pub struct HtSig {
    #[bits(7)]
    pub mcs: u8,
    pub is_40mhz: bool,
    pub ht_length: u16,
    pub smoothing_recommended: bool,
    pub not_sounding: bool,
    #[bits(1, default = 1)]
    reserved: u8,
    pub is_ampdu: bool,
    #[bits(2)]
    pub spatial_stream_count: u8,
    pub is_ldpc: bool,
    pub short_gi: bool
}
