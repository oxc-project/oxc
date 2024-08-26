pub struct Reader<'a> {
    source_text: &'a str,
    /// Current index for `units`.
    index: usize,
    /// Iteration units.
    /// We use `char` and its index regardless of the unicode mode or non-unicode mode.
    /// However, in non-unicode mode, the parser may mark it as the surrogate pairs.
    units: Vec<(usize, char)>,
}

impl<'a> Reader<'a> {
    pub fn new(source_text: &'a str) -> Self {
        // NOTE: Collecting `Vec` may not be efficient if the source is too large.
        // Implements lookahead cache with `VecDeque` is better...?
        // But when I tried once, there are no notable improvements.
        let units = source_text.char_indices().collect::<Vec<_>>();

        Self { source_text, index: 0, units }
    }

    pub fn offset(&mut self) -> usize {
        self.units.get(self.index).map_or(self.source_text.len(), |(idx, _)| *idx)
    }

    // NOTE: For now, `usize` is enough for the checkpoint.
    pub fn checkpoint(&self) -> usize {
        self.index
    }

    pub fn rewind(&mut self, checkpoint: usize) {
        self.index = checkpoint;
    }

    pub fn advance(&mut self) {
        self.index += 1;
    }

    // We can iterate over `char` and parse the pattern.
    // But we need a code point, not a char, for AST results.
    fn peek_nth(&self, n: usize) -> Option<u32> {
        let nth = self.index + n;
        self.units.get(nth).map(|&(_, ch)| ch as u32)
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
        let mut reader = Reader::new(source_text);

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

    #[test]
    fn index_unicode() {
        let source_text = "𠮷野家は👈🏻あっち";
        let mut reader = Reader::new(source_text);

        assert!(reader.eat('𠮷')); // Can eat
        assert!(reader.eat2('野', '家'));
        let checkpoint = reader.checkpoint();
        assert!(reader.eat('は'));

        // Emoji + Skin tone
        reader.advance();
        reader.advance();

        assert!(reader.eat('あ'));
        assert_eq!(reader.peek(), Some('っ' as u32));
        assert_eq!(reader.peek2(), Some('ち' as u32));

        reader.rewind(checkpoint);
        assert!(reader.eat('は'));
    }

    #[test]
    fn span_position() {
        let source_text = "^ Catch😎 @ symbols🇺🇳 $";
        let mut reader = Reader::new(source_text);

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
