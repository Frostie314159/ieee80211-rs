use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ieee80211::IEEE80211Frame;
use scroll::{Pread, Pwrite};

macro_rules! gen_benchmark {
    ($name:ident) => {
        pub fn $name(criterion: &mut Criterion) {
            let bytes = include_bytes!(concat!("../bins/", concat!(stringify!($name), ".bin")));
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
gen_benchmark!(qos_data);
gen_benchmark!(beacon);
gen_benchmark!(action_vendor);
criterion_group!(benches, beacon, action_vendor, qos_data);

criterion_main!(benches);
