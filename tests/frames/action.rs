use ieee80211::{
    match_frames,
    mgmt_frame::{
        body::action::{CategoryCode, RawVendorSpecificActionFrame},
        RawActionFrame,
    },
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
