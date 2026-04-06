# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-03-20

### Breaking Changes

- **`ExternalId::new()` now returns `Result<ExternalId, EidError>`** instead of
  `ExternalId`. All call sites must handle the `Result` (e.g. `.unwrap()` or `?`).
- **Serde format changed from struct to string.** `ExternalId` now serializes as
  a flat JSON string (`"user-0abc..."`) instead of `{"prefix":"user","bytes":[...]}`.
  Existing serialized data (databases, caches, message queues) is **incompatible**
  and must be migrated or re-generated.
- **Struct fields are now private.** Use `.prefix()` and `.uuid()` accessors
  instead of `.prefix` and `.bytes`.
- **Inherent `to_string()` removed.** Use `Display` (`.to_string()` via the trait)
  which produces the same format.

### Added
- **Canonical encoding format:** lowercase prefix (`[a-z0-9]`, max 63 chars) +
  dash + exactly 25 lowercase base36 characters. Parsing strictly rejects
  uppercase, wrong-length, or otherwise non-canonical representations.
- `ExternalId::from_uuid(prefix, uuid)` constructor
- `FromStr` impl — parse `"prefix-payload"` strings back into `ExternalId`
- `EidError` enum with variants: `EmptyPrefix`, `PrefixTooLong`,
  `InvalidPrefixChar`, `MissingDash`, `EmptyPayload`, `InvalidPayload`
- Prefix validation: only `[a-z0-9]`, 1-63 characters
- 19 unit tests + 6 property-based tests (proptest)

### Changed
- Replaced `base36` crate (v0.0.1, depends on deprecated `failure`) with inline
  fixed-width base36 encoder/decoder using `u128` arithmetic

### Removed
- `base36` dependency (and its transitive `failure` + `base-x` deps)

## [0.3.0] - 2026-02-22

### Added
- Configurable request ID width (6 or 11 characters)
- `WideRequestIdGenerator` and const generic `RequestIdGenerator<N>`
- Criterion benchmarks for `timestamp`, `eid`, and `request_id` modules

## [0.2.0] - 2026-01-13

### Added
- `request_id` module: Fast request ID generator using sequential counter mapped to base64-like string
  - `encode_request_id()` - encode u64 to 6-char ASCII
  - `encode_request_id_mixed()` - encode with splitmix64 mixing for random-looking output
  - `RequestIdGenerator` - thread-safe generator with atomic counter
- `eid` module: External ID system with prefix and UUID bytes (now publicly exported)
  - `ExternalId::new(prefix)` - generate prefixed UUID-based IDs
  - Format: `prefix-{base36uuid}`

### Changed
- Updated crate description to reflect all modules

## [0.1.0] - 2025-12-26

### Added
- Initial release
- `timestamp` module: UTC timestamp utilities
  - `get_utc_timestamp()` - get current UTC time as `YYYY-MM-DDTHH:MM:SSZ`
  - `get_utc_formatter()` - get reusable format description

[0.4.0]: https://github.com/vectorian-rs/kiters/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/vectorian-rs/kiters/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/vectorian-rs/kiters/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/vectorian-rs/kiters/releases/tag/v0.1.0
