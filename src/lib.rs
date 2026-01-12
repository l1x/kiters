//! # Kiters
//!
//! `kiters` is a collection of useful Rust functions and utilities used across various projects.
//!
//! ## Modules
//!
//! - [`timestamp`]: Utilities for working with timestamps (specifically UTC formatted strings).
//! - [`request_id`]: Fast request ID generator using sequential counter mapped to base64-like string.

pub mod request_id;
pub mod timestamp;
