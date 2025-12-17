//! Snapshot tests for regex parse error diagnostics.
//!
//! Each test case triggers one specific error type to verify
//! that error messages and help text are correct.

use oxc_allocator::Allocator;
use oxc_diagnostics::{GraphicalReportHandler, GraphicalTheme, NamedSource};
use oxc_regular_expression::{LiteralParser, Options};
use std::fmt::Write;

fn test_err(allocator: &Allocator, pattern: &str, flags: Option<&str>) -> String {
    let source = format!("/{pattern}/{}", flags.unwrap_or(""));
    let err = LiteralParser::new(
        allocator,
        pattern,
        flags,
        Options {
            pattern_span_offset: 1,
            flags_span_offset: u32::try_from(pattern.len()).unwrap() + 2,
        },
    )
    .parse()
    .expect_err(&format!("{source} should fail to parse"));

    let error = err.with_source_code(NamedSource::new(source.clone(), source));
    let handler = GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor());
    let mut output = String::new();
    handler.render_report(&mut output, error.as_ref()).unwrap();
    output
}

#[test]
fn test() {
    let allocator = Allocator::default();

    let cases: &[(&str, Option<&str>, &str)] = &[
        // Flag errors
        ("a", Some("z"), "unknown_flag"),
        ("a", Some("gg"), "duplicated_flags"),
        ("a", Some("uv"), "invalid_unicode_flags"),
        // Capturing group errors
        (r"(?<n>.)(?<n>.)", Some(""), "duplicated_capturing_group_names"),
        (r"(?<>a)", Some(""), "empty_group_specifier"),
        // Quantifier errors
        ("+", Some("u"), "lone_quantifier"),
        ("a|+", Some(""), "invalid_braced_quantifier"),
        ("a{", Some("u"), "invalid_braced_quantifier_unicode"),
        (
            r"x{99999999999999999999999999999999999999999999999999}",
            Some(""),
            "too_large_number_in_braced_quantifier",
        ),
        (r"a{2,1}", Some(""), "braced_quantifier_out_of_order"),
        // Unterminated patterns
        ("(", Some(""), "unterminated_group"),
        ("[", Some(""), "unterminated_character_class"),
        // Reference errors
        (r"\1", Some("u"), "invalid_indexed_reference"),
        (r"\k<a>", Some("u"), "invalid_named_reference"),
        // Unicode property errors
        (r"\P{Basic_Emoji}", Some("v"), "invalid_unicode_property_name_negative_strings"),
        (r"\p{Basic_Emoji}", Some("u"), "invalid_unicode_property_of_strings"),
        (r"\p{Foo}", Some("u"), "invalid_unicode_property"),
        // Character class errors
        ("[z-a]", Some(""), "character_class_range_out_of_order"),
        (r"[\d-z]", Some("u"), "character_class_range_invalid_atom"),
        (r"[a-\d]", Some("u"), "invalid_class_atom"),
        // Unicode sets mode (v flag) errors
        (r"[&&]", Some("v"), "empty_class_set_expression"),
        (r"[a&&&b]", Some("v"), "class_intersection_unexpected_ampersand"),
        (r"[a&&]", Some("v"), "class_set_expression_invalid_character"),
        (r"[a&&b(]", Some("v"), "class_set_expression_invalid_character"),
        (r"[[^\q{ng}]]", Some("v"), "character_class_contents_invalid_operands"),
        // Unicode escape errors
        (r"\u{FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF}", Some("u"), "too_large_number_digits"),
        (r"\ua", Some("u"), "invalid_unicode_escape_sequence"),
        (r"\u{110000}", Some("u"), "invalid_surrogate_pair"),
        // Escape errors
        (r"\c0", Some("u"), "invalid_extended_atom_escape"),
        // Modifier errors
        (r"(?ii:.)", Some(""), "invalid_modifiers"),
        (r"(?a:.)", Some(""), "unknown_modifiers"),
        // Parse errors
        (")", Some("v"), "parse_pattern_incomplete"),
    ];

    let mut snapshot = String::new();
    for (pattern, flags, name) in cases {
        writeln!(snapshot, "# {name}").unwrap();
        snapshot.push_str(&test_err(&allocator, pattern, *flags));
        snapshot.push('\n');
    }

    insta::assert_snapshot!(snapshot);
}
