//! Key normalization and the comparator shared by every sorting target.

use std::{borrow::Cow, cmp::Ordering};

use cow_utils::CowUtils;

use super::options::{SortCommonOptions, SortOrder, SortType, SpecialCharacters};

/// A "letter" for `SpecialCharacters`: ASCII letters plus the Latin extended blocks.
fn is_sort_letter(c: char) -> bool {
    c.is_ascii_alphabetic()
        || ('\u{C0}'..='\u{24F}').contains(&c)
        || ('\u{1E00}'..='\u{1EFF}').contains(&c)
}

/// Build the comparison key: lowercase when `ignore_case`, always drop whitespace,
/// then apply `special_characters`. Borrows when nothing changes.
pub fn normalize_key(
    name: &str,
    ignore_case: bool,
    special_characters: SpecialCharacters,
) -> Cow<'_, str> {
    let needs_lowercase = ignore_case && name.chars().any(char::is_uppercase);
    let has_whitespace = name.chars().any(char::is_whitespace);
    let needs_special = match special_characters {
        SpecialCharacters::Keep => false,
        SpecialCharacters::Trim => name.chars().next().is_some_and(|c| !is_sort_letter(c)),
        SpecialCharacters::Remove => name.chars().any(|c| !is_sort_letter(c)),
    };
    if !needs_lowercase && !has_whitespace && !needs_special {
        return Cow::Borrowed(name);
    }

    let mut key =
        if needs_lowercase { name.cow_to_lowercase().into_owned() } else { name.to_string() };
    if has_whitespace {
        key.retain(|c| !c.is_whitespace());
    }
    match special_characters {
        SpecialCharacters::Keep => {}
        SpecialCharacters::Trim => {
            let start = key.find(is_sort_letter).unwrap_or(key.len());
            key.drain(..start);
        }
        SpecialCharacters::Remove => key.retain(is_sort_letter),
    }
    Cow::Owned(key)
}

/// One comparison by `sort_type`, with `order` applied per comparison
/// (never by reversing a sorted slice: that would flip `Equal` pairs and break idempotency).
pub fn compare_names(
    a: &str,
    a_size: u32,
    b: &str,
    b_size: u32,
    sort_type: SortType,
    order: SortOrder,
) -> Ordering {
    let ordering = match sort_type {
        SortType::Natural => natord::compare(a, b),
        SortType::Alphabetical => a.cmp(b),
        SortType::LineLength => a_size.cmp(&b_size),
        SortType::Unsorted => Ordering::Equal,
    };
    if order.is_desc() { ordering.reverse() } else { ordering }
}

/// Primary comparison, then `fallback_sort` when the primary is `Equal`.
/// `Equal` from both means "keep source order" (the callers use stable sorts).
pub fn compare(
    a: &str,
    a_size: u32,
    b: &str,
    b_size: u32,
    options: &SortCommonOptions,
) -> Ordering {
    match compare_names(a, a_size, b, b_size, options.sort_type, options.order) {
        Ordering::Equal => compare_names(
            a,
            a_size,
            b,
            b_size,
            options.fallback_sort.sort_type,
            options.fallback_sort.order.unwrap_or(options.order),
        ),
        ordering => ordering,
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::*;
    use crate::ir_transform::sort_common::options::{
        FallbackSort, SortOrder, SortType, SpecialCharacters,
    };

    #[test]
    fn normalize_lowercases_and_drops_whitespace() {
        assert_eq!(normalize_key("Foo Bar", true, SpecialCharacters::Keep), "foobar");
        assert_eq!(normalize_key("Foo Bar", false, SpecialCharacters::Keep), "FooBar");
        assert!(matches!(normalize_key("plain", true, SpecialCharacters::Keep), Cow::Borrowed(_)));
    }

    #[test]
    fn normalize_trim_strips_leading_non_letters_only() {
        assert_eq!(normalize_key("./foo-bar", true, SpecialCharacters::Trim), "foo-bar");
        assert_eq!(normalize_key("@scope/x", true, SpecialCharacters::Trim), "scope/x");
        assert_eq!(normalize_key("abc", true, SpecialCharacters::Trim), "abc");
    }

    #[test]
    fn normalize_remove_strips_every_non_letter_including_digits() {
        // Digits count as special characters, like perfectionist.
        assert_eq!(normalize_key("a-b.c/d1", true, SpecialCharacters::Remove), "abcd");
        assert_eq!(normalize_key("Élan über", false, SpecialCharacters::Remove), "Élanüber");
    }

    #[test]
    fn natural_vs_alphabetical() {
        let asc = SortOrder::Asc;
        assert_eq!(compare_names("a2", 0, "a10", 0, SortType::Natural, asc), Ordering::Less);
        assert_eq!(
            compare_names("a2", 0, "a10", 0, SortType::Alphabetical, asc),
            Ordering::Greater
        );
        // Code-point order: uppercase before lowercase when case is significant.
        assert_eq!(compare_names("B", 0, "a", 0, SortType::Alphabetical, asc), Ordering::Less);
    }

    #[test]
    fn line_length_and_unsorted() {
        let asc = SortOrder::Asc;
        assert_eq!(compare_names("zz", 2, "a", 1, SortType::LineLength, asc), Ordering::Greater);
        assert_eq!(compare_names("zz", 5, "a", 5, SortType::LineLength, asc), Ordering::Equal);
        assert_eq!(compare_names("b", 0, "a", 0, SortType::Unsorted, asc), Ordering::Equal);
    }

    #[test]
    fn desc_reverses_per_comparison_but_keeps_equal() {
        let desc = SortOrder::Desc;
        assert_eq!(compare_names("a", 0, "b", 0, SortType::Natural, desc), Ordering::Greater);
        assert_eq!(compare_names("a", 0, "a", 0, SortType::Natural, desc), Ordering::Equal);
    }

    #[test]
    fn fallback_applies_only_on_equal() {
        let mut options = SortCommonOptions::default();
        assert_eq!(compare("a", 9, "a", 1, &options), Ordering::Equal);
        options.fallback_sort = FallbackSort { sort_type: SortType::LineLength, order: None };
        assert_eq!(compare("a", 9, "a", 1, &options), Ordering::Greater);
        options.fallback_sort.order = Some(SortOrder::Desc);
        assert_eq!(compare("a", 9, "a", 1, &options), Ordering::Less);
        // Primary decides when it can.
        assert_eq!(compare("a", 9, "b", 1, &options), Ordering::Less);
    }

    #[test]
    fn enum_parsing() {
        assert_eq!("line-length".parse::<SortType>().unwrap(), SortType::LineLength);
        assert!("locale".parse::<SortType>().is_err());
        assert_eq!("remove".parse::<SpecialCharacters>().unwrap(), SpecialCharacters::Remove);
        for t in SortType::ALL {
            assert_eq!(t.name().parse::<SortType>().unwrap(), *t);
        }
        for s in SpecialCharacters::ALL {
            assert_eq!(s.name().parse::<SpecialCharacters>().unwrap(), *s);
        }
    }
}
