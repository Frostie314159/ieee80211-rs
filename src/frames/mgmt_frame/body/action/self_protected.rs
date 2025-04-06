use macro_bits::serializable_enum;

use core::marker::PhantomData;

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite, Endian
};

use crate::{elements::ReadElements, mgmt_frame::ManagementFrame};

use crate::common::CapabilitiesInformation;

use super::{ActionBody, CategoryCode, RawActionBody};


serializable_enum! {
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum SelfProtectedActionCode: u8 {
        MeshPeeringOpen => 1,
        MeshPeeringConfirm => 2,
        MeshPeeringClose => 3,
        MeshGroupKeyInform => 4,
        MeshGroupKeyAcknowledge => 5
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MeshPeeringOpenBody<'a, ElementContainer = ReadElements<'a>> {
    pub capabilities_info: CapabilitiesInformation,
    pub elements: ElementContainer,
    pub _phantom: PhantomData<&'a ()>,
}

impl<'a> TryFromCtx<'a> for MeshPeeringOpenBody<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let category_code = CategoryCode::from_bits(from.gread(&mut offset)?);
        if category_code != CategoryCode::SelfProtected {
            return Err(scroll::Error::BadInput {
                size: offset,
                msg: "Category code wasn't self-protected.",
            });
        }
        let selfprotected_action_code = SelfProtectedActionCode::from_bits(from.gread(&mut offset)?);
        if selfprotected_action_code != SelfProtectedActionCode::MeshPeeringOpen {
            return Err(scroll::Error::BadInput {
                size: offset,
                msg: "Self-protected action code wasn't Mesh peering open.",
            });
        }

        let capabilities_info = CapabilitiesInformation::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        let elements = from.gread(&mut offset)?;

        Ok((
            Self {
                capabilities_info,
                elements,
                _phantom: PhantomData,
            },
            offset,
        ))
    }
}
impl<ElementContainer: MeasureWith<()>> MeasureWith<()> for MeshPeeringOpenBody<'_, ElementContainer> {
    fn measure_with(&self, ctx: &()) -> usize {
        1 + 2 + self.elements.measure_with(ctx)
    }
}
impl<ElementContainer: TryIntoCtx<Error = scroll::Error>> TryIntoCtx
    for MeshPeeringOpenBody<'_, ElementContainer>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(CategoryCode::SelfProtected.into_bits(), &mut offset)?;
        buf.gwrite(SelfProtectedActionCode::MeshPeeringOpen.into_bits(), &mut offset)?;
        buf.gwrite_with(
            self.capabilities_info.into_bits(),
            &mut offset,
            Endian::Little,
        )?;
        buf.gwrite(self.elements, &mut offset)?;

        Ok(offset)
    }
}
impl<ElementContainer> ActionBody for MeshPeeringOpenBody<'_, ElementContainer> {
    const CATEGORY_CODE: CategoryCode = CategoryCode::SelfProtected;
    fn matches(action_body: RawActionBody<'_>) -> bool {
        action_body.category_code == Self::CATEGORY_CODE && action_body.payload.pread::<u8>(0)
        .map(|subtype| subtype == SelfProtectedActionCode::MeshPeeringOpen.into_bits())
        .unwrap_or_default()
    }
}
pub type MeshPeeringOpenFrame<'a, ElementContainer = ReadElements<'a>> =
    ManagementFrame<MeshPeeringOpenBody<'a, ElementContainer>>;
