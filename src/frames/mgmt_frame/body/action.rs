use macro_bits::serializable_enum;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

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
pub enum ActionFrameBody<'a> {
    /// This is a vendor specific body.
    VendorSpecific { oui: [u8; 3], payload: &'a [u8] },
}
impl ActionFrameBody<'_> {
    /// The total length in bytes.
    pub const fn length_in_bytes(&self) -> usize {
        1 + match self {
            Self::VendorSpecific { payload, .. } => 3 + payload.len(),
        }
    }
}
impl MeasureWith<()> for ActionFrameBody<'_> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.length_in_bytes()
    }
}
impl<'a> TryFromCtx<'a> for ActionFrameBody<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let category_code = CategoryCode::from_representation(from.gread(&mut offset)?);
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
                        size: category_code.to_representation() as usize,
                        msg: "Category code not yet implented.",
                    })
                }
            },
            offset,
        ))
    }
}
impl<'a> TryIntoCtx for ActionFrameBody<'a> {
    type Error = scroll::Error;
    fn try_into_ctx(self, data: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;
        match self {
            Self::VendorSpecific { oui, payload } => {
                data.gwrite(
                    CategoryCode::VendorSpecific.to_representation(),
                    &mut offset,
                )?;
                data.gwrite(oui, &mut offset)?;
                data.gwrite(payload, &mut offset)?;
            }
        }
        Ok(offset)
    }
}
