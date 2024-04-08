use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{common::reason::IEEE80211Reason, elements::Elements};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// A frame sent to disassociate from a BSS.
pub struct DisassociationFrameBody<ElementContainer> {
    /// The reason for the disassociation.
    pub reason: IEEE80211Reason,
    pub body: ElementContainer,
}
impl DisassociationFrameBody<Elements<'_>> {
    pub const fn length_in_bytes(&self) -> usize {
        2 + self.body.bytes.len()
    }
}
impl<ElementContainer: MeasureWith<()>> MeasureWith<()>
    for DisassociationFrameBody<ElementContainer>
{
    fn measure_with(&self, ctx: &()) -> usize {
        2 + self.body.measure_with(ctx)
    }
}
impl<'a> TryFromCtx<'a> for DisassociationFrameBody<Elements<'a>> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let reason = IEEE80211Reason::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        let body = Elements {
            bytes: &from[offset..],
        };

        Ok((Self { reason, body }, offset))
    }
}
impl<ElementContainer: TryIntoCtx<Error = scroll::Error>> TryIntoCtx
    for DisassociationFrameBody<ElementContainer>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(self.reason.into_bits(), &mut offset, Endian::Little)?;
        buf.gwrite(self.body, &mut offset)?;

        Ok(offset)
    }
}
