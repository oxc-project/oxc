use memchr::memmem;
use oxc_str::JSStr;

// A UTF-8 needle cannot match part of a WTF-8 code point or a lone surrogate.
// Search bytes once, then translate the result to a UTF-16 code unit index.
pub fn index_of_utf8(value: JSStr<'_>, search: &str, start: usize) -> Option<usize> {
    debug_assert_ne!(search, "");
    // If `start` lies inside a surrogate pair, the next code point is the first
    // possible match for a UTF-8 needle.
    let (byte_start, _) = code_point_indices(value).find(|&(_, index)| index >= start)?;
    let found = byte_start + memmem::find(&value.as_bytes()[byte_start..], search.as_bytes())?;
    utf16_index(value, found)
}

pub fn last_index_of_utf8(value: JSStr<'_>, search: &str, bound: usize) -> Option<usize> {
    debug_assert_ne!(search, "");
    let (byte_bound, _) =
        code_point_indices(value).take_while(|&(_, index)| index <= bound).last()?;
    // The bound limits the match's start; the needle may extend beyond it.
    let end = byte_bound.saturating_add(search.len()).min(value.len());
    let found = memmem::rfind(&value.as_bytes()[..end], search.as_bytes())?;
    utf16_index(value, found)
}

pub fn index_of_code_units(value: JSStr<'_>, search: JSStr<'_>, start: usize) -> Option<usize> {
    code_unit_matches(value.encode_utf16().skip(start), search).next().map(|index| start + index)
}

pub fn last_index_of_code_units(
    value: JSStr<'_>,
    search: JSStr<'_>,
    bound: usize,
) -> Option<usize> {
    // The bound limits the match's start; the needle may extend beyond it.
    let end = bound.saturating_add(search.len_utf16());
    code_unit_matches(value.encode_utf16().take(end), search).last()
}

// KMP searches UTF-16 in linear time, including matches that split surrogate pairs.
// Only the needle is buffered; the receiver is decoded once as matches are requested.
fn code_unit_matches(
    value: impl Iterator<Item = u16>,
    search: JSStr<'_>,
) -> impl Iterator<Item = usize> {
    let needle: Vec<_> = search.encode_utf16().collect();
    debug_assert_ne!(needle.len(), 0);
    let mut prefixes = vec![0; needle.len()];
    let mut matched = 0;
    for index in 1..needle.len() {
        while matched > 0 && needle[index] != needle[matched] {
            matched = prefixes[matched - 1];
        }
        if needle[index] == needle[matched] {
            matched += 1;
        }
        prefixes[index] = matched;
    }

    let mut matched = 0;
    value.enumerate().filter_map(move |(index, unit)| {
        while matched > 0 && unit != needle[matched] {
            matched = prefixes[matched - 1];
        }
        if unit == needle[matched] {
            matched += 1;
        }
        if matched == needle.len() {
            matched = prefixes[matched - 1];
            Some(index + 1 - needle.len())
        } else {
            None
        }
    })
}

fn utf16_index(value: JSStr<'_>, byte_index: usize) -> Option<usize> {
    code_point_indices(value).find_map(|(byte, index)| (byte == byte_index).then_some(index))
}

fn code_point_indices(value: JSStr<'_>) -> impl Iterator<Item = (usize, usize)> + '_ {
    value.chars().scan((0, 0), |offsets, c| {
        let result = *offsets;
        let width = c.len_bytes();
        offsets.0 += width;
        offsets.1 += 1 + usize::from(width == 4);
        Some(result)
    })
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_str::JSStrBuilder;

    use crate::{StringIndexOf, StringLastIndexOf};

    #[test]
    fn surrogate_needles_preserve_overlapping_matches_and_bounds() {
        let allocator = Allocator::new();
        let js_str = |units: &[u16]| {
            let mut builder = JSStrBuilder::new_in(&&allocator);
            builder.push_utf16(units);
            builder.into_js_str()
        };
        let value = js_str(&[0xD800, 0x61, 0xD800, 0x61, 0xD800]);
        let search = js_str(&[0xD800, 0x61, 0xD800]);
        assert_eq!(value.index_of(Some(search), None), 0);
        assert_eq!(value.index_of(Some(search), Some(1.0)), 2);
        assert_eq!(value.index_of(Some(search), Some(3.0)), -1);
        assert_eq!(value.last_index_of(Some(search), None), 2);
        assert_eq!(value.last_index_of(Some(search), Some(1.0)), 0);
        assert_eq!(value.last_index_of(Some(search), Some(0.0)), 0);

        // Both ends of a match may split formed surrogate pairs.
        let value = js_str(&[0xD83D, 0xDE00, 0x61, 0xD83D, 0xDE00]);
        let search = js_str(&[0xDE00, 0x61, 0xD83D]);
        assert_eq!(value.index_of(Some(search), Some(1.0)), 1);
        assert_eq!(value.index_of(Some(search), Some(2.0)), -1);
        assert_eq!(value.last_index_of(Some(search), Some(1.0)), 1);
        assert_eq!(value.last_index_of(Some(search), Some(0.0)), -1);
    }

    #[test]
    fn surrogate_needles_with_long_repeated_prefixes() {
        let allocator = Allocator::new();
        let prefix = "a".repeat(16_000);
        let value = "a".repeat(32_000);
        let mut builder = JSStrBuilder::new_in(&&allocator);
        builder.push_str(&prefix);
        builder.push_code_unit(0xD800);
        let search = builder.into_js_str();
        assert_eq!(value.as_str().index_of(Some(search), None), -1);
        assert_eq!(value.as_str().last_index_of(Some(search), None), -1);
    }
}
