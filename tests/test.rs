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
fn test_id_unique() {
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

#[cfg(feature = "float-safe")]
mod float_safe_tests {
    use twitter_snowflake::{Snowflake, SnowflakeError};

    /// In float-safe mode, the generated ID must fit within the exact
    /// integer range of an IEEE 754 double-precision float (53-bit mantissa).
    const MAX_SAFE_INTEGER: u64 = (1u64 << 53) - 1; // 2^53 - 1

    #[test]
    fn test_generate() {
        let mut snowflake = Snowflake::new(1).unwrap();
        let id = snowflake.generate().unwrap();
        assert!(id < MAX_SAFE_INTEGER, "ID {id} exceeds IEEE 754 safe integer range");
    }

    #[test]
    fn test_id_unique() {
        let mut snowflake = Snowflake::new(1).unwrap();
        let id1 = snowflake.generate().unwrap();
        let id2 = snowflake.generate().unwrap();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_id_order() {
        let mut snowflake = Snowflake::new(1).unwrap();
        let id1 = snowflake.generate().unwrap();
        let id2 = snowflake.generate().unwrap();
        assert!(id1 < id2);
    }

    #[test]
    fn test_many_ids_stay_in_safe_range() {
        let mut snowflake = Snowflake::new(1).unwrap();
        for _ in 0..1000 {
            let id = snowflake.generate().unwrap();
            assert!(id < MAX_SAFE_INTEGER, "ID {id} exceeds IEEE 754 safe integer range");
        }
    }

    #[test]
    fn test_invalid_worker_id() {
        // float-safe mode uses 4 worker-id bits → max worker ID = 15
        let snowflake = Snowflake::new(16);
        assert!(matches!(snowflake.err(), Some(SnowflakeError::ArgumentError(..))));
    }

    #[test]
    fn test_invalid_epoch() {
        // float-safe mode expects epoch in seconds — supply a far-future second value
        let epoch = 1_000_000_000_000;
        let snowflake = Snowflake::builder().with_worker_id(1).with_epoch(epoch).build();
        assert!(matches!(snowflake.err(), Some(SnowflakeError::InvalidEpoch)));
    }
}
