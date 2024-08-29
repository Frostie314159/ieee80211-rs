use bitfield_struct::bitfield;

#[bitfield(u16, defmt = cfg(feature = "defmt"))]
#[derive(PartialEq, Eq, Hash)]
/// This bitfield contains the capabilities of the sender.
pub struct CapabilitiesInformation {
    pub is_ess: bool,
    pub is_ibss: bool,
    #[bits(2)]
    __: u8,
    pub is_confidentiality_required: bool,
    pub is_short_preamble_allowed: bool,
    #[bits(2)]
    __: u8,
    pub is_spectrum_management_implemented: bool,
    pub is_qos_implemented: bool,
    pub is_short_time_slot_in_use: bool,
    pub is_auto_power_save_implemented: bool,
    pub is_radio_measurement_implemented: bool,
    pub is_epd_implemented: bool,
    #[bits(2)]
    __: u8,
}
