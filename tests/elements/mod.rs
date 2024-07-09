use std::marker::PhantomData;

use ieee80211::{
    elements::{ElementID, RawIEEE80211Element, ReadElements, SSIDElement},
    ssid,
};

mod dsss_parameter_set;
mod ibss_parameter_set;
mod rsn_element;
mod ssid;
mod supported_rates;

#[macro_export]
macro_rules! gen_element_rw_test {
    ($test_name:ident, $element_type:ty, $expected_element:expr, $expected_bytes:expr) => {
        #[test]
        fn $test_name() {
            use ::scroll::{Pread, Pwrite, ctx::MeasureWith};

            let read_element = $expected_bytes.pread::<$element_type>(0).unwrap();
            assert_eq!(read_element, $expected_element, "The {} read from the supplied bytes didn't match, what was expected. 
                Either check the test case for correctness or the TryFromCtx implementation.", stringify!($element_type));

            let expected_length = read_element.measure_with(&());
            let mut buf = ::std::vec![0x00; expected_length];
            let written = buf.pwrite($expected_element, 0).unwrap();
            assert_eq!(written, expected_length, "The amount of bytes, written by TryIntoCtx, for {}, didn't match the amount returned by MeasureWith. 
                Either check the test case for correctness or the TryIntoCtx and or MeasureWith implementation.", stringify!($element_type));
            assert_eq!(buf, $expected_bytes, "The bytes, written by TryIntoCtx, for {}, didn't match what was expected.
                Either check the test case for correctness or the TryIntoCtx implementation.", stringify!($element_type));
        }
    };
}
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
                oui: [0x00, 0x80, 0x41],
                subtype: 0
            })
            .unwrap(),
        RawIEEE80211Element {
            tlv_type: 0xdd,
            slice: &[0x00, 0x80, 0x41, 0x00, 0x13, 0x37],
            _phantom: PhantomData
        }
    );
    assert_eq!(
        elements.get_first_element_raw(ElementID::ExtId(0x00)).unwrap(),
        RawIEEE80211Element {
            tlv_type: 0xff,
            slice: &[0x00, 0x13, 0x37],
            _phantom: PhantomData
        }
    );
}
