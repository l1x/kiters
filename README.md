# kiters

Kiters (kit-rs) is a collection of Rust utilities for timestamps, request IDs, and external IDs.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
kiters = "0.3.0"
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

Fast, thread-safe request ID generator. Two widths available:
- **6 characters** (36 bits, ~68 billion unique IDs) — no heap allocation
- **11 characters** (66 bits, captures full `u64`) — no heap allocation

```rust
use kiters::request_id::{
    RequestIdGenerator, WideRequestIdGenerator,
    encode_request_id, encode_request_id_wide, as_str,
};

// 6-char generator (default)
let generator: RequestIdGenerator = RequestIdGenerator::new();
let id = generator.next_id();          // [u8; 6]
let id_str = generator.next_id_string(); // String

// 11-char wide generator (full u64 fidelity)
let wide = WideRequestIdGenerator::new();
let wide_id = wide.next_id();          // [u8; 11]

// Free functions
let id6 = encode_request_id(12345);
println!("ID: {}", as_str(&id6));       // "7CBAAA"

let id11 = encode_request_id_wide(12345);
println!("Wide: {}", as_str(&id11));    // 11-char output

// Mixed mode for random-looking output
let mixed = WideRequestIdGenerator::new_mixed();
let random_looking = mixed.next_id_string();
```

#### Benchmarks

Compared against the `nanoid` crate using Criterion (`cargo bench`):

| Implementation | Time | Throughput |
|----------------|------|------------|
| `encode_request_id` (6 chars) | **4.95 ns** | 202 M/s |
| `encode_request_id_mixed` (6 chars) | 5.73 ns | 175 M/s |
| `encode_request_id_wide` (11 chars) | 8.13 ns | 123 M/s |
| `encode_request_id_mixed_wide` (11 chars) | 9.98 ns | 98 M/s |
| `RequestIdGenerator::next_id` (6) | 5.99 ns | 167 M/s |
| `RequestIdGenerator::next_id` (6, mixed) | 5.97 ns | 168 M/s |
| `WideRequestIdGenerator::next_id` (11) | 7.35 ns | 134 M/s |
| `WideRequestIdGenerator::next_id` (11, mixed) | 9.44 ns | 106 M/s |
| `RequestIdGenerator::next_id_string` (6) | 21.9 ns | 45 M/s |
| `WideRequestIdGenerator::next_id_string` (11) | 24.9 ns | 40 M/s |
| `nanoid!(6)` | 1,625 ns | 615 K/s |
| `nanoid!()` (21 chars) | 1,654 ns | 604 K/s |

Wide variants cost ~3 ns more than their 6-char counterparts — still ~170x faster than nanoid. The performance advantage comes from:
- No heap allocation (returns `[u8; N]`)
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
| 0.3.0 | 2026-02-22 | Add configurable request ID width (6 or 11 chars), const generic `RequestIdGenerator<N>`, `WideRequestIdGenerator` |
| 0.2.0 | 2026-01-13 | Add `request_id` module, export `eid` module |
| 0.1.0 | 2025-12-01 | Initial release with `timestamp` module |

## License

AGPL-3.0-or-later
