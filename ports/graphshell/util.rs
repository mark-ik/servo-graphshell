/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// Truncate a string to `max_chars` characters, appending an ellipsis if truncated.
/// Uses character counting (not byte length) so it's safe for multi-byte UTF-8.
pub(crate) fn truncate_with_ellipsis(input: &str, max_chars: usize) -> String {
    if input.chars().count() > max_chars {
        let truncated: String = input.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{truncated}\u{2026}")
    } else {
        input.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_string_unchanged() {
        assert_eq!(truncate_with_ellipsis("short", 20), "short");
    }

    #[test]
    fn test_long_string_truncated() {
        let result =
            truncate_with_ellipsis("this is a very long title that should be truncated", 20);
        assert_eq!(result.chars().count(), 20);
        assert!(result.ends_with('\u{2026}'));
    }

    #[test]
    fn test_exact_length_unchanged() {
        assert_eq!(
            truncate_with_ellipsis("exactly twenty chars", 20),
            "exactly twenty chars"
        );
    }

    #[test]
    fn test_emoji_safe() {
        // Emoji are multi-byte but single chars — should not panic
        let input = "Hello \u{1F600} World! This is long enough to truncate";
        let result = truncate_with_ellipsis(input, 15);
        assert!(result.chars().count() <= 15);
        assert!(result.ends_with('\u{2026}'));
    }

    #[test]
    fn test_cjk_safe() {
        // CJK characters are 3 bytes each — byte slicing would panic
        let input = "\u{4F60}\u{597D}\u{4E16}\u{754C}\u{4F60}\u{597D}\u{4E16}\u{754C}"; // 8 chars
        let result = truncate_with_ellipsis(input, 5);
        assert_eq!(result.chars().count(), 5);
        assert!(result.ends_with('\u{2026}'));
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(truncate_with_ellipsis("", 20), "");
    }

    #[test]
    fn test_max_one() {
        // Edge case: max_chars = 1, saturating_sub(1) = 0, so just ellipsis
        let result = truncate_with_ellipsis("hello", 1);
        assert_eq!(result, "\u{2026}");
    }
}
