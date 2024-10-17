mod reader_impl;
mod string_literal_parser;

pub use reader_impl::Reader;

#[cfg(test)]
mod test {
    use crate::parser::reader::Reader;
    use oxc_allocator::Allocator;

    #[test]
    fn index_basic() {
        let allocator = Allocator::default();

        for mut reader in [
            Reader::new(&allocator, "RegExp!", true, false),
            Reader::new(&allocator, "RegExp!", false, false),
            Reader::new(&allocator, r#""RegExp!""#, true, true),
            Reader::new(&allocator, r#""RegExp!""#, false, true),
            Reader::new(&allocator, "'RegExp!'", true, true),
            Reader::new(&allocator, "'RegExp!'", false, true),
        ] {
            reader.collect_units().unwrap();

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
    fn index_unicode() {
        let allocator = Allocator::default();
        let source_text = "𠮷野家は👈🏻あっち";

        let mut unicode_reader = Reader::new(&allocator, source_text, true, false);
        unicode_reader.collect_units().unwrap();
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

        let mut legacy_reader = Reader::new(&allocator, source_text, false, false);
        legacy_reader.collect_units().unwrap();
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
        let allocator = Allocator::default();

        let source_text1 = r"^ Catch😎 @@ symbols🇺🇳 $";
        let reader1 = Reader::new(&allocator, source_text1, true, false);

        let source_text2 = format!("\"{source_text1}\"");
        let reader2 = Reader::new(&allocator, &source_text2, true, true);

        for mut reader in [reader1, reader2] {
            reader.collect_units().unwrap();

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
}
