
use macro_bits::serializable_enum;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};
use core::cmp;

use crate::elements::{Element, ElementID};

serializable_enum! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum MeshPeeringProtocolIdentifier : u16 {
        MeshPeeringManagementProtocol => 0,
        AuthenticatedMeshPeeringExchangeProtocol => 1,
        VendorSpecific => 255
    }
}


#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Hash)]
/// The Mesh Peering Management element is used to manage a mesh peering with a neighbor mesh STA.
pub struct MeshPeeringManagement {
    pub mesh_peering_protocol_identifier: MeshPeeringProtocolIdentifier,
    pub local_link_id: u16,
    rest: [u8; 2+2+16],
    rest_len: usize
}

impl TryFromCtx<'_> for MeshPeeringManagement {
    type Error = scroll::Error;
    fn try_from_ctx(from: &[u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let mesh_peering_protocol_identifier = MeshPeeringProtocolIdentifier::from_bits(from.gread(&mut offset)?);
        let local_link_id = from.gread(&mut offset)?;
        let mut rest: [u8;2+2+16] = [0; 2+2+16];

        let rest_len = cmp::min(rest.len(), from[offset..].len());
        rest[0..rest_len].copy_from_slice( &from[offset..]);

        Ok((
            Self {
                mesh_peering_protocol_identifier,
                local_link_id,
                rest,
                rest_len
            },
            offset,
        ))
    }
}

impl MeasureWith<()> for MeshPeeringManagement {
    fn measure_with(&self, _ctx: &()) -> usize {
        2 + 2 + self.rest_len
    }
}

impl TryIntoCtx for MeshPeeringManagement {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.mesh_peering_protocol_identifier.into_bits(), &mut offset)?;
        buf.gwrite(self.local_link_id, &mut offset)?;
        buf.gwrite(self.rest, &mut offset)?;

        Ok(offset)
    }
}


impl Element for MeshPeeringManagement {
    const ELEMENT_ID: ElementID = ElementID::Id(117);
    type ReadType<'a> = Self;
}
