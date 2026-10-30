use oxc_allocator::Allocator;
use oxc_regular_expression::{ConstructorParser, Options, ast::Term};

#[test]
fn decoded_unicode_escapes_keep_source_spans() {
    let allocator = Allocator::default();
    let source = r#""\\u{\x30\x30\x36\x31}b""#;
    let parsed = ConstructorParser::new(&allocator, source, Some(r#""u""#), Options::default())
        .parse()
        .unwrap();
    let terms = &parsed.body.body[0].body;
    assert_eq!(terms.len(), 2);
    let Term::Character(first) = &terms[0] else {
        panic!("expected decoded character");
    };
    let Term::Character(second) = &terms[1] else {
        panic!("expected trailing character");
    };
    assert_eq!(first.value, u32::from('a'));
    assert_eq!(first.span.source_text(source), r"\\u{\x30\x30\x36\x31}");
    assert_eq!(second.value, u32::from('b'));
    assert_eq!(second.span.source_text(source), "b");
}
