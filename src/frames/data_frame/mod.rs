use either::Either;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::common::{subtypes::DataFrameSubtype, FCFFlags};

use self::{
    amsdu::{AMSDUPayload, AMSDUSubframeIterator},
    header::DataFrameHeader,
};

pub mod amsdu;
pub mod builder;
pub mod header;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataFramePayload<'a> {
    Single(&'a [u8]),
    AMSDU(AMSDUPayload<'a>),
}
impl DataFramePayload<'_> {
    pub const fn length_in_bytes(&self) -> usize {
        match self {
            Self::Single(slice) => slice.len(),
            Self::AMSDU(amsdu_payload) => amsdu_payload.length_in_bytes(),
        }
    }
}
impl TryIntoCtx for DataFramePayload<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        match self {
            DataFramePayload::Single(slice) => buf.pwrite(slice, 0),
            DataFramePayload::AMSDU(amsdu_payload) => {
                let mut offset = 0;
                for amsdu_sub_frame in amsdu_payload.sub_frames {
                    buf.gwrite(*amsdu_sub_frame, &mut offset)?;
                }
                Ok(offset)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataFrame<'a> {
    pub header: DataFrameHeader,
    payload: Option<DataFramePayload<'a>>,
}
impl DataFrame<'_> {
    pub const fn length_in_bytes(&self) -> usize {
        self.header.length_in_bytes()
            + if let Some(payload) = self.payload {
                payload.length_in_bytes()
            } else {
                0
            }
    }
    pub fn payload<'a>(&'a self) -> Option<Either<&'a [u8], AMSDUSubframeIterator<'a>>> {
        if let Some(DataFramePayload::Single(payload)) = self.payload {
            Some(if self.header.is_amsdu() {
                Either::Right(AMSDUSubframeIterator::from_bytes(payload))
            } else {
                Either::Left(payload)
            })
        } else {
            None
        }
    }
}
impl MeasureWith<()> for DataFrame<'_> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.length_in_bytes()
    }
}
impl<'a> TryFromCtx<'a, (DataFrameSubtype, FCFFlags)> for DataFrame<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(
        from: &'a [u8],
        (subtype, fcf_flags): (DataFrameSubtype, FCFFlags),
    ) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let header: DataFrameHeader = from.gread_with(&mut offset, (subtype, fcf_flags))?;
        let payload = if header.subtype.has_payload() {
            let len = from.len() - offset;
            Some(DataFramePayload::Single(from.gread_with(&mut offset, len)?))
        } else {
            None
        };
        Ok((Self { header, payload }, offset))
    }
}
impl TryIntoCtx for DataFrame<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.header, &mut offset)?;
        if let Some(payload) = self.payload {
            buf.gwrite(payload, &mut offset)?;
        }
        Ok(offset)
    }
}
