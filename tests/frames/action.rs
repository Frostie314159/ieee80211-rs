use ieee80211::{
    elements::mesh::{MeshCapability, MeshConfigurationElement, MeshFormationInfo, MeshIDElement}, match_frames, mgmt_frame::{
        body::action::{CategoryCode, MeshPeeringOpenFrame, RawVendorSpecificActionFrame},
        RawActionFrame,
    }
};

#[test]
fn test_raw_action_frame() {
    let bytes = include_bytes!("../../bins/frames/awdl_action.bin");
    match_frames! {
        bytes,
        raw_action_frame = RawActionFrame => {
            assert_eq!(raw_action_frame.category_code, CategoryCode::VendorSpecific);
        }
    }
    .expect("Failed to match raw action frame.");
}
#[test]
fn test_raw_vendor_action_frame() {
    let bytes = include_bytes!("../../bins/frames/awdl_action.bin");
    match_frames! {
        bytes,
        raw_action_frame = RawVendorSpecificActionFrame => {
            assert_eq!(raw_action_frame.oui, [0x00, 0x17, 0xf2]);
        }
    }
    .expect("Failed to match raw action frame.");
}


#[test]
fn test_action_mesh_open() {
    // Taken from a real packet capture
    let bytes = include_bytes!("../../bins/frames/action_mesh_open.bin");
    match_frames! {
        bytes,
        mesh_peering = MeshPeeringOpenFrame => {
            let mesh_id = mesh_peering.elements
                .get_first_element::<MeshIDElement>()
                .map(MeshIDElement::take_mesh_id);
            assert_eq!(mesh_id, Some("meshtest"));
            let mesh_configuration = mesh_peering.elements
                .get_first_element::<MeshConfigurationElement>();
            let expected_mesh_configuration = Some(MeshConfigurationElement {
                active_path_selection_protocol_identifier: ieee80211::elements::mesh::MeshConfigurationActivePathSelectionProtocolIdentifier::HWMP,
                active_path_selection_metric_identifier: ieee80211::elements::mesh::MeshConfigurationActivePathSelectionMetricIdentifier::AirtimeLinkMetric,
                congestion_control_mode_identifier: ieee80211::elements::mesh::MeshConfigurationCongestionControlModeIdentifier::NotActivated,
                syncronization_method_identifier: ieee80211::elements::mesh::MeshConfigurationSynchronizationMethodIdentifier::NeighborOffsetSynchronization,
                authentication_protocol_identifier: ieee80211::elements::mesh::MeshConfigurationAuthenticationProtocolIdentifier::NoAuthentication,
                mesh_formation_info: MeshFormationInfo::new().with_connected_to_mesh_gate(false).with_num_peerings(0).with_connected_to_as(false),
                mesh_capability: MeshCapability::new().with_accept_additional_mesh_peerings(true).with_forwarding(true)
            });
            assert_eq!(mesh_configuration, expected_mesh_configuration);
        }
    }
    .expect("Failed to match action mesh open frame.");
}