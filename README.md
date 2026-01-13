# kiters

Kiters (kit-rs) is a collection of Rust functions that I use across projects.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
kiters = "0.1.0"
```

## Usage

### Timestamp

Get the current UTC timestamp in `YYYY-MM-DDTHH:MM:SSZ` format.

```rust
use kiters::timestamp::get_utc_timestamp;

fn main() {
    let ts = get_utc_timestamp();
    println!("Current timestamp: {}", ts);
}
```
