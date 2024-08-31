use ieee80211::data_frame::{builder::DataFrameBuilder, DataFrame, DataFrameReadPayload};
use mac_parser::MACAddress;
use scroll::{ctx::MeasureWith, Pread, Pwrite};

const OUR_MAC_ADDRESS: MACAddress = MACAddress::new([0x00, 0x20, 0x91, 0x13, 0x37, 0x00]);
const AP_MAC_ADDRESS: MACAddress = MACAddress::new([0x00, 0x20, 0x91, 0x13, 0x37, 0x01]);

const EXPECTED_DATA_FRAME: DataFrame<'_, &[u8]> = DataFrameBuilder::new()
    .to_ds()
    .category_data()
    .payload::<&[u8]>(&[0x13, 0x37])
    .destination_address(AP_MAC_ADDRESS)
    .source_address(OUR_MAC_ADDRESS)
    .bssid(AP_MAC_ADDRESS)
    .build();
const EXPECTED_BYTES: &[u8] = &[
    0x08, 0x01, 0x00, 0x00, 0x00, 0x20, 0x91, 0x13, 0x37, 0x01, 0x00, 0x20, 0x91, 0x13, 0x37, 0x00,
    0x00, 0x20, 0x91, 0x13, 0x37, 0x01, 0x00, 0x00, 0x13, 0x37,
];

#[test]
fn test_data_frame_rw() {
    let read = EXPECTED_BYTES.pread_with::<DataFrame>(0, false).unwrap();
    assert_eq!(read.header, EXPECTED_DATA_FRAME.header);
    let Some(DataFrameReadPayload::Single(payload)) = read.payload else {
        unreachable!()
    };
    assert_eq!(payload, EXPECTED_DATA_FRAME.payload.unwrap());

    let mut buf = vec![0x00u8; EXPECTED_DATA_FRAME.measure_with(&false)];
    buf.pwrite(EXPECTED_DATA_FRAME, 0).unwrap();
    assert_eq!(buf, EXPECTED_BYTES);
}
