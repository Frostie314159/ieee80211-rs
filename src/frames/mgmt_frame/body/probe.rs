use core::marker::PhantomData;

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread,
};

use crate::{
    common::Empty,
    elements::{ReadElements, SSIDElement},
};

use super::{
    beacon::ProbeResponseSubtype, BeaconLikeFrameBody, ManagementFrameBody, ToManagementFrameBody,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// The body of a probe request.
pub struct ProbeRequestBody<'a, ElementContainer = ReadElements<'a>> {
    pub elements: ElementContainer,
    pub _phantom: PhantomData<&'a ()>,
}
impl<'a> ProbeRequestBody<'a> {
    /// The entire length in bytes.
    pub const fn length_in_bytes(&self) -> usize {
        self.elements.bytes.len()
    }
    /// Extract the SSID from the tlvs.
    pub fn ssid(&self) -> Option<&'a str> {
        // SSID should be the first TLV.
        self.elements
            .get_first_element::<SSIDElement>()
            .map(SSIDElement::take_ssid)
    }
}
impl<ElementContainer: MeasureWith<()>> MeasureWith<()> for ProbeRequestBody<'_, ElementContainer> {
    fn measure_with(&self, ctx: &()) -> usize {
        self.elements.measure_with(ctx)
    }
}
impl<'a> TryFromCtx<'a> for ProbeRequestBody<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;
        let elements = from.gread(&mut offset)?;
        Ok((
            Self {
                elements,
                _phantom: PhantomData,
            },
            offset,
        ))
    }
}
impl<ElementContainer: TryIntoCtx<Error = scroll::Error>> TryIntoCtx
    for ProbeRequestBody<'_, ElementContainer>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], ctx: ()) -> Result<usize, Self::Error> {
        <ElementContainer as TryIntoCtx>::try_into_ctx(self.elements, buf, ctx)
    }
}
impl<'a, ElementContainer: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>>
    ToManagementFrameBody<'a, ElementContainer, Empty> for ProbeRequestBody<'a, ElementContainer>
{
    fn to_management_frame_body(self) -> ManagementFrameBody<'a, ElementContainer, Empty> {
        ManagementFrameBody::ProbeRequest(self)
    }
}

pub type ProbeResponeBody<'a, ElementContainer = ReadElements<'a>> =
    BeaconLikeFrameBody<'a, ProbeResponseSubtype, ElementContainer>;
