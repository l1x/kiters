# kiters

Kiters (kit-rs) is a collection of Rust utilities for timestamps, request IDs, and external IDs.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
kiters = "0.2.0"
```

## Modules

### timestamp

Get the current UTC timestamp in `YYYY-MM-DDTHH:MM:SSZ` format.

```rust
use kiters::timestamp::get_utc_timestamp;

let ts = get_utc_timestamp();
println!("Current timestamp: {}", ts);  // 2026-01-13T12:00:00Z
```

#### Benchmarks

Using Criterion (`cargo bench --bench timestamp_bench`):

| Implementation | Time | Throughput |
|----------------|------|------------|
| `OffsetDateTime::now_utc()` | **37 ns** | 27 M/s |
| `get_utc_formatter()` | 0.36 ns | 2.8 G/s |
| Format only (pre-obtained time) | 194 ns | 5.1 M/s |
| `get_utc_timestamp()` (full) | 220 ns | 4.5 M/s |
| RFC3339 formatting | 199 ns | 5.0 M/s |

Getting the current time is fast (~37 ns). String formatting dominates the cost.

### request_id

Fast, thread-safe request ID generator. Produces 6-character URL-safe strings with no heap allocation.

```rust
use kiters::request_id::{RequestIdGenerator, encode_request_id, as_str};

// Using the generator (thread-safe)
let generator = RequestIdGenerator::new();
let id1 = generator.next_id();  // [u8; 6]
let id2 = generator.next_id_string();  // String

// Or encode a specific u64 directly
let id = encode_request_id(12345);
println!("ID: {}", as_str(&id));  // "7CBAAA"

// Use mixed mode for random-looking output
let gen_mixed = RequestIdGenerator::new_mixed();
let random_looking = gen_mixed.next_id_string();
```

#### Benchmarks

Compared against the `nanoid` crate using Criterion (`cargo bench`):

| Implementation | Time | Throughput |
|----------------|------|------------|
| `encode_request_id` (plain) | **1.78 ns** | 560 M/s |
| `encode_request_id_mixed` | 2.58 ns | 387 M/s |
| `RequestIdGenerator::next_id` | 2.64 ns | 378 M/s |
| `RequestIdGenerator::next_id_string` | 17.3 ns | 57 M/s |
| `nanoid!(6)` | 1.27 us | 771 K/s |
| `nanoid!()` (21 chars) | 1.28 us | 778 K/s |

Our implementation is ~480x faster than nanoid due to:
- No heap allocation (returns `[u8; 6]`)
- No RNG calls (deterministic counter)
- Simple bit-shift encoding vs cryptographic randomness

### eid

External ID system combining a prefix with UUID bytes encoded in base36.

```rust
use kiters::eid::ExternalId;

let user_id = ExternalId::new("user");
println!("{}", user_id);  // user-abc123xyz...

let order_id = ExternalId::new("order");
println!("{}", order_id);  // order-def456...
```

#### Benchmarks

Compared against raw UUID v4 generation (`cargo bench --bench eid_bench`):

| Implementation | Time | Throughput |
|----------------|------|------------|
| `Uuid::new_v4()` | 937 ns | 1.07 M/s |
| `ExternalId::new` | 966 ns | 1.03 M/s |
| `ExternalId::to_string` (pre-generated) | **191 ns** | 5.2 M/s |
| `ExternalId::new` + `to_string` | 1.17 us | 854 K/s |

The bottleneck is UUID v4 generation (RNG). The base36 encoding adds minimal overhead (~3%).

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.2.0 | 2026-01-13 | Add `request_id` module, export `eid` module |
| 0.1.0 | 2025-12-01 | Initial release with `timestamp` module |

## License

AGPL-3.0-or-later
