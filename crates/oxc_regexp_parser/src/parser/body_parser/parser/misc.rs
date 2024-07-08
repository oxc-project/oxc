use crate::parser::body_parser::unicode;

impl<'a> super::parse::PatternParser<'a> {
    // ```
    // DecimalDigits[Sep] ::
    //   DecimalDigit
    //   DecimalDigits[?Sep] DecimalDigit
    //   [+Sep] DecimalDigits[+Sep] NumericLiteralSeparator DecimalDigit
    // ```
    // <https://tc39.es/ecma262/#prod-DecimalDigits>
    pub(super) fn consume_decimal_digits(&mut self) -> Option<usize> {
        let checkpoint = self.reader.checkpoint();

        let mut value = 0;
        while let Some(cp) = self.reader.peek().filter(|&cp| unicode::is_decimal_digits(cp)) {
            // `- '0' as u32`: convert code point to digit
            value = (10 * value) + (cp - '0' as u32) as usize;
            self.reader.advance();
        }

        if self.reader.checkpoint() != checkpoint {
            return Some(value);
        }

        None
    }

    pub(super) fn consume_hex_digits(&mut self) -> Option<u32> {
        let checkpoint = self.reader.checkpoint();

        let mut value = 0;
        while let Some(hex) = self.reader.peek().and_then(unicode::map_hex_digit) {
            value = (16 * value) + hex;
            self.reader.advance();
        }

        if self.reader.checkpoint() != checkpoint {
            return Some(value);
        }

        None
    }

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
