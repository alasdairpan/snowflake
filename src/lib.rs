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
//! The number of bits used for the worker ID and sequence
//! number can be customized. The total number of bits must be 64, and the
//! worker ID and sequence number must be at least 1 bit each.
//!
//!
//! # Examples
//!
//! ```
//! use twitter_snowflake::Snowflake;
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
//!   time must be greater than the current time.
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

const MIN_BITS: u64 = 1;
const TIMEOUT_MILLIS: u128 = 1000;

#[cfg(feature = "float-safe")]
const WORKER_ID_BITS: u64 = 4;
#[cfg(not(feature = "float-safe"))]
const WORKER_ID_BITS: u64 = 10;

#[cfg(not(feature = "float-safe"))]
const EPOCH_MILLIS: u64 = 1704038400000; // 2024-01-01 00:00:00.000
#[cfg(feature = "float-safe")]
const EPOCH_SECS: u64 = 1704038400; // 2024-01-01 00:00:00

#[cfg(feature = "float-safe")]
const TIMESTAMP_BITS: u64 = 32;
#[cfg(not(feature = "float-safe"))]
const TIMESTAMP_BITS: u64 = 41;

#[cfg(feature = "float-safe")]
const SAFE_UNUSED_BITS: u64 = 11;
#[cfg(not(feature = "float-safe"))]
const SIGN_BITS: u64 = 1;

#[cfg(not(feature = "float-safe"))]
const MAX_ADJUSTABLE_BITS: u64 = 64 - SIGN_BITS - TIMESTAMP_BITS;
#[cfg(feature = "float-safe")]
const MAX_ADJUSTABLE_BITS: u64 = 64 - SAFE_UNUSED_BITS - TIMESTAMP_BITS;

#[derive(Debug)]
pub struct Snowflake {
    epoch: u64,                   // The epoch time used as a reference
    last_timestamp: u64,          // The most recent generation time
    worker_id: u64,               // The ID of the worker
    sequence: u64,                // The sequence within a time period
    timeout_millis: Option<u128>, // The timeout duration for waiting for the next time period

    max_sequence: u64,    // The maximum sequence value
    timestamp_shift: u64, // The number of bits to shift the timestamp value
    worker_id_shift: u64, // The number of bits to shift the worker ID value
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum SnowflakeError {
    #[error("argument error: {0}")]
    ArgumentError(String),
    #[error("clock move backwards")]
    ClockMoveBackwards,
    #[error("wait for next period timeout")]
    WaitForNextPeriodTimeout,
    #[error("epoch must be greater than the current time")]
    InvalidEpoch,
    #[error("failed to convert timestamp to milliseconds")]
    FailedConvertToMillis,
}

impl Snowflake {
    ///  Create a new Snowflake generator with the default configuration.
    /// The worker ID is the only required parameter.
    /// # Examples
    /// ```
    /// use twitter_snowflake::Snowflake;
    /// let worker_id = 1;
    /// let mut snowflake = Snowflake::new(worker_id).unwrap();
    /// ```
    /// # Errors
    /// Returns an error if the worker ID is greater than the maximum worker ID.
    /// ```
    /// use twitter_snowflake::Snowflake;
    /// let worker_id = 1024;
    /// let snowflake = Snowflake::new(worker_id);
    /// assert!(snowflake.is_err());
    /// ```
    pub fn new(worker_id: u64) -> Result<Self, SnowflakeError> { Self::builder().with_worker_id(worker_id).build() }

    /// Create a new Snowflake builder with the default configuration.
    /// # Examples
    /// ```
    /// use twitter_snowflake::Snowflake;
    /// let mut snowflake = Snowflake::builder().build().unwrap();
    /// ```
    pub fn builder() -> SnowflakeBuilder {
        SnowflakeBuilder {
            worker_id: 0,
            worker_id_bits: Some(WORKER_ID_BITS),
            timeout_millis: Some(TIMEOUT_MILLIS),
            #[cfg(feature = "float-safe")]
            epoch: Some(EPOCH_SECS),
            #[cfg(not(feature = "float-safe"))]
            epoch: Some(EPOCH_MILLIS),
        }
    }

    /// Create a new Snowflake generator with custom configuration.
    /// # Parameters
    /// - `worker_id`: The ID of the worker.
    /// - `worker_id_bits`: The number of bits used for the worker ID. The
    ///   default value is 10 bits.
    /// - `timeout_millis`: The timeout duration for waiting for the next time
    ///   period. The default value is 1000 milliseconds.
    /// - `epoch`: The epoch time used as a reference. The default value is
    ///   1704038400000 (2024-01-01 00:00:00.000).
    fn with_config(
        worker_id: u64,
        worker_id_bits: Option<u64>,
        timeout_millis: Option<u128>,
        epoch: Option<u64>,
    ) -> Result<Self, SnowflakeError> {
        let worker_id_bits = worker_id_bits.unwrap_or(WORKER_ID_BITS);
        if !(MIN_BITS .. MAX_ADJUSTABLE_BITS).contains(&worker_id_bits) {
            return  Err(SnowflakeError::ArgumentError(
                format!(
                    "invalid worker id bits(={worker_id_bits}), expected worker id bits ∈ [{MIN_BITS},{MAX_ADJUSTABLE_BITS})"
                ))
            );
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

        #[cfg(feature = "float-safe")]
        let epoch = epoch.unwrap_or(EPOCH_SECS);
        #[cfg(not(feature = "float-safe"))]
        let epoch = epoch.unwrap_or(EPOCH_MILLIS);

        #[cfg(feature = "float-safe")]
        if epoch >= Self::timestamp()? {
            return Err(SnowflakeError::InvalidEpoch);
        }

        #[cfg(not(feature = "float-safe"))]
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

    /// Generate a new Snowflake ID.
    /// # Examples
    /// ```
    /// use twitter_snowflake::Snowflake;
    /// let worker_id = 1;
    /// let mut snowflake = Snowflake::new(worker_id).unwrap();
    /// let id = snowflake.generate().unwrap();
    /// println!("Generated ID: {}", id);
    /// ```
    pub fn generate(&mut self) -> Result<u64, SnowflakeError> {
        #[cfg(feature = "float-safe")]
        let mut now = self.current_timestamp_since_epoch()?;
        #[cfg(not(feature = "float-safe"))]
        let mut now = self.current_timestamp_millis_since_epoch()?;
        match now.cmp(&self.last_timestamp) {
            // The clock has moved backwards
            Ordering::Less => {
                let possible_sequence = (self.sequence + 1) & self.max_sequence;
                if possible_sequence > 0 {
                    // Continue to use the remaining sequence in the last time period
                    self.sequence = possible_sequence;
                    return Ok((self.last_timestamp << self.timestamp_shift)
                        | (self.worker_id << self.worker_id_shift)
                        | (self.sequence));
                }
                // The sequence of the last period has been used up, throw an error
                return Err(SnowflakeError::ClockMoveBackwards);
            }
            // Same time period, increase the sequence
            Ordering::Equal => {
                self.sequence = (self.sequence + 1) & self.max_sequence;
                if self.sequence == 0 {
                    // The sequence of the current period has been used up, waiting for the next
                    // period
                    let timeout_start = Instant::now();
                    while now <= self.last_timestamp {
                        if let Some(timeout_millis) = self.timeout_millis {
                            if Instant::now().duration_since(timeout_start).as_millis() > timeout_millis {
                                return Err(SnowflakeError::WaitForNextPeriodTimeout);
                            }
                        }
                        #[cfg(feature = "float-safe")]
                        if let Ok(latest_timestamp) = self.current_timestamp_since_epoch() {
                            now = latest_timestamp;
                        }
                        #[cfg(not(feature = "float-safe"))]
                        if let Ok(latest_timestamp_millis) = self.current_timestamp_millis_since_epoch() {
                            now = latest_timestamp_millis;
                        }
                        spin_loop();
                    }
                }
            }
            // New time period, reset the sequence
            Ordering::Greater => {
                self.sequence = 0;
            }
        }
        // Update the most recent generation time
        self.last_timestamp = now;
        Ok((now << self.timestamp_shift) | (self.worker_id << self.worker_id_shift) | (self.sequence))
    }

    #[cfg(feature = "float-safe")]
    fn timestamp() -> Result<u64, SnowflakeError> {
        Ok(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| SnowflakeError::ClockMoveBackwards)?
            .as_secs())
    }

    #[cfg(not(feature = "float-safe"))]
    fn timestamp_millis() -> Result<u64, SnowflakeError> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| SnowflakeError::ClockMoveBackwards)?
            .as_millis()
            .try_into()
            .map_err(|_| SnowflakeError::FailedConvertToMillis)
    }

    #[cfg(feature = "float-safe")]
    fn current_timestamp_since_epoch(&self) -> Result<u64, SnowflakeError> {
        let now = Self::timestamp()?;
        match now.cmp(&self.epoch) {
            Ordering::Less => Err(SnowflakeError::ClockMoveBackwards),
            _ => Ok(now - self.epoch),
        }
    }

    #[cfg(not(feature = "float-safe"))]
    fn current_timestamp_millis_since_epoch(&self) -> Result<u64, SnowflakeError> {
        let now = Self::timestamp_millis()?;
        match now.cmp(&self.epoch) {
            Ordering::Less => Err(SnowflakeError::ClockMoveBackwards),
            _ => Ok(now - self.epoch),
        }
    }
}

/// A builder for creating a Snowflake generator with custom configuration.
pub struct SnowflakeBuilder {
    worker_id: u64,
    worker_id_bits: Option<u64>,
    timeout_millis: Option<u128>,
    epoch: Option<u64>,
}

impl SnowflakeBuilder {
    /// Set the worker ID for the Snowflake generator.
    pub fn with_worker_id(mut self, worker_id: u64) -> Self {
        self.worker_id = worker_id;
        self
    }

    /// Set the number of bits used for the worker ID.
    pub fn with_worker_id_bits(mut self, worker_id_bits: u64) -> Self {
        self.worker_id_bits = Some(worker_id_bits);
        self
    }

    /// Set the timeout duration for waiting for the next time period.
    pub fn with_timeout_millis(mut self, timeout_millis: u128) -> Self {
        self.timeout_millis = Some(timeout_millis);
        self
    }

    /// Set the epoch time.
    pub fn with_epoch(mut self, epoch: u64) -> Self {
        self.epoch = Some(epoch);
        self
    }

    /// Build the Snowflake generator with the specified configuration.
    pub fn build(self) -> Result<Snowflake, SnowflakeError> {
        Snowflake::with_config(self.worker_id, self.worker_id_bits, self.timeout_millis, self.epoch)
    }
}
