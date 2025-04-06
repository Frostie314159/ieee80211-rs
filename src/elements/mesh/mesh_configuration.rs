use scroll::{
    ctx::{MeasureWith, SizeWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use bitfield_struct::bitfield;

use crate::elements::{Element, ElementID};

macro_rules! mesh_configuration_property {
    (
        $(
            #[$enum_meta:meta]
        )*
        $enum_vis:vis enum $enum_name:ident {
            $(
                $(
                    #[$mesh_configuration_property_meta:meta]
                )*
                $mesh_configuration_property_name:ident => $mesh_configuration_property_type:expr
            ),*
        }
    ) => {
        $(
            #[$enum_meta]
        )*
        #[non_exhaustive]
        $enum_vis enum $enum_name {
            Unknown {
                mesh_configuration_property_selector: u8
            },
            $(
                $(
                    #[$mesh_configuration_property_meta]
                )*
                $mesh_configuration_property_name
            ),*
        }
        impl $enum_name {

            #[inline]
            pub const fn with_mesh_configuration_property_selector(mesh_configuration_property_selector: u8) -> Self {
                match mesh_configuration_property_selector {
                    $(
                        $mesh_configuration_property_type => Self::$mesh_configuration_property_name,
                    )*
                    identifier => Self::Unknown {
                        mesh_configuration_property_selector: identifier
                    }
                }
            }
            #[inline]
            pub const fn mesh_configuration_property_selector(&self) -> u8 {
                match *self {
                    $(
                        Self::$mesh_configuration_property_name => $mesh_configuration_property_type,
                    )*
                    Self::Unknown { mesh_configuration_property_selector } => mesh_configuration_property_selector
                }
            }
        }
        impl<'a> TryFromCtx<'a> for $enum_name {
            type Error = scroll::Error;
            #[inline]
            fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
                let mut offset = 0;

                let mesh_configuration_property_selector = from.gread_with(&mut offset, Endian::Little)?;

                Ok((
                    Self::with_mesh_configuration_property_selector(mesh_configuration_property_selector),
                    offset
                ))
            }
        }
        impl SizeWith for $enum_name {
            #[inline]
            fn size_with(_ctx: &()) -> usize {
                1
            }
        }
        impl TryIntoCtx for $enum_name {
            type Error = scroll::Error;
            #[inline]
            fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
                let mut offset = 0;

                buf.gwrite_with(self.mesh_configuration_property_selector(), &mut offset, Endian::Little)?;

                Ok(offset)
            }
        }
    };
}

mesh_configuration_property! {
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub enum MeshConfigurationActivePathSelectionProtocolIdentifier {
        #[default]
        HWMP => 1,
        VendorSpecific => 255
    }
}

mesh_configuration_property! {
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub enum MeshConfigurationActivePathSelectionMetricIdentifier {
        #[default]
        AirtimeLinkMetric => 1,
        HighPHYRateAirtimeLinkMetric => 2,
        VendorSpecific => 255
    }
}

mesh_configuration_property! {
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub enum MeshConfigurationCongestionControlModeIdentifier {
        #[default]
        NotActivated => 0,
        SignalingProtocol => 1,
        VendorSpecific => 255
    }
}

mesh_configuration_property! {
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub enum MeshConfigurationSynchronizationMethodIdentifier {
        #[default]
        NeighborOffsetSynchronization => 1,
        VendorSpecific => 255
    }
}

mesh_configuration_property! {
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub enum MeshConfigurationAuthenticationProtocolIdentifier {
        #[default]
        NoAuthentication => 0,
        SAE => 1,
        IEEE80211XAuthentication => 2,
        VendorSpecific => 255
    }
}

#[bitfield(u8, defmt = cfg(feature = "defmt"))]
#[derive(PartialEq, Eq, Hash, Pread, Pwrite)]
pub struct MeshFormationInfo {
    pub connected_to_mesh_gate: bool,
    #[bits(6)]
    pub num_peerings: u8,
    pub connected_to_as: bool
}

#[bitfield(u8, defmt = cfg(feature = "defmt"))]
#[derive(PartialEq, Eq, Hash)]
pub struct MeshCapability {
    pub accept_additional_mesh_peerings: bool,
    pub mcca_supported: bool,
    pub mcca_enabled: bool,
    pub forwarding: bool,
    pub mbca_enabled: bool,
    pub tbtt_adjusting: bool,
    pub mesh_power_save_level: bool,
    pub _reserved: bool
}


#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
/// The Mesh Configuration element is used to advertise mesh services.
pub struct MeshConfigurationElement {
    pub active_path_selection_protocol_identifier: MeshConfigurationActivePathSelectionProtocolIdentifier,
    pub active_path_selection_metric_identifier: MeshConfigurationActivePathSelectionMetricIdentifier,
    pub congestion_control_mode_identifier: MeshConfigurationCongestionControlModeIdentifier,
    pub syncronization_method_identifier: MeshConfigurationSynchronizationMethodIdentifier,
    pub authentication_protocol_identifier: MeshConfigurationAuthenticationProtocolIdentifier,
    pub mesh_formation_info: MeshFormationInfo,
    pub mesh_capability: MeshCapability,
}

impl MeasureWith<()> for MeshConfigurationElement {
    fn measure_with(&self, _ctx: &()) -> usize {
        7
    }
}

impl TryFromCtx<'_> for MeshConfigurationElement {
    type Error = scroll::Error;
    fn try_from_ctx(from: &[u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let active_path_selection_protocol_identifier = from.gread(&mut offset)?;
        let active_path_selection_metric_identifier = from.gread(&mut offset)?;
        let congestion_control_mode_identifier = from.gread(&mut offset)?;
        let syncronization_method_identifier = from.gread(&mut offset)?;
        let authentication_protocol_identifier = from.gread(&mut offset)?;
        let mesh_formation_info = MeshFormationInfo::from_bits(from.gread(&mut offset)?);
        let mesh_capability = MeshCapability::from_bits(from.gread(&mut offset)?);

        Ok((
            Self {
                active_path_selection_protocol_identifier,
                active_path_selection_metric_identifier,
                congestion_control_mode_identifier,
                syncronization_method_identifier,
                authentication_protocol_identifier,
                mesh_formation_info,
                mesh_capability
            },
            offset,
        ))
    }
}

impl TryIntoCtx for MeshConfigurationElement {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.active_path_selection_protocol_identifier, &mut offset)?;
        buf.gwrite(self.active_path_selection_metric_identifier, &mut offset)?;
        buf.gwrite(self.congestion_control_mode_identifier, &mut offset)?;
        buf.gwrite(self.syncronization_method_identifier, &mut offset)?;
        buf.gwrite(self.authentication_protocol_identifier, &mut offset)?;
        buf.gwrite(self.mesh_formation_info.into_bits(), &mut offset)?;
        buf.gwrite(self.mesh_capability.into_bits(), &mut offset)?;

        Ok(offset)
    }
}

impl Element for MeshConfigurationElement {
    const ELEMENT_ID: ElementID = ElementID::Id(113);
    type ReadType<'a> = Self;
}
