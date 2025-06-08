use core::cmp;
use macro_bits::serializable_enum;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::{
    common::IEEE80211Reason,
    elements::{Element, ElementID},
};

serializable_enum! {
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
    rest: [u8; 2 + 2 + 16],
    rest_len: usize,
}

#[non_exhaustive]
pub struct ParsedMeshPeeringManagement {
    pub mesh_peering_protocol_identifier: MeshPeeringProtocolIdentifier,
    pub local_link_id: u16,
    pub peer_link_id: Option<u16>,
    pub reason_code: Option<IEEE80211Reason>,
    pub pmk: Option<[u8; 16]>,
}

impl MeshPeeringManagement {
    // We need specific constructors/parsers because the Mesh Peering Managment element is context-dependent:
    // depending on which frame (Open/Confirm/Close) it occurs in, the data needs to be parsed differently. This is a very unfortunate
    // design in 802.11, as it means that our current approach of parsing elements does not cleanly work.

    pub fn new_open(
        mesh_peering_protocol_identifier: MeshPeeringProtocolIdentifier,
        local_link_id: u16,
        pmk: Option<[u8; 16]>,
    ) -> Self {
        let mut rest: [u8; 2 + 2 + 16] = [0; 2 + 2 + 16];
        let mut offset = 0;

        if let Some(key) = pmk {
            rest.gwrite(key, &mut offset).unwrap();
        }

        Self {
            mesh_peering_protocol_identifier,
            local_link_id,
            rest,
            rest_len: offset,
        }
    }

    pub fn new_confirm(
        mesh_peering_protocol_identifier: MeshPeeringProtocolIdentifier,
        local_link_id: u16,
        peer_link_id: u16,
        pmk: Option<[u8; 16]>,
    ) -> Self {
        let mut rest: [u8; 2 + 2 + 16] = [0; 2 + 2 + 16];
        let mut offset = 0;

        rest.gwrite(peer_link_id, &mut offset).unwrap();

        if let Some(key) = pmk {
            rest.gwrite(key, &mut offset).unwrap();
        }

        Self {
            mesh_peering_protocol_identifier,
            local_link_id,
            rest,
            rest_len: offset,
        }
    }

    pub fn new_close(
        mesh_peering_protocol_identifier: MeshPeeringProtocolIdentifier,
        local_link_id: u16,
        peer_link_id: Option<u16>,
        reason_code: IEEE80211Reason,
        pmk: Option<[u8; 16]>,
    ) -> Self {
        let mut rest: [u8; 2 + 2 + 16] = [0; 2 + 2 + 16];
        let mut offset = 0;

        if let Some(peer_link_id) = peer_link_id {
            rest.gwrite(peer_link_id, &mut offset).unwrap();
        }
        rest.gwrite(reason_code.into_bits(), &mut offset).unwrap();

        if let Some(key) = pmk {
            rest.gwrite(key, &mut offset).unwrap();
        }

        Self {
            mesh_peering_protocol_identifier,
            local_link_id,
            rest,
            rest_len: offset,
        }
    }

    pub fn parse_as_open(&self) -> Option<ParsedMeshPeeringManagement> {
        let mut offset = 0;

        let pmk = if self.rest_len == 0 {
            None
        } else if self.rest_len == 16 {
            let pmk_inner = self.rest.gread(&mut offset).unwrap();
            Some(pmk_inner)
        } else {
            return None;
        };

        Some(ParsedMeshPeeringManagement {
            mesh_peering_protocol_identifier: self.mesh_peering_protocol_identifier,
            local_link_id: self.local_link_id,
            peer_link_id: None,
            reason_code: None,
            pmk,
        })
    }

    pub fn parse_as_confirm(&self) -> Option<ParsedMeshPeeringManagement> {
        let peer_link_id: u16;
        let pmk;
        let mut offset = 0;

        if self.rest_len == 2 {
            pmk = None;
            peer_link_id = self.rest.gread(&mut offset).unwrap();
        } else if self.rest_len == 2 + 16 {
            peer_link_id = self.rest.gread(&mut offset).unwrap();
            let pmk_inner = self.rest.gread(&mut offset).unwrap();
            pmk = Some(pmk_inner);
        } else {
            return None;
        }

        Some(ParsedMeshPeeringManagement {
            mesh_peering_protocol_identifier: self.mesh_peering_protocol_identifier,
            local_link_id: self.local_link_id,
            peer_link_id: Some(peer_link_id),
            reason_code: None,
            pmk,
        })
    }

    pub fn parse_as_close(&self) -> Option<ParsedMeshPeeringManagement> {
        let peer_link_id: Option<u16>;
        let reason_code: IEEE80211Reason;
        let pmk;
        let mut offset = 0;

        if self.rest_len == 2 {
            reason_code = IEEE80211Reason::from_bits(self.rest.gread(&mut offset).unwrap());
            peer_link_id = None;
            pmk = None;
        } else if self.rest_len == 2 + 2 {
            let peer_link_id_ = self.rest.gread(&mut offset).unwrap();
            peer_link_id = Some(peer_link_id_);
            reason_code = IEEE80211Reason::from_bits(self.rest.gread(&mut offset).unwrap());
            pmk = None;
        } else if self.rest_len == 2 + 16 {
            peer_link_id = None;
            reason_code = IEEE80211Reason::from_bits(self.rest.gread(&mut offset).unwrap());
            let pmk_inner = self.rest.gread(&mut offset).unwrap();
            pmk = Some(pmk_inner);
        } else if self.rest_len == 2 + 2 + 16 {
            let peer_link_id_ = self.rest.gread(&mut offset).unwrap();
            peer_link_id = Some(peer_link_id_);
            reason_code = IEEE80211Reason::from_bits(self.rest.gread(&mut offset).unwrap());
            let pmk_inner = self.rest.gread(&mut offset).unwrap();
            pmk = Some(pmk_inner);
        } else {
            return None;
        }

        Some(ParsedMeshPeeringManagement {
            mesh_peering_protocol_identifier: self.mesh_peering_protocol_identifier,
            local_link_id: self.local_link_id,
            peer_link_id,
            reason_code: Some(reason_code),
            pmk,
        })
    }
}

impl TryFromCtx<'_> for MeshPeeringManagement {
    type Error = scroll::Error;
    fn try_from_ctx(from: &[u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let mesh_peering_protocol_identifier =
            MeshPeeringProtocolIdentifier::from_bits(from.gread(&mut offset)?);
        let local_link_id = from.gread(&mut offset)?;
        let mut rest: [u8; 2 + 2 + 16] = [0; 2 + 2 + 16];

        let rest_len = cmp::min(rest.len(), from[offset..].len());
        rest[0..rest_len].copy_from_slice(&from[offset..]);

        Ok((
            Self {
                mesh_peering_protocol_identifier,
                local_link_id,
                rest,
                rest_len,
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

        buf.gwrite(
            self.mesh_peering_protocol_identifier.into_bits(),
            &mut offset,
        )?;
        buf.gwrite(self.local_link_id, &mut offset)?;
        buf.gwrite(self.rest, &mut offset)?;

        Ok(offset)
    }
}

impl Element for MeshPeeringManagement {
    const ELEMENT_ID: ElementID = ElementID::Id(117);
    type ReadType<'a> = Self;
}
