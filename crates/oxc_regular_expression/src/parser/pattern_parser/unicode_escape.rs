use crate::{
    parser::{pattern_parser::character, reader::CodePoint},
    surrogate_pair,
};

pub(super) enum UnicodeEscape {
    Value {
        code_point: u32,
        length: usize,
    },
    Invalid,
    /// Number of valid digits consumed after `u{` before the overflowing digit.
    Overflow {
        digits: usize,
    },
}

/// Decode RegExpUnicodeEscapeSequence from cooked code points, beginning with `u`.
/// <https://tc39.es/ecma262/multipage/text-processing.html#prod-RegExpUnicodeEscapeSequence>
pub(super) fn decode(units: &[CodePoint], unicode_mode: bool) -> UnicodeEscape {
    if units.first().is_none_or(|unit| unit.value != 'u' as u32) {
        return UnicodeEscape::Invalid;
    }

    if let Some(value) = fixed_hex_digits(units, 1) {
        if unicode_mode
            && surrogate_pair::is_lead_surrogate(value)
            && units.get(5).is_some_and(|unit| unit.value == '\\' as u32)
            && units.get(6).is_some_and(|unit| unit.value == 'u' as u32)
            && let Some(trail) = fixed_hex_digits(units, 7)
            && surrogate_pair::is_trail_surrogate(trail)
        {
            return UnicodeEscape::Value {
                code_point: surrogate_pair::combine_surrogate_pair(value, trail),
                length: 11,
            };
        }
        return UnicodeEscape::Value { code_point: value, length: 5 };
    }

    if unicode_mode && units.get(1).is_some_and(|unit| unit.value == '{' as u32) {
        let mut index = 2;
        let mut value = 0_u32;
        while let Some(digit) =
            units.get(index).and_then(|unit| character::map_hex_digit(unit.value))
        {
            let Some(next) = value.checked_mul(16).and_then(|value| value.checked_add(digit))
            else {
                return UnicodeEscape::Overflow { digits: index - 2 };
            };
            value = next;
            index += 1;
        }
        if index > 2
            && units.get(index).is_some_and(|unit| unit.value == '}' as u32)
            && character::is_valid_unicode(value)
        {
            return UnicodeEscape::Value { code_point: value, length: index + 1 };
        }
    }

    UnicodeEscape::Invalid
}

fn fixed_hex_digits(units: &[CodePoint], start: usize) -> Option<u32> {
    let mut value = 0_u32;
    for unit in units.get(start..start + 4)? {
        value = value * 16 + character::map_hex_digit(unit.value)?;
    }
    Some(value)
}
