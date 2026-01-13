# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

## [0.1.0] - 2025-12-01

### Added
- Initial release
- `timestamp` module: UTC timestamp utilities
  - `get_utc_timestamp()` - get current UTC time as `YYYY-MM-DDTHH:MM:SSZ`
  - `get_utc_formatter()` - get reusable format description

[0.2.0]: https://github.com/l1x/kiters/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/l1x/kiters/releases/tag/v0.1.0
