pub struct Reader<'a> {
    source: &'a str,
    // NOTE: For now, this exists only for `span_position()` method.
    char_indices: std::str::CharIndices<'a>,
    unicode_mode: bool,
    index: usize,
}

impl<'a> Reader<'a> {
    pub fn new(source: &'a str, unicode_mode: bool) -> Self {
        Self { source, char_indices: source.char_indices(), unicode_mode, index: 0 }
    }

    // NOTE: Should be decoupled from the reader...?
    // ```
    // let reader_idx = reader.index;
    // SpanPosition::new(source, unicode_mode).get(reader_idx);
    // ````
    pub fn span_position(&self) -> usize {
        let mut char_indices = self.char_indices.clone();

        if self.unicode_mode {
            char_indices.nth(self.index).map_or(self.source.len(), |(i, _)| i)
        } else {
            let mut utf16_units = 0;
            let mut byte_index = 0;
            for (idx, ch) in char_indices {
                if utf16_units == self.index {
                    return idx;
                }

                utf16_units += ch.len_utf16();
                byte_index = idx + ch.len_utf8();
            }
            byte_index
        }
    }

    pub fn checkpoint(&self) -> usize {
        self.index
    }
    pub fn rewind(&mut self, checkpoint: usize) {
        self.index = checkpoint;
    }

    pub fn advance(&mut self) {
        self.index += 1;
    }

    fn peek_nth(&self, n: usize) -> Option<u32> {
        let nth = self.index + n;

        // TODO: This is not efficient.
        // Refs oxc_parser/src/lexer/mod.rs using `VecDeque` for this?
        if self.unicode_mode {
            self.source.chars().nth(nth).map(|c| c as u32)
        } else {
            #[allow(clippy::cast_lossless)]
            self.source.encode_utf16().nth(nth).map(|u| u as u32)
        }
    }

    pub fn peek(&self) -> Option<u32> {
        self.peek_nth(0)
    }
    pub fn peek2(&self) -> Option<u32> {
        self.peek_nth(1)
    }
    pub fn peek3(&self) -> Option<u32> {
        self.peek_nth(2)
    }
    pub fn peek4(&self) -> Option<u32> {
        self.peek_nth(3)
    }

    // NOTE: Consider `peek(ch: char): bool` style API?

    pub fn eat(&mut self, ch: char) -> bool {
        if self.peek() == Some(ch as u32) {
            self.advance();
            return true;
        }
        false
    }
    pub fn eat2(&mut self, ch: char, ch2: char) -> bool {
        if self.peek() == Some(ch as u32) && self.peek2() == Some(ch2 as u32) {
            self.advance();
            self.advance();
            return true;
        }
        false
    }
    pub fn eat3(&mut self, ch: char, ch2: char, ch3: char) -> bool {
        if self.peek() == Some(ch as u32)
            && self.peek2() == Some(ch2 as u32)
            && self.peek3() == Some(ch3 as u32)
        {
            self.advance();
            self.advance();
            self.advance();
            return true;
        }
        false
    }
    pub fn eat4(&mut self, ch: char, ch2: char, ch3: char, ch4: char) -> bool {
        if self.peek() == Some(ch as u32)
            && self.peek2() == Some(ch2 as u32)
            && self.peek3() == Some(ch3 as u32)
            && self.peek4() == Some(ch4 as u32)
        {
            self.advance();
            self.advance();
            self.advance();
            self.advance();
            return true;
        }
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn index_basic() {
        let source_text = "/RegExp✨/i";
        let unicode_reader = Reader::new(source_text, true);
        let legacy_reader = Reader::new(source_text, false);

        for mut reader in [unicode_reader, legacy_reader] {
            assert_eq!(reader.index, 0);
            assert_eq!(reader.peek(), Some('/' as u32));

            reader.advance();
            assert_eq!(reader.index, 1);
            assert_eq!(reader.peek(), Some('R' as u32));
            assert_eq!(reader.peek2(), Some('e' as u32));
            assert_eq!(reader.peek3(), Some('g' as u32));

            assert!(reader.eat('R'));
            assert!(!reader.eat('R'));
            assert!(reader.eat('e'));
            assert!(reader.eat('g'));
            assert!(reader.eat('E'));
            assert!(!reader.eat3('E', 'x', 'p'));
            assert!(reader.eat2('x', 'p'));

            let checkpoint = reader.checkpoint();
            assert_eq!(checkpoint, 7);
            assert_eq!(reader.peek(), Some('✨' as u32));

            reader.advance();
            reader.advance();
            assert_eq!(reader.peek(), Some('i' as u32));

            reader.advance();
            assert_eq!(reader.peek(), None);

            reader.rewind(checkpoint);
            assert_eq!(reader.peek(), Some('✨' as u32));
        }
    }

    #[test]
    fn index_unicode() {
        let source_text = "𠮷野家は👈🏻あっち";

        let mut unicode_reader = Reader::new(source_text, true);

        assert!(unicode_reader.eat('𠮷')); // Can eat
        assert!(unicode_reader.eat2('野', '家'));
        let checkpoint = unicode_reader.checkpoint();
        assert!(unicode_reader.eat('は'));

        // Emoji + Skin tone
        unicode_reader.advance();
        unicode_reader.advance();

        assert!(unicode_reader.eat('あ'));
        assert_eq!(unicode_reader.peek(), Some('っ' as u32));
        assert_eq!(unicode_reader.peek2(), Some('ち' as u32));
        assert_eq!(unicode_reader.peek3(), None);

        unicode_reader.rewind(checkpoint);
        assert!(unicode_reader.eat('は'));

        let mut legacy_reader = Reader::new(source_text, false);

        assert!(!legacy_reader.eat('𠮷')); // Can not eat
        legacy_reader.advance();
        assert!(!legacy_reader.eat('𠮷')); // Also can not
        legacy_reader.advance();

        assert!(legacy_reader.eat('野'));
        assert!(legacy_reader.eat('家'));
        let checkpoint = unicode_reader.checkpoint();
        assert!(legacy_reader.eat('は'));

        legacy_reader.advance();
        legacy_reader.advance();
        legacy_reader.advance();
        legacy_reader.advance();

        assert_eq!(legacy_reader.peek(), Some('あ' as u32));
        assert_eq!(legacy_reader.peek2(), Some('っ' as u32));
        assert_eq!(legacy_reader.peek3(), Some('ち' as u32));
        assert!(legacy_reader.eat3('あ', 'っ', 'ち'));

        legacy_reader.rewind(checkpoint);
        assert!(legacy_reader.eat('は'));
    }

    #[test]
    fn span_position() {
        let source_text = "^ Catch😎 @ symbols🇺🇳 $";

        let unicode_reader = Reader::new(source_text, true);
        let legacy_reader = Reader::new(source_text, false);

        for mut reader in [unicode_reader, legacy_reader] {
            while reader.peek() != Some('^' as u32) {
                reader.advance();
            }
            let s1 = reader.span_position();
            assert!(reader.eat('^'));
            let e1 = reader.span_position();

            while reader.peek() != Some('@' as u32) {
                reader.advance();
            }
            let s2 = reader.span_position();
            assert!(reader.eat('@'));
            let e2 = reader.span_position();

            while reader.peek() != Some('$' as u32) {
                reader.advance();
            }
            let s3 = reader.span_position();
            assert!(reader.eat('$'));
            let e3 = reader.span_position();

            assert_eq!(&source_text[s1..e1], "^");
            assert_eq!(&source_text[s2..e2], "@");
            assert_eq!(&source_text[s3..e3], "$");
        }
    }
}
