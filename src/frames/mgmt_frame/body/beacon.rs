use core::{marker::PhantomData, time::Duration};

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{
    common::{capabilities::CapabilitiesInformation, Empty, TU},
    elements::{Elements, SSIDElement},
};

use super::{ManagementFrameBody, ToManagementFrameBody};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub struct BeaconSubtype;
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub struct ProbeResponseSubtype;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
/// This is a generic body of a beacon like frame. This includes beacons and probe responses.
pub struct BeaconLikeFrameBody<'a, Subtype, ElementContainer = Elements<'a>> {
    pub timestamp: u64,
    pub beacon_interval: u16,
    pub capabilities_info: CapabilitiesInformation,
    pub elements: ElementContainer,
    pub _phantom: PhantomData<(&'a (), Subtype)>,
}
impl<'a, Subtype> BeaconLikeFrameBody<'a, Subtype> {
    pub const fn length_in_bytes(&'a self) -> usize {
        8 + // Timestamp
        2 + // Beacon interval
        2 + // Capabilities information
        self.elements.bytes.len()
    }
}
impl<'a, Subtype> BeaconLikeFrameBody<'a, Subtype> {
    pub const fn beacon_interval_as_duration(&self) -> Duration {
        Duration::from_micros(self.beacon_interval as u64 * TU.as_micros() as u64)
    }
    /// Extract the SSID from the tlvs.
    pub fn ssid(&'a self) -> Option<&'a str> {
        // SSID should be the first TLV.
        self.elements
            .get_first_element::<SSIDElement>()
            .map(SSIDElement::take_ssid)
    }
}
/* impl<LhsElements, RhsElements, Subtype> PartialEq<BeaconLikeFrameBody<RhsElements, Subtype>>
    for BeaconLikeFrameBody<LhsElements, Subtype>
{
    fn eq(&self, other: &BeaconLikeFrameBody<RhsElements, Subtype>) -> bool {
        self.timestamp == other.timestamp
            && self.beacon_interval == other.beacon_interval
            && self.capabilities_info == other.capabilities_info
            && self.elements
    }
} */
impl<'a, ElementContainer: MeasureWith<()>, Subtype> MeasureWith<()>
    for BeaconLikeFrameBody<'a, Subtype, ElementContainer>
{
    fn measure_with(&self, ctx: &()) -> usize {
        12 + self.elements.measure_with(ctx)
    }
}
impl<'a, Subtype: 'a> TryFromCtx<'a> for BeaconLikeFrameBody<'a, Subtype> {
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
                _phantom: PhantomData,
            },
            offset,
        ))
    }
}
impl<'a, ElementContainer: TryIntoCtx<Error = scroll::Error>, Subtype> TryIntoCtx
    for BeaconLikeFrameBody<'a, Subtype, ElementContainer>
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
    ToManagementFrameBody<'a, ElementContainer, Empty>
    for BeaconLikeFrameBody<'a, BeaconSubtype, ElementContainer>
{
    fn to_management_frame_body(self) -> ManagementFrameBody<'a, ElementContainer, Empty> {
        ManagementFrameBody::Beacon(self)
    }
}
impl<'a, ElementContainer: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>>
    ToManagementFrameBody<'a, ElementContainer, Empty>
    for BeaconLikeFrameBody<'a, ProbeResponseSubtype, ElementContainer>
{
    fn to_management_frame_body(self) -> ManagementFrameBody<'a, ElementContainer, Empty> {
        ManagementFrameBody::ProbeRespone(self)
    }
}

/// The body of a beacon frame.
///
/// This is derived from a [generic type](BeaconLikeFrameBody) over beacon like frames, since Beacons and Probe Responses have exactly the same frame format.
pub type BeaconFrameBody<'a, ElementContainer = Elements<'a>> =
    BeaconLikeFrameBody<'a, BeaconSubtype, ElementContainer>;
