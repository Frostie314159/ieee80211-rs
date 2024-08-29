use core::{fmt::Display, marker::PhantomData};

use mac_parser::MACAddress;
use scroll::{
    ctx::{MeasureWith, StrCtx, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::common::WIFI_ALLIANCE_OUI;

use super::{Element, ElementID};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pread, Pwrite)]
/// Information about the band and channel, on which the AP in OWE mode is operating.
pub struct BandAndChannelInfo {
    /// The band is encoded as a global operating class.
    pub band: u8,
    /// This is the channel number in the band specified by [Self::band].
    pub channel: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// The OWE transition mode element describes the characteristics of the second BSS operating in OWE-only mode.
///
/// The OWE transition mode works, by operating not one but two BSS, from one AP.
/// These have different BSSIDs and SSIDs.
/// The first BSS is operated in open mode and includes this element in it's beacons.
/// The information included allows the connecting STA to find the BSS operating in OWE-only mode, which is commonly using a hidden SSID, but on the same channel.
pub struct OWETransitionModeElement<'a, SSID = &'a str> {
    /// The BSSID of the OWE BSS.
    pub bssid: MACAddress,
    /// The SSID of the OWE BSS.
    pub ssid: SSID,
    /// Information about the band and operating channel.
    /// This is only present, if the OWE BSS operates on a different channel.
    pub band_and_channel_info: Option<BandAndChannelInfo>,
    pub _phantom: PhantomData<&'a ()>,
}
impl<SSID: AsRef<str>> Display for OWETransitionModeElement<'_, SSID> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut binding = f.debug_struct("OWETransitionModeElement");
        let debug_struct = binding
            .field("bssid", &self.bssid)
            .field("ssid", &self.ssid.as_ref());
        if let Some(ref band_and_channel_info) = self.band_and_channel_info {
            debug_struct.field("band_and_channel_info", band_and_channel_info)
        } else {
            debug_struct
        }
        .finish()
    }
}
impl<'a> TryFromCtx<'a> for OWETransitionModeElement<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let bssid = from.gread(&mut offset)?;
        let ssid_len = from.gread::<u8>(&mut offset)? as usize;
        let ssid = from.gread_with(&mut offset, StrCtx::Length(ssid_len))?;
        let band_and_channel_info = from.gread(&mut offset).ok();

        Ok((
            Self {
                bssid,
                ssid,
                band_and_channel_info,
                _phantom: PhantomData,
            },
            offset,
        ))
    }
}
impl<SSID: AsRef<str>> MeasureWith<()> for OWETransitionModeElement<'_, SSID> {
    fn measure_with(&self, _ctx: &()) -> usize {
        6 + 1
            + self.ssid.as_ref().len()
            + if self.band_and_channel_info.is_some() {
                2
            } else {
                0
            }
    }
}
impl<SSID: AsRef<str>> TryIntoCtx for OWETransitionModeElement<'_, SSID> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.bssid, &mut offset)?;
        buf.gwrite(self.ssid.as_ref().len() as u8, &mut offset)?;
        buf.gwrite(self.ssid.as_ref(), &mut offset)?;
        if let Some(band_and_channel_info) = self.band_and_channel_info {
            buf.gwrite(band_and_channel_info, &mut offset)?;
        }

        Ok(offset)
    }
}
impl<SSID: AsRef<str>> Element for OWETransitionModeElement<'_, SSID> {
    const ELEMENT_ID: ElementID = ElementID::VendorSpecific {
        prefix: &[
            WIFI_ALLIANCE_OUI[0],
            WIFI_ALLIANCE_OUI[1],
            WIFI_ALLIANCE_OUI[2],
            0x1c,
        ],
    };
    type ReadType<'a> = OWETransitionModeElement<'a>;
}
