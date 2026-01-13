# PRD: Timestamp Module

## Overview

The `timestamp` module provides utilities for working with UTC timestamps in a consistent ISO 8601 format (`YYYY-MM-DDTHH:MM:SSZ`). It solves the problem of needing a standardized, human-readable timestamp format across applications without timezone ambiguity.

**Primary Goal:** Provide a simple, zero-configuration API for generating UTC timestamps in ISO 8601 format.

## Goals

1. **Consistency** - All timestamps follow the exact format `YYYY-MM-DDTHH:MM:SSZ` (20 characters)
2. **Simplicity** - Single function call to get current UTC timestamp
3. **Performance** - Use compile-time format description for optimal runtime performance
4. **Interoperability** - ISO 8601 format is universally parseable by databases, APIs, and logging systems
5. **No Configuration** - Works out of the box with sensible defaults

## Job Stories

- As a developer, I can call `get_utc_timestamp()` to get the current time as a formatted string, so that I can log events with consistent timestamps.
- As a developer, I can use `get_utc_formatter()` to get the format description, so that I can format my own `OffsetDateTime` values consistently.
- As a system integrator, I can rely on the Z suffix indicating UTC, so that I avoid timezone conversion bugs.

## Assumptions

- Users want UTC specifically (not local time)
- Second-level precision is sufficient (no milliseconds)
- The `time` crate is an acceptable dependency
- ISO 8601 format is the desired output format

## Functional Requirements

```
FR-1: get_utc_timestamp() returns current UTC time as String
- Acceptance: Returns exactly 20 characters in format YYYY-MM-DDTHH:MM:SSZ

FR-2: get_utc_formatter() returns reusable format description
- Acceptance: Returns &'static [FormatItem<'static>] usable with time crate

FR-3: Output format follows ISO 8601
- Acceptance: Contains T separator between date and time, ends with Z suffix

FR-4: All numeric components are zero-padded
- Acceptance: Month, day, hour, minute, second are always 2 digits
```

## Non-functional Requirements

```
NFR-1: Format description compiled at compile-time
- Acceptance: Uses time::macros::format_description! macro

NFR-2: No heap allocation in format description
- Acceptance: get_utc_formatter() returns static reference

NFR-3: Thread-safe
- Acceptance: Can be called from multiple threads concurrently
```

## Non-Goals

- Local timezone support
- Millisecond/microsecond precision
- Custom format strings
- Timestamp parsing (only generation)
- Date arithmetic or manipulation

## Success Metrics

| Metric | Target |
|--------|--------|
| Format correctness | 100% of outputs match `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$` |
| API surface | 2 public functions |
| Test coverage | 3+ test cases covering format validation |

## Technical Constraints

- Depends on `time` crate with `formatting` and `macros` features
- Rust edition 2024
