use core::marker::PhantomData;

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{
    common::{IEEE80211AuthenticationAlgorithmNumber, IEEE80211StatusCode},
    elements::ReadElements,
};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// This is the body of an authentication frame.
///
/// # Note
/// This is currently only valid for open system authentication, since I haven't worked out a good way for other options yet.
pub struct AuthenticationBody<'a, ElementContainer = ReadElements<'a>> {
    pub authentication_algorithm_number: IEEE80211AuthenticationAlgorithmNumber,
    pub authentication_transaction_sequence_number: u16,
    pub status_code: IEEE80211StatusCode,
    pub elements: ElementContainer,
    pub _phantom: PhantomData<&'a ()>,
}
impl<'a> AuthenticationBody<'a> {
    /// Returns the total length in bytes.
    pub const fn length_in_bytes(&self) -> usize {
        6 + self.elements.bytes.len()
    }
}
impl<'a> TryFromCtx<'a> for AuthenticationBody<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let authentication_algorithm_number = IEEE80211AuthenticationAlgorithmNumber::from_bits(
            from.gread_with(&mut offset, Endian::Little)?,
        );
        let authentication_transaction_sequence_number =
            from.gread_with(&mut offset, Endian::Little)?;
        let status_code =
            IEEE80211StatusCode::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        let elements = from.gread(&mut offset)?;

        Ok((
            Self {
                authentication_algorithm_number,
                authentication_transaction_sequence_number,
                status_code,
                elements,
                _phantom: PhantomData,
            },
            offset,
        ))
    }
}
impl<'a, ElementContainer: TryIntoCtx<Error = scroll::Error>> TryIntoCtx
    for AuthenticationBody<'a, ElementContainer>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(
            self.authentication_algorithm_number.into_bits(),
            &mut offset,
            Endian::Little,
        )?;
        buf.gwrite_with(
            self.authentication_transaction_sequence_number,
            &mut offset,
            Endian::Little,
        )?;
        buf.gwrite_with(self.status_code.into_bits(), &mut offset, Endian::Little)?;
        buf.gwrite(self.elements, &mut offset)?;

        Ok(offset)
    }
}
impl<'a, ElementContainer: MeasureWith<()>> MeasureWith<()>
    for AuthenticationBody<'a, ElementContainer>
{
    fn measure_with(&self, ctx: &()) -> usize {
        6 + self.elements.measure_with(ctx)
    }
}
