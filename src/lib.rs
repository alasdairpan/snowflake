//! Snowflake is a unique ID generator that generates IDs based on the current
//! time, a worker ID, and a sequence value.
//!
//! Snowflake IDs are not strictly sequential but are designed to be ordered.
//! The ordering is based on the timestamp part of the ID, so IDs generated at
//! later times will be numerically larger than those generated earlier.
//! However, within the same millisecond, IDs can vary based on the sequence
//! number. Here are key points about ordering:
//! - **Temporal Ordering**: IDs are ordered by the timestamp portion, so IDs
//!   generated at different times are in chronological order.
//! - **Same Timestamp**: When multiple IDs are generated within the same
//!   millisecond, the sequence number differentiates them. This ensures
//!   uniqueness but doesn't guarantee a strict numerical order within that
//!   millisecond.
//! - **Clock Skew**: If system clocks are not synchronized across workers, or
//!   if a machine's clock goes backward, it might lead to IDs that don't
//!   strictly follow the expected order.
//!
//! Default Snowflake ID structure:
//! - **Sign bit**: Always 0.
//! - **Timestamp**: 41 bits, representing milliseconds since a custom epoch.
//! - **Worker ID**: 10 bits, identifying the worker that generated the ID.
//! - **Sequence**: 12 bits, providing uniqueness within the same millisecond.
//! - **Total**: 64 bits.
//!
//! You can customize the number of bits used for the worker ID and sequence
//! number. The total number of bits must be 64, and the worker ID and sequence
//! number must be at least 1 bit each.
//!
//!
//! # Examples
//!
//! ```
//! use snowflake::Snowflake;
//!
//! // Create a new snowflake generator with a worker ID
//! let mut snowflake = Snowflake::new(1).unwrap();
//!
//! // Generate a snowflake ID
//! let id = snowflake.generate().unwrap();
//! println!("Generated ID: {}", id);
//! ```
//!
//! # Errors
//!
//! The Snowflake generator can return the following errors:
//!
//! - [`ArgumentError`](SnowflakeError::ArgumentError): Indicates an invalid
//!   argument was provided to the Snowflake generator.
//! - [`ClockMoveBackwards`](SnowflakeError::ClockMoveBackwards): Indicates that
//!   the system clock has moved backwards.
//! - [`WaitForNextPeriodTimeout`](SnowflakeError::WaitForNextPeriodTimeout):
//!   Indicates that the generator has timed out while waiting for the next time
//!   period.
//! - [`InvalidEpoch`](SnowflakeError::InvalidEpoch): Indicates that the epoch
//!  time must be greater than the current time.
//! - [`FailedConvertToMillis`](SnowflakeError::FailedConvertToMillis):
//!   Indicates that the generator failed to convert the timestamp to
//!   milliseconds.
//!
//!
//! # Safety
//!
//! The Snowflake generator is safe to use in a multi-threaded environment as
//! long as each thread has its own instance of the generator.

use std::{
    cmp::Ordering,
    hint::spin_loop,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

const EPOCH: u64 = 1704038400000; // 2024-01-01 00:00:00.000
const SIGN_BITS: u64 = 1;
const TIMESTAMP_BITS: u64 = 41;
const WORKER_ID_BITS: u64 = 10;

const TIMEOUT_MILLIS: u128 = 1000;

const MIN_BITS: u64 = 1;
const MAX_ADJUSTABLE_BITS: u64 = 64 - SIGN_BITS - TIMESTAMP_BITS;

#[derive(Debug)]
pub struct Snowflake {
    epoch: u64,          // The epoch time used as a reference
    last_timestamp: u64, // The most recent generation time
    worker_id: u64,      // The ID of the worker generating the snowflakes
    sequence: u64,       // The sequence within a time period

    max_sequence: u64,    // A mask to limit the sequence value within the allowed range
    timestamp_shift: u64, // The number of bits to shift the time value
    worker_id_shift: u64, // The number of bits to shift the worker ID value

    timeout_millis: u128, // The timeout duration for waiting for the next time period
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum SnowflakeError {
    #[error("argument error: {0}")]
    ArgumentError(String),
    #[error("clock move backwards")]
    ClockMoveBackwards,
    #[error("waiting for the next period has timed out")]
    WaitForNextPeriodTimeout,
    #[error("epoch must be greater than the current time")]
    InvalidEpoch,
    #[error("failed to convert timestamp to milliseconds")]
    FailedConvertToMillis,
}

impl Snowflake {
    pub fn new(worker_id: u64) -> Result<Self, SnowflakeError> {
        Self::with_config(worker_id, WORKER_ID_BITS, TIMEOUT_MILLIS, EPOCH)
    }

    pub fn with_config(
        worker_id: u64,
        worker_id_bits: u64,
        timeout_millis: u128,
        epoch: u64,
    ) -> Result<Self, SnowflakeError> {
        if !(MIN_BITS .. MAX_ADJUSTABLE_BITS).contains(&worker_id_bits)
            || !(MIN_BITS .. MAX_ADJUSTABLE_BITS).contains(&(MAX_ADJUSTABLE_BITS - worker_id_bits))
        {
            return  Err(SnowflakeError::ArgumentError(format!(
                    "invalid worker id bits(={worker_id_bits}), expected worker id bits ∈ [{MIN_BITS},{MAX_ADJUSTABLE_BITS})",
                )));
        }

        let sequence_bits = MAX_ADJUSTABLE_BITS - worker_id_bits;
        let max_worker_id = (1 << worker_id_bits) - 1;
        let max_sequence = (1 << sequence_bits) - 1;
        let worker_id_shift = sequence_bits;
        let timestamp_shift = worker_id_bits + sequence_bits;

        if worker_id > max_worker_id {
            return Err(SnowflakeError::ArgumentError(format!(
                "invalid worker id(={worker_id}), expected worker id ∈ [0,{max_worker_id}]",
            )));
        }

        if epoch >= Self::timestamp_millis()? {
            return Err(SnowflakeError::InvalidEpoch);
        }

        Ok(Self {
            epoch,
            last_timestamp: 0,
            worker_id,
            sequence: 0,
            timeout_millis,
            max_sequence,
            timestamp_shift,
            worker_id_shift,
        })
    }

    pub fn generate(&mut self) -> Result<u64, SnowflakeError> {
        let mut now = self.current_timestamp_millis_since_epoch()?;
        match now.cmp(&self.last_timestamp) {
            // Clock move backwards
            Ordering::Less => {
                let possible_sequence = (self.sequence + 1) & self.max_sequence;
                if possible_sequence > 0 {
                    // Continue to use the remaining sequence in the last time period
                    self.sequence = possible_sequence;
                    return Ok((self.last_timestamp << self.timestamp_shift)
                        | (self.worker_id << self.worker_id_shift)
                        | (self.sequence));
                }
                return Err(SnowflakeError::ClockMoveBackwards);
            }
            // Multiple calls within the same time period can increase sequence
            Ordering::Equal => {
                self.sequence = (self.sequence + 1) & self.max_sequence;
                if self.sequence == 0 {
                    // The sequence of the current period has been used up, waiting for the next
                    // period
                    let timeout_start = Instant::now();
                    while now <= self.last_timestamp {
                        if Instant::now().duration_since(timeout_start).as_millis() > self.timeout_millis {
                            return Err(SnowflakeError::WaitForNextPeriodTimeout);
                        }
                        if let Ok(latest_timestamp_millis) = self.current_timestamp_millis_since_epoch() {
                            now = latest_timestamp_millis;
                        }
                        spin_loop();
                    }
                }
            }
            // First call in a time period
            Ordering::Greater => {
                self.sequence = 0;
            }
        }
        // Update the most recent generation time
        self.last_timestamp = now;
        Ok((now << self.timestamp_shift) | (self.worker_id << self.worker_id_shift) | (self.sequence))
    }

    fn timestamp_millis() -> Result<u64, SnowflakeError> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| SnowflakeError::ClockMoveBackwards)?
            .as_millis()
            .try_into()
            .map_err(|_| SnowflakeError::FailedConvertToMillis)
    }

    fn current_timestamp_millis_since_epoch(&self) -> Result<u64, SnowflakeError> {
        let now = Self::timestamp_millis()?;
        match now.cmp(&self.epoch) {
            Ordering::Less => Err(SnowflakeError::ClockMoveBackwards),
            _ => Ok(now - self.epoch),
        }
    }
}
