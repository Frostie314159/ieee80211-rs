use core::marker::PhantomData;

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use super::{Element, ElementID};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// The vendor specific element carries information, which is not defined in IEEE 802.11, but rather by individual vendors.
pub struct VendorSpecificElement<'a, Payload = &'a [u8]> {
    /// The OUI of the vendor.
    pub oui: [u8; 3],
    /// The subtype of the element.
    pub subtype: u8,
    /// The payload of the frame.
    pub payload: Payload,
    pub _phantom: PhantomData<&'a ()>,
}
impl<Payload: MeasureWith<()>> MeasureWith<()> for VendorSpecificElement<'_, Payload> {
    fn measure_with(&self, ctx: &()) -> usize {
        4 + self.payload.measure_with(ctx)
    }
}
impl<'a> TryFromCtx<'a> for VendorSpecificElement<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let oui = from.gread(&mut offset)?;
        let subtype = from.gread(&mut offset)?;
        let payload_len = from.len() - offset;
        let payload = from.gread_with(&mut offset, payload_len)?;

        Ok((
            Self {
                oui,
                subtype,
                payload,
                _phantom: PhantomData,
            },
            offset,
        ))
    }
}
impl<Payload: TryIntoCtx<Error = scroll::Error>> TryIntoCtx for VendorSpecificElement<'_, Payload> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.oui, &mut offset)?;
        buf.gwrite(self.subtype, &mut offset)?;
        buf.gwrite(self.payload, &mut offset)?;

        Ok(offset)
    }
}
impl<'a> Element for VendorSpecificElement<'a> {
    const ELEMENT_ID: ElementID = ElementID::Id(0xdd);
    type ReadType<'b> = VendorSpecificElement<'b>;
}
