//! Display-width measurement matching Prettier's `getStringWidth`.
//!
//! Prettier measures `printWidth` with a deliberately simple per-code-point rule
//! (`src/utilities/get-string-width.js`): every code point counts as width 1, except
//!
//! - C0 / C1 control characters                  -> 0
//! - combining diacritical marks `U+0300..=U+036F` -> 0
//! - variation selectors `U+FE00..=U+FE0F`         -> 0
//! - East Asian Wide / Fullwidth code points       -> 2
//!
//! and emoji grapheme clusters are collapsed to a single glyph (1 for the handful of
//! "narrow" emoji, 2 otherwise) by an earlier emoji-regex pass.
//!
//! This intentionally differs from [`unicode_width`], whose `wcwidth`-style tables assign
//! width 0 to *every* non-spacing mark (`Mn`/`Me`) and format character (`Cf`). Prettier only
//! zeroes the three ranges above, so e.g. Tibetan / Arabic / Devanagari / Hebrew combining
//! marks each count as 1. Measuring those as 0 makes lines that Prettier would break look like
//! they fit, producing a formatting divergence (oxc issue #23863).
//!
//! We still lean on [`unicode_width`] for emoji clusters only: Prettier's emoji-regex step
//! collapses a ZWJ sequence / flag / VS16 presentation to one glyph, which is exactly the
//! grapheme-cluster width [`unicode_width`] computes.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Display width of a single code point, following Prettier's per-code-point rule.
///
/// Unlike [`unicode_width`], combining marks *outside* `U+0300..=U+036F` (and format
/// characters) count as 1, matching Prettier. Emoji sequences are not handled here; use
/// [`get_string_width`] for anything that may contain a multi-code-point grapheme cluster.
///
/// Kept crate-internal (the module is private); consumers go through [`get_string_width`].
pub fn char_width(c: char) -> usize {
    let cp = c as u32;
    // C0 / C1 control characters.
    if cp <= 0x1F || (0x7F..=0x9F).contains(&cp) {
        return 0;
    }
    // Combining diacritical marks: the only combining block Prettier zeroes.
    if (0x0300..=0x036F).contains(&cp) {
        return 0;
    }
    // Variation selectors.
    if (0xFE00..=0xFE0F).contains(&cp) {
        return 0;
    }
    // East Asian Wide / Fullwidth -> 2, everything else (incl. non-`U+0300..=U+036F`
    // combining marks, which `unicode_width` would zero) -> 1.
    if UnicodeWidthChar::width(c) == Some(2) { 2 } else { 1 }
}

/// Display width of `text` as Prettier measures `printWidth`.
///
/// Tabs and newlines count as zero width (they are control characters); a caller that needs a
/// tab expanded to `indent_width`, or the line width reset on each newline, must split on them
/// first (as [`crate::format_element::TextWidth::from_text`] does). Grapheme segmentation is only
/// used to keep emoji clusters collapsed; every other code point is summed via [`char_width`].
pub fn get_string_width(text: &str) -> usize {
    // Printable ASCII (no controls) has width == byte length.
    if text.bytes().all(|b| matches!(b, 0x20..=0x7E)) {
        return text.len();
    }

    let mut width = 0;
    for cluster in text.graphemes(true) {
        let mut chars = cluster.chars();
        // `graphemes(true)` never yields an empty cluster, so `first` is always `Some`.
        let Some(first) = chars.next() else { continue };
        if chars.next().is_none() {
            // Lone code point: the overwhelmingly common case, including every CJK ideograph
            // and standalone emoji. Skips the emoji-cluster check.
            width += char_width(first);
        } else if is_emoji_cluster(cluster) {
            // Prettier collapses emoji sequences to one glyph; `unicode_width` computes that
            // collapsed width (1 narrow / 2 wide).
            width += UnicodeWidthStr::width(cluster);
        } else {
            // A base character plus combining marks: Prettier counts each code point.
            width += cluster.chars().map(char_width).sum::<usize>();
        }
    }
    width
}

/// Whether a multi-code-point grapheme cluster is an emoji sequence Prettier collapses to a
/// single glyph, as opposed to a base character carrying combining marks.
///
/// A variation selector (emoji presentation) or any scalar in the emoji planes (pictographs,
/// regional-indicator flags, skin-tone modifiers) marks the cluster as emoji. Notably a bare
/// ZWJ is *not* treated as an emoji signal, so combining sequences that use ZWJ purely as a
/// text joiner (some Indic scripts) keep counting each code point.
fn is_emoji_cluster(cluster: &str) -> bool {
    cluster.chars().any(|c| {
        let cp = c as u32;
        (0xFE00..=0xFE0F).contains(&cp) || (0x1_F000..=0x1_FAFF).contains(&cp)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_is_byte_length() {
        assert_eq!(get_string_width(""), 0);
        assert_eq!(get_string_width("abc"), 3);
        assert_eq!(get_string_width("a b c"), 5);
        assert_eq!(get_string_width("~!@#$%^&*()"), 11);
    }

    #[test]
    fn east_asian_wide_counts_two() {
        assert_eq!(get_string_width("你好"), 4);
        assert_eq!(get_string_width("日本語"), 6);
        // Fullwidth forms.
        assert_eq!(get_string_width("ＡＢ"), 4);
    }

    #[test]
    fn control_and_combining_diacriticals_are_zero() {
        // C0 / C1 controls.
        assert_eq!(char_width('\u{0007}'), 0);
        assert_eq!(char_width('\u{0085}'), 0);
        // Combining acute accent (in the zeroed block): "cafe\u{301}" == "café" == 4.
        assert_eq!(get_string_width("cafe\u{301}"), 4);
    }

    #[test]
    fn combining_marks_outside_the_zeroed_block_count_as_one() {
        // The regression from oxc #23863: `unicode_width` zeroes these, Prettier counts them.
        // Tibetan subjoined letter LA (U+0FB3), a non-spacing mark.
        assert_eq!(char_width('\u{0FB3}'), 1);
        // ཟླ་ = ZA (1) + subjoined LA (1) + tsheg (1) = 3.
        assert_eq!(get_string_width("\u{0F5F}\u{0FB3}\u{0F0B}"), 3);
        // Arabic letter meem + fatha (U+064E, non-spacing mark) = 2.
        assert_eq!(get_string_width("\u{0645}\u{064E}"), 2);
        // The full string from the issue.
        let s = "ཟླ་1_ཟླ་2_ཟླ་3_ཟླ་4_ཟླ་5_ཟླ་6_ཟླ་7_ཟླ་8_ཟླ་9_ཟླ་10_ཟླ་11_ཟླ་12";
        assert_eq!(get_string_width(s), 62);
    }

    #[test]
    fn zwj_joining_non_emoji_is_counted() {
        // A ZWJ between non-emoji is a text joiner, not an emoji sequence: e (1) + ZWJ (1) + b (1).
        assert_eq!(get_string_width("e\u{200D}b"), 3);
    }

    #[test]
    fn emoji_clusters_stay_collapsed() {
        // Single emoji.
        assert_eq!(get_string_width("👍"), 2);
        // Emoji with variation selector.
        assert_eq!(get_string_width("🗑️"), 2);
        assert_eq!(get_string_width("⚠️"), 2);
        // ZWJ sequence (family) collapses to one glyph.
        assert_eq!(get_string_width("👨‍👩‍👧‍👦"), 2);
        // Regional-indicator flag.
        assert_eq!(get_string_width("🇯🇵"), 2);
        // Skin-tone modifier.
        assert_eq!(get_string_width("👍🏽"), 2);
        // Emoji mixed with text.
        assert_eq!(get_string_width("🗑️ DELETE"), 9);
    }

    #[test]
    fn bare_narrow_symbol_without_presentation_selector_is_one() {
        // Bare U+26A0 (no VS16) is a narrow symbol; Prettier counts it as 1.
        assert_eq!(get_string_width("\u{26A0}"), 1);
    }
}
