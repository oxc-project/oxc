use cow_utils::CowUtils;

/// Call `f` with the shortest JavaScript source representation of a non-negative finite number.
///
/// # Panics
///
/// Panics if the float formatter produces malformed scientific notation.
// Adapted from Terser's `get_minified_number`:
// https://github.com/terser/terser/blob/c5315c3fd6321d6b2e076af35a70ef532f498505/lib/output.js#L2418
#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_possible_wrap)]
pub fn with_number_literal<R>(num: f64, f: impl FnOnce(&str) -> R) -> R {
    let mut buffer = dragonbox_ecma::Buffer::new();
    if num < 1000.0 && num.fract() == 0.0 {
        return f(buffer.format(num));
    }

    let mut s = buffer.format(num);

    if s.starts_with("0.") {
        s = &s[1..];
    }

    let mut best_candidate = s.cow_replacen("e+", "e", 1);
    let mut is_hex = false;

    // Track the best candidate found so far
    if num.fract() == 0.0 {
        // For integers, check hex format and other optimizations
        let hex_candidate = format!("0x{:x}", num as u128);
        if hex_candidate.len() < best_candidate.len() {
            is_hex = true;
            best_candidate = hex_candidate.into();
        }
    }
    // Check for scientific notation optimizations for numbers starting with ".0"
    else if best_candidate.starts_with(".0") {
        // Skip the first '0' since we know it's there from the starts_with check
        if let Some(i) = best_candidate.bytes().skip(2).position(|c| c != b'0') {
            let len = i + 2; // `+2` to include the dot and first zero.
            let digits = &best_candidate[len..];
            let exp = digits.len() + len - 1;
            let exp_str_len = itoa::Buffer::new().format(exp).len();
            // Calculate expected length: digits + 'e-' + exp_length
            let expected_len = digits.len() + 2 + exp_str_len;
            if expected_len < best_candidate.len() {
                best_candidate = format!("{digits}e-{exp}").into();
                debug_assert_eq!(best_candidate.len(), expected_len);
            }
        }
    }

    // Check for numbers ending with zeros (but not hex numbers)
    // The `!is_hex` check is necessary to prevent hex numbers like `0x8000000000000000`
    // from being incorrectly converted to scientific notation
    if !is_hex
        && best_candidate.ends_with('0')
        && let Some(len) = best_candidate.bytes().rev().position(|c| c != b'0')
    {
        let base = &best_candidate[0..best_candidate.len() - len];
        let exp_str_len = itoa::Buffer::new().format(len).len();
        // Calculate expected length: base + 'e' + len
        let expected_len = base.len() + 1 + exp_str_len;
        if expected_len < best_candidate.len() {
            best_candidate = format!("{base}e{len}").into();
            debug_assert_eq!(best_candidate.len(), expected_len);
        }
    }

    // Check for scientific notation optimization: `1.2e101` -> `12e100`
    if let Some((integer, point, exponent)) =
        best_candidate.split_once('.').and_then(|(a, b)| b.split_once('e').map(|e| (a, e.0, e.1)))
    {
        let new_expr = exponent.parse::<isize>().unwrap() - point.len() as isize;
        let new_exp_str_len = itoa::Buffer::new().format(new_expr).len();
        // Calculate expected length: integer + point + 'e' + new_exp_str_len
        let expected_len = integer.len() + point.len() + 1 + new_exp_str_len;
        if expected_len < best_candidate.len() {
            best_candidate = format!("{integer}{point}e{new_expr}").into();
            debug_assert_eq!(best_candidate.len(), expected_len);
        }
    }

    f(&best_candidate)
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
