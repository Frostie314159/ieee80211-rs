use ieee80211::{
    tlvs::{TLVReadIterator, IEEE80211TLV, SSIDTLV},
    {beacon::BeaconFrameBody, IEEE80211Frame, ManagementFrame, ManagementFrameBody},
};
use scroll::{ctx::MeasureWith, Pread, Pwrite};

const INITIAL_BYTES: &[u8] = &[
    0x80, 0x00, // FCF
    0x00, 0x00, // Duartion
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // TA
    0x00, 0x80, 0x41, 0xff, 0xff, 0xff, // RA
    0x00, 0x80, 0x41, 0x13, 0x37, 0x42, // BSSID
    0x00, 0x00, // Fragment and sequence number
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // Timestamp
    0x04, 0x00, // Beacon interval
    0x00, 0x00, // Capabilities
    0x00, 0x06, // TLV header
    b'L', b'a', b'm', b'b', b'd', b'a', // SSID
];
const NEW_BYTES: &[u8] = &[
    0x80, 0x00, // FCF
    0x00, 0x00, // Duartion
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // TA
    0x00, 0x80, 0x41, 0xff, 0xff, 0xff, // RA
    0x00, 0x80, 0x41, 0x13, 0x37, 0x42, // BSSID
    0x00, 0x00, // Fragment and sequence number
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // Timestamp
    0x04, 0x00, // Beacon interval
    0x00, 0x00, // Capabilities
    0x00, 0x06, // TLV header
    b'O', b'p', b'e', b'n', b'R', b'F', // SSID
];

fn main() {
    let frame = INITIAL_BYTES
        .pread::<IEEE80211Frame<'_, TLVReadIterator>>(0)
        .unwrap();
    let IEEE80211Frame::Management(management_frame) = frame else {
        panic!()
    };
    let ManagementFrameBody::Beacon(beacon) = management_frame.body else {
        panic!()
    };
    println!(
        "BSSID: {:?}, SSID: {:#?}",
        management_frame.header.bssid,
        beacon.ssid()
    );

    let ssid_tlv = IEEE80211TLV::SSID(SSIDTLV::new("OpenRF").unwrap());
    let beacon = BeaconFrameBody {
        capabilities_info: beacon.capabilities_info,
        timestamp: beacon.timestamp,
        beacon_interval: beacon.beacon_interval,
        tagged_payload: [ssid_tlv],
    };
    let management_frame = ManagementFrame {
        header: management_frame.header,
        body: ManagementFrameBody::Beacon(beacon),
    };
    let frame = IEEE80211Frame::Management(management_frame);

    let mut buf = vec![0x00u8; frame.measure_with(&false)];
    buf.pwrite(frame, 0).unwrap();

    assert_eq!(buf, NEW_BYTES);
}
