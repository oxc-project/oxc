//! Byte-order-mark handling shared by every formatter's file entries.

/// Splits one leading U+FEFF off `source`, returning whether it was present and the remainder.
///
/// The contract every formatter follows: ENTRIES own the strip, parse layers
/// assume BOM-free input (spans and gap scans never see it).
/// A physical root entry keeps the flag and re-emits the BOM exactly once at byte 0 of its IR;
/// an embedded entry (`format_to_ir`) strips as input hygiene and never re-emits.
/// The one exception is JS: its AST-in entry (`format_program`) means the formatter
/// never owns pre-parse text — oxc_parser lexes U+FEFF as whitespace itself,
/// so the root printer detects at print time instead.
///
/// Exactly ONE BOM is split: a doubled BOM keeps its second copy in the remainder
/// and reaches the parser as ordinary input. (matching Prettier, which strips only `charAt(0)`)
pub fn split_bom(source: &str) -> (bool, &str) {
    match source.strip_prefix('\u{feff}') {
        Some(rest) => (true, rest),
        None => (false, source),
    }
}

#[cfg(test)]
mod tests {
    use super::split_bom;

    #[test]
    fn no_bom_passes_through() {
        assert_eq!(split_bom("a: 1"), (false, "a: 1"));
        assert_eq!(split_bom(""), (false, ""));
    }

    #[test]
    fn one_bom_is_split() {
        assert_eq!(split_bom("\u{feff}a"), (true, "a"));
    }

    #[test]
    fn bom_only_leaves_an_empty_remainder() {
        assert_eq!(split_bom("\u{feff}"), (true, ""));
    }

    #[test]
    fn doubled_bom_keeps_the_second_copy() {
        assert_eq!(split_bom("\u{feff}\u{feff}a"), (true, "\u{feff}a"));
    }
}
