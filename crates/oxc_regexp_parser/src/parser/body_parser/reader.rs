pub struct Reader {
    units: Vec<u32>,
    pub index: usize,
}

impl<'a> Reader {
    pub fn new(source: &'a str, unicode_mode: bool) -> Self {
        // NOTE: Is there a better way to avoid using `Vec`?
        // - To implement `peek2`, `peek3`, `Peekable` is not enough
        // - To `rewind` at any point, consuming `Iter` need more efforts(e.g. cache)
        let units = if unicode_mode {
            source.chars().map(|c| c as u32).collect()
        } else {
            #[allow(clippy::cast_lossless)]
            source.encode_utf16().map(|u| u as u32).collect()
        };

        Self { units, index: 0 }
    }

    // NOTE: How to know global unicode(utf8?) `Span` position?
    // - If reader is non-unicode mode, the `index` is not a valid position anymore
    // - Need map or something with using `ch.len_utf8|16`?
    #[allow(clippy::unused_self, dead_code)]
    pub fn position(&self) -> usize {
        0
    }

    pub fn rewind(&mut self, index: usize) {
        self.index = index;
    }

    pub fn advance(&mut self) {
        self.index += 1;
    }

    pub fn peek1(&self) -> Option<u32> {
        self.units.get(self.index).copied()
    }
    pub fn peek2(&self) -> Option<u32> {
        self.units.get(self.index + 1).copied()
    }
    pub fn peek3(&self) -> Option<u32> {
        self.units.get(self.index + 2).copied()
    }

    pub fn eat1(&mut self, ch: char) -> bool {
        if self.peek1() == Some(ch as u32) {
            self.advance();
            return true;
        }
        false
    }
    pub fn eat2(&mut self, ch: char, ch2: char) -> bool {
        if self.peek1() == Some(ch as u32) && self.peek2() == Some(ch2 as u32) {
            self.advance();
            self.advance();
            return true;
        }
        false
    }
    pub fn eat3(&mut self, ch: char, ch2: char, ch3: char) -> bool {
        if self.peek1() == Some(ch as u32)
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_index_basic() {
        let source_text = "/RegExp✨/i";
        let unicode_reader = Reader::new(source_text, true);
        let legacy_reader = Reader::new(source_text, false);

        for mut reader in [unicode_reader, legacy_reader] {
            assert_eq!(reader.index, 0);
            assert_eq!(reader.peek1(), Some('/' as u32));

            reader.advance();
            assert_eq!(reader.index, 1);
            assert_eq!(reader.peek1(), Some('R' as u32));
            assert_eq!(reader.peek2(), Some('e' as u32));
            assert_eq!(reader.peek3(), Some('g' as u32));

            assert!(reader.eat1('R'));
            assert!(!reader.eat1('R'));
            assert!(reader.eat1('e'));
            assert!(reader.eat1('g'));
            assert!(reader.eat1('E'));
            assert!(!reader.eat3('E', 'x', 'p'));
            assert!(reader.eat2('x', 'p'));

            let start = reader.index;
            assert_eq!(start, 7);
            assert_eq!(reader.peek1(), Some('✨' as u32));

            reader.advance();
            reader.advance();
            assert_eq!(reader.peek1(), Some('i' as u32));

            reader.advance();
            assert_eq!(reader.peek1(), None);

            reader.rewind(start);
            assert_eq!(reader.peek1(), Some('✨' as u32));
        }
    }

    #[test]
    fn test_index_unicode() {
        let source_text = "𠮷野家は👈🏻あっち";

        let mut unicode_reader = Reader::new(source_text, true);

        assert!(unicode_reader.eat1('𠮷')); // Can eat
        assert!(unicode_reader.eat2('野', '家'));
        let start = unicode_reader.index;
        assert!(unicode_reader.eat1('は'));

        // Emoji + Skin tone
        unicode_reader.advance();
        unicode_reader.advance();

        assert!(unicode_reader.eat1('あ'));
        assert_eq!(unicode_reader.peek1(), Some('っ' as u32));
        assert_eq!(unicode_reader.peek2(), Some('ち' as u32));
        assert_eq!(unicode_reader.peek3(), None);

        unicode_reader.rewind(start);
        assert!(unicode_reader.eat1('は'));

        let mut legacy_reader = Reader::new(source_text, false);

        assert!(!legacy_reader.eat1('𠮷')); // Can not eat
        legacy_reader.advance();
        assert!(!legacy_reader.eat1('𠮷')); // Also can not
        legacy_reader.advance();

        assert!(legacy_reader.eat1('野'));
        assert!(legacy_reader.eat1('家'));
        let start = unicode_reader.index;
        assert!(legacy_reader.eat1('は'));

        legacy_reader.advance();
        legacy_reader.advance();
        legacy_reader.advance();
        legacy_reader.advance();

        assert_eq!(legacy_reader.peek1(), Some('あ' as u32));
        assert_eq!(legacy_reader.peek2(), Some('っ' as u32));
        assert_eq!(legacy_reader.peek3(), Some('ち' as u32));
        assert!(legacy_reader.eat3('あ', 'っ', 'ち'));

        legacy_reader.rewind(start);
        assert!(legacy_reader.eat1('は'));
    }
}
