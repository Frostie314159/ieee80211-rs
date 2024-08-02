use ieee80211::{match_frames, mgmt_frame::BeaconFrame};

fn main() {
    let bytes = include_bytes!("../bins/frames/beacon.bin");
    match_frames! {
        bytes,
        beacon_frame = BeaconFrame => {
            println!("SSID: {}", beacon_frame.body.ssid().unwrap());
        }
    }
    .unwrap();
}
