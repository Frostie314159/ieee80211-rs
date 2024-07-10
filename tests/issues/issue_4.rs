use ieee80211::{
    elements::VendorSpecificElement, mgmt_frame::body::ManagementFrameBody, IEEE80211Frame,
};
use scroll::Pread;

#[allow(unused)]
pub fn borrow_element<'a>(packet: &'a [u8]) -> Option<VendorSpecificElement<'a>> {
    let frame = packet.pread::<IEEE80211Frame>(0).unwrap();
    if let IEEE80211Frame::Management(mgmt) = frame {
        if let ManagementFrameBody::Beacon(beacon) = mgmt.body {
            if let Some(vendor_specific) =
                beacon.elements.get_first_element::<VendorSpecificElement>()
            {
                return Some(vendor_specific);
            }
        }
    }

    None
}
