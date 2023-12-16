use core::{iter::repeat, str::FromStr};

use alloc::vec::Vec;
use mac_parser::MACAddress;
use macro_bits::{bit, bitfield};
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{
    tlvs::{
        ssid::{SSID, SSIDTLV},
        RawIEEE80211TLV, IEEE80211TLV,
    },
    FragSeqInfo,
};

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
#[derive(Debug, Clone, PartialEq)]
pub struct BeaconFrame<'a> {
    pub duration: u16,
    pub destination: MACAddress,
    pub source: MACAddress,
    pub bssid: MACAddress,
    pub frag_seq_info: FragSeqInfo,
    pub timestamp: u64,
    pub beacon_interval: u16,
    pub capabilities_info: CapabilitiesInformation,
    pub ssid: SSID,
    pub ext_tlvs: Vec<IEEE80211TLV<'a>>,
}
impl<'a> Default for BeaconFrame<'a> {
    fn default() -> Self {
        Self {
            duration: 0,
            destination: mac_parser::BROADCAST,
            source: MACAddress::default(),
            bssid: MACAddress::default(),
            frag_seq_info: FragSeqInfo::default(),
            timestamp: 0,
            beacon_interval: 100,
            capabilities_info: CapabilitiesInformation::default(),
            ssid: SSID::from_str("").unwrap(), // Wildcard
            ext_tlvs: Vec::default(),
        }
    }
}
impl<'a> MeasureWith<()> for BeaconFrame<'a> {
    fn measure_with(&self, _ctx: &()) -> usize {
        36 + self
            .ext_tlvs
            .iter()
            .map(|x| x.measure_with(&()))
            .sum::<usize>()
            + self.ssid.len()
    }
}
impl<'a> TryFromCtx<'a> for BeaconFrame<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let duration = from.gread_with(&mut offset, Endian::Little)?;
        let destination = MACAddress::new(from.gread(&mut offset)?);
        let source = MACAddress::new(from.gread(&mut offset)?);
        let bssid = MACAddress::new(from.gread(&mut offset)?);
        let frag_seq_info =
            FragSeqInfo::from_representation(from.gread_with(&mut offset, Endian::Little)?);
        let timestamp = from.gread_with(&mut offset, Endian::Little)?;
        let beacon_interval = from.gread_with::<u16>(&mut offset, Endian::Little)?;
        let capabilities_info = CapabilitiesInformation::from_representation(
            from.gread_with(&mut offset, Endian::Little)?,
        );
        let IEEE80211TLV::SSID(SSIDTLV { ssid }) =
            IEEE80211TLV::from_raw_tlv(from.gread::<RawIEEE80211TLV<'a>>(&mut offset)?)?
        else {
            return Err(scroll::Error::BadInput {
                size: offset,
                msg: "First TLV wasn't SSID.",
            });
        };
        let ext_tlvs = repeat(())
            .map_while(|_| from.gread(&mut offset).ok())
            .map(IEEE80211TLV::from_raw_tlv)
            .try_collect()?;
        Ok((
            Self {
                duration,
                destination,
                source,
                bssid,
                frag_seq_info,
                timestamp,
                beacon_interval,
                capabilities_info,
                ssid,
                ext_tlvs,
            },
            offset,
        ))
    }
}
impl<'a> TryIntoCtx for BeaconFrame<'a> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;
        buf.gwrite_with(self.duration, &mut offset, Endian::Little)?;
        buf.gwrite(self.destination.as_slice(), &mut offset)?;
        buf.gwrite(self.source.as_slice(), &mut offset)?;
        buf.gwrite(self.bssid.as_slice(), &mut offset)?;
        buf.gwrite_with(
            self.frag_seq_info.to_representation(),
            &mut offset,
            Endian::Little,
        )?;
        buf.gwrite_with(self.timestamp, &mut offset, Endian::Little)?;
        buf.gwrite_with(self.beacon_interval, &mut offset, Endian::Little)?;
        buf.gwrite_with(
            self.capabilities_info.to_representation(),
            &mut offset,
            Endian::Little,
        )?;
        offset += 1;
        buf.gwrite(self.ssid.len() as u8, &mut offset)?;
        buf.gwrite(self.ssid.as_bytes(), &mut offset)?;
        if !self.ext_tlvs.is_empty() {
            for tlv in self.ext_tlvs.into_iter().map(IEEE80211TLV::to_raw_tlv) {
                buf.gwrite(tlv?, &mut offset).unwrap();
            }
        }
        Ok(offset)
    }
}
/* impl Read for BeaconFrame {
    fn from_bytes(data: &mut impl ExactSizeIterator<Item = u8>) -> Result<Self, bin_utils::ParserError> {
        let mut header = try_take(data, 36).map_err(ParserError::TooLittleData)?;
        let destination = MACAddress::from_bytes(&header.next_chunk().unwrap())?;
        let source = MACAddress::from_bytes(&header.next_chunk().unwrap())?;
        let bssid = MACAddress::from_bytes(&header.next_chunk().unwrap())?;
    }
} */
