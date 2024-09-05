# 🚀 Snowflake

**Snowflake** is a lightweight, efficient Rust library that implements Twitter's Snowflake ❄️ algorithm for generating unique, time-based IDs. Designed for distributed systems, it provides a scalable solution to ID generation, ensuring no collisions even across multiple nodes. Perfect for building high-throughput, reliable systems.

## 📚 Features

- **Unique IDs**: Generates 64-bit unique, time-based IDs.
- **High Scalability**: Designed for distributed systems with multiple nodes.
- **Efficient**: Low-latency ID generation with no contention.
- **Customizable**: Easy to tweak the bit allocation (timestamp, node ID, sequence).
- **Rusty** 🦀: Written in pure Rust for performance and safety.

## 📐 How It Works

The Snowflake algorithm generates IDs based on:

- **Timestamp** (32 bits) - Time in seconds since a custom epoch.
- **Node ID** (8 bits) - A unique identifier for the node.
- **Sequence** (23 bits) - A per-node counter that resets every millisecond.

The default bit allocation follows the original Snowflake design but can be customized for your specific needs.

## 🚀 Usage

Add **Snowflake** to your `Cargo.toml`:

```toml
[dependencies]
snowflake = { git = "ssh://git@github.com/trayvonpan/snowflake.git" }
```

Then, import it in your Rust code:

```rust
use snowflake::Snowflake;

fn main() {
    let mut generator = Snowflake::new(1).unwrap();
    let id = generator.generate().unwrap();
    println!("Generated ID: {}", id);
}
```

### Custom Epoch

You can also set a custom epoch for ID generation:

```rust
let epoch = 1609459200000; // January 1, 2021
let mut generator = Snowflake::with_config(1, 4, 1000, epoch).unwrap();
```

## 🔧 Configuration

You can adjust the bit allocation if needed by customizing the timestamp, node ID, or sequence length. This allows flexibility for systems with more nodes or longer sequences per second.

### 🧪 Running Tests

To run the test suite, use:

```bash
cargo test
```

## 🚧 Roadmap

- [ ] Add support for 128-bit Snowflakes
- [ ] More flexible bit partitioning
- [ ] Benchmarking tools for performance testing

## 🤝 Contributing

Contributions are welcome! Feel free to submit issues, feature requests, or pull requests.

## 📄 License

Snowflake is licensed under the MIT License. See the [LICENSE](./LICENSE) file for more details.
