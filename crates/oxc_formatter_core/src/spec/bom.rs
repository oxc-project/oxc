//! Byte-order-mark handling shared by every formatter's file entries.

/// Splits the entire leading run of U+FEFF off `source`,
/// returning whether any was present and the remainder.
///
/// The contract every formatter follows:
/// ENTRIES own the strip, parse layers assume BOM-free input (spans and gap scans never see it).
/// A physical root entry keeps the flag and re-emits the BOM exactly once at byte 0 of its IR;
/// an embedded entry (`format_to_ir`) never sees a leading U+FEFF at all:
/// in an embedded position it is content, not a BOM,
/// so `FormatSession::dispatch` answers `PreserveOriginal` for BOM-headed inputs before any child entry runs.
/// The one exception is JS:
/// its AST-in entry (`format_program`) means the formatter never owns pre-parse text,
/// `oxc_parser` lexes U+FEFF as whitespace itself, so the root printer detects at print time instead.
///
/// The WHOLE leading run is split, not just the first,
/// so the remainder is BOM-free by construction and a doubled BOM comes back out as one BOM.
/// (Prettier strips only `charAt(0)` and lets its parsers swallow the rest; the observable output is the same.)
pub fn split_bom(source: &str) -> (bool, &str) {
    (source.starts_with('\u{feff}'), source.trim_start_matches('\u{feff}'))
}

#[cfg(test)]
mod tests {
    use super::split_bom;

    #[test]
    fn splits_the_whole_leading_bom_run() {
        for (input, expected) in [
            ("a: 1", (false, "a: 1")),
            ("", (false, "")),
            ("\u{feff}a", (true, "a")),
            ("\u{feff}", (true, "")),
            ("\u{feff}\u{feff}a", (true, "a")),
            // A non-leading U+FEFF is content and stays
            ("\u{feff}a\u{feff}", (true, "a\u{feff}")),
        ] {
            assert_eq!(split_bom(input), expected);
        }
    }
}
