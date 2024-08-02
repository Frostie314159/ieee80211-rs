use ieee80211::{elements::VendorSpecificElement, mgmt_frame::BeaconFrame};
use scroll::Pread;

#[allow(unused)]
pub fn borrow_element(packet: &[u8]) -> Option<VendorSpecificElement<'_>> {
    let beacon = packet.pread::<BeaconFrame>(0).unwrap();
    if let Some(vendor_specific) = beacon.elements.get_first_element::<VendorSpecificElement>() {
        return Some(vendor_specific);
    }

    None
}
#[allow(unused)]
pub fn borrow_ssid(packet: &[u8]) -> Option<&str> {
    let beacon = packet.pread::<BeaconFrame>(0).unwrap();
    beacon.ssid()
}
