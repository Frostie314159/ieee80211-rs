use core::time::Duration;

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{
    common::{capabilities::CapabilitiesInformation, Empty, TU},
    elements::{types::SSIDRepr, Elements, SSIDElement},
};

use super::{ManagementFrameBody, ToManagementFrameBody};

/// This is the body of a beacon frame.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub struct BeaconFrameBody<ElementContainer> {
    pub timestamp: u64,
    pub beacon_interval: u16,
    pub capabilities_info: CapabilitiesInformation,
    pub elements: ElementContainer,
}
impl<'a> BeaconFrameBody<Elements<'a>> {
    pub const fn length_in_bytes(&'a self) -> usize {
        8 + // Timestamp
        2 + // Beacon interval
        2 + // Capabilities information
        self.elements.bytes.len()
    }
}
impl<'a> BeaconFrameBody<Elements<'a>> {
    pub const fn beacon_interval_as_duration(&self) -> Duration {
        Duration::from_micros(self.beacon_interval as u64 * TU.as_micros() as u64)
    }
    /// Extract the SSID from the tlvs.
    pub fn ssid(&'a self) -> Option<&'a str> {
        // SSID should be the first TLV.
        self.elements
            .get_first_element::<SSIDRepr>()
            .map(SSIDElement::take_ssid)
    }
}
impl<ElementContainer: MeasureWith<()>> MeasureWith<()> for BeaconFrameBody<ElementContainer> {
    fn measure_with(&self, ctx: &()) -> usize {
        12 + self.elements.measure_with(ctx)
    }
}
impl<'a> TryFromCtx<'a> for BeaconFrameBody<Elements<'a>> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let timestamp = from.gread_with(&mut offset, Endian::Little)?;
        let beacon_interval = from.gread_with(&mut offset, Endian::Little)?;
        let capabilities_info =
            CapabilitiesInformation::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        let elements = from.gread(&mut offset)?;

        Ok((
            Self {
                timestamp,
                beacon_interval,
                capabilities_info,
                elements,
            },
            offset,
        ))
    }
}
impl<ElementContainer: TryIntoCtx<Error = scroll::Error>> TryIntoCtx
    for BeaconFrameBody<ElementContainer>
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
        buf.gwrite(self.elements, &mut offset)?;

        Ok(offset)
    }
}
impl<'a, ElementContainer: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>>
    ToManagementFrameBody<'a, ElementContainer, Empty> for BeaconFrameBody<ElementContainer>
{
    fn to_management_frame_body(self) -> ManagementFrameBody<'a, ElementContainer, Empty> {
        ManagementFrameBody::Beacon(self)
    }
}
