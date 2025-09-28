# ❄️ Snowflake

**Snowflake** is a lightweight, efficient Rust library that implements Twitter's Snowflake algorithm for generating unique, time-based IDs. Designed for distributed systems, it provides a scalable solution to ID generation, ensuring no collisions even across multiple workers. Perfect for building high-throughput, reliable systems.

## 📚 Features

- **Unique IDs**: Generates 64-bit unique, time-based IDs.
- **High Scalability**: Designed for distributed systems with multiple workers.
- **Efficient**: Low-latency ID generation with no contention.
- **Customizable**: Easy to tweak the bit allocation (worker ID, sequence).
- **Rusty**: Written in pure Rust for performance and safety.
- **Float Safe**: The `float-safe` feature ensures that the maximum ID is less than 2^53, making it compatible with floating-point number precision.

## 📐 How It Works

https://en.wikipedia.org/wiki/Snowflake_ID

The Snowflake algorithm generates IDs based on:

- **Timestamp** (41 bits) - Time in milliseconds since a custom epoch.
- **Worker ID** (10 bits) - A unique identifier for the worker.
- **Sequence** (12 bits) - A per-worker counter that resets every millisecond.

The default bit allocation follows the original Snowflake design but can be customized for your specific needs.

## 🚀 Usage

Add **Snowflake** to your `Cargo.toml`:

```toml
[dependencies]
twitter_snowflake = "1.0.2"
```

Then, import it in your Rust code:

```rust
use {std::error::Error, twitter_snowflake::Snowflake};

fn main() -> Result<(), Box<dyn Error>> {
    let worker_id = 1;
    let mut snowflake = Snowflake::new(worker_id)?;
    let sfid = snowflake.generate()?;
    println!("Snowflake ID: {}", sfid);

    Ok(())
}
```

### Custom Config

You can also set a custom config for ID generation:

```rust
use {std::error::Error, twitter_snowflake::Snowflake};

fn main() -> Result<(), Box<dyn Error>> {
    let worker_id = 1;
    let worker_id_bits = 4;
    let epoch: u64 = 1609459200000; // 2021-01-01 00:00:00.000 UTC

    let mut snowflake = Snowflake::builder()
        .with_worker_id_bits(worker_id_bits)
        .with_worker_id(worker_id)
        .with_epoch(epoch)
        .build()?;

    let sfid = snowflake.generate()?;
    println!("Snowflake ID: {}", sfid);
    Ok(())
}
```

### Float-Safe IDs

To ensure that the generated IDs are compatible with floating-point numbers, enable the `float-safe` feature:

```toml
[dependencies]
twitter_snowflake = { version = "1", features = ["float-safe"] }
```

See all [examples](./examples/).

### 🧪 Running Tests

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

## 🤝 Contributing

Contributions are welcome! Feel free to submit issues, feature requests, or pull requests.

## 📄 License

Snowflake is licensed under the MIT License. See the [LICENSE](./LICENSE) file for more details.
