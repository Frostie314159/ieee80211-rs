use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{common::reason::IEEE80211Reason, elements::Elements};

/// This is the body of a deauthentication frame.
pub struct DeauthenticationBody<ElementContainer> {
    /// The reason for the deauthentication.
    pub reason: IEEE80211Reason,
    pub elements: ElementContainer,
}
impl<'a> DeauthenticationBody<Elements<'a>> {
    /// The total length in bytes.
    pub const fn length_in_bytes(&self) -> usize {
        2 + self.elements.bytes.len()
    }
}
impl<'a> TryFromCtx<'a> for DeauthenticationBody<Elements<'a>> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let reason = IEEE80211Reason::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        let elements = from.gread(&mut offset)?;

        Ok((Self { reason, elements }, offset))
    }
}
impl<ElementContainer: TryIntoCtx<Error = scroll::Error>> TryIntoCtx
    for DeauthenticationBody<ElementContainer>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(self.reason.into_bits(), &mut offset, Endian::Little)?;
        buf.gwrite(self.elements, &mut offset)?;

        Ok(offset)
    }
}
impl<ElementContainer: MeasureWith<()>> MeasureWith<()> for DeauthenticationBody<ElementContainer> {
    fn measure_with(&self, ctx: &()) -> usize {
        2 + self.elements.measure_with(ctx)
    }
}
