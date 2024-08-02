use ieee80211::{
    elements::{rates::SupportedRatesElement, rsn::RSNElement, SSIDElement},
    mgmt_frame::BeaconFrame,
};
use scroll::Pread;

fn main() {
    let bytes = include_bytes!("../bins/frames/beacon.bin");
    let beacon = bytes.pread::<BeaconFrame>(0).unwrap();
    // There is a method that does this automatically, see beacon.rs.
    let _ssid_element = beacon.elements.get_first_element::<SSIDElement>().unwrap();
    let _supported_rates = beacon.elements.get_first_element::<SupportedRatesElement>();
    let _rsn_element = beacon.elements.get_first_element::<RSNElement>();
}
