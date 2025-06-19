use core::marker::PhantomData;
use ieee80211::{
    common::FCFFlags,
    crypto::{
        deserialize_eapol_data_frame,
        eapol::{EapolKeyFrame, KeyDescriptorVersion, KeyInformation},
        serialize_eapol_data_frame,
    },
    data_frame::{header::DataFrameHeader, DataFrame},
    element_chain,
    elements::rsn::IEEE80211AkmType,
};
use llc_rs::{EtherType, SnapLlcFrame};
use scroll::Pread;

#[test]
fn test_eapol_serialization() {
    let eapol_key_frame = EapolKeyFrame {
        key_information: KeyInformation::new()
            .with_key_descriptor_version(KeyDescriptorVersion::AesHmacSha1)
            .with_is_pairwise(true)
            .with_key_mic(true),
        key_length: 16,
        key_replay_counter: 1,
        key_nonce: [0xff; 32],
        key_iv: 0,
        key_rsc: 0,
        key_mic: &[0x00u8; 16],
        key_data: element_chain! {
            ieee80211::elements::rsn::RsnElement::WPA2_PERSONAL
        },
        _phantom: PhantomData,
    };
    let data_frame = DataFrame {
        header: DataFrameHeader {
            fcf_flags: FCFFlags::new().with_to_ds(true),
            ..Default::default()
        },
        payload: Some(SnapLlcFrame {
            oui: [0x00; 3],
            ether_type: EtherType::Eapol,
            payload: eapol_key_frame,
            _phantom: PhantomData,
        }),
        _phantom: PhantomData,
    };
    let mut buf = [0x00u8; 500];
    let kck = [0xaa; 16];
    let kek = [0xbb; 16];
    let mut temp_buffer = [0x00u8; 100];
    let written = serialize_eapol_data_frame(
        Some(&kck),
        Some(&kek),
        data_frame,
        buf.as_mut_slice(),
        temp_buffer.as_mut_slice(),
    )
    .unwrap();
    panic!("{:?}", &buf[..written]);
}
#[test]
fn test_eapol_deserialization() {
    const EAPOL_KEY_FRAME: &[u8] = &[
        0x08, 0x02, 0x2c, 0x00, 0x00, 0x0d, 0x93, 0x82, 0x36, 0x3a, 0x00, 0x0c, 0x41, 0x82, 0xb2,
        0x55, 0x00, 0x0c, 0x41, 0x82, 0xb2, 0x55, 0xc0, 0xfc, 0xaa, 0xaa, 0x03, 0x00, 0x00, 0x00,
        0x88, 0x8e, 0x02, 0x03, 0x00, 0xaf, 0x02, 0x13, 0xca, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x01, 0x3e, 0x8e, 0x96, 0x7d, 0xac, 0xd9, 0x60, 0x32, 0x4c, 0xac, 0x5b,
        0x6a, 0xa7, 0x21, 0x23, 0x5b, 0xf5, 0x7b, 0x94, 0x97, 0x71, 0xc8, 0x67, 0x98, 0x9f, 0x49,
        0xd0, 0x4e, 0xd4, 0x7c, 0x69, 0x33, 0xf5, 0x7b, 0x94, 0x97, 0x71, 0xc8, 0x67, 0x98, 0x9f,
        0x49, 0xd0, 0x4e, 0xd4, 0x7c, 0x69, 0x34, 0xcf, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7d, 0x0a, 0xf6, 0xdf, 0x51, 0xe9, 0x9c,
        0xde, 0x7a, 0x18, 0x74, 0x53, 0xf0, 0xf9, 0x35, 0x37, 0x00, 0x50, 0xcf, 0xa7, 0x2c, 0xde,
        0x35, 0xb2, 0xc1, 0xe2, 0x31, 0x92, 0x55, 0x80, 0x6a, 0xb3, 0x64, 0x17, 0x9f, 0xd9, 0x67,
        0x30, 0x41, 0xb9, 0xa5, 0x93, 0x9f, 0xa1, 0xa2, 0x01, 0x0d, 0x2a, 0xc7, 0x94, 0xe2, 0x51,
        0x68, 0x05, 0x5f, 0x79, 0x4d, 0xdc, 0x1f, 0xdf, 0xae, 0x35, 0x21, 0xf4, 0x44, 0x6b, 0xfd,
        0x11, 0xda, 0x98, 0x34, 0x5f, 0x54, 0x3d, 0xf6, 0xce, 0x19, 0x9d, 0xf8, 0xfe, 0x48, 0xf8,
        0xcd, 0xd1, 0x7a, 0xdc, 0xa8, 0x7b, 0xf4, 0x57, 0x11, 0x18, 0x3c, 0x49, 0x6d, 0x41, 0xaa,
        0x0c,
    ];
    let mut eapol_frame = EAPOL_KEY_FRAME.to_vec();
    let data_frame_header = eapol_frame.pread_with::<DataFrame>(0, false).unwrap().header;
    let kck = hex::decode("b1cd792716762903f723424cd7d16511").unwrap();
    let kek = hex::decode("82a644133bfa4e0b75d96d2308358433").unwrap();
    let mut temp_buffer = [0u8; 100];
    let eapol_key_frame = deserialize_eapol_data_frame(
        Some(kck.as_slice().try_into().unwrap()),
        Some(kek.as_slice().try_into().unwrap()),
        &mut eapol_frame,
        &mut temp_buffer,
        IEEE80211AkmType::Psk,
        false,
    )
    .unwrap();
    temp_buffer = [0u8; 100];
    let mut out = vec![0x00u8; 300];
    let written = serialize_eapol_data_frame(
        Some(kck.as_slice().try_into().unwrap()),
        Some(kek.as_slice().try_into().unwrap()),
        DataFrame {
            header: data_frame_header,
            payload: Some(SnapLlcFrame {
                oui: [0x00u8; 3],
                ether_type: EtherType::Eapol,
                payload: EapolKeyFrame {
                    // key_information: eapol_key_frame.key_information.with_key_mic(false),
                    key_mic: [0u8; 16].as_slice(),
                    ..eapol_key_frame
                },
                _phantom: PhantomData,
            }),
            _phantom: PhantomData,
        },
        &mut out,
        &mut temp_buffer,
    ).unwrap();
    assert_eq!(&out[..written], EAPOL_KEY_FRAME);
}
