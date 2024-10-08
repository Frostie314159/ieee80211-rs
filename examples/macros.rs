//! This example demonstrates how to use some of the macros for generating elements.

use ieee80211::{ssid, supported_rates, tim_bitmap};

fn main() {
    let _ssid = ssid!("Test");
    let _rates = supported_rates![
        1 B,
        1.5 B,
        3,
        54,
        21,
        1,
        1,
        1
    ];
    let _tim_bitmap = tim_bitmap!(2000 => 2007);
}
