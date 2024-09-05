//! Snowflake is a unique ID generator that generates IDs based on the current
//! time, a node ID, and a step value.
//!
//! The Snowflake struct represents a snowflake generator and provides methods
//! for generating snowflake IDs.
//!
//! # Examples
//!
//! ```
//! use snowflake::Snowflake;
//!
//! // Create a new snowflake generator with a node ID
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
//! - `ArgumentError`: Indicates an invalid argument was provided to the
//!   Snowflake generator.
//! - `ClockMoveBackwards`: Indicates that the system clock has moved backwards.
//! - `WaitForNextSecondTimeout`: Indicates that waiting for the next second has
//!   timed out.
//! - `EpochEarlierThanCurrentTime`: Indicates that the epoch time is earlier
//!   than the current time.
//!
//! # Safety
//!
//! The Snowflake generator is safe to use in a multi-threaded environment as
//! long as each thread has its own instance of the generator.
//!
//! # Notes
//!
//! - The Snowflake generator uses a combination of the current time, a node ID,
//!   and a step value to generate unique IDs.
//! - The node ID should be unique for each node in a distributed system to
//!   avoid ID collisions.
//! - The step value is incremented for each ID generated within the same time
//!   period to ensure uniqueness.
//! - The Snowflake generator has a timeout duration for waiting for the next
//!   time period, which can be configured during initialization.
//! - The epoch time is used as a reference for generating snowflake IDs and
//!   should be set to a time before the current time.
//! - The Snowflake generator supports a maximum of 256 nodes (2^8) and a
//!   maximum of 16777215 steps (2^24 - 1) within a time period.
//! - The Snowflake generator guarantees that generated IDs will be unique as
//!   long as the node ID and step value are unique within a time period.
//! - The Snowflake generator does not guarantee the order of generated IDs, as
//!   the step value can be incremented multiple times within the same time
//!   period.
//! - The Snowflake generator is not suitable for generating sequential IDs or
//!   for use cases that require strict ordering of IDs.
//! - The Snowflake generator is designed to be fast and efficient, using
//!   bitwise operations and system time functions for ID generation.
//! - The Snowflake generator is inspired by Twitter's Snowflake ID generation
//!   algorithm.

use std::{
    cmp::Ordering,
    hint::spin_loop,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const NODE_BITS: u8 = 8;
const MIN_BITS: u8 = 1;
const TOTAL_BITS: u8 = 31;
const TIMEOUT_MILLIS: u128 = 1000;
const EPOCH: u64 = 1704038400; // 2024-01-01 00:00:00

#[derive(Debug, Clone)]
pub struct Snowflake {
    epoch: SystemTime, // The epoch time used as a reference for generating snowflakes
    time: i64,         // The most recent generation time
    node_id: i64,      // The ID of the node generating the snowflakes
    step: i64,         // The step within a time period

    step_mask: i64, // A mask to limit the step value within the allowed range
    time_shift: u8, // The number of bits to shift the time value
    node_shift: u8, // The number of bits to shift the node ID value

    timeout_millis: u128, // The timeout duration for waiting for the next time period
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum SnowflakeError {
    #[error("argument error, expect: {0}")]
    ArgumentError(String),
    #[error("clock move backwards")]
    ClockMoveBackwards,
    #[error("waiting for the next second has timed out")]
    WaitForNextSecondTimeout,
    #[error("epoch must be less than the current time")]
    EpochEarlierThanCurrentTime,
}

impl Snowflake {
    pub fn new(node_id: i64) -> Result<Self, SnowflakeError> {
        Self::with_config(node_id, NODE_BITS, TIMEOUT_MILLIS, EPOCH)
    }

    pub fn with_config(node_id: i64, node_bits: u8, timeout_millis: u128, epoch: u64) -> Result<Self, SnowflakeError> {
        let step_bits = TOTAL_BITS - node_bits;
        if node_bits < MIN_BITS || step_bits < MIN_BITS {
            return Err(SnowflakeError::ArgumentError(format!(
                "{} <= node bits <= {}",
                MIN_BITS,
                TOTAL_BITS - MIN_BITS
            )));
        }

        let node_max = -1 ^ (-1 << node_bits);
        let step_mask = -1 ^ (-1 << step_bits);
        let time_shift = node_bits + step_bits;
        let node_shift = step_bits;

        if node_id < 0 || node_id > node_max {
            return Err(SnowflakeError::ArgumentError(format!("0 <= node id <= {node_max}")));
        }

        let adjusted_epoch = UNIX_EPOCH + Duration::from_secs(epoch);
        match SystemTime::now().duration_since(adjusted_epoch) {
            Err(_) => Err(SnowflakeError::ArgumentError(
                "epoch must be less than the current time".to_string(),
            )),
            Ok(dur) => Ok(Self {
                epoch: adjusted_epoch,
                time: dur.as_secs() as i64,
                node_id,
                step: 0,
                timeout_millis,
                step_mask,
                time_shift,
                node_shift,
            }),
        }
    }

    pub fn generate(&mut self) -> Result<i64, SnowflakeError> {
        let duration_since = |epoch| SystemTime::now().duration_since(epoch);
        match duration_since(self.epoch) {
            Ok(dur) => {
                let mut now = dur.as_secs() as i64;
                match now.cmp(&self.time) {
                    // Clock move backwards
                    Ordering::Less => {
                        let possible_step = (self.step + 1) & self.step_mask;
                        if possible_step > 0 {
                            // Continue to use the remaining step in the last time period
                            self.step = possible_step;
                            return Ok((self.time << self.time_shift)
                                | (self.node_id << self.node_shift)
                                | (self.step));
                        }
                        return Err(SnowflakeError::ClockMoveBackwards);
                    }
                    // Multiple calls within the same time period can increase step
                    Ordering::Equal => {
                        self.step = (self.step + 1) & self.step_mask;
                        if self.step == 0 {
                            // The step of the current period has been used up, waiting for the next period
                            let timeout_start = Instant::now();
                            while now <= self.time {
                                if Instant::now().duration_since(timeout_start).as_millis() > self.timeout_millis {
                                    return Err(SnowflakeError::WaitForNextSecondTimeout);
                                }
                                if let Ok(dur) = duration_since(self.epoch) {
                                    now = dur.as_secs() as i64;
                                }
                                spin_loop();
                            }
                        }
                    }
                    // First call in a time period
                    Ordering::Greater => {
                        self.step = 0;
                    }
                }
                self.time = now; // Update the most recent generation time
                Ok((self.time << self.time_shift) | (self.node_id << self.node_shift) | (self.step))
            }
            // Clock earlier than epoch directly reports an error
            Err(_) => Err(SnowflakeError::EpochEarlierThanCurrentTime),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_unique_ids() {
        let mut snowflake = Snowflake::new(1).unwrap();

        let id1 = snowflake.generate().unwrap();
        let id2 = snowflake.generate().unwrap();
        let id3 = snowflake.generate().unwrap();

        assert_ne!(id1, id2);
        assert_ne!(id1, id3);
        assert_ne!(id2, id3);
    }

    #[test]
    fn test_generate_in_order() {
        let mut snowflake = Snowflake::new(1).unwrap();

        let id1 = snowflake.generate().unwrap();
        let id2 = snowflake.generate().unwrap();
        let id3 = snowflake.generate().unwrap();

        assert!(id1 < id2);
        assert!(id2 < id3);
    }

    #[test]
    fn test_generate_with_multiple_nodes() {
        let mut snowflake1 = Snowflake::new(1).unwrap();
        let mut snowflake2 = Snowflake::new(2).unwrap();

        let id1 = snowflake1.generate().unwrap();
        let id2 = snowflake2.generate().unwrap();
        let id3 = snowflake1.generate().unwrap();
        let id4 = snowflake2.generate().unwrap();

        assert_ne!(id1, id2);
        assert_ne!(id1, id3);
        assert_ne!(id1, id4);
        assert_ne!(id2, id3);
        assert_ne!(id2, id4);
        assert_ne!(id3, id4);
    }

    #[test]
    fn test_generate_with_timeout() {
        let mut snowflake = Snowflake::with_config(1, 8, 100, EPOCH).unwrap();

        let id1 = snowflake.generate().unwrap();
        std::thread::sleep(Duration::from_millis(200));
        let id2 = snowflake.generate().unwrap();

        assert_ne!(id1, id2);
    }
}
