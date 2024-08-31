use core::marker::PhantomData;

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{common::IEEE80211Reason, elements::ReadElements};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// This is the body of a deauthentication frame.
pub struct DeauthenticationBody<'a, ElementContainer = ReadElements<'a>> {
    /// The reason for the deauthentication.
    pub reason: IEEE80211Reason,
    /// These are the tagged parameters of the frame body.
    pub elements: ElementContainer,
    pub _phantom: PhantomData<&'a ()>,
}
impl<'a> DeauthenticationBody<'a> {
    /// Returns the total length in bytes.
    pub const fn length_in_bytes(&self) -> usize {
        2 + self.elements.bytes.len()
    }
}
impl<'a> TryFromCtx<'a> for DeauthenticationBody<'a> {
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
impl<'a, ElementContainer: TryIntoCtx<Error = scroll::Error>> TryIntoCtx
    for DeauthenticationBody<'a, ElementContainer>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(self.reason.into_bits(), &mut offset, Endian::Little)?;
        buf.gwrite(self.elements, &mut offset)?;

        Ok(offset)
    }
}
impl<'a, ElementContainer: MeasureWith<()>> MeasureWith<()>
    for DeauthenticationBody<'a, ElementContainer>
{
    fn measure_with(&self, ctx: &()) -> usize {
        2 + self.elements.measure_with(ctx)
    }
}
