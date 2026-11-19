use oxc_allocator::Allocator;
use oxc_regular_expression::{ConstructorParser, LiteralParser, Options, ast::Term};

#[test]
fn unicode_property_error_ranges() {
    let allocator = Allocator::default();
    for (pattern, message, start, len) in [
        (r"\p{foo1bar}", "Invalid unicode property name and/or value", 3, 7),
        (r"\p{1ASCII}", "Invalid unicode property name and/or value", 3, 6),
        (r"\p{foo1bar=Latin}", "Invalid unicode property name and/or value", 3, 7),
        (r"\p{Script=}", "Invalid unicode property name and/or value", 3, 6),
        (r"\p{Script=Latin", "Invalid unicode property name and/or value", 3, 6),
        (r"\p{ASCII=}", "Unterminated unicode property escape", 0, 8),
        (r"\p{ASCII", "Unterminated unicode property escape", 0, 8),
        (r"\p{A1b", "Unterminated unicode property escape", 0, 6),
        (r"\p{}", "Unterminated unicode property escape", 0, 3),
        (r"\p{=Latin}", "Unterminated unicode property escape", 0, 3),
        (r"\p{", "Unterminated unicode property escape", 0, 3),
    ] {
        let error = LiteralParser::new(&allocator, pattern, Some("v"), Options::default())
            .parse()
            .unwrap_err();
        assert!(error.to_string().ends_with(message), "/{pattern}/v: {error}");
        assert_eq!(error.labels.len(), 1, "/{pattern}/v");
        assert_eq!((error.labels[0].offset(), error.labels[0].len()), (start, len), "/{pattern}/v");
    }
}

#[test]
fn constructor_unicode_property_names() {
    let allocator = Allocator::default();
    for (source, expected_name, expected_value) in [
        (r#""\\p{\x41SCII}""#, "ASCII", None),
        (r#""\\p{\x53cript=Latin}""#, "Script", Some("Latin")),
        (r#""\\p{Script=\x4catin}""#, "Script", Some("Latin")),
    ] {
        let parsed = ConstructorParser::new(&allocator, source, Some(r#""u""#), Options::default())
            .parse()
            .unwrap();
        let Term::UnicodePropertyEscape(property) = &parsed.body.body[0].body[0] else {
            panic!("expected Unicode property escape");
        };
        assert_eq!(property.name.as_str(), expected_name);
        assert_eq!(property.value.as_ref().map(oxc_str::Str::as_str), expected_value);
    }
}
