//! Unicode case-folding primitives matching the Go standard library functions tsgo uses.
//!
//! tsgo leans on three distinct Go behaviors, each with its own quirks, so each gets a
//! dedicated port here rather than approximating all of them with one lowercase pass:
//!
//! - [`to_file_name_lower_case`] — `tspath.ToFileNameLowerCase`, canonical dedup keys.
//! - [`str_fold_eq`] — `strings.EqualFold`, the matcher's case-insensitive equality.
//! - [`compare_case_insensitive`] — `stringutil.CompareStringsCaseInsensitive`, sort order.

use std::cmp::Ordering;

use cow_utils::CowUtils;

/// U+0130 LATIN CAPITAL LETTER I WITH DOT ABOVE. Its lowercase form (`i` + combining dot)
/// has its own uppercase form, so tsgo keeps it un-lowered in canonical file names, and its
/// case-fold orbit contains only itself.
const CAPITAL_I_WITH_DOT: char = '\u{0130}';
/// U+0131 LATIN SMALL LETTER DOTLESS I. Like U+0130, it case-folds only to itself.
const SMALL_DOTLESS_I: char = '\u{0131}';

/// The single-character lowercase mapping (Go `unicode.ToLower`).
///
/// Rust's `char::to_lowercase` is the *full* mapping, which differs from the simple one
/// only for U+0130 (two chars); callers that reach this either exclude U+0130 or want its
/// first char (`i`), which equals the simple mapping.
fn simple_lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

/// Port of tsgo `tspath.ToFileNameLowerCase`: lowercase for canonical file-name keys,
/// leaving U+0130 untouched (its lowercase form can coexist with it on disk).
pub fn to_file_name_lower_case(file_name: &str) -> String {
    if file_name.is_ascii() {
        return file_name.cow_to_ascii_lowercase().into_owned();
    }
    file_name.chars().map(|c| if c == CAPITAL_I_WITH_DOT { c } else { simple_lower(c) }).collect()
}

/// Port of Go `strings.EqualFold` (simple Unicode case-folding) for a single char pair.
fn chars_fold_eq(a: char, b: char) -> bool {
    if a == b {
        return true;
    }
    if a.is_ascii() && b.is_ascii() {
        return a.eq_ignore_ascii_case(&b);
    }
    // The Turkish dotted/dotless i fold only to themselves (Go SimpleFold orbit of one).
    if matches!(a, CAPITAL_I_WITH_DOT | SMALL_DOTLESS_I)
        || matches!(b, CAPITAL_I_WITH_DOT | SMALL_DOTLESS_I)
    {
        return false;
    }
    if simple_lower(a) == simple_lower(b) {
        return true;
    }
    // Same simple uppercase catches orbits the lowercase map misses (σ/ς, µ/μ). Skip
    // multi-char (full-only) uppercase expansions like ß -> SS, which simple folding
    // does not equate.
    let mut ua = a.to_uppercase();
    let mut ub = b.to_uppercase();
    match (ua.next(), ua.next(), ub.next(), ub.next()) {
        (Some(x), None, Some(y), None) => x == y,
        _ => false,
    }
}

/// Port of Go `strings.EqualFold`: char-wise simple case-folded equality.
pub fn str_fold_eq(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    if a.is_ascii() && b.is_ascii() {
        return a.eq_ignore_ascii_case(b);
    }
    let mut ca = a.chars();
    let mut cb = b.chars();
    loop {
        match (ca.next(), cb.next()) {
            (None, None) => return true,
            (Some(x), Some(y)) if chars_fold_eq(x, y) => {}
            _ => return false,
        }
    }
}

/// Byte-safe case-folded suffix test: `false` when `suffix`'s byte length lands inside a
/// multi-byte char of `s` (mirroring Go, where the byte slice then fails `EqualFold`).
pub fn has_suffix_fold(s: &str, suffix: &str) -> bool {
    s.len() >= suffix.len()
        && s.get(s.len() - suffix.len()..).is_some_and(|tail| str_fold_eq(tail, suffix))
}

/// Port of tsgo `stringutil.CompareStringsCaseInsensitive`: char-wise comparison of the
/// simple-lowercased chars (here U+0130 *does* lower to `i`, unlike the canonical key).
pub fn compare_case_insensitive(a: &str, b: &str) -> Ordering {
    if a == b {
        return Ordering::Equal;
    }
    let mut ca = a.chars();
    let mut cb = b.chars();
    loop {
        match (ca.next(), cb.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => match simple_lower(x).cmp(&simple_lower(y)) {
                Ordering::Equal => {}
                unequal => return unequal,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_name_lower_case() {
        assert_eq!(to_file_name_lower_case("/User/JOSÉ/X.ts"), "/user/josé/x.ts");
        assert_eq!(to_file_name_lower_case("/a/Σ.ts"), "/a/σ.ts");
        // U+0130 is kept; ASCII around it still lowers.
        assert_eq!(to_file_name_lower_case("A\u{0130}B"), "a\u{0130}b");
    }

    #[test]
    fn fold_eq() {
        assert!(str_fold_eq("Ä.ts", "ä.ts"));
        assert!(str_fold_eq("σ", "ς"));
        assert!(str_fold_eq("Σ", "ς"));
        assert!(str_fold_eq("µ", "μ"));
        assert!(str_fold_eq("\u{212A}", "k")); // Kelvin sign
        assert!(str_fold_eq("ſ", "S")); // long s
        assert!(!str_fold_eq("ß", "SS")); // simple folding, not full
        assert!(str_fold_eq("ß", "ẞ"));
        assert!(!str_fold_eq("\u{0130}", "i"));
        assert!(!str_fold_eq("\u{0131}", "I"));
        assert!(!str_fold_eq("a", "ab"));
    }

    #[test]
    fn suffix_fold() {
        assert!(has_suffix_fold("lib.MIN.js", ".min.js"));
        // 12 bytes; the last-7-bytes window starts inside the emoji -> no match, no panic.
        assert!(!has_suffix_fold("abcd🎉xxxx", ".min.js"));
        assert!(!has_suffix_fold("x", ".min.js"));
    }

    #[test]
    fn case_insensitive_compare() {
        assert_eq!(compare_case_insensitive("a", "B"), Ordering::Less);
        assert_eq!(compare_case_insensitive("Ä", "ä"), Ordering::Equal);
        // U+0130 lowers to plain `i` here, per Go unicode.ToLower.
        assert_eq!(compare_case_insensitive("\u{0130}", "i"), Ordering::Equal);
    }
}
