# Snowflake

**Snowflake** is a lightweight, efficient Rust library that implements Twitter's Snowflake algorithm for generating unique, time-based IDs. Designed for distributed systems, it provides a scalable solution to ID generation, ensuring no collisions even across multiple workers. Perfect for building high-throughput, reliable systems.

## Features

- **Unique IDs**: Generates 64-bit unique, time-based IDs.
- **High Scalability**: Designed for distributed systems with multiple workers.
- **Efficient**: Low-latency ID generation with no contention.
- **Customizable**: Easy to tweak the bit allocation (worker ID, sequence).
- **Rusty**: Written in pure Rust for performance and safety.
- **Float Safe**: The `float-safe` feature keeps all IDs below 2^53, the exact integer limit of IEEE 754 double-precision floats.

## How It Works

[https://en.wikipedia.org/wiki/Snowflake_ID](https://en.wikipedia.org/wiki/Snowflake_ID)

The Snowflake algorithm generates IDs based on:

- **Timestamp** (41 bits) - Time in milliseconds since a custom epoch.
- **Worker ID** (10 bits) - A unique identifier for the worker.
- **Sequence** (12 bits) - A per-worker counter that resets every millisecond.

The default bit allocation follows the original Snowflake design but can be customized for your specific needs.

## Usage

Add **Snowflake** to your `Cargo.toml`:

```toml
[dependencies]
twitter_snowflake = "1.0.2"
```

Then, import it in your Rust code:

```rust
use twitter_snowflake::Snowflake;

fn main() {
    let worker_id = 1;
    let mut snowflake = Snowflake::new(worker_id).unwrap();
    let sfid = snowflake.generate().unwrap();
    println!("Snowflake ID: {}", sfid);
}
```

### Custom Config

You can also set a custom config for ID generation:

```rust
use twitter_snowflake::Snowflake;

fn main() {
    let worker_id = 1;
    let worker_id_bits = 4;
    let epoch = 1609459200000; // 2021-01-01 00:00:00.000 UTC

    let mut snowflake = Snowflake::builder()
        .with_worker_id_bits(worker_id_bits)
        .with_worker_id(worker_id)
        .with_epoch(epoch)
        .build()
        .unwrap();

    let sfid = snowflake.generate().unwrap();
    println!("Snowflake ID: {}", sfid);
}
```

### Float-Safe IDs

To keep generated IDs within the exact integer range of IEEE 754 double-precision floats (53-bit mantissa), enable the `float-safe` feature:

```toml
[dependencies]
twitter_snowflake = { version = "1", features = ["float-safe"] }
```

When `float-safe` is enabled, the timestamp shrinks to 32 bits (seconds instead of milliseconds) and 11 unused bits are reserved, keeping all IDs below 2^53. The worker ID and sequence bits remain customizable within the remaining 21 adjustable bits.

See all [examples](./examples/).

### Running Tests

To run the test suite, use:

```bash
cargo test
```

### Benchmark

Benchmarks are run using [Criterion](https://github.com/bheisler/criterion.rs) and work on stable Rust.

- Rust version: rustc 1.90.0 (1159e78c4 2025-09-14)
- Machine setup: Apple M4 4.46 GHz CPU 32GB RAM

| Benchmark        | Min (ns) | Mean (ns) | Max (ns) | Description                           |
| ---------------- | -------- | --------- | -------- | ------------------------------------- |
| `bench_new`      | 22.088   | 22.131    | 22.176   | Creating a new Snowflake instance     |
| `bench_builder`  | 22.368   | 22.395    | 22.423   | Building Snowflake with custom config |
| `bench_generate` | 243.83   | 244.12    | 244.44   | Generating a new Snowflake ID         |

## Contributing

Contributions are welcome! Feel free to submit issues, feature requests, or pull requests.

## License

Snowflake is licensed under the MIT License. See the [LICENSE](./LICENSE) file for more details.
