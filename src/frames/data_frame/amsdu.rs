use mac_parser::MACAddress;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// A single subframe from an aggregate MSDU.
///
/// The payload is generic, to avoid the need for an intermediate step when writing.
/// It can be any type, which implements [`TryIntoCtx<Ctx = (), Error = scroll::Error>`](TryIntoCtx).
pub struct AMSDUSubframe<Payload> {
    pub destination_address: MACAddress,
    pub source_address: MACAddress,
    /// This is the payload of the subframe.
    /// Although it's generic, it's always a byte slice, when returned by parsing.
    pub payload: Payload,
}
impl AMSDUSubframe<&'_ [u8]> {
    /// Returns the length in bytes.
    /// This is currently only const for byte slices, since const traits are unstable.
    pub const fn length_in_bytes(&self) -> usize {
        14 + self.payload.len()
    }
}
impl<Payload: MeasureWith<()>> MeasureWith<()> for AMSDUSubframe<Payload> {
    fn measure_with(&self, ctx: &()) -> usize {
        14 + self.payload.measure_with(ctx)
    }
}
impl<'a> TryFromCtx<'a> for AMSDUSubframe<&'a [u8]> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let destination_address = from.gread(&mut offset)?;
        let source_address = from.gread(&mut offset)?;
        let length = from.gread_with::<u16>(&mut offset, Endian::Little)?;
        let payload = from.gread_with(&mut offset, length as usize)?;
        // Round to the nearest multiple of four.
        offset += 3;
        offset &= !0b0000_0011;
        Ok((
            Self {
                destination_address,
                source_address,
                payload,
            },
            offset,
        ))
    }
}
impl<Payload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>> TryIntoCtx
    for AMSDUSubframe<Payload>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.destination_address, &mut offset)?;
        buf.gwrite(self.source_address, &mut offset)?;
        buf.gwrite_with(
            self.payload.measure_with(&()) as u16,
            &mut offset,
            Endian::Little,
        )?;
        buf.gwrite(self.payload, &mut offset)?;
        // Round to the nearest multiple of four.
        offset += 3;
        offset &= !0b0000_0011;

        Ok(offset)
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// An iterator over the subframes of an A-MSDU.
///
/// This internally keeps the bytes slice and the offset and returns [Some] until [scroll] returns an error.
/// This has the side effect, that if an error is encoutered while reading, the iterator may stop early, even if data is still left.
pub struct AMSDUSubframeIterator<'a> {
    // Making this an option comes with the advantage, that after encoutering an error, subsequent iterations will be almost instant.
    pub(crate) bytes: Option<&'a [u8]>,
}
impl<'a> AMSDUSubframeIterator<'a> {
    /// Initializes the iterator with the offset set to zero.
    pub const fn from_bytes(bytes: &'a [u8]) -> Self {
        Self { bytes: Some(bytes) }
    }
    /// Returns the complete length in bytes.
    pub const fn length_in_bytes(&self) -> usize {
        match self.bytes {
            Some(bytes) => bytes.len(),
            None => 0,
        }
    }
}
impl<'a> Iterator for AMSDUSubframeIterator<'a> {
    type Item = AMSDUSubframe<&'a [u8]>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut offset = 0;
        let bytes = self.bytes?;
        let sub_frame = bytes.gread(&mut offset).ok();
        match sub_frame {
            Some(sub_frame) => {
                self.bytes = Some(&bytes[offset..]);
                Some(sub_frame)
            }
            None => {
                self.bytes = None;
                None
            }
        }
    }
}
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// This can be used for writing an aggregate MSDU.
///
/// The generic paramter can be any type, which implements [IntoIterator].
/// It can only be written.
pub struct AMSDUPayload<Frames> {
    pub sub_frames: Frames,
}
impl<'a> AMSDUPayload<&'a [AMSDUSubframe<&'a [u8]>]> {
    /// Returns the total length in bytes.
    pub const fn length_in_bytes(&self) -> usize {
        let mut size = 0;
        let mut i = 0;
        while i != self.sub_frames.len() {
            size += self.sub_frames[i].length_in_bytes();
            i += 1;
        }
        size
    }
}
impl<'a, Frames: IntoIterator<Item = &'a Payload> + Clone, Payload: MeasureWith<()> + 'a>
    MeasureWith<()> for AMSDUPayload<Frames>
{
    fn measure_with(&self, _ctx: &()) -> usize {
        self.sub_frames
            .clone()
            .into_iter()
            .map(|sub_frame| sub_frame.measure_with(&()))
            .sum()
    }
}
impl<Frames: IntoIterator<Item = Payload>, Payload: Copy + TryIntoCtx<Error = scroll::Error>>
    TryIntoCtx for AMSDUPayload<Frames>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;
        for sub_frame in self.sub_frames.into_iter() {
            buf.gwrite(sub_frame, &mut offset)?;
        }
        Ok(offset)
    }
}
