use time::{OffsetDateTime, format_description::FormatItem, macros::format_description};

/// Returns a static reference to the UTC date-time format description.
///
/// The format is ISO 8601-like: `YYYY-MM-DDTHH:MM:SSZ`.
///
/// # Returns
///
/// A slice of `FormatItem`s that can be used with the `time` crate.
pub fn get_utc_formatter() -> &'static [FormatItem<'static>] {
    format_description!(
        "[year]-[month padding:zero]-[day padding:zero]T[hour padding:zero]:[minute padding:zero]:[second padding:zero]Z"
    )
}

/// Returns the current UTC timestamp as a formatted String.
///
/// The format is `YYYY-MM-DDTHH:MM:SSZ`.
///
/// # Example
///
/// ```
/// use kiters::timestamp::get_utc_timestamp;
///
/// let ts = get_utc_timestamp();
/// println!("Current time: {}", ts); // e.g., 2023-10-27T10:00:00Z
/// ```
pub fn get_utc_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(get_utc_formatter())
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_format_length() {
        let ts = get_utc_timestamp();

        // Should be exactly 20 chars: YYYY-MM-DDTHH:MM:SSZ
        assert_eq!(ts.len(), 20);
    }

    #[test]
    fn test_timestamp_has_required_chars() {
        let ts = get_utc_timestamp();

        assert!(ts.ends_with('Z'), "Must end with Z (UTC)");
        assert!(ts.contains('T'), "Must contain T separator");
        assert_eq!(ts.chars().nth(4), Some('-'), "Year separator");
        assert_eq!(ts.chars().nth(7), Some('-'), "Month separator");
        assert_eq!(ts.chars().nth(10), Some('T'), "Date/time separator");
        assert_eq!(ts.chars().nth(13), Some(':'), "Hour separator");
        assert_eq!(ts.chars().nth(16), Some(':'), "Minute separator");
    }

    #[test]
    fn test_timestamp_components_are_digits() {
        let ts = get_utc_timestamp();

        // Year
        assert!(ts[0..4].chars().all(|c| c.is_ascii_digit()));
        // Month
        assert!(ts[5..7].chars().all(|c| c.is_ascii_digit()));
        // Day
        assert!(ts[8..10].chars().all(|c| c.is_ascii_digit()));
        // Hour
        assert!(ts[11..13].chars().all(|c| c.is_ascii_digit()));
        // Minute
        assert!(ts[14..16].chars().all(|c| c.is_ascii_digit()));
        // Second
        assert!(ts[17..19].chars().all(|c| c.is_ascii_digit()));
    }
}
