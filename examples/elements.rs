use ieee80211::{
    elements::{rates::SupportedRatesElement, rsn::RSNElement, SSIDElement},
    mgmt_frame::body::ManagementFrameBody,
    IEEE80211Frame,
};
use scroll::Pread;

fn main() {
    let bytes = include_bytes!("../bins/frames/beacon.bin");
    let frame = bytes.pread::<IEEE80211Frame>(0).unwrap();
    let IEEE80211Frame::Management(mgmt_frame) = frame else {
        todo!()
    };
    let ManagementFrameBody::Beacon(beacon) = mgmt_frame.body else {
        todo!()
    };
    // There is a method that does this automatically, see beacon.rs.
    let _ssid_element = beacon.elements.get_first_element::<SSIDElement>().unwrap();
    let _supported_rates = beacon.elements.get_first_element::<SupportedRatesElement>();
    let _rsn_element = beacon.elements.get_first_element::<RSNElement>();
}
