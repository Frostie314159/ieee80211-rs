//! This example demonstrates, how to use the [DynamicManagementFrame].

use std::marker::PhantomData;

use ieee80211::{
    common::CapabilitiesInformation,
    element_chain,
    elements::{
        rsn::RsnElement,
        tim::{TIMBitmap, TIMElement},
        DSSSParameterSetElement,
    },
    mgmt_frame::{body::BeaconBody, BeaconFrame, DynamicManagementFrame, ManagementFrameHeader},
    ssid, supported_rates,
};
use mac_parser::{MACAddress, BROADCAST};

const MAC_ADDRESS: MACAddress = MACAddress::new([0x00, 0x80, 0x41, 0x13, 0x37, 0x42]);

fn main() {
    let fixed_beacon_frame = BeaconFrame {
        header: ManagementFrameHeader {
            receiver_address: BROADCAST,
            transmitter_address: MAC_ADDRESS,
            bssid: MAC_ADDRESS,
            ..Default::default()
        },
        body: BeaconBody {
            timestamp: 0,
            beacon_interval: 100,
            capabilities_info: CapabilitiesInformation::new().with_is_ess(true),
            elements: element_chain! {
                ssid!("Test"),
                supported_rates![
                    1 B,
                    2 B,
                    5.5 B,
                    11 B,
                    6,
                    9,
                    12,
                    18
                ],
                DSSSParameterSetElement {
                    current_channel: 1
                },
                TIMElement {
                    dtim_count: 1,
                    dtim_period: 2,
                    bitmap: None::<TIMBitmap<&[u8]>>,
                    _phantom: PhantomData
                }
            },
            ..Default::default()
        },
    };
    let mut buf = [0u8; 300];
    let mut dynamic_frame =
        DynamicManagementFrame::new(fixed_beacon_frame, buf.as_mut_slice()).unwrap();
    dynamic_frame
        .add_element(RsnElement::WPA2_PERSONAL)
        .unwrap();
    let written_length = dynamic_frame.finish(false).unwrap();
    let frame = &buf[..written_length];
    println!("Frame bytes: {frame:02x?}");
}
