use core::marker::PhantomData;

use mac_parser::MACAddress;
use scroll::{
    ctx::{MeasureWith, StrCtx, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::common::WIFI_ALLIANCE_OUI;

use super::{Element, ElementID};

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
    pub band_and_channel_info: Option<(u8, u8)>,
    pub _phantom: PhantomData<&'a ()>,
}
impl<'a> TryFromCtx<'a> for OWETransitionModeElement<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let bssid = from.gread(&mut offset)?;
        let ssid_len = from.gread::<u8>(&mut offset)? as usize;
        let ssid = from.gread_with(&mut offset, StrCtx::Length(ssid_len))?;
        let band_and_channel_info = if from.len() - offset >= 2 {
            Some((from.gread(&mut offset)?, from.gread(&mut offset)?))
        } else {
            None
        };

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
        if let Some((band_info, channel_info)) = self.band_and_channel_info {
            buf.gwrite(band_info, &mut offset)?;
            buf.gwrite(channel_info, &mut offset)?;
        }

        Ok(offset)
    }
}
impl<SSID: AsRef<str>> Element for OWETransitionModeElement<'_, SSID> {
    const ELEMENT_ID: ElementID = ElementID::VendorSpecific {
        oui: WIFI_ALLIANCE_OUI,
        subtype: 0x1c,
    };
    type ReadType<'a> = OWETransitionModeElement<'a>;
}
