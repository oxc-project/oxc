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
