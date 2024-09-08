use snowflake::{Snowflake, SnowflakeError};

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
    let worker_id_bits = Some(100);
    let snowflake = Snowflake::with_config(worker_id, worker_id_bits, None, None);
    assert!(matches!(snowflake.err(), Some(SnowflakeError::ArgumentError(..))));
}

#[test]
fn test_invalid_epoch() {
    let worker_id = 1;
    let epoch = Some(1_000_000_000_000_000);
    let snowflake = Snowflake::with_config(worker_id, None, None, epoch);
    assert!(matches!(snowflake.err(), Some(SnowflakeError::InvalidEpoch)));
}
