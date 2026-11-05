use oxc_allocator::{Allocator, ArenaStringBuilder};
use oxc_str::Str;

use crate::parser::{
    pattern_parser::{
        character,
        unicode_escape::{UnicodeEscape, decode},
    },
    reader::CodePoint,
};
use crate::surrogate_pair;

/// Decode a group name found by the capture pre-scan. `units` have already been decoded from
/// a string or template literal, but may still contain RegExp Unicode escapes.
/// Names without source escapes keep borrowing the source text. Malformed names return `None` so
/// the pre-scan cannot report a duplicate before the main parser diagnoses the invalid name.
/// <https://tc39.es/ecma262/multipage/text-processing.html#sec-static-semantics-capturinggroupname>
pub(super) fn capturing_group_name<'a>(
    raw: &'a str,
    units: &[CodePoint],
    unicode_mode: bool,
    allocator: &'a Allocator,
) -> Option<Str<'a>> {
    if units.is_empty() {
        return None;
    }

    let mut name =
        raw.contains('\\').then(|| ArenaStringBuilder::with_capacity_in(raw.len(), allocator));
    let mut index = 0;
    while index < units.len() {
        let (mut value, mut next) = decode_code_point(units, index)?;

        // In non-Unicode mode, RegExpIdentifierCodePoint combines raw surrogate pairs.
        // https://tc39.es/ecma262/multipage/text-processing.html#sec-static-semantics-regexpidentifiercodepoint
        if !unicode_mode
            && units[index].value != '\\' as u32
            && surrogate_pair::is_lead_surrogate(value)
            && let Some(trail) = units.get(next).map(|unit| unit.value)
            && surrogate_pair::is_trail_surrogate(trail)
        {
            value = surrogate_pair::combine_surrogate_pair(value, trail);
            next += 1;
        }

        let valid = if index == 0 {
            character::is_identifier_start_char(value)
        } else {
            character::is_identifier_part_char(value)
        };
        if !valid {
            return None;
        }
        if let Some(name) = &mut name {
            name.push(char::from_u32(value)?);
        }
        index = next;
    }

    Some(name.map_or_else(|| Str::from(raw), |name| Str::from(name.into_str())))
}

fn decode_code_point(units: &[CodePoint], start: usize) -> Option<(u32, usize)> {
    let value = units.get(start)?.value;
    if value != '\\' as u32 {
        return Some((value, start + 1));
    }

    match decode(&units[start + 1..], true) {
        UnicodeEscape::Value { code_point, length } => Some((code_point, start + 1 + length)),
        UnicodeEscape::Invalid | UnicodeEscape::Overflow { .. } => None,
    }
}
