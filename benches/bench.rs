#![feature(test)]

use snowflake::Snowflake;
extern crate test;

#[bench]
fn bench_new(b: &mut test::Bencher) {
    b.iter(|| {
        let worker_id = 1;
        let _ = Snowflake::new(worker_id).unwrap();
    });
}

#[bench]
fn bench_with_config(b: &mut test::Bencher) {
    b.iter(|| {
        let worker_id = 1;
        let worker_id_bits = Some(4);
        let epoch: Option<u64> = Some(1609459200000); // 2021-01-01 00:00:00.000 UTC
        let _ = Snowflake::with_config(worker_id, worker_id_bits, None, epoch).unwrap();
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
