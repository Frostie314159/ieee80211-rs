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

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// This the body of an action frame.
pub enum ActionBody<'a, VendorSpecificPayload = &'a [u8]> {
    /// This is a vendor specific body.
    VendorSpecific {
        oui: [u8; 3],
        payload: VendorSpecificPayload,
    },
    Unknown {
        category_code: u8,
        payload: &'a [u8],
    },
}
impl ActionBody<'_> {
    /// Returns the total length in bytes.
    pub const fn length_in_bytes(&self) -> usize {
        1 + match self {
            Self::VendorSpecific { payload, .. } => 3 + payload.len(),
            Self::Unknown { payload, .. } => payload.len(),
        }
    }
}
impl<VendorSpecificPayload: MeasureWith<()>> MeasureWith<()>
    for ActionBody<'_, VendorSpecificPayload>
{
    fn measure_with(&self, ctx: &()) -> usize {
        1 + match self {
            Self::VendorSpecific { payload, .. } => 3 + payload.measure_with(ctx),
            Self::Unknown { payload, .. } => payload.len(),
        }
    }
}
impl<'a> TryFromCtx<'a> for ActionBody<'a> {
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
                    ActionBody::VendorSpecific { oui, payload }
                }
                CategoryCode::Unknown(category_code) => {
                    offset = from.len();
                    Self::Unknown {
                        category_code,
                        payload: from,
                    }
                }
            },
            offset,
        ))
    }
}
impl<VendorSpecificPayload: TryIntoCtx<Error = scroll::Error>> TryIntoCtx
    for ActionBody<'_, VendorSpecificPayload>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, data: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        // Since we don't want to repeat, the call to gwrite for the category code, in every match arm, we store it in a variable and set the offset to one.
        let mut offset = 1;
        let category_code;

        match self {
            Self::VendorSpecific { oui, payload } => {
                category_code = CategoryCode::VendorSpecific;
                data.gwrite(oui, &mut offset)?;
                data.gwrite(payload, &mut offset)?;
            }
            Self::Unknown {
                category_code: unknown_category_code,
                payload,
            } => {
                category_code = CategoryCode::Unknown(unknown_category_code);
                data.gwrite(payload, &mut offset)?;
            }
        }
        // Here we write the category code at a fixed offset.
        // Specifying an Endian is useless here, since it's just one byte.
        data.pwrite(category_code.into_bits(), 0)?;
        Ok(offset)
    }
}
