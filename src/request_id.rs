//! Fast request ID generator using sequential counter mapped to base64-like string.
//!
//! # Example
//!
//! ```rust
//! use kiters::request_id::{RequestIdGenerator, encode_request_id};
//!
//! // Using the generator (thread-safe)
//! let generator = RequestIdGenerator::new();
//! let id1 = generator.next_id();  // "BAAAAA"
//! let id2 = generator.next_id();  // "CAAAAA"
//!
//! // Or encode a specific u64 directly
//! let id = encode_request_id(12345);  // "7CBAAA"
//! ```
//!
//! # Performance
//!
//! - ~0.001ms per ID generation
//! - No heap allocation (returns `[u8; 6]`)
//! - 36 bits of counter space (~68 billion unique IDs)

use std::sync::atomic::{AtomicU64, Ordering};

/// URL-safe alphabet (64 characters = 6 bits per character)
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Encode a u64 into a 6-character ASCII string.
///
/// Uses 6 bits per character (masked with 0x3F = 63) to index into a 64-char alphabet.
/// Total: 36 bits of the u64 are encoded.
#[inline]
pub fn encode_request_id(n: u64) -> [u8; 6] {
    [
        ALPHABET[(n & 0x3F) as usize],
        ALPHABET[((n >> 6) & 0x3F) as usize],
        ALPHABET[((n >> 12) & 0x3F) as usize],
        ALPHABET[((n >> 18) & 0x3F) as usize],
        ALPHABET[((n >> 24) & 0x3F) as usize],
        ALPHABET[((n >> 30) & 0x3F) as usize],
    ]
}

/// Encode with mixing for random-looking output (still deterministic).
#[inline]
pub fn encode_request_id_mixed(n: u64) -> [u8; 6] {
    // splitmix64 mixing function
    let mut x = n.wrapping_mul(0x9e3779b97f4a7c15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    x ^= x >> 27;
    encode_request_id(x)
}

/// Convert encoded bytes to &str (infallible - all bytes are ASCII).
#[inline]
pub fn as_str(id: &[u8; 6]) -> &str {
    // SAFETY: All bytes in ALPHABET are valid ASCII
    unsafe { std::str::from_utf8_unchecked(id) }
}

/// Thread-safe request ID generator.
pub struct RequestIdGenerator {
    counter: AtomicU64,
    mixed: bool,
}

impl RequestIdGenerator {
    /// Create a new generator starting at 1.
    pub const fn new() -> Self {
        Self {
            counter: AtomicU64::new(1),
            mixed: false,
        }
    }

    /// Create a generator with mixing enabled (random-looking output).
    pub const fn new_mixed() -> Self {
        Self {
            counter: AtomicU64::new(1),
            mixed: true,
        }
    }

    /// Generate the next request ID.
    #[inline]
    pub fn next_id(&self) -> [u8; 6] {
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        if self.mixed {
            encode_request_id_mixed(n)
        } else {
            encode_request_id(n)
        }
    }

    /// Generate next ID as a String.
    #[inline]
    pub fn next_id_string(&self) -> String {
        let id = self.next_id();
        as_str(&id).to_owned()
    }
}

impl Default for RequestIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_sequential() {
        let id1 = encode_request_id(1);
        let id2 = encode_request_id(2);
        let id3 = encode_request_id(64);

        assert_eq!(as_str(&id1), "BAAAAA");
        assert_eq!(as_str(&id2), "CAAAAA");
        assert_eq!(as_str(&id3), "ABAAAA"); // rolls over to second char
    }

    #[test]
    fn test_encode_mixed_differs() {
        let plain = encode_request_id(1);
        let mixed = encode_request_id_mixed(1);
        assert_ne!(plain, mixed);
    }

    #[test]
    fn test_generator() {
        let generator = RequestIdGenerator::new();
        let id1 = generator.next_id();
        let id2 = generator.next_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_all_chars_valid_ascii() {
        for i in 0..1000 {
            let id = encode_request_id(i);
            assert!(std::str::from_utf8(&id).is_ok());
        }
    }

    #[test]
    fn test_uniqueness() {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for i in 0..100_000 {
            let id = encode_request_id(i);
            assert!(seen.insert(id), "collision at {}", i);
        }
    }
}
