use ieee80211::{
    mgmt_frame::{header::ManagementFrameHeader, ManagementFrame},
    IEEE80211Frame,
};

const EXPECTED_BEACON_FRAME: IEEE80211Frame = ManagementFrame {
    header: ManagementFrameHeader {},
};
