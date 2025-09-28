use criterion::{black_box, criterion_group, criterion_main, Criterion};
use twitter_snowflake::Snowflake;

fn bench_new(c: &mut Criterion) {
    c.bench_function("bench_new", |b| {
        b.iter(|| {
            let worker_id = 1;
            let _ = black_box(Snowflake::new(worker_id).unwrap());
        });
    });
}

fn bench_builder(c: &mut Criterion) {
    c.bench_function("bench_builder", |b| {
        b.iter(|| {
            let worker_id = 1;
            let worker_id_bits = 4;
            let epoch: u64 = 1609459200000; // 2021-01-01 00:00:00.000 UTC

            let _ = black_box(
                Snowflake::builder()
                    .with_worker_id_bits(worker_id_bits)
                    .with_worker_id(worker_id)
                    .with_epoch(epoch)
                    .build()
                    .unwrap(),
            );
        });
    });
}

fn bench_generate(c: &mut Criterion) {
    let worker_id = 1;
    let mut snowflake = Snowflake::new(worker_id).unwrap();
    c.bench_function("bench_generate", |b| {
        b.iter(|| {
            let _ = black_box(snowflake.generate().unwrap());
        });
    });
}

criterion_group!(benches, bench_new, bench_builder, bench_generate);
criterion_main!(benches);
