pub struct Reader<'a> {
    source: &'a str,
    unicode_mode: bool,
    index: usize,
    u8_units: Vec<(usize, char)>,
    u16_units: Vec<u16>,
}

impl<'a> Reader<'a> {
    pub fn new(source: &'a str, unicode_mode: bool) -> Self {
        // NOTE: This may not be efficient if the source is too large.
        // Implements lookahead cache with `VecDeque` is better...?
        let u8_units = source.char_indices().collect::<Vec<_>>();
        let u16_units = if unicode_mode { "" } else { source }.encode_utf16().collect::<Vec<_>>();
        Self { source, unicode_mode, index: 0, u8_units, u16_units }
    }

    pub fn offset(&self) -> usize {
        if self.unicode_mode {
            self.u8_units.get(self.index).map_or(self.source.len(), |(idx, _)| *idx)
        } else {
            // NOTE: This can be optimized by saving the last idx?
            let mut u16_idx = 0;
            for (idx, ch) in &self.u8_units {
                if self.index <= u16_idx {
                    return *idx;
                }

                u16_idx += ch.len_utf16();
            }
            self.source.len()
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

        if self.unicode_mode {
            self.u8_units.get(nth).map(|&(_, ch)| ch as u32)
        } else {
            #[allow(clippy::cast_lossless)]
            self.u16_units.get(nth).map(|&cu| cu as u32)
        }
    }

    pub fn peek(&self) -> Option<u32> {
        self.peek_nth(0)
    }
    pub fn peek2(&self) -> Option<u32> {
        self.peek_nth(1)
    }

    pub fn eat(&mut self, ch: char) -> bool {
        if self.peek_nth(0) == Some(ch as u32) {
            self.advance();
            return true;
        }
        false
    }
    pub fn eat2(&mut self, ch: char, ch2: char) -> bool {
        if self.peek_nth(0) == Some(ch as u32) && self.peek_nth(1) == Some(ch2 as u32) {
            self.advance();
            self.advance();
            return true;
        }
        false
    }
    pub fn eat3(&mut self, ch: char, ch2: char, ch3: char) -> bool {
        if self.peek_nth(0) == Some(ch as u32)
            && self.peek_nth(1) == Some(ch2 as u32)
            && self.peek_nth(2) == Some(ch3 as u32)
        {
            self.advance();
            self.advance();
            self.advance();
            return true;
        }
        false
    }
    pub fn eat4(&mut self, ch: char, ch2: char, ch3: char, ch4: char) -> bool {
        if self.peek_nth(0) == Some(ch as u32)
            && self.peek_nth(1) == Some(ch2 as u32)
            && self.peek_nth(2) == Some(ch3 as u32)
            && self.peek_nth(3) == Some(ch4 as u32)
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
            let s1 = reader.offset();
            assert!(reader.eat('^'));
            let e1 = reader.offset();

            while reader.peek() != Some('@' as u32) {
                reader.advance();
            }
            let s2 = reader.offset();
            assert!(reader.eat('@'));
            let e2 = reader.offset();

            while reader.peek() != Some('$' as u32) {
                reader.advance();
            }
            let s3 = reader.offset();
            assert!(reader.eat('$'));
            let e3 = reader.offset();

            assert_eq!(&source_text[s1..e1], "^");
            assert_eq!(&source_text[s2..e2], "@");
            assert_eq!(&source_text[s3..e3], "$");
        }
    }
}
