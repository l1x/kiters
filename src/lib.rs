//! # Kiters
//!
//! `kiters` is a collection of useful Rust functions and utilities used across various projects.
//!
//! ## Modules
//!
//! - [`timestamp`]: Utilities for working with timestamps (specifically UTC formatted strings).
//! - [`request_id`]: Fast request ID generator using sequential counter mapped to base64-like string.
//! - [`eid`]: External ID system with prefix and UUID bytes encoded in base36.

pub mod eid;
pub mod request_id;
pub mod timestamp;
