mod ast;
mod characters;
mod options;
mod reader_impl;
mod string_literal_parser;
mod template_literal_parser;

pub use ast::*;
pub use options::Options;
pub use reader_impl::Reader;

#[cfg(test)]
mod test {
    use crate::parser::reader::Reader;

    #[test]
    fn should_fail() {
        for reader in [
            Reader::initialize(r#""Unterminated"#, true, true),
            Reader::initialize(r#""Unterminated"#, false, true),
            Reader::initialize("'Unterminated!", true, true),
            Reader::initialize("'Unterminated!", false, true),
        ] {
            assert!(reader.is_err());
        }
    }

    #[test]
    fn should_pass_basic() {
        for mut reader in [
            Reader::initialize("RegExp!", true, false).unwrap(),
            Reader::initialize("RegExp!", false, false).unwrap(),
            Reader::initialize(r#""RegExp!""#, true, true).unwrap(),
            Reader::initialize(r#""RegExp!""#, false, true).unwrap(),
            Reader::initialize("'RegExp!'", true, true).unwrap(),
            Reader::initialize("'RegExp!'", false, true).unwrap(),
        ] {
            assert_eq!(reader.peek(), Some('R' as u32));
            assert_eq!(reader.peek2(), Some('e' as u32));
            assert!(reader.eat('R'));
            assert!(!reader.eat('R'));
            assert!(reader.eat('e'));
            assert!(reader.eat('g'));
            assert!(reader.eat('E'));
            assert!(!reader.eat3('E', 'x', 'p'));
            assert!(reader.eat2('x', 'p'));

            let checkpoint = reader.checkpoint();
            assert_eq!(reader.peek(), Some('!' as u32));
            reader.advance();
            reader.advance();

            reader.rewind(checkpoint);
            assert_eq!(reader.peek(), Some('!' as u32));

            assert!(reader.eat('!'));
            assert_eq!(reader.peek(), None);
        }
    }

    #[test]
    fn should_pass_unicode() {
        let source_text = "𠮷野家は👈🏻あっち";

        let mut unicode_reader = Reader::initialize(source_text, true, false).unwrap();
        assert!(unicode_reader.eat('𠮷')); // Can eat
        assert!(unicode_reader.eat2('野', '家'));
        let checkpoint = unicode_reader.checkpoint();
        assert!(unicode_reader.eat('は'));
        unicode_reader.advance(); // Emoji
        unicode_reader.advance(); // Skin tone
        assert!(unicode_reader.eat('あ'));
        assert_eq!(unicode_reader.peek(), Some('っ' as u32));
        assert_eq!(unicode_reader.peek2(), Some('ち' as u32));
        unicode_reader.rewind(checkpoint);
        assert!(unicode_reader.eat('は'));

        let mut legacy_reader = Reader::initialize(source_text, false, false).unwrap();
        assert!(!legacy_reader.eat('𠮷')); // Can not eat
        legacy_reader.advance();
        assert!(!legacy_reader.eat('𠮷')); // Also can not
        legacy_reader.advance();
        assert!(legacy_reader.eat('野'));
        assert!(legacy_reader.eat('家'));
        let checkpoint = unicode_reader.checkpoint();
        assert!(legacy_reader.eat('は'));
        legacy_reader.advance(); // Emoji(High surrogate)
        legacy_reader.advance(); // Emoji(Low surrogate)
        legacy_reader.advance(); // Skin tone(High surrogate)
        legacy_reader.advance(); // Skin tone(Low surrogate)
        assert_eq!(legacy_reader.peek(), Some('あ' as u32));
        assert_eq!(legacy_reader.peek2(), Some('っ' as u32));
        assert!(legacy_reader.eat3('あ', 'っ', 'ち'));
        legacy_reader.rewind(checkpoint);
        assert!(legacy_reader.eat('は'));
    }

    #[test]
    fn span_position() {
        let source_text1 = r"^ Catch😎 @@ symbols🇺🇳 $";
        let reader1 = Reader::initialize(source_text1, true, false).unwrap();

        let source_text2 = format!("\"{source_text1}\"");
        let reader2 = Reader::initialize(&source_text2, true, true).unwrap();

        for mut reader in [reader1, reader2] {
            while reader.peek() != Some('^' as u32) {
                reader.advance();
            }
            let s1 = reader.offset();
            assert!(reader.eat('^'));
            let e1 = reader.offset();
            assert_eq!(&reader.atom(s1, e1), "^");

            while reader.peek() != Some('@' as u32) {
                reader.advance();
            }
            let s2 = reader.offset();
            assert!(reader.eat('@'));
            assert!(reader.eat('@'));
            let e2 = reader.offset();
            assert_eq!(&reader.atom(s2, e2), "@@");

            while reader.peek() != Some('$' as u32) {
                reader.advance();
            }
            let s3 = reader.offset();
            assert!(reader.eat('$'));
            let e3 = reader.offset();

            assert_eq!(&reader.atom(s3, e3), "$");
        }
    }

    #[test]
    fn handle_escapes() {
        let mut reader1 = Reader::initialize("こんにちWorld2024", true, false).unwrap();
        let mut reader2 = Reader::initialize(
            r#""\u3053\u3093\u306B\u3061\u0057\u006F\u0072\u006C\u0064\u0032\u0030\u0032\u0034""#,
            false,
            true,
        )
        .unwrap();

        assert_eq!(reader1.eat('こ'), reader2.eat('こ'));
        assert_eq!(reader1.eat('ん'), reader2.eat('ん'));

        loop {
            match (reader1.peek(), reader2.peek()) {
                (None, None) => {
                    break; // All passed
                }
                (Some(cp1), Some(cp2)) if cp1 == cp2 => {
                    reader1.advance();
                    reader2.advance();
                    // println!("{} == {}: {:?}", cp1, cp2, char::try_from(cp1));
                }
                _ => {
                    panic!("Mismatched characters");
                }
            }
        }
    }

    #[test]
    fn escape_kind_tracking() {
        use crate::parser::reader::EscapeKind;

        // For regexp literal, escape kind should be None (escapes are handled by pattern parser)
        let mut reader1 = Reader::initialize(r"A\u0301", true, false).unwrap();
        assert_eq!(reader1.peek_escape_kind(), EscapeKind::None); // 'A'
        reader1.advance();
        assert_eq!(reader1.peek_escape_kind(), EscapeKind::None); // '\' - literal in regexp
        reader1.advance();
        assert_eq!(reader1.peek_escape_kind(), EscapeKind::None); // 'u'

        // For string literal, unicode escapes should be tracked
        let mut reader2 = Reader::initialize(r#""A\u0301""#, true, true).unwrap();
        assert_eq!(reader2.peek_escape_kind(), EscapeKind::None); // 'A'
        reader2.advance();
        assert_eq!(reader2.peek_escape_kind(), EscapeKind::Unicode); // \u0301 resolved to U+0301

        // For string literal with hex escape
        let mut reader3 = Reader::initialize(r#""A\x41""#, true, true).unwrap();
        assert_eq!(reader3.peek_escape_kind(), EscapeKind::None); // 'A'
        reader3.advance();
        assert_eq!(reader3.peek_escape_kind(), EscapeKind::Hexadecimal); // \x41 resolved to 'A'
    }
}
