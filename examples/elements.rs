use ieee80211::{
    elements::types::{SupportedRates, SSID},
    mgmt_frame::body::ManagementFrameBody,
    IEEE80211Frame,
};
use scroll::Pread;

fn main() {
    let bytes = include_bytes!("../bins/beacon.bin");
    let frame = bytes.pread::<IEEE80211Frame>(0).unwrap();
    let IEEE80211Frame::Management(mgmt_frame) = frame else {
        todo!()
    };
    let ManagementFrameBody::Beacon(beacon) = mgmt_frame.body else {
        todo!()
    };
    // There is a method that does this automatically, see beacon.rs.
    let ssid_element = beacon.elements.get_first_element::<SSID>().unwrap();
    let supported_rates = beacon.elements.get_first_element::<SupportedRates>();

    println!("SSID: {}", ssid_element.ssid());
    println!("Supported rates: {:?}", supported_rates);
}
