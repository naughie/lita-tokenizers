//! Utilities for pure string manipulation.

/// Returns true if the input string ends with end-of-sentence punctuation marks.
///
/// ```
/// # use lita_tokenizers_utils::str::ends_with_eos_marker;
/// assert!(ends_with_eos_marker("."));
/// assert!(ends_with_eos_marker("abc."));
/// assert!(!ends_with_eos_marker(".abc"));
/// ```
pub fn ends_with_eos_marker(s: &str) -> bool {
    s.ends_with(['.', '!', '?', '．', '！', '？', '。', '‼', '⁉'])
}
/// Returns true if the input string equals to end-of-sentence punctuation marks.
///
/// ```
/// # use lita_tokenizers_utils::str::matches_eos_marker;
/// assert!(matches_eos_marker("."));
/// assert!(!matches_eos_marker("abc."));
/// assert!(!matches_eos_marker(".abc"));
/// ```
pub fn matches_eos_marker(s: &str) -> bool {
    matches!(s, "." | "!" | "?" | "．" | "！" | "？" | "。" | "‼" | "⁉")
}
