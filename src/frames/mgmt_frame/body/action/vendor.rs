use core::marker::PhantomData;

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::mgmt_frame::ManagementFrame;

use super::{ActionBody, CategoryCode, RawActionBody};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// A raw vendor specific action frame body.
pub struct RawVendorSpecificActionBody<'a, Payload = &'a [u8]> {
    pub oui: [u8; 3],
    pub payload: Payload,
    pub _phantom: PhantomData<&'a ()>,
}
impl<'a> TryFromCtx<'a> for RawVendorSpecificActionBody<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let category_code = CategoryCode::from_bits(from.gread(&mut offset)?);
        if category_code != CategoryCode::VendorSpecific {
            return Err(scroll::Error::BadInput {
                size: offset,
                msg: "Category code wasn't vendor specific.",
            });
        }
        let oui = from.gread(&mut offset)?;
        let payload = &from[offset..];

        Ok((
            Self {
                oui,
                payload,
                _phantom: PhantomData,
            },
            offset,
        ))
    }
}
impl<Payload: MeasureWith<()>> MeasureWith<()> for RawVendorSpecificActionBody<'_, Payload> {
    fn measure_with(&self, ctx: &()) -> usize {
        3 + self.payload.measure_with(ctx)
    }
}
impl<Payload: TryIntoCtx<Error = scroll::Error>> TryIntoCtx
    for RawVendorSpecificActionBody<'_, Payload>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        append_vendor_action_header(buf, self.oui)?;
        buf.gwrite(self.payload, &mut offset)?;

        Ok(offset)
    }
}
impl<Payload> ActionBody for RawVendorSpecificActionBody<'_, Payload> {
    const CATEGORY_CODE: CategoryCode = CategoryCode::VendorSpecific;
    fn matches(action_body: RawActionBody<'_>) -> bool {
        action_body.category_code == Self::CATEGORY_CODE
    }
}
pub type RawVendorSpecificActionFrame<'a, Payload = &'a [u8]> =
    ManagementFrame<RawVendorSpecificActionBody<'a, Payload>>;

/// This appends the vendor specific action frame header (including the category code) to the buffer.
pub fn append_vendor_action_header(buf: &mut [u8], oui: [u8; 3]) -> Result<usize, scroll::Error> {
    let mut offset = 0;

    buf.gwrite(CategoryCode::VendorSpecific.into_bits(), &mut offset)?;
    buf.gwrite(oui.as_slice(), &mut offset)?;

    Ok(offset)
}
/// Checks the category code and OUI, of the supplied vendor specific action frame body, and advances the `offset`.
pub fn strip_and_check_vendor_action_header(
    buf: &[u8],
    offset: &mut usize,
    expected_oui: [u8; 3],
) -> Result<(), scroll::Error> {
    if CategoryCode::from_bits(buf.gread(offset)?) != CategoryCode::VendorSpecific {
        return Err(scroll::Error::BadInput {
            size: *offset,
            msg: "The category code wasn't vendor specific.",
        });
    }
    if buf.gread::<[u8; 3]>(offset)? != expected_oui {
        return Err(scroll::Error::BadInput {
            size: *offset,
            msg: "The OUI didn't match, what was expected.",
        });
    }
    Ok(())
}
