use ieee80211::{ssid, supported_rates};

fn main() {
    let _ssid = ssid!("OpenRF");
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
}
