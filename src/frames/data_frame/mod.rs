use core::marker::PhantomData;

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{
    common::{attach_fcs, strip_and_validate_fcs, DataFrameSubtype, FrameControlField, FrameType},
    crypto::{CryptoHeader, CryptoWrapper, MicState},
};

use self::{amsdu::AMSDUSubframeIterator, header::DataFrameHeader};

use super::IEEE80211Frame;

/// This contains types related to aggregate MSDUs.
pub mod amsdu;
pub mod builder;
/// This contains the header.
pub mod header;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
                Self::AMSDU(AMSDUSubframeIterator::from_bytes(from))
            } else {
                Self::Single(from)
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

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A payload, that may be crypto wrapped.
pub enum PotentiallyWrappedPayload<P> {
    /// A non crypto wrapped payload.
    Unwrapped(P),
    /// A crypto wrapped payload.
    CryptoWrapped(CryptoWrapper<P>),
}
impl<P> PotentiallyWrappedPayload<P> {
    /// Get a reference to the inner payload.
    pub const fn payload(&self) -> &P {
        match self {
            Self::Unwrapped(payload) | Self::CryptoWrapped(CryptoWrapper { payload, .. }) => {
                payload
            }
        }
    }
    /// Get a mutable reference to the inner payload.
    pub const fn payload_mut(&mut self) -> &mut P {
        match self {
            Self::Unwrapped(payload) | Self::CryptoWrapped(CryptoWrapper { payload, .. }) => {
                payload
            }
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// This is a data frame.
///
/// The individual subtypes don't have there own seperate structs, since the only difference is the header.
pub struct DataFrame<'a, DataFramePayload = &'a [u8]> {
    /// This is the header of the data frame.
    pub header: DataFrameHeader,
    /// This is the payload of the data frame.
    /// It will be set to [None] for all null function frames.
    /// NOTE: This may be crypto wrapped, so using [DataFrame::potentially_wrapped_payload] is
    /// recommended.
    pub payload: Option<DataFramePayload>,

    pub _phantom: PhantomData<&'a ()>,
}
impl DataFrame<'_> {
    /// The total length in bytes.
    pub const fn length_in_bytes(&self) -> usize {
        self.header.length_in_bytes()
            + if let Some(payload) = self.payload {
                payload.len()
            } else {
                0
            }
    }
    /// Get the potentially wrapped inner payload.
    ///
    /// If the payload is protected, but `mic_state` is None, None will be returned.
    pub fn potentially_wrapped_payload(
        &self,
        mic_state: Option<MicState>,
    ) -> Option<PotentiallyWrappedPayload<DataFrameReadPayload<'_>>> {
        let payload = self.payload?;
        Some(if self.header.fcf_flags.protected() {
            PotentiallyWrappedPayload::CryptoWrapped(
                payload
                    .pread_with(0, (mic_state?, self.header.is_amsdu()))
                    .unwrap(),
            )
        } else {
            PotentiallyWrappedPayload::Unwrapped(
                payload.pread_with(0, self.header.is_amsdu()).ok()?,
            )
        })
    }
}
impl<'a, P> DataFrame<'a, P> {
    /// Wrap the payload in a [CryptoWrapper].
    pub fn crypto_wrap(
        self,
        crypto_header: CryptoHeader,
        mic_state: MicState,
    ) -> DataFrame<'a, CryptoWrapper<P>> {
        DataFrame {
            header: DataFrameHeader {
                fcf_flags: self.header.fcf_flags.with_protected(true),
                ..self.header
            },
            payload: self.payload.map(|payload| CryptoWrapper {
                crypto_header,
                payload,
                mic_state,
            }),
            _phantom: self._phantom,
        }
    }
}
impl<DataFramePayload: MeasureWith<()>> MeasureWith<bool> for DataFrame<'_, DataFramePayload> {
    fn measure_with(&self, with_fcs: &bool) -> usize {
        self.header.length_in_bytes()
            + if let Some(payload) = self.payload.as_ref() {
                payload.measure_with(&())
            } else {
                0
            }
            + if *with_fcs { 4 } else { 0 }
    }
}
impl<'a> TryFromCtx<'a, bool> for DataFrame<'a> {
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
            Some(&from[offset..])
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
impl<Payload: TryIntoCtx<Error = scroll::Error>> TryIntoCtx<bool> for DataFrame<'_, Payload> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], with_fcs: bool) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(
            FrameControlField::new()
                .with_frame_type(FrameType::Data(self.header.subtype))
                .with_flags(self.header.fcf_flags)
                .into_bits(),
            &mut offset,
            Endian::Little,
        )?;
        buf.gwrite(self.header, &mut offset)?;
        if let Some(payload) = self.payload {
            buf.gwrite(payload, &mut offset)?;
        }
        if with_fcs {
            attach_fcs(buf, &mut offset)?;
        }
        Ok(offset)
    }
}
impl<Payload> IEEE80211Frame for DataFrame<'_, Payload> {
    const TYPE: FrameType = FrameType::Data(DataFrameSubtype::Data);
}
