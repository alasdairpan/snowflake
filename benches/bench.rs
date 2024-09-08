#![feature(test)]

use twitter_snowflake::Snowflake;
extern crate test;

#[bench]
fn bench_new(b: &mut test::Bencher) {
    b.iter(|| {
        let worker_id = 1;
        let _ = Snowflake::new(worker_id).unwrap();
    });
}

#[bench]
fn bench_builder(b: &mut test::Bencher) {
    b.iter(|| {
        let worker_id = 1;
        let worker_id_bits = 4;
        let epoch: u64 = 1609459200000; // 2021-01-01 00:00:00.000 UTC

        let _ = Snowflake::builder()
            .with_worker_id_bits(worker_id_bits)
            .with_worker_id(worker_id)
            .with_epoch(epoch)
            .build()
            .unwrap();
    });
}

#[bench]
fn bench_generate(b: &mut test::Bencher) {
    let worker_id = 1;
    let mut snowflake = Snowflake::new(worker_id).unwrap();
    b.iter(|| {
        let _ = snowflake.generate().unwrap();
    });
}
