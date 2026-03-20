//! External ID system with prefix and UUID bytes encoded in fixed-width base36.
//!
//! An [`ExternalId`] combines a human-readable prefix with a UUIDv4 encoded as
//! exactly 25 base36 characters, producing strings like `user-0i4x3k7a8m1p9q2r5t7v0w3y6`.

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// The exact number of base36 characters used to encode a full `u128`.
///
/// Since `log2(36^25) ≈ 129.25 > 128`, 25 base36 digits can represent every
/// possible 128-bit value. The encoder always emits exactly 25 characters
/// (zero-padded), making the representation canonical — there is exactly one
/// valid encoding for any given byte sequence.
const BASE36_LEN: usize = 25;

/// Maximum allowed prefix length (bytes). Keeps total ID length bounded.
const MAX_PREFIX_LEN: usize = 63;

const BASE36_DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors produced when creating or parsing an [`ExternalId`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EidError {
    /// Prefix must not be empty.
    EmptyPrefix,
    /// Prefix exceeds the maximum allowed length.
    PrefixTooLong,
    /// Prefix contains an invalid character (only lowercase ASCII alphanumeric allowed).
    InvalidPrefixChar(char),
    /// Serialized form is missing the `-` separator.
    MissingDash,
    /// Payload part (after the dash) is empty.
    EmptyPayload,
    /// Payload contains invalid characters or has wrong length.
    InvalidPayload,
}

impl fmt::Display for EidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPrefix => write!(f, "prefix must not be empty"),
            Self::PrefixTooLong => {
                write!(f, "prefix exceeds maximum length of {MAX_PREFIX_LEN} bytes")
            }
            Self::InvalidPrefixChar(c) => {
                write!(f, "invalid prefix character: '{c}' (only a-z, 0-9 allowed)")
            }
            Self::MissingDash => write!(f, "missing '-' separator"),
            Self::EmptyPayload => write!(f, "payload must not be empty"),
            Self::InvalidPayload => write!(f, "invalid base36 payload"),
        }
    }
}

impl std::error::Error for EidError {}

// ---------------------------------------------------------------------------
// Base36 encode / decode (fixed-width, module-private)
// ---------------------------------------------------------------------------

fn encode_base36(bytes: &[u8; 16]) -> [u8; BASE36_LEN] {
    let mut n = u128::from_be_bytes(*bytes);
    let mut out = [b'0'; BASE36_LEN];
    for i in (0..BASE36_LEN).rev() {
        out[i] = BASE36_DIGITS[(n % 36) as usize];
        n /= 36;
    }
    out
}

fn decode_base36(s: &str) -> Result<[u8; 16], EidError> {
    if s.len() != BASE36_LEN {
        return Err(EidError::InvalidPayload);
    }
    let mut n: u128 = 0;
    for &b in s.as_bytes() {
        let digit = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'z' => b - b'a' + 10,
            _ => return Err(EidError::InvalidPayload),
        };
        n = n
            .checked_mul(36)
            .and_then(|v| v.checked_add(digit as u128))
            .ok_or(EidError::InvalidPayload)?;
    }
    Ok(n.to_be_bytes())
}

// ---------------------------------------------------------------------------
// Prefix validation (module-private)
// ---------------------------------------------------------------------------

fn validate_prefix(s: &str) -> Result<(), EidError> {
    if s.is_empty() {
        return Err(EidError::EmptyPrefix);
    }
    if s.len() > MAX_PREFIX_LEN {
        return Err(EidError::PrefixTooLong);
    }
    for c in s.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() {
            return Err(EidError::InvalidPrefixChar(c));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// ExternalId
// ---------------------------------------------------------------------------

/// External ID with a validated prefix and 128-bit UUID payload.
///
/// Display format: `{prefix}-{25-char-base36}`
///
/// The encoding is **canonical**: the payload is always exactly 25 lowercase
/// base36 characters (`[0-9a-z]`), zero-padded on the left. Parsing rejects
/// uppercase, wrong-length, or otherwise non-canonical representations.
///
/// # Examples
///
/// ```
/// use kiters::eid::ExternalId;
///
/// let id = ExternalId::new("user").unwrap();
/// let s = id.to_string();
/// assert!(s.starts_with("user-"));
///
/// let parsed: ExternalId = s.parse().unwrap();
/// assert_eq!(id, parsed);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalId {
    prefix: String,
    bytes: [u8; 16],
}

impl ExternalId {
    /// Create a new `ExternalId` with a random UUIDv4.
    ///
    /// The prefix must be non-empty and consist only of lowercase ASCII
    /// alphanumeric characters (`a-z`, `0-9`).
    pub fn new(prefix: &str) -> Result<Self, EidError> {
        validate_prefix(prefix)?;
        Ok(Self {
            prefix: prefix.to_string(),
            bytes: *Uuid::new_v4().as_bytes(),
        })
    }

    /// Create an `ExternalId` from an existing UUID.
    pub fn from_uuid(prefix: &str, uuid: Uuid) -> Result<Self, EidError> {
        validate_prefix(prefix)?;
        Ok(Self {
            prefix: prefix.to_string(),
            bytes: *uuid.as_bytes(),
        })
    }

    /// The prefix part of the ID.
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// The underlying UUID.
    pub fn uuid(&self) -> Uuid {
        Uuid::from_bytes(self.bytes)
    }
}

// ---------------------------------------------------------------------------
// Display / FromStr
// ---------------------------------------------------------------------------

impl fmt::Display for ExternalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encoded = encode_base36(&self.bytes);
        // SAFETY: encoded contains only ASCII base36 digits
        let payload = std::str::from_utf8(&encoded).unwrap();
        write!(f, "{}-{}", self.prefix, payload)
    }
}

impl FromStr for ExternalId {
    type Err = EidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dash = s.find('-').ok_or(EidError::MissingDash)?;
        let prefix = &s[..dash];
        let payload = &s[dash + 1..];

        validate_prefix(prefix)?;

        if payload.is_empty() {
            return Err(EidError::EmptyPayload);
        }

        let bytes = decode_base36(payload)?;
        Ok(Self {
            prefix: prefix.to_string(),
            bytes,
        })
    }
}

// ---------------------------------------------------------------------------
// Serde (serializes as a string, not a struct)
// ---------------------------------------------------------------------------

impl Serialize for ExternalId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ExternalId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_display_parse() {
        let id = ExternalId::new("run").unwrap();
        let s = id.to_string();
        let parsed: ExternalId = s.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_roundtrip_uuid() {
        let id = ExternalId::new("task").unwrap();
        let uuid = id.uuid();
        let id2 = ExternalId::from_uuid("task", uuid).unwrap();
        assert_eq!(id, id2);
    }

    #[test]
    fn test_roundtrip_serde_json() {
        let id = ExternalId::new("org").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        // Must be a JSON string, not an object
        assert!(json.starts_with('"'));
        assert!(json.ends_with('"'));
        let id2: ExternalId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, id2);
    }

    #[test]
    fn test_fixed_width_encoding() {
        for _ in 0..100 {
            let id = ExternalId::new("x").unwrap();
            let s = id.to_string();
            // "x-" + 25 chars = 27
            assert_eq!(s.len(), 2 + BASE36_LEN);
        }
    }

    #[test]
    fn test_all_zeros_roundtrip() {
        let nil = Uuid::nil();
        let id = ExternalId::from_uuid("zero", nil).unwrap();
        let s = id.to_string();
        assert_eq!(s, "zero-0000000000000000000000000");
        let parsed: ExternalId = s.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_all_ff_roundtrip() {
        let max = Uuid::max();
        let id = ExternalId::from_uuid("max", max).unwrap();
        let s = id.to_string();
        let parsed: ExternalId = s.parse().unwrap();
        assert_eq!(id, parsed);
        // Payload must be exactly 25 chars
        assert_eq!(s.len(), 4 + BASE36_LEN); // "max-" + 25
    }

    #[test]
    fn test_invalid_prefix_empty() {
        assert_eq!(ExternalId::new(""), Err(EidError::EmptyPrefix));
    }

    #[test]
    fn test_invalid_prefix_uppercase() {
        assert_eq!(
            ExternalId::new("User"),
            Err(EidError::InvalidPrefixChar('U'))
        );
    }

    #[test]
    fn test_invalid_prefix_underscore() {
        assert_eq!(
            ExternalId::new("my_id"),
            Err(EidError::InvalidPrefixChar('_'))
        );
    }

    #[test]
    fn test_invalid_prefix_dash() {
        assert_eq!(
            ExternalId::new("my-id"),
            Err(EidError::InvalidPrefixChar('-'))
        );
    }

    #[test]
    fn test_parse_missing_dash() {
        assert_eq!(
            "nopayload".parse::<ExternalId>(),
            Err(EidError::MissingDash)
        );
    }

    #[test]
    fn test_parse_empty_payload() {
        assert_eq!("user-".parse::<ExternalId>(), Err(EidError::EmptyPayload));
    }

    #[test]
    fn test_parse_invalid_payload_chars() {
        assert_eq!(
            "user-ZZZZZZZZZZZZZZZZZZZZZZZZ!".parse::<ExternalId>(),
            Err(EidError::InvalidPayload)
        );
    }

    #[test]
    fn test_parse_payload_too_short() {
        assert_eq!(
            "user-abc".parse::<ExternalId>(),
            Err(EidError::InvalidPayload)
        );
    }

    #[test]
    fn test_parse_payload_too_long() {
        assert_eq!(
            "user-00000000000000000000000000".parse::<ExternalId>(),
            Err(EidError::InvalidPayload)
        );
    }

    #[test]
    fn test_parse_uppercase_payload_rejected() {
        // Canonical encoding is lowercase; uppercase must be rejected, not normalized.
        assert_eq!(
            "user-0I4X3K7A8M1P9Q2R5T7V0W3Y6".parse::<ExternalId>(),
            Err(EidError::InvalidPayload)
        );
    }

    #[test]
    fn test_parse_multiple_dashes() {
        // Only the first dash is the separator; remaining dashes are part of payload
        // and must be rejected as invalid base36 characters.
        assert_eq!(
            "user-abc-defghijklmnopqrstuv".parse::<ExternalId>(),
            Err(EidError::InvalidPayload)
        );
    }

    #[test]
    fn test_prefix_too_long() {
        let long_prefix = "a".repeat(MAX_PREFIX_LEN + 1);
        assert_eq!(ExternalId::new(&long_prefix), Err(EidError::PrefixTooLong));
    }

    #[test]
    fn test_prefix_at_max_length() {
        let prefix = "a".repeat(MAX_PREFIX_LEN);
        let id = ExternalId::new(&prefix).unwrap();
        assert_eq!(id.prefix(), prefix);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// Strategy that generates valid prefixes: 1-20 chars of [a-z0-9].
    fn valid_prefix() -> impl Strategy<Value = String> {
        "[a-z0-9]{1,20}"
    }

    proptest! {
        #[test]
        fn display_parse_roundtrip(prefix in valid_prefix()) {
            let id = ExternalId::new(&prefix).unwrap();
            let s = id.to_string();
            let parsed: ExternalId = s.parse().unwrap();
            prop_assert_eq!(&id, &parsed);
        }

        #[test]
        fn serde_roundtrip(prefix in valid_prefix()) {
            let id = ExternalId::new(&prefix).unwrap();
            let json = serde_json::to_string(&id).unwrap();
            let id2: ExternalId = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(&id, &id2);
        }

        #[test]
        fn encoding_always_fixed_width(bytes in any::<[u8; 16]>()) {
            let encoded = encode_base36(&bytes);
            prop_assert_eq!(encoded.len(), BASE36_LEN);
            // All chars must be valid base36
            for &b in &encoded {
                prop_assert!(b.is_ascii_lowercase() || b.is_ascii_digit());
            }
        }

        #[test]
        fn encode_decode_roundtrip(bytes in any::<[u8; 16]>()) {
            let encoded = encode_base36(&bytes);
            let s = std::str::from_utf8(&encoded).unwrap();
            let decoded = decode_base36(s).unwrap();
            prop_assert_eq!(&bytes, &decoded);
        }

        #[test]
        fn uuid_roundtrip(prefix in valid_prefix()) {
            let id = ExternalId::new(&prefix).unwrap();
            let uuid = id.uuid();
            let id2 = ExternalId::from_uuid(&prefix, uuid).unwrap();
            prop_assert_eq!(&id, &id2);
        }

        #[test]
        fn invalid_prefix_rejected(c in "[^a-z0-9]") {
            let result = ExternalId::new(&c);
            prop_assert!(result.is_err());
        }
    }
}
