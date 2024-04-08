use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use super::{Element, ElementID};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// The vendor specific element carries information, which is not defined in IEEE 802.11, but rather by individual vendors.
pub struct VendorSpecificElement<'a> {
    /// The OUI of the vendor.
    pub oui: [u8; 3],
    /// The payload of the frame.
    pub payload: &'a [u8],
}
impl MeasureWith<()> for VendorSpecificElement<'_> {
    fn measure_with(&self, _ctx: &()) -> usize {
        3 + self.payload.len()
    }
}
impl<'a> TryFromCtx<'a> for VendorSpecificElement<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let oui = from.gread(&mut offset)?;
        let payload_len = from.len() - offset;
        let payload = from.gread_with(&mut offset, payload_len)?;

        Ok((Self { oui, payload }, offset))
    }
}
impl TryIntoCtx for VendorSpecificElement<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.oui, &mut offset)?;
        buf.gwrite(self.payload, &mut offset)?;

        Ok(offset)
    }
}
impl<'a> Element<'a> for VendorSpecificElement<'a> {
    const ELEMENT_ID: ElementID = ElementID::Id(0xdd);
    type ReadType = Self;
}
