use crate::parser::body_parser::unicode;

impl<'a> super::parse::PatternParser<'a> {
    pub(super) fn consume_fixed_hex_digits(&mut self, len: usize) -> Option<u32> {
        let checkpoint = self.reader.checkpoint();

        let mut value = 0;
        for _ in 0..len {
            let Some(hex) = self.reader.peek().and_then(unicode::map_hex_digit) else {
                self.reader.rewind(checkpoint);
                return None;
            };

            value = (16 * value) + hex;
            self.reader.advance();
        }

        Some(value)
    }
}
