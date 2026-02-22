//! Fast request ID generator using sequential counter mapped to base64-like string.
//!
//! Two widths are available:
//! - **6 characters** (36 bits of counter space, ~68 billion unique IDs)
//! - **11 characters** (66 bits, captures a full `u64` — the 11th character uses only
//!   16 of 64 alphabet positions since only 4 bits remain)
//!
//! # Example
//!
//! ```rust
//! use kiters::request_id::{RequestIdGenerator, WideRequestIdGenerator, encode_request_id, encode_request_id_wide, as_str};
//!
//! // 6-char generator (default)
//! let generator: RequestIdGenerator = RequestIdGenerator::new();
//! let id = generator.next_id();  // [u8; 6]
//!
//! // 11-char wide generator
//! let wide = WideRequestIdGenerator::new();
//! let wide_id = wide.next_id();  // [u8; 11]
//!
//! // Free functions
//! let id6 = encode_request_id(12345);       // [u8; 6]
//! let id11 = encode_request_id_wide(12345); // [u8; 11]
//! ```

use std::sync::atomic::{AtomicU64, Ordering};

/// URL-safe alphabet (64 characters = 6 bits per character)
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Internal base64 encoder: extracts 6 bits per character, LSB first.
#[inline]
fn encode_base64<const N: usize>(n: u64) -> [u8; N] {
    const { assert!(N <= 11, "N > 11 would shift past u64 width (11 * 6 = 66 >= 64)") }
    let mut buf = [ALPHABET[0]; N];
    let mut i = 0;
    while i < N {
        buf[i] = ALPHABET[((n >> (i * 6)) & 0x3F) as usize];
        i += 1;
    }
    buf
}

/// splitmix64 mixing function — deterministic bijection on u64.
#[inline]
fn splitmix64(n: u64) -> u64 {
    let mut x = n.wrapping_mul(0x9e3779b97f4a7c15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    x ^= x >> 27;
    x
}

/// Encode a u64 into a 6-character ASCII string.
///
/// Uses 6 bits per character (masked with 0x3F = 63) to index into a 64-char alphabet.
/// Total: 36 bits of the u64 are encoded.
#[inline]
pub fn encode_request_id(n: u64) -> [u8; 6] {
    encode_base64(n)
}

/// Encode a u64 into an 11-character ASCII string.
///
/// Captures all 64 bits of the input. The 11th character uses only
/// 4 remaining bits (16 of 64 alphabet positions).
#[inline]
pub fn encode_request_id_wide(n: u64) -> [u8; 11] {
    encode_base64(n)
}

/// Encode with mixing for random-looking output (still deterministic), 6 chars.
#[inline]
pub fn encode_request_id_mixed(n: u64) -> [u8; 6] {
    encode_base64(splitmix64(n))
}

/// Encode with mixing for random-looking output (still deterministic), 11 chars.
#[inline]
pub fn encode_request_id_mixed_wide(n: u64) -> [u8; 11] {
    encode_base64(splitmix64(n))
}

/// Convert encoded bytes to &str (infallible — all bytes are ASCII).
#[inline]
pub fn as_str<const N: usize>(id: &[u8; N]) -> &str {
    // SAFETY: All bytes in ALPHABET are valid ASCII
    unsafe { std::str::from_utf8_unchecked(id) }
}

/// Thread-safe request ID generator.
///
/// `N` is the output width in characters: 6 (default, 36 bits) or 11 (66 bits, full u64).
pub struct RequestIdGenerator<const N: usize = 6> {
    counter: AtomicU64,
    mixed: bool,
}

/// Wide (11-character) request ID generator capturing all 64 bits.
pub type WideRequestIdGenerator = RequestIdGenerator<11>;

impl<const N: usize> RequestIdGenerator<N> {
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
    pub fn next_id(&self) -> [u8; N] {
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        if self.mixed {
            encode_base64(splitmix64(n))
        } else {
            encode_base64(n)
        }
    }

    /// Generate next ID as a String.
    #[inline]
    pub fn next_id_string(&self) -> String {
        let id = self.next_id();
        as_str(&id).to_owned()
    }
}

impl<const N: usize> Default for RequestIdGenerator<N> {
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
        let generator: RequestIdGenerator = RequestIdGenerator::new();
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

    // --- Wide variant tests ---

    #[test]
    fn test_encode_wide_captures_all_bits() {
        // A value that differs only in the upper bits (beyond 36)
        let lo: u64 = 42;
        let hi: u64 = 42 | (1 << 40);

        // 6-char encoding truncates — both map to the same output
        assert_eq!(encode_request_id(lo), encode_request_id(hi));

        // 11-char encoding preserves the difference
        assert_ne!(encode_request_id_wide(lo), encode_request_id_wide(hi));
    }

    #[test]
    fn test_wide_all_chars_valid_ascii() {
        for i in 0..1000 {
            let id = encode_request_id_wide(i);
            assert!(std::str::from_utf8(&id).is_ok());
        }
    }

    #[test]
    fn test_wide_uniqueness() {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for i in 0..100_000u64 {
            let id = encode_request_id_wide(i);
            assert!(seen.insert(id), "collision at {}", i);
        }
    }

    #[test]
    fn test_wide_generator() {
        let generator = WideRequestIdGenerator::new();
        let id1 = generator.next_id();
        let id2 = generator.next_id();
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 11);
        assert_eq!(id2.len(), 11);
    }

    #[test]
    fn test_wide_mixed_generator() {
        let generator = WideRequestIdGenerator::new_mixed();
        let id1 = generator.next_id();
        let id2 = generator.next_id();
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 11);
    }

    #[test]
    fn test_wide_mixed_differs_from_plain() {
        let plain = encode_request_id_wide(1);
        let mixed = encode_request_id_mixed_wide(1);
        assert_ne!(plain, mixed);
    }

    #[test]
    fn test_as_str_works_for_both_widths() {
        let id6 = encode_request_id(99);
        let id11 = encode_request_id_wide(99);
        let s6 = as_str(&id6);
        let s11 = as_str(&id11);
        assert_eq!(s6.len(), 6);
        assert_eq!(s11.len(), 11);
    }
}
