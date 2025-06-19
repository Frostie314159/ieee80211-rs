//! This example demonstrates how to use the element API.

use ieee80211::{
    elements::{rsn::RsnElement, tim::TIMElement, SSIDElement},
    mgmt_frame::BeaconFrame,
};
use scroll::Pread;

fn main() {
    let bytes = include_bytes!("../bins/frames/beacon.bin");
    let beacon = bytes.pread::<BeaconFrame>(0).unwrap();
    // There is a method that does this automatically, see beacon.rs.
    let ssid_element = beacon.elements.get_first_element::<SSIDElement>().unwrap();
    let rsn_element = beacon.elements.get_first_element::<RsnElement>().unwrap();
    let tim_element = beacon.elements.get_first_element::<TIMElement>().unwrap();
    println!("SSID: {}", ssid_element.ssid());
    println!("RSN: {rsn_element:#?}");
    println!("{tim_element}");
}
