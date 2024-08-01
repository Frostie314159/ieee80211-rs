use body::{ActionBody, AssociationRequestBody, AssociationResponseBody, AuthenticationBody, BeaconBody, DeauthenticationBody, DisassociationBody, ProbeRequestBody, ProbeResponseBody};
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{
    common::{FCFFlags, FrameControlField, FrameType},
    data_frame::DataFrameReadPayload,
    elements::ReadElements,
    IEEE80211Frame, IEEE80211FrameTrait, ToFrame,
};

use self::{
    body::{ManagementFrameBody, ManagementFrameSubtype},
    header::ManagementFrameHeader,
};

pub mod body;
pub mod header;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// An IEEE 802.11 Management Frame.
pub struct GenericManagementFrame<
    'a,
    ElementContainer = ReadElements<'a>,
    ActionFramePayload = &'a [u8],
> where
    ElementContainer: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
    ActionFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
{
    pub header: ManagementFrameHeader,
    pub body: ManagementFrameBody<'a, ElementContainer, ActionFramePayload>,
}
impl<
        'a,
        ElementContainer: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
        ActionFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
    > GenericManagementFrame<'a, ElementContainer, ActionFramePayload>
{
    pub const fn get_fcf(&self) -> FrameControlField {
        FrameControlField::new()
            .with_frame_type(FrameType::Management(self.body.get_subtype()))
            .with_flags(self.header.fcf_flags)
    }
}
impl GenericManagementFrame<'_> {
    pub const fn length_in_bytes(&self) -> usize {
        self.header.length_in_bytes() + self.body.length_in_bytes()
    }
}
impl<
        'a,
        ElementContainer: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
        ActionFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
    > MeasureWith<()> for GenericManagementFrame<'a, ElementContainer, ActionFramePayload>
{
    fn measure_with(&self, ctx: &()) -> usize {
        self.header.length_in_bytes() + self.body.measure_with(ctx)
    }
}
impl<'a> TryFromCtx<'a, (ManagementFrameSubtype, FCFFlags)> for GenericManagementFrame<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(
        from: &'a [u8],
        (subtype, fcf_flags): (ManagementFrameSubtype, FCFFlags),
    ) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let header = from.gread_with(&mut offset, fcf_flags)?;
        let body = from.gread_with(&mut offset, subtype)?;

        Ok((Self { header, body }, offset))
    }
}
impl<
        'a,
        ElementContainer: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
        ActionFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()>,
    > TryIntoCtx for GenericManagementFrame<'a, ElementContainer, ActionFramePayload>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.header, &mut offset)?;
        buf.gwrite(self.body, &mut offset)?;
        Ok(offset)
    }
}
impl<
        'a,
        ElementContainer: TryIntoCtx<Error = scroll::Error> + MeasureWith<()> + 'a,
        ActionFramePayload: TryIntoCtx<Error = scroll::Error> + MeasureWith<()> + 'a,
    > ToFrame<'a, ElementContainer, ActionFramePayload, DataFrameReadPayload<'a>>
    for GenericManagementFrame<'a, ElementContainer, ActionFramePayload>
{
    fn to_frame(
        self,
    ) -> IEEE80211Frame<'a, ElementContainer, ActionFramePayload, DataFrameReadPayload<'a>> {
        IEEE80211Frame::Management(self)
    }
}

pub trait ManagementFrameBodyTrait {
    const SUBTYPE: ManagementFrameSubtype;
}
pub struct TypedManagementFrame<Body> {
    pub header: ManagementFrameHeader,
    pub body: Body,
}
impl<Body: ManagementFrameBodyTrait> IEEE80211FrameTrait for TypedManagementFrame<Body> {
    const TYPE: FrameType = FrameType::Management(Body::SUBTYPE);
}
impl<Body: MeasureWith<()>> MeasureWith<()> for TypedManagementFrame<Body> {
    fn measure_with(&self, ctx: &()) -> usize {
        2 + self.header.length_in_bytes() + self.body.measure_with(ctx)
    }
}
impl<'a, Body: TryFromCtx<'a, Error = scroll::Error>> TryFromCtx<'a>
    for TypedManagementFrame<Body>
{
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        // We don't care about the FCF, since the information is already encoded in the type.
        let mut offset = 1;

        let fcf_flags = FCFFlags::from_bits(from.gread(&mut offset)?);
        let header = from.gread_with(&mut offset, fcf_flags)?;
        let body = from.gread(&mut offset)?;

        Ok((Self { header, body }, offset))
    }
}
impl<Body: TryIntoCtx<Error = scroll::Error> + ManagementFrameBodyTrait> TryIntoCtx
    for TypedManagementFrame<Body>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(
            FrameControlField::new()
                .with_frame_type(<Self as IEEE80211FrameTrait>::TYPE)
                .with_flags(self.header.fcf_flags)
                .into_bits(),
            &mut offset,
            Endian::Little,
        )?;
        buf.gwrite(self.header, &mut offset)?;
        buf.gwrite(self.body, &mut offset)?;

        Ok(offset)
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
            pub type $frame<'a, ElementContainer = ReadElements<'a>> = TypedManagementFrame<$frame_body<'a, ElementContainer>>;
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
    DeauthenticationFrame => DeauthenticationBody,
    ActionFrame => ActionBody
}
