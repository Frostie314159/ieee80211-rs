//! This example demonstrates how to use the [match_frames] API.

use ieee80211::{
    data_frame::DataFrame,
    match_frames,
    mgmt_frame::{body::action::RawVendorSpecificActionFrame, BeaconFrame, DeauthenticationFrame},
};

fn main() {
    let bytes = include_bytes!("../bins/frames/beacon.bin");
    match_frames! {
        bytes,
        _ = DeauthenticationFrame => {}
        beacon_frame = BeaconFrame => {
            println!("SSID: {}", beacon_frame.body.ssid().unwrap());
        }
        _ = DataFrame => {}
        // You can even match specific action frames.
        _ = RawVendorSpecificActionFrame => {}
    }
    .unwrap();
}
