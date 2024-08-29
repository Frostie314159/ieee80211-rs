use core::ops::{Deref, DerefMut};

use body::{
    ActionBody, AssociationRequestBody, AssociationResponseBody, AuthenticationBody, BeaconBody,
    DeauthenticationBody, DisassociationBody, ManagementFrameBody, ProbeRequestBody,
    ProbeResponseBody,
};
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{
    common::{attach_fcs, strip_and_validate_fcs, FCFFlags, FrameControlField, FrameType},
    elements::{Element, ReadElements, WrappedIEEE80211Element},
    IEEE80211Frame,
};

use self::header::ManagementFrameHeader;

pub mod body;
pub mod header;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// A generic management frame.
pub struct ManagementFrame<Body> {
    pub header: ManagementFrameHeader,
    pub body: Body,
}
impl<Body: TryIntoCtx<Error = scroll::Error> + ManagementFrameBody> ManagementFrame<Body> {
    /// Create a [DynamicManagementFrame] from a statically typed one.
    pub fn into_dynamic(
        self,
        buffer: &mut [u8],
    ) -> Result<DynamicManagementFrame<'_>, scroll::Error> {
        DynamicManagementFrame::new(self, buffer)
    }
}
impl<Body: ManagementFrameBody> IEEE80211Frame for ManagementFrame<Body> {
    const TYPE: FrameType = FrameType::Management(Body::SUBTYPE);
    const MATCH_ONLY_TYPE: bool = false;
}
impl<Body: MeasureWith<()>> MeasureWith<bool> for ManagementFrame<Body> {
    fn measure_with(&self, with_fcs: &bool) -> usize {
        2 + self.header.length_in_bytes()
            + self.body.measure_with(&())
            + if *with_fcs { 4 } else { 0 }
    }
}
impl<'a, Ctx: Copy, Body: TryFromCtx<'a, Ctx, Error = scroll::Error>> TryFromCtx<'a, (bool, Ctx)>
    for ManagementFrame<Body>
{
    type Error = scroll::Error;
    fn try_from_ctx(
        from: &'a [u8],
        (with_fcs, body_ctx): (bool, Ctx),
    ) -> Result<(Self, usize), Self::Error> {
        // We don't care about the FCF, since the information is already encoded in the type.
        let mut offset = 1;

        let from = if with_fcs {
            strip_and_validate_fcs(from)?
        } else {
            from
        };
        let fcf_flags = FCFFlags::from_bits(from.gread(&mut offset)?);
        let header = from.gread_with(&mut offset, fcf_flags)?;
        let body = from.gread_with(&mut offset, body_ctx)?;

        Ok((Self { header, body }, offset))
    }
}
impl<Body: TryIntoCtx<Error = scroll::Error> + ManagementFrameBody> TryIntoCtx<bool>
    for ManagementFrame<Body>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], with_fcs: bool) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(
            FrameControlField::new()
                .with_frame_type(<Self as IEEE80211Frame>::TYPE)
                .with_flags(self.header.fcf_flags)
                .into_bits(),
            &mut offset,
            Endian::Little,
        )?;
        buf.gwrite(self.header, &mut offset)?;
        buf.gwrite(self.body, &mut offset)?;
        if with_fcs {
            attach_fcs(buf, &mut offset)?;
        }

        Ok(offset)
    }
}
impl<Body> Deref for ManagementFrame<Body> {
    type Target = Body;
    fn deref(&self) -> &Self::Target {
        &self.body
    }
}
impl<Body> DerefMut for ManagementFrame<Body> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.body
    }
}
macro_rules! mgmt_frames {
    (
        $(
            $(
                #[$frame_meta:meta]
            )*
            $frame:ident => $frame_body:ident
        ),*
    ) => {
        $(
            $(
                #[$frame_meta]
            )*
            pub type $frame<'a, ElementContainer = ReadElements<'a>> = ManagementFrame<$frame_body<'a, ElementContainer>>;
        )*
    };
}
mgmt_frames! {
    AssociationRequestFrame => AssociationRequestBody,
    AssociationResponseFrame => AssociationResponseBody,
    ProbeRequestFrame => ProbeRequestBody,
    ProbeResponseFrame => ProbeResponseBody,
    BeaconFrame => BeaconBody,
    DisassociationFrame => DisassociationBody,
    AuthenticationFrame => AuthenticationBody,
    DeauthenticationFrame => DeauthenticationBody
}
pub type ActionFrame<'a, VendorSpecificPayload = &'a [u8]> =
    ManagementFrame<ActionBody<'a, VendorSpecificPayload>>;

#[derive(Debug, PartialEq, Eq, Hash)]
/// A dynamic management frame.
///
/// This frame allows writing a frame, with a fixed header and set of elements, and dynamically adding [Elements](Element) to it.
/// One potential use case for this is, generating a [BeaconFrame] and optionally for example a channel switch announcement element.
pub struct DynamicManagementFrame<'a> {
    buffer: &'a mut [u8],
    offset: usize,
}
impl<'a> DynamicManagementFrame<'a> {
    /// Create a new dynamic frame.
    ///
    /// This writes the frame into the buffer.
    pub fn new(
        frame: impl TryIntoCtx<bool, Error = scroll::Error>,
        buffer: &'a mut [u8],
    ) -> Result<Self, scroll::Error> {
        let offset = buffer.pwrite(frame, 0)?;
        Ok(Self { buffer, offset })
    }
    /// Attach an element to the frame body.
    ///
    /// This will return an error, if writing the element failed.
    pub fn add_element(&mut self, element: impl Element) -> Result<(), scroll::Error> {
        self.buffer
            .gwrite(WrappedIEEE80211Element(element), &mut self.offset)?;
        Ok(())
    }
    /// Finish writing the dynamic frame.
    ///
    /// # Returns
    /// If `with_fcs` is `true` and the remaining length of the buffer is less then four, an error will be returned.
    /// Otherwise, this will always return [Ok].
    pub fn finish(mut self, with_fcs: bool) -> Result<usize, scroll::Error> {
        if with_fcs {
            self.buffer.gwrite(
                crc32fast::hash(&self.buffer[..self.offset]),
                &mut self.offset,
            )?;
        }
        Ok(self.offset)
    }
}
