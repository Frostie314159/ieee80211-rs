use std::marker::PhantomData;

use ieee80211::{
    elements::{ElementID, RawIEEE80211Element, ReadElements, SSIDElement},
    ssid,
};

mod dsss_parameter_set;
#[allow(unused)]
mod element_chain;
mod ibss_parameter_set;
mod rsn;
mod ssid;
mod supported_rates;
mod tim;
mod mesh_id;

#[test]
fn test_read_elements() {
    let elements = ReadElements {
        bytes: &[
            0x00, 0x04, b'T', b'e', b's', b't', 0xdd, 0x06, 0x00, 0x80, 0x41, 0x00, 0x13, 0x37,
            0xff, 0x03, 0x00, 0x13, 0x37,
        ],
    };
    assert_eq!(
        elements.get_first_element::<SSIDElement>().unwrap(),
        ssid!("Test")
    );
    assert_eq!(
        elements
            .get_first_element_raw(ElementID::VendorSpecific {
                prefix: &[0x00, 0x80, 0x41, 0x00]
            })
            .unwrap(),
        RawIEEE80211Element {
            tlv_type: 0xdd,
            slice: &[0x00, 0x80, 0x41, 0x00, 0x13, 0x37],
            _phantom: PhantomData
        }
    );
    assert_eq!(
        elements
            .get_first_element_raw(ElementID::ExtId(0x00))
            .unwrap(),
        RawIEEE80211Element {
            tlv_type: 0xff,
            slice: &[0x00, 0x13, 0x37],
            _phantom: PhantomData
        }
    );
}
