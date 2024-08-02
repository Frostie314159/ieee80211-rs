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
    common::{strip_and_validate_fcs, FCFFlags, FrameControlField, FrameType},
    elements::ReadElements,
    IEEE80211Frame,
};

use self::header::ManagementFrameHeader;

pub mod body;
pub mod header;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// A generic management frame.
pub struct ManagementFrame<Body> {
    pub header: ManagementFrameHeader,
    pub body: Body,
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
impl<'a, Body: TryFromCtx<'a, Error = scroll::Error>> TryFromCtx<'a, bool>
    for ManagementFrame<Body>
{
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], with_fcs: bool) -> Result<(Self, usize), Self::Error> {
        // We don't care about the FCF, since the information is already encoded in the type.
        let mut offset = 1;

        let fcf_flags = FCFFlags::from_bits(from.gread(&mut offset)?);
        let header = from.gread_with(&mut offset, fcf_flags)?;
        let body_slice = if with_fcs {
            strip_and_validate_fcs(from)?
        } else {
            from
        };
        let body = body_slice.gread(&mut offset)?;

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
            let fcs = crc32fast::hash(&buf[..offset]);
            buf.gwrite_with(fcs, &mut offset, Endian::Little)?;
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
