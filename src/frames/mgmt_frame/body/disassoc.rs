use core::marker::PhantomData;

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{common::IEEE80211Reason, elements::ReadElements};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// A frame sent to disassociate from a BSS.
pub struct DisassociationBody<'a, ElementContainer = ReadElements<'a>> {
    /// The reason for the disassociation.
    pub reason: IEEE80211Reason,
    /// These are the tagged parameters of the frame body.
    pub elements: ElementContainer,
    pub _phantom: PhantomData<&'a ()>,
}
impl DisassociationBody<'_> {
    /// Returns the total length in bytes.
    pub const fn length_in_bytes(&self) -> usize {
        2 + self.elements.bytes.len()
    }
}
impl<ElementContainer: MeasureWith<()>> MeasureWith<()>
    for DisassociationBody<'_, ElementContainer>
{
    fn measure_with(&self, ctx: &()) -> usize {
        2 + self.elements.measure_with(ctx)
    }
}
impl<'a> TryFromCtx<'a> for DisassociationBody<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let reason = IEEE80211Reason::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        let elements = from.gread(&mut offset)?;

        Ok((
            Self {
                reason,
                elements,
                _phantom: PhantomData,
            },
            offset,
        ))
    }
}
impl<ElementContainer: TryIntoCtx<Error = scroll::Error>> TryIntoCtx
    for DisassociationBody<'_, ElementContainer>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(self.reason.into_bits(), &mut offset, Endian::Little)?;
        buf.gwrite(self.elements, &mut offset)?;

        Ok(offset)
    }
}
