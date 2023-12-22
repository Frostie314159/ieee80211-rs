use core::iter::repeat;

use macro_bits::{bit, bitfield};
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::tlvs::{supported_rates::ReadIterator, IEEE80211TLV};

bitfield! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct CapabilitiesInformation: u16 {
        pub is_ap: bool => bit!(0),
        pub is_ibss: bool => bit!(1),
        pub is_confidentially_required: bool => bit!(4),
        pub is_short_preamble_allowed: bool => bit!(5),
        pub is_spectrum_management_implemented: bool => bit!(8),
        pub is_qos_implemented: bool => bit!(9),
        pub is_short_time_slot_in_use: bool => bit!(10),
        pub is_auto_power_save_implemented: bool => bit!(11),
        pub is_radio_measurement_implemented: bool => bit!(12),
        pub is_epd_implemented: bool => bit!(13)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BeaconFrameBody<'a> {
    pub timestamp: u64,
    pub beacon_interval: u16,
    pub capabilities_info: CapabilitiesInformation,
    pub tagged_payload: &'a [u8],
}
impl<'a> BeaconFrameBody<'a> {
    pub const fn length_in_bytes(&self) -> usize {
        8 + // Timestamp
        2 + // Beacon interval
        2 + // Capabilities information
        self.tagged_payload.len()
    }
    /// Get an [Iterator] over the contained [IEEE80211TLV]s.
    pub fn tlv_iter(&'a self) -> impl Iterator<Item = IEEE80211TLV<'a, ReadIterator>> + 'a {
        repeat(()).scan(0usize, |offset, _| self.tagged_payload.gread(offset).ok())
    }
    /// Extract the SSID from the tlvs.
    pub fn ssid(&'a self) -> Option<&'a str> {
        // SSID should be the first TLV.
        self.tlv_iter().find_map(|tlv| {
            if let IEEE80211TLV::SSID(ssid_tlv) = tlv {
                Some(ssid_tlv.take_ssid())
            } else {
                None
            }
        })
    }
}
impl<'a> MeasureWith<()> for BeaconFrameBody<'a> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.length_in_bytes()
    }
}
impl<'a> TryFromCtx<'a> for BeaconFrameBody<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let timestamp = from.gread_with(&mut offset, Endian::Little)?;
        let beacon_interval = from.gread_with(&mut offset, Endian::Little)?;
        let capabilities_info = CapabilitiesInformation::from_representation(
            from.gread_with(&mut offset, Endian::Little)?,
        );
        let tagged_payload_length = from.len() - offset;
        let tagged_payload = from.gread_with(&mut offset, tagged_payload_length)?;
        Ok((
            Self {
                timestamp,
                beacon_interval,
                capabilities_info,
                tagged_payload,
            },
            offset,
        ))
    }
}
impl<'a> TryIntoCtx for BeaconFrameBody<'a> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(self.timestamp, &mut offset, Endian::Little)?;
        buf.gwrite_with(self.beacon_interval, &mut offset, Endian::Little)?;
        buf.gwrite_with(
            self.capabilities_info.to_representation(),
            &mut offset,
            Endian::Little,
        )?;
        buf.gwrite(self.tagged_payload, &mut offset)?;

        Ok(offset)
    }
}
