use core::marker::PhantomData;

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pwrite,
};

use super::{Element, ElementID};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// The vendor specific element carries information, which is not defined in IEEE 802.11, but rather by individual vendors.
///
/// Vendor specific elements are prefixed with an OUI and potentially more bytes identifying the specific type.
/// Due to this, the identifier isn't parsed while reading and remains part of the payload slice.
/// This must be taken in to account, when using the payload.
/// When creating a new [VendorSpecificElement], the [Self::new_prefixed] method should be used.
pub struct VendorSpecificElement<'a, Payload = &'a [u8]> {
    /// The payload of the frame.
    payload: Payload,
    _phantom: PhantomData<&'a ()>,
}
impl<'a> VendorSpecificElement<'a> {
    /// Returns the truncated payload, if it starts with the specified prefix.
    pub fn get_payload_if_prefix_matches(&self, prefix: &'static [u8]) -> Option<&'a [u8]> {
        if self.payload.starts_with(prefix) {
            Some(&self.payload[prefix.len()..])
        } else {
            None
        }
    }
}
impl<'a, Payload> VendorSpecificElement<'a, Payload> {
    pub const fn get_payload(&self) -> &Payload {
        &self.payload
    }
    pub fn get_payload_mut(&mut self) -> &mut Payload {
        &mut self.payload
    }
}
impl<'a, InnerPayload> VendorSpecificElement<'a, PrefixedVendorPayload<InnerPayload>> {
    pub const fn new_prefixed(prefix: &'static [u8], payload: InnerPayload) -> Self {
        VendorSpecificElement {
            payload: PrefixedVendorPayload { prefix, payload },
            _phantom: PhantomData,
        }
    }
}
impl<Payload: MeasureWith<()>> MeasureWith<()> for VendorSpecificElement<'_, Payload> {
    fn measure_with(&self, ctx: &()) -> usize {
        self.payload.measure_with(ctx)
    }
}
impl<'a> TryFromCtx<'a> for VendorSpecificElement<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        Ok((
            Self {
                payload: from,
                _phantom: PhantomData,
            },
            from.len(),
        ))
    }
}
impl<Payload: TryIntoCtx<Error = scroll::Error>> TryIntoCtx for VendorSpecificElement<'_, Payload> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.payload, &mut offset)?;

        Ok(offset)
    }
}
impl<'a> Element for VendorSpecificElement<'a> {
    const ELEMENT_ID: ElementID = ElementID::Id(0xdd);
    type ReadType<'b> = VendorSpecificElement<'b>;
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// A payload of a [VendorSpecificElement], prefixed with an identifier.
///
/// This is mostly used internaly, but is also required, if you want to implement your own vendor specific element outside this crate.
pub struct PrefixedVendorPayload<Payload> {
    pub prefix: &'static [u8],
    pub payload: Payload,
}
impl<Payload: MeasureWith<()>> MeasureWith<()> for PrefixedVendorPayload<Payload> {
    fn measure_with(&self, ctx: &()) -> usize {
        self.prefix.len() + self.payload.measure_with(ctx)
    }
}
impl<Payload: TryIntoCtx<Error = scroll::Error>> TryIntoCtx for PrefixedVendorPayload<Payload> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.prefix, &mut offset)?;
        buf.gwrite(self.payload, &mut offset)?;

        Ok(offset)
    }
}
