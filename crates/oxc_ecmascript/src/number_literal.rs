use oxc_data_structures::inline_string::InlineString;

type LiteralString = InlineString<31, u8>;

/// Call `f` with the shortest JavaScript source representation of a non-negative finite number.
// Adapted from Terser's `get_minified_number`:
// https://github.com/terser/terser/blob/c5315c3fd6321d6b2e076af35a70ef532f498505/lib/output.js#L2418
pub fn with_number_literal<R>(value: f64, f: impl FnOnce(&str) -> R) -> R {
    let literal = number_literal(value);
    f(literal.as_str())
}

#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_possible_wrap)]
fn number_literal(value: f64) -> LiteralString {
    let mut buffer = dragonbox_ecma::Buffer::new();
    if value < 1000.0 && value.fract() == 0.0 {
        return LiteralString::from_str(buffer.format(value));
    }

    let formatted = buffer.format(value);
    let mut best_candidate = LiteralString::new();
    for (index, byte) in formatted.bytes().enumerate() {
        if (index == 0 && byte == b'0' && formatted.as_bytes().get(1) == Some(&b'.'))
            || (index > 0 && byte == b'+' && formatted.as_bytes()[index - 1] == b'e')
        {
            continue;
        }
        best_candidate.push(byte);
    }

    let mut is_hex = false;
    if value.fract() == 0.0 {
        let integer = value as u128;
        let hex_digits = (128 - integer.leading_zeros() as usize).div_ceil(4);
        let hex_len = 2 + hex_digits;
        if hex_len < best_candidate.len_usize() {
            let mut candidate = LiteralString::new();
            candidate.push(b'0');
            candidate.push(b'x');
            push_hex(integer, &mut candidate);
            best_candidate = candidate;
            is_hex = true;
        }
    } else if best_candidate.starts_with(".0")
        && let Some(index) = best_candidate.bytes().skip(2).position(|byte| byte != b'0')
    {
        let digits_start = index + 2;
        let digits = &best_candidate[digits_start..];
        let exponent = digits.len() + digits_start - 1;
        let mut exponent_buffer = itoa::Buffer::new();
        let exponent = exponent_buffer.format(exponent);
        let expected_len = digits.len() + 2 + exponent.len();
        if expected_len < best_candidate.len_usize() {
            let mut candidate = LiteralString::new();
            for byte in digits.bytes().chain(b"e-".iter().copied()).chain(exponent.bytes()) {
                candidate.push(byte);
            }
            best_candidate = candidate;
        }
    }

    if !is_hex
        && best_candidate.ends_with('0')
        && let Some(exponent) = best_candidate.bytes().rev().position(|byte| byte != b'0')
    {
        let base_len = best_candidate.len_usize() - exponent;
        let mut exponent_buffer = itoa::Buffer::new();
        let exponent = exponent_buffer.format(exponent);
        let expected_len = base_len + 1 + exponent.len();
        if expected_len < best_candidate.len_usize() {
            let mut candidate = LiteralString::new();
            for byte in best_candidate.as_bytes()[..base_len]
                .iter()
                .copied()
                .chain(std::iter::once(b'e'))
                .chain(exponent.bytes())
            {
                candidate.push(byte);
            }
            best_candidate = candidate;
        }
    }

    if let Some((integer, point, exponent)) =
        best_candidate.split_once('.').and_then(|(integer, rest)| {
            rest.split_once('e').map(|(point, exponent)| (integer, point, exponent))
        })
        && let Ok(exponent) = exponent.parse::<isize>()
    {
        let exponent = exponent - point.len() as isize;
        let mut exponent_buffer = itoa::Buffer::new();
        let exponent = exponent_buffer.format(exponent);
        let expected_len = integer.len() + point.len() + 1 + exponent.len();
        if expected_len < best_candidate.len_usize() {
            let mut candidate = LiteralString::new();
            for byte in integer
                .bytes()
                .chain(point.bytes())
                .chain(std::iter::once(b'e'))
                .chain(exponent.bytes())
            {
                candidate.push(byte);
            }
            best_candidate = candidate;
        }
    }

    best_candidate
}

fn push_hex(mut value: u128, output: &mut LiteralString) {
    let mut digits = [0; 32];
    let mut len = 0;
    loop {
        let digit = (value & 0xf) as u8;
        digits[len] = if digit < 10 { b'0' + digit } else { b'a' + digit - 10 };
        len += 1;
        value >>= 4;
        if value == 0 {
            break;
        }
    }
    for byte in digits[..len].iter().rev() {
        output.push(*byte);
    }
}

#[cfg(test)]
mod tests {
    use super::with_number_literal;

    #[test]
    fn shortest_number_literal() {
        for (value, expected) in [
            (0.0, "0"),
            (0.05, ".05"),
            (0.000_001, "1e-6"),
            (1000.0, "1e3"),
            (281_474_976_710_655.0, "0xffffffffffff"),
            (1.2e101, "12e100"),
            (f64::MAX, "17976931348623157e292"),
        ] {
            with_number_literal(value, |literal| assert_eq!(literal, expected));
        }
    }
}
