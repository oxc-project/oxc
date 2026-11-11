use oxc_allocator::Allocator;
use oxc_regular_expression::{
    LiteralParser, Options,
    ast::{CharacterClassContents, Term},
};
use oxc_span::Span;

#[test]
fn class_set_character_values_and_ranges() {
    let allocator = Allocator::default();
    let pattern = r"[\u{61}\x62c-e]";
    let parsed =
        LiteralParser::new(&allocator, pattern, Some("v"), Options::default()).parse().unwrap();
    let Term::CharacterClass(class) = &parsed.body.body[0].body[0] else {
        panic!("expected character class");
    };
    assert_eq!(class.body.len(), 3);
    for (entry, value, span) in
        [(&class.body[0], 'a', Span::new(1, 7)), (&class.body[1], 'b', Span::new(7, 11))]
    {
        let CharacterClassContents::Character(character) = entry else {
            panic!("expected class character");
        };
        assert_eq!(character.value, u32::from(value));
        assert_eq!(character.span, span);
    }
    let CharacterClassContents::CharacterClassRange(range) = &class.body[2] else {
        panic!("expected class range");
    };
    assert_eq!((range.min.value, range.max.value), (u32::from('c'), u32::from('e')));
    assert_eq!(range.span, Span::new(11, 14));

    for (pattern, flags, message, span) in [
        ("[a-]", "v", "Unterminated character class", Span::new(0, 2)),
        ("[a-b&&c]", "v", "Unterminated character class", Span::new(0, 4)),
        ("[z-a]", "v", "Character class set range out of order", Span::new(1, 4)),
    ] {
        let error = LiteralParser::new(&allocator, pattern, Some(flags), Options::default())
            .parse()
            .unwrap_err();
        assert!(error.to_string().contains(message), "/{pattern}/{flags}: {error}");
        assert_eq!(error.labels[0].offset(), span.start);
        assert_eq!(error.labels[0].len(), span.size());
    }
}
