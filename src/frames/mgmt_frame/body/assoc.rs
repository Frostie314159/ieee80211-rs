use core::marker::PhantomData;

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{
    common::{AssociationID, CapabilitiesInformation, IEEE80211StatusCode},
    elements::ReadElements,
};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// This is the body of an association request frame.
pub struct AssociationRequestBody<'a, ElementContainer = ReadElements<'a>> {
    pub capabilities_info: CapabilitiesInformation,
    pub listen_interval: u16,
    pub elements: ElementContainer,
    pub _phantom: PhantomData<&'a ()>,
}
impl<'a> AssociationRequestBody<'a> {
    /// Returns the total length in bytes.
    pub const fn length_in_bytes(&self) -> usize {
        4 + self.elements.bytes.len()
    }
}
impl<'a> TryFromCtx<'a> for AssociationRequestBody<'a> {
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
                _phantom: PhantomData,
            },
            offset,
        ))
    }
}
impl<ElementContainer: MeasureWith<()>> MeasureWith<()>
    for AssociationRequestBody<'_, ElementContainer>
{
    fn measure_with(&self, ctx: &()) -> usize {
        4 + self.elements.measure_with(ctx)
    }
}
impl<ElementContainer: TryIntoCtx<Error = scroll::Error>> TryIntoCtx
    for AssociationRequestBody<'_, ElementContainer>
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

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// This is the body of an association response frame.
pub struct AssociationResponseBody<'a, ElementContainer = ReadElements<'a>> {
    pub capabilities_info: CapabilitiesInformation,
    pub status_code: IEEE80211StatusCode,
    pub association_id: AssociationID,
    pub elements: ElementContainer,
    pub _phantom: PhantomData<&'a ()>,
}
impl<'a> AssociationResponseBody<'a> {
    pub const fn length_in_bytes(&self) -> usize {
        6 + self.elements.bytes.len()
    }
}
impl<'a> TryFromCtx<'a> for AssociationResponseBody<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let capabilities_info =
            CapabilitiesInformation::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        let status_code =
            IEEE80211StatusCode::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        let association_id = AssociationID::new_checked(
            from.gread_with(&mut offset, Endian::Little)?,
        )
        .ok_or(scroll::Error::BadInput {
            size: offset,
            msg: "Association ID is out of bounds.",
        })?;
        let elements = from.gread(&mut offset)?;

        Ok((
            Self {
                capabilities_info,
                status_code,
                association_id,
                elements,
                _phantom: PhantomData,
            },
            offset,
        ))
    }
}
impl<ElementContainer: MeasureWith<()>> MeasureWith<()>
    for AssociationResponseBody<'_, ElementContainer>
{
    fn measure_with(&self, ctx: &()) -> usize {
        6 + self.elements.measure_with(ctx)
    }
}
impl<ElementContainer: TryIntoCtx<Error = scroll::Error>> TryIntoCtx
    for AssociationResponseBody<'_, ElementContainer>
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
        buf.gwrite_with(self.association_id.into_bits(), &mut offset, Endian::Little)?;
        buf.gwrite(self.elements, &mut offset)?;

        Ok(offset)
    }
}
