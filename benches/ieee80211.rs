use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ieee80211::frames::Frame;
use scroll::Pread;

pub fn criterion_benchmark_data(criterion: &mut Criterion) {
    let bytes = include_bytes!("../bins/qos_data.bin");
    criterion.bench_function("bench_qos_data_read", |b| {
        b.iter(|| {
            let _ = black_box(bytes).pread::<Frame>(0);
        })
    });
}

criterion_group!(benches, criterion_benchmark_data);
criterion_main!(benches);
