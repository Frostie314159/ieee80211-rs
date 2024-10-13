use ieee80211::{
    common::{ControlFrameSubtype, FrameControlField, FrameType, SequenceControl},
    GenericFrame,
};
use mac_parser::{MACAddress, BROADCAST};

const ACK_FRAME_BYTES: &[u8] = &[0xd4, 0x00, 0x37, 0x13, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06];

/// This test covers the FCF, duration and first address.
/// It also checks, that the second address is `None`.
#[test]
fn test_gf_ack() {
    let generic_frame = GenericFrame::new(ACK_FRAME_BYTES, false)
        .expect("Creating a GenericFrame for an ACK failed, even though it's valid.");
    assert_eq!(
        generic_frame.frame_control_field(),
        FrameControlField::new().with_frame_type(FrameType::Control(ControlFrameSubtype::Ack)),
        "Frame type wasn't ACK."
    );
    assert_eq!(generic_frame.duration(), 0x1337, "Duration didn't match.");
    assert_eq!(
        generic_frame.address_1(),
        MACAddress::new([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
        "First MAC address didn't match."
    );
    assert!(
        generic_frame.address_2().is_none(),
        "Second adddress wasn't None."
    );
    // Here we check, that GenericFrame actually queries the subtype for info about which fields
    // are present.
    const ACK_PLUS_ADDRESS_LENGTH: usize = ACK_FRAME_BYTES.len() + 6;
    let mut buf = [0x00u8; ACK_PLUS_ADDRESS_LENGTH];
    buf[..(ACK_FRAME_BYTES.len())].copy_from_slice(ACK_FRAME_BYTES);
    assert!(
        GenericFrame::new(buf.as_slice(), false)
            .unwrap()
            .address_2()
            .is_none(),
        "Second address for extended ACK frame wasn't None."
    );
}

/// This is technically an invalid beacon frame, since some parts in the body are missing.
const BEACON_FRAME_BYTES: &[u8] = &[
    0x80, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
];
#[test]
fn test_gf_beacon() {
    let generic_frame = GenericFrame::new(BEACON_FRAME_BYTES, false).unwrap();
    assert_eq!(
        generic_frame.address_2(),
        Some(MACAddress::new([0x01, 0x02, 0x03, 0x04, 0x05, 0x06])),
        "Second address didn't match."
    );
    assert_eq!(
        generic_frame.address_3(),
        Some(BROADCAST),
        "Third address didn't match."
    );
    assert_eq!(
        generic_frame.sequence_control(),
        Some(SequenceControl::new().with_sequence_number(0x1)),
        "Sequence control didn't match."
    );
}
