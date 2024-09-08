use twitter_snowflake::{Snowflake, SnowflakeError};

#[test]
fn test_new() {
    let worker_id = 1;
    let snowflake = Snowflake::new(worker_id);
    assert!(snowflake.is_ok());
}

#[test]
fn test_invalid_worker_id() {
    let worker_id = 1024;
    let snowflake = Snowflake::new(worker_id);
    assert!(matches!(snowflake.err(), Some(SnowflakeError::ArgumentError(..))));
}

#[test]
fn test_generate() {
    let worker_id = 1;
    let mut snowflake = Snowflake::new(worker_id).unwrap();
    let sfid = snowflake.generate();
    assert!(sfid.is_ok());
}

#[test]
fn test_id_unqiue() {
    let worker_id = 1;
    let mut snowflake = Snowflake::new(worker_id).unwrap();
    let sfid1 = snowflake.generate().unwrap();
    let sfid2 = snowflake.generate().unwrap();
    assert_ne!(sfid1, sfid2);
}

#[test]
fn test_id_order() {
    let worker_id = 1;
    let mut snowflake = Snowflake::new(worker_id).unwrap();
    let sfid1 = snowflake.generate().unwrap();
    let sfid2 = snowflake.generate().unwrap();
    assert!(sfid1 < sfid2);
}

#[test]
fn test_invalid_worker_id_bits() {
    let worker_id = 1;
    let worker_id_bits = 100;
    let snowflake = Snowflake::builder()
        .with_worker_id(worker_id)
        .with_worker_id_bits(worker_id_bits)
        .build();
    assert!(matches!(snowflake.err(), Some(SnowflakeError::ArgumentError(..))));
}

#[test]
fn test_invalid_epoch() {
    let worker_id = 1;
    let epoch = 1_000_000_000_000_000;
    let snowflake = Snowflake::builder().with_worker_id(worker_id).with_epoch(epoch).build();
    assert!(matches!(snowflake.err(), Some(SnowflakeError::InvalidEpoch)));
}
