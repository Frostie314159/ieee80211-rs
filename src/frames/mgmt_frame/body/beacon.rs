use core::time::Duration;

use macro_bits::{bit, bitfield};
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{
    common::TU,
    elements::{
        rates::{EncodedRate, RatesReadIterator},
        ElementReadIterator, IEEE80211Element,
    },
};

use super::{ManagementFrameBody, ToManagementFrameBody};

bitfield! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// This bitfield contains the capabilities of the sender.
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
        pub is_epd_implemented: bool => bit!(13),
        pub reserved: u8 => bit!(14, 15)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// This is the body of a beacon frame.
/// The generic parameter can be any type, which implements [IntoIterator<Item = IEEE80211TLV<'_>>](IntoIterator).
/// When reading this struct the generic parameter is set to [ElementReadIterator].
pub struct BeaconFrameBody<
    'a,
    RateIterator = RatesReadIterator<'a>,
    ExtendedRateIterator = RatesReadIterator<'a>,
    ElementIterator = ElementReadIterator<'a>,
> where
    RateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ElementIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>>,
{
    pub timestamp: u64,
    pub beacon_interval: u16,
    pub capabilities_info: CapabilitiesInformation,
    pub tagged_payload: ElementIterator,
}
impl<'a> BeaconFrameBody<'a> {
    pub const fn length_in_bytes(&'a self) -> usize {
        8 + // Timestamp
        2 + // Beacon interval
        2 + // Capabilities information
        match self.tagged_payload.bytes {
            Some(bytes) => bytes.len(),
            None => 0
        }
    }
}
impl<
        'a,
        RateIterator: IntoIterator<Item = EncodedRate> + Clone,
        ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
        TLVIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>> + Clone + 'a,
    > BeaconFrameBody<'a, RateIterator, ExtendedRateIterator, TLVIterator>
{
    pub const fn beacon_interval_as_duration(&self) -> Duration {
        Duration::from_micros(self.beacon_interval as u64 * TU.as_micros() as u64)
    }
    /// Extract the SSID from the tlvs.
    pub fn ssid(&'a self) -> Option<&'a str> {
        // SSID should be the first TLV.
        self.tagged_payload.clone().into_iter().find_map(|tlv| {
            if let IEEE80211Element::SSID(ssid_tlv) = tlv {
                Some(ssid_tlv.take_ssid())
            } else {
                None
            }
        })
    }
}
impl<
        'a,
        RateIterator: IntoIterator<Item = EncodedRate> + Clone,
        ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
        TLVIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>> + Clone + 'a,
    > MeasureWith<()> for BeaconFrameBody<'a, RateIterator, ExtendedRateIterator, TLVIterator>
{
    fn measure_with(&self, ctx: &()) -> usize {
        12 + self
            .tagged_payload
            .clone()
            .into_iter()
            .map(|tlv| tlv.measure_with(ctx))
            .sum::<usize>()
    }
}
impl<'a> TryFromCtx<'a> for BeaconFrameBody<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let timestamp = from.gread_with(&mut offset, Endian::Little)?;
        let beacon_interval = from.gread_with(&mut offset, Endian::Little)?;
        let capabilities_info =
            CapabilitiesInformation::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        let tagged_payload_len = from.len() - offset;
        let tagged_payload =
            ElementReadIterator::new(from.gread_with(&mut offset, tagged_payload_len)?);
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
impl<
        'a,
        RateIterator: IntoIterator<Item = EncodedRate> + Clone,
        ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
        TLVIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>> + 'a,
    > TryIntoCtx for BeaconFrameBody<'a, RateIterator, ExtendedRateIterator, TLVIterator>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(self.timestamp, &mut offset, Endian::Little)?;
        buf.gwrite_with(self.beacon_interval, &mut offset, Endian::Little)?;
        buf.gwrite_with(
            self.capabilities_info.into_bits(),
            &mut offset,
            Endian::Little,
        )?;
        for tlv in self.tagged_payload {
            buf.gwrite(tlv, &mut offset)?;
        }

        Ok(offset)
    }
}
impl<'a, RateIterator, ExtendedRateIterator, TLVIterator>
    ToManagementFrameBody<'a, RateIterator, ExtendedRateIterator, TLVIterator>
    for BeaconFrameBody<'a, RateIterator, ExtendedRateIterator, TLVIterator>
where
    RateIterator: IntoIterator<Item = EncodedRate> + Clone,
    ExtendedRateIterator: IntoIterator<Item = EncodedRate> + Clone,
    TLVIterator: IntoIterator<Item = IEEE80211Element<'a, RateIterator, ExtendedRateIterator>>,
{
    fn to_management_frame_body(
        self,
    ) -> ManagementFrameBody<'a, RateIterator, ExtendedRateIterator, TLVIterator> {
        ManagementFrameBody::Beacon(self)
    }
}
