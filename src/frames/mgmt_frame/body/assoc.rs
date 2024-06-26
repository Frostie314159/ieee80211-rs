use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{
    common::{capabilities::CapabilitiesInformation, status_code::IEEE80211Status},
    elements::Elements,
};

/// This is the body of an association request frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AssociationRequestBody<ElementContainer> {
    pub capabilities_info: CapabilitiesInformation,
    pub listen_interval: u16,
    pub elements: ElementContainer,
}
impl<'a> AssociationRequestBody<Elements<'a>> {
    pub const fn length_in_bytes(&self) -> usize {
        4 + self.elements.bytes.len()
    }
}
impl<'a> TryFromCtx<'a> for AssociationRequestBody<Elements<'a>> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let capabilities_info =
            CapabilitiesInformation::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        let listen_interval = from.gread_with(&mut offset, Endian::Little)?;
        let elements = from.gread(&mut offset)?;

        Ok((
            Self {
                capabilities_info,
                listen_interval,
                elements,
            },
            offset,
        ))
    }
}
impl<ElementContainer: MeasureWith<()>> MeasureWith<()>
    for AssociationRequestBody<ElementContainer>
{
    fn measure_with(&self, ctx: &()) -> usize {
        4 + self.elements.measure_with(ctx)
    }
}
impl<ElementContainer: TryIntoCtx<Error = scroll::Error>> TryIntoCtx
    for AssociationRequestBody<ElementContainer>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(
            self.capabilities_info.into_bits(),
            &mut offset,
            Endian::Little,
        )?;
        buf.gwrite_with(self.listen_interval, &mut offset, Endian::Little)?;
        buf.gwrite(self.elements, &mut offset)?;

        Ok(offset)
    }
}

/// This is the body of an association response frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AssociationResponseBody<ElementContainer> {
    pub capabilities_info: CapabilitiesInformation,
    pub status_code: IEEE80211Status,
    pub association_id: u16,
    pub elements: ElementContainer,
}
impl<'a> AssociationResponseBody<Elements<'a>> {
    pub const fn length_in_bytes(&self) -> usize {
        6 + self.elements.bytes.len()
    }
}
impl<'a> TryFromCtx<'a> for AssociationResponseBody<Elements<'a>> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let capabilities_info =
            CapabilitiesInformation::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        let status_code = IEEE80211Status::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        let association_id = from.gread_with(&mut offset, Endian::Little)?;
        let elements = from.gread(&mut offset)?;

        Ok((
            Self {
                capabilities_info,
                status_code,
                association_id,
                elements,
            },
            offset,
        ))
    }
}
impl<ElementContainer: MeasureWith<()>> MeasureWith<()>
    for AssociationResponseBody<ElementContainer>
{
    fn measure_with(&self, ctx: &()) -> usize {
        6 + self.elements.measure_with(ctx)
    }
}
impl<ElementContainer: TryIntoCtx<Error = scroll::Error>> TryIntoCtx
    for AssociationResponseBody<ElementContainer>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(
            self.capabilities_info.into_bits(),
            &mut offset,
            Endian::Little,
        )?;
        buf.gwrite_with(self.status_code.into_bits(), &mut offset, Endian::Little)?;
        buf.gwrite_with(self.association_id, &mut offset, Endian::Little)?;
        buf.gwrite(self.elements, &mut offset)?;

        Ok(offset)
    }
}
