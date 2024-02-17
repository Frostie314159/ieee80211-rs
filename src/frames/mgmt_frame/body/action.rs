use core::iter::Empty;

use macro_bits::serializable_enum;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::elements::{rates::EncodedRate, IEEE80211Element};

use super::{ManagementFrameBody, ToManagementFrameBody};

serializable_enum! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    /// This enum contains the category code specified in the body of an [Action Frame](ActionFrameBody).
    pub enum CategoryCode: u8 {
        #[default]
        VendorSpecific => 127
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// This the body of an action frame.
pub enum ActionFrameBody<P> {
    /// This is a vendor specific body.
    VendorSpecific { oui: [u8; 3], payload: P },
}
impl ActionFrameBody<&[u8]> {
    /// The total length in bytes.
    pub const fn length_in_bytes(&self) -> usize {
        1 + match self {
            Self::VendorSpecific { payload, .. } => 3 + payload.len(),
        }
    }
}
impl<P: MeasureWith<()>> MeasureWith<()> for ActionFrameBody<P> {
    fn measure_with(&self, ctx: &()) -> usize {
        1 + match self {
            Self::VendorSpecific { payload, .. } => 3 + payload.measure_with(ctx),
        }
    }
}
impl<'a> TryFromCtx<'a> for ActionFrameBody<&'a [u8]> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let category_code = CategoryCode::from_bits(from.gread(&mut offset)?);
        Ok((
            match category_code {
                CategoryCode::VendorSpecific => {
                    let oui = from.gread(&mut offset)?;
                    let slice_len = from.len() - offset;
                    let payload = from.gread_with(&mut offset, slice_len)?;
                    ActionFrameBody::VendorSpecific { oui, payload }
                }
                _ => {
                    return Err(scroll::Error::BadInput {
                        size: category_code.into_bits() as usize,
                        msg: "Category code not yet implented.",
                    })
                }
            },
            offset,
        ))
    }
}
impl<P: TryIntoCtx<Error = scroll::Error>> TryIntoCtx for ActionFrameBody<P> {
    type Error = scroll::Error;
    fn try_into_ctx(self, data: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;
        match self {
            Self::VendorSpecific { oui, payload } => {
                data.gwrite(CategoryCode::VendorSpecific.into_bits(), &mut offset)?;
                data.gwrite(oui, &mut offset)?;
                data.gwrite(payload, &mut offset)?;
            }
        }
        Ok(offset)
    }
}
impl<'a, ActionFramePayload>
    ToManagementFrameBody<
        'a,
        Empty<EncodedRate>,
        Empty<EncodedRate>,
        Empty<IEEE80211Element<'a, Empty<EncodedRate>, Empty<EncodedRate>>>,
        ActionFramePayload,
    > for ActionFrameBody<ActionFramePayload>
{
    fn to_management_frame_body(
        self,
    ) -> ManagementFrameBody<
        'a,
        Empty<EncodedRate>,
        Empty<EncodedRate>,
        Empty<IEEE80211Element<'a, Empty<EncodedRate>, Empty<EncodedRate>>>,
        ActionFramePayload,
    > {
        ManagementFrameBody::Action(self)
    }
}
