use crate::parser::body_parser::unicode;

impl<'a> super::parse::PatternParser<'a> {
    pub(super) fn consume_decimal_digits(&mut self) -> Option<usize> {
        let span_start = self.reader.span_position();

        let mut value = 0;
        while let Some(cp) = self.reader.peek() {
            if !unicode::is_decimal_digits(cp) {
                break;
            }

            // `- '0' as u32`: convert code point to digit
            value = (10 * value) + (cp - '0' as u32) as usize;
            self.reader.advance();
        }

        if self.reader.span_position() != span_start {
            return Some(value);
        }

        None
    }
}
