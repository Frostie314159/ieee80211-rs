use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ieee80211::{
    elements::{
        element_chain::{ChainElement, ElementChainEnd},
        rsn::RsnElement,
        ElementID, ReadElements,
    },
    mgmt_frame::{body::BeaconBody, BeaconFrame, ManagementFrameHeader, RawActionFrame},
    ssid, supported_rates,
};
use scroll::{Pread, Pwrite};

macro_rules! gen_frame_benchmark {
    ($name:ident, $frame_type:ty) => {
        pub fn $name(criterion: &mut Criterion) {
            let bytes = include_bytes!(concat!(
                "../bins/frames/",
                concat!(stringify!($name), ".bin")
            ));
            criterion.bench_function(concat!(stringify!($name), "_read"), |b| {
                b.iter(|| {
                    let _ = black_box(bytes).pread::<$frame_type>(0).unwrap();
                })
            });
            let parsed = bytes.pread::<$frame_type>(0).unwrap();
            let mut buf = [0x00; 8000];
            criterion.bench_function(concat!(stringify!($name), "_write"), |b| {
                b.iter(|| {
                    let _ = buf.pwrite(black_box(parsed), 0).unwrap();
                })
            });
        }
    };
}
// gen_frame_benchmark!(qos_data);
gen_frame_benchmark!(beacon, BeaconFrame);
gen_frame_benchmark!(action_vendor, RawActionFrame);
pub fn element_chain(criterion: &mut Criterion) {
    let frame = BeaconFrame {
        header: ManagementFrameHeader::default(),
        body: BeaconBody {
            elements: ElementChainEnd::new(ssid!("OpenRF")).append(supported_rates![1]),
            ..Default::default()
        },
    };
    let mut buf = [0x00; 0xff];
    criterion.bench_function("element_chain_write", |b| {
        b.iter(|| {
            let _ = buf.pwrite(black_box(frame), 0).unwrap();
        })
    });
}
pub fn get_element(criterion: &mut Criterion) {
    let read_elements = ReadElements {
        bytes: &[0x00, 0x04, b'T', b'e', b's', b't', 0x03, 0x01, 0x13],
    };
    criterion.bench_function("get_first_element_raw", |b| {
        b.iter(|| {
            let _ = read_elements.get_first_element_raw(ElementID::Id(0x00));
        })
    });
}
macro_rules! gen_element_benchmarks {
    ($(
        ($element:ty, $file_name:expr)
    ),*) => {
        pub fn bench_elements(criterion: &mut Criterion) {
            $(
                {
                    const BYTES: &[u8] = include_bytes!(concat!("../bins/elements/", concat!($file_name, ".bin")));
                    criterion.bench_function(concat!($file_name, "_read"), |b| {
                        b.iter(|| {
                            let _ = black_box(BYTES).pread::<$element>(0).unwrap();
                        })
                    });
                    let parsed = BYTES.pread::<$element>(0).unwrap();
                    let mut buf = [0x00; 8000];
                    criterion.bench_function(concat!($file_name, "_write"), |b| {
                        b.iter(|| {
                            let _ = buf.pwrite(black_box(parsed), 0).unwrap();
                        })
                    });
                }
            ),*
        }
    };
}
gen_element_benchmarks!((RsnElement, "rsn"));

criterion_group!(
    benches,
    beacon,
    action_vendor,
    // qos_data,
    element_chain,
    bench_elements,
    get_element
);

criterion_main!(benches);
