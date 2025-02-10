use oxc_syntax::identifier::{is_identifier_part_ascii, is_identifier_start};

use crate::diagnostics;

use super::{Kind, Lexer, Span};

impl Lexer<'_> {
    /// 12.9.3 Numeric Literals with `0` prefix
    pub(super) fn read_zero(&mut self) -> Kind {
        match self.peek_byte() {
            Some(b'b' | b'B') => self.read_non_decimal(Kind::Binary),
            Some(b'o' | b'O') => self.read_non_decimal(Kind::Octal),
            Some(b'x' | b'X') => self.read_non_decimal(Kind::Hex),
            Some(b'e' | b'E') => {
                self.consume_char();
                self.read_decimal_exponent()
            }
            Some(b'.') => {
                self.consume_char();
                self.decimal_literal_after_decimal_point_after_digits()
            }
            Some(b'n') => {
                self.consume_char();
                self.check_after_numeric_literal(Kind::Decimal)
            }
            Some(n) if n.is_ascii_digit() => self.read_legacy_octal(),
            _ => self.check_after_numeric_literal(Kind::Decimal),
        }
    }

    pub(super) fn decimal_literal_after_first_digit(&mut self) -> Kind {
        self.read_decimal_digits_after_first_digit();
        if self.next_ascii_byte_eq(b'.') {
            return self.decimal_literal_after_decimal_point_after_digits();
        } else if self.next_ascii_byte_eq(b'n') {
            return self.check_after_numeric_literal(Kind::Decimal);
        }

        let kind = self.optional_exponent().map_or(Kind::Decimal, |kind| kind);
        self.check_after_numeric_literal(kind)
    }

    // Inline into the 3 calls from `read_zero` so that value of `kind` is known
    // and `kind.matches_number_byte` can be statically reduced to just the match arm
    // that applies for this specific kind. `matches_number_byte` is also marked `#[inline]`.
    #[inline]
    fn read_non_decimal(&mut self, kind: Kind) -> Kind {
        self.consume_char();

        if self.peek_byte().is_some_and(|b| kind.matches_number_byte(b)) {
            self.consume_char();
        } else {
            self.unexpected_err();
            return Kind::Undetermined;
        }

        while let Some(b) = self.peek_byte() {
            match b {
                b'_' => {
                    self.consume_char();
                    // NOTE: it looks invalid numeric tokens are still parsed.
                    // This seems to be a waste. It also requires us to put this
                    // call here instead of after we ensure the next character
                    // is a number character
                    self.token.set_has_separator();
                    if self.peek_byte().is_some_and(|b| kind.matches_number_byte(b)) {
                        self.consume_char();
                    } else {
                        self.unexpected_err();
                        return Kind::Undetermined;
                    }
                }
                b if kind.matches_number_byte(b) => {
                    self.consume_char();
                }
                _ => break,
            }
        }
        self.next_ascii_byte_eq(b'n');
        self.check_after_numeric_literal(kind)
    }

    fn read_legacy_octal(&mut self) -> Kind {
        let mut kind = Kind::Octal;
        loop {
            match self.peek_byte() {
                Some(b'0'..=b'7') => {
                    self.consume_char();
                }
                Some(b'8'..=b'9') => {
                    self.consume_char();
                    kind = Kind::Decimal;
                }
                _ => break,
            }
        }

        match self.peek_byte() {
            // allow 08.5 and 09.5
            Some(b'.') if kind == Kind::Decimal => {
                self.consume_char();
                self.decimal_literal_after_decimal_point_after_digits()
            }
            // allow 08e1 and 09e1
            Some(b'e') if kind == Kind::Decimal => {
                self.consume_char();
                self.read_decimal_exponent()
            }
            _ => self.check_after_numeric_literal(kind),
        }
    }

    fn read_decimal_exponent(&mut self) -> Kind {
        let kind = match self.peek_byte() {
            Some(b'-') => {
                self.consume_char();
                Kind::NegativeExponential
            }
            Some(b'+') => {
                self.consume_char();
                Kind::PositiveExponential
            }
            _ => Kind::PositiveExponential,
        };
        self.read_decimal_digits();
        kind
    }

    fn read_decimal_digits(&mut self) {
        if self.peek_byte().is_some_and(|b| b.is_ascii_digit()) {
            self.consume_char();
        } else {
            self.unexpected_err();
            return;
        }

        self.read_decimal_digits_after_first_digit();
    }

    fn read_decimal_digits_after_first_digit(&mut self) {
        while let Some(b) = self.peek_byte() {
            match b {
                b'_' => {
                    self.consume_char();
                    // NOTE: it looks invalid numeric tokens are still parsed.
                    // This seems to be a waste. It also requires us to put this
                    // call here instead of after we ensure the next character
                    // is an ASCII digit
                    self.token.set_has_separator();
                    if self.peek_byte().is_some_and(|b| b.is_ascii_digit()) {
                        self.consume_char();
                    } else {
                        self.unexpected_err();
                        return;
                    }
                }
                b'0'..=b'9' => {
                    self.consume_char();
                }
                _ => break,
            }
        }
    }

    pub(super) fn decimal_literal_after_decimal_point(&mut self) -> Kind {
        self.read_decimal_digits();
        self.optional_exponent();
        self.check_after_numeric_literal(Kind::Float)
    }

    fn decimal_literal_after_decimal_point_after_digits(&mut self) -> Kind {
        self.optional_decimal_digits();
        self.optional_exponent();
        self.check_after_numeric_literal(Kind::Float)
    }

    fn optional_decimal_digits(&mut self) {
        if self.peek_byte().is_some_and(|b| b.is_ascii_digit()) {
            self.consume_char();
            self.read_decimal_digits_after_first_digit();
        }
    }

    fn optional_exponent(&mut self) -> Option<Kind> {
        if matches!(self.peek_byte(), Some(b'e' | b'E')) {
            self.consume_char();
            return Some(self.read_decimal_exponent());
        }
        None
    }

    fn check_after_numeric_literal(&mut self, kind: Kind) -> Kind {
        // The SourceCharacter immediately following a NumericLiteral must not be
        // an IdentifierStart or DecimalDigit.
        // Use a fast path for common case where next char is ASCII.
        // NB: `!is_identifier_part_ascii(b as char)` is equivalent to
        // `!b.is_ascii_digit() && !is_identifier_start_ascii(b as char)`
        match self.peek_byte() {
            Some(b) if b.is_ascii() => {
                if !is_identifier_part_ascii(b as char) {
                    return kind;
                }
            }
            Some(_) => {
                // Unicode
                let c = self.peek_char().unwrap();
                if !is_identifier_start(c) {
                    return kind;
                }
            }
            None => return kind,
        }

        // Invalid next char
        let offset = self.offset();
        self.consume_char();
        while let Some(c) = self.peek_char() {
            if is_identifier_start(c) {
                self.consume_char();
            } else {
                break;
            }
        }
        self.error(diagnostics::invalid_number_end(Span::new(offset, self.offset())));
        Kind::Undetermined
    }
}
