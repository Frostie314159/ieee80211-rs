use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ieee80211::{
    elements::{
        element_chain::{ChainElement, ElementChainEnd},
        types::RSNRepr,
    },
    mgmt_frame::{
        body::{BeaconFrameBody, ToManagementFrameBody},
        header::ManagementFrameHeader,
        ManagementFrame,
    },
    ssid, supported_rates, IEEE80211Frame, ToFrame,
};
use scroll::{Pread, Pwrite};

macro_rules! gen_frame_benchmark {
    ($name:ident) => {
        pub fn $name(criterion: &mut Criterion) {
            let bytes = include_bytes!(concat!(
                "../bins/frames/",
                concat!(stringify!($name), ".bin")
            ));
            criterion.bench_function(concat!(stringify!($name), "_read"), |b| {
                b.iter(|| {
                    let _ = black_box(bytes).pread::<IEEE80211Frame>(0).unwrap();
                })
            });
            let parsed = bytes.pread::<IEEE80211Frame>(0).unwrap();
            let mut buf = [0x00; 8000];
            criterion.bench_function(concat!(stringify!($name), "_write"), |b| {
                b.iter(|| {
                    let _ = buf.pwrite(black_box(parsed), 0).unwrap();
                })
            });
        }
    };
}
gen_frame_benchmark!(qos_data);
gen_frame_benchmark!(beacon);
gen_frame_benchmark!(action_vendor);
pub fn element_chain(criterion: &mut Criterion) {
    let frame = ManagementFrame {
        header: ManagementFrameHeader::default(),
        body: BeaconFrameBody {
            elements: ElementChainEnd::new(ssid!("OpenRF")).append(supported_rates![1]),
            ..Default::default()
        }
        .to_management_frame_body(),
    }
    .to_frame();
    let mut buf = [0x00; 0xff];
    criterion.bench_function("element_chain_write", |b| {
        b.iter(|| {
            let _ = buf.pwrite(black_box(frame), 0).unwrap();
        })
    });
}
macro_rules! gen_element_benchmarks {
    ($(
        ($element:ty, $file_name:expr)
    ),*) => {
        pub fn bench_elements(criterion: &mut Criterion) {
            use ::ieee80211::elements::types::ElementTypeRepr;
            $(
                {
                    const BYTES: &[u8] = include_bytes!(concat!("../bins/elements/", concat!($file_name, ".bin")));
                    criterion.bench_function(concat!($file_name, "_read"), |b| {
                        b.iter(|| {
                            let _ = black_box(BYTES).pread::<<$element as ElementTypeRepr>::ElementType<'_>>(0).unwrap();
                        })
                    });
                    let parsed = BYTES.pread::<<$element as ElementTypeRepr>::ElementType<'_>>(0).unwrap();
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
gen_element_benchmarks!((RSNRepr, "rsn"));

criterion_group!(
    benches,
    beacon,
    action_vendor,
    qos_data,
    element_chain,
    bench_elements
);

criterion_main!(benches);
