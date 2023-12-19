use mac_parser::MACAddress;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AMSDUSubframe<'a> {
    pub destination_address: MACAddress,
    pub source_address: MACAddress,
    pub payload: &'a [u8],
}
impl AMSDUSubframe<'_> {
    pub const FIXED_HEADER_LENGTH: usize = 14;
    pub const fn length_in_bytes(&self) -> usize {
        Self::FIXED_HEADER_LENGTH + self.payload.len()
    }
}
impl MeasureWith<()> for AMSDUSubframe<'_> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.length_in_bytes()
    }
}
impl<'a> TryFromCtx<'a> for AMSDUSubframe<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let destination_address = from.gread(&mut offset)?;
        let source_address = from.gread(&mut offset)?;
        let length = from.gread_with::<u16>(&mut offset, Endian::Little)?;
        let payload = from.gread_with(&mut offset, length as usize)?;
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
impl TryIntoCtx for AMSDUSubframe<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.destination_address, &mut offset)?;
        buf.gwrite(self.source_address, &mut offset)?;
        buf.gwrite_with(self.payload.len() as u16, &mut offset, Endian::Little)?;
        buf.gwrite(self.payload, &mut offset)?;

        Ok(offset)
    }
}

// An iterator over the sub frames of an A-MSDU.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AMSDUSubframeIterator<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> AMSDUSubframeIterator<'a> {
    pub const fn from_bytes(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
}
impl<'a> Iterator for AMSDUSubframeIterator<'a> {
    type Item = AMSDUSubframe<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.bytes.gread(&mut self.offset).ok()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// Used for writting data frames.
pub struct AMSDUPayload<'a> {
    pub sub_frames: &'a [AMSDUSubframe<'a>],
}
impl AMSDUPayload<'_> {
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
impl MeasureWith<()> for AMSDUPayload<'_> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.length_in_bytes()
    }
}
impl TryIntoCtx for AMSDUPayload<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;
        for sub_frame in self.sub_frames {
            buf.gwrite(*sub_frame, &mut offset)?;
        }
        Ok(offset)
    }
}
