use core::marker::PhantomData;

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::common::{attach_fcs, strip_and_validate_fcs, DataFrameSubtype, FrameType};

use self::{amsdu::AMSDUSubframeIterator, header::DataFrameHeader};

use super::IEEE80211Frame;

/// This contains types related to aggregate MSDUs.
pub mod amsdu;
/// This is a type state based data frame builder.
pub mod builder;
/// This contains the header.
pub mod header;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// This is the payload of a data frame.
/// The payload can be either one chunk or multiple aggregate MSDU subframes.
pub enum DataFrameReadPayload<'a> {
    Single(&'a [u8]),
    AMSDU(AMSDUSubframeIterator<'a>),
}
impl DataFrameReadPayload<'_> {
    /// The total length in bytes.
    pub const fn length_in_bytes(&self) -> usize {
        match self {
            Self::Single(bytes) => bytes.len(),
            Self::AMSDU(amsdu_sub_frame_iter) => amsdu_sub_frame_iter.length_in_bytes(),
        }
    }
}
impl MeasureWith<()> for DataFrameReadPayload<'_> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.length_in_bytes()
    }
}
impl<'a> TryFromCtx<'a, bool> for DataFrameReadPayload<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], is_amsdu: bool) -> Result<(Self, usize), Self::Error> {
        Ok((
            if is_amsdu {
                Self::Single(from)
            } else {
                Self::AMSDU(AMSDUSubframeIterator::from_bytes(from))
            },
            from.len(),
        ))
    }
}
impl TryIntoCtx for DataFrameReadPayload<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        match self {
            Self::Single(data) => buf.pwrite(data, 0),
            Self::AMSDU(data) => buf.pwrite(data.bytes.unwrap_or_default(), 0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// This is a data frame. The individual subtypes don't have there own seperate structs, since the only difference is the header.
pub struct DataFrame<'a, DataFramePayload = DataFrameReadPayload<'a>> {
    /// This is the header of the data frame.
    pub header: DataFrameHeader,
    /// This is the payload of the data frame.
    /// It will be set to [None] for all null function frames.
    pub payload: Option<DataFramePayload>,

    pub _phantom: PhantomData<&'a ()>,
}
impl DataFrame<'_> {
    /// The total length in bytes.
    pub const fn length_in_bytes(&self) -> usize {
        self.header.length_in_bytes()
            + if let Some(payload) = self.payload {
                payload.length_in_bytes()
            } else {
                0
            }
    }
}
impl<DataFramePayload: MeasureWith<()>> MeasureWith<()> for DataFrame<'_, DataFramePayload> {
    fn measure_with(&self, ctx: &()) -> usize {
        self.header.length_in_bytes()
            + if let Some(payload) = self.payload.as_ref() {
                payload.measure_with(ctx)
            } else {
                0
            }
    }
}
impl<'a> TryFromCtx<'a, bool> for DataFrame<'a, DataFrameReadPayload<'a>> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], with_fcs: bool) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let from = if with_fcs {
            strip_and_validate_fcs(from)?
        } else {
            from
        };
        let header: DataFrameHeader = from.gread(&mut offset)?;
        let payload = if header.subtype.has_payload() {
            let len = from.len() - offset;
            Some(DataFrameReadPayload::Single(
                from.gread_with(&mut offset, len)?,
            ))
        } else {
            None
        };
        Ok((
            Self {
                header,
                payload,
                _phantom: PhantomData,
            },
            offset,
        ))
    }
}
impl<Payload: TryIntoCtx<Error = scroll::Error>> TryIntoCtx for DataFrame<'_, Payload> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.header, &mut offset)?;
        if let Some(payload) = self.payload {
            buf.gwrite(payload, &mut offset)?;
        }
        attach_fcs(buf, &mut offset)?;
        Ok(offset)
    }
}
impl<'a, Payload> IEEE80211Frame for DataFrame<'a, Payload> {
    const TYPE: FrameType = FrameType::Data(DataFrameSubtype::Data);
    const MATCH_ONLY_TYPE: bool = true;
}
