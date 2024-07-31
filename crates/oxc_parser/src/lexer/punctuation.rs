use super::{Kind, Lexer, Token};

impl<'a> Lexer<'a> {
    /// Section 12.8 Punctuators
    pub(super) fn read_dot(&mut self) -> Kind {
        if self.peek_2_bytes() == Some([b'.', b'.']) {
            self.consume_2_chars();
            return Kind::Dot3;
        }
        if self.peek_byte().is_some_and(|b| b.is_ascii_digit()) {
            self.decimal_literal_after_decimal_point()
        } else {
            Kind::Dot
        }
    }

    /// returns None for `SingleLineHTMLOpenComment` `<!--` in script mode
    pub(super) fn read_left_angle(&mut self) -> Option<Kind> {
        if self.next_ascii_byte_eq(b'<') {
            if self.next_ascii_byte_eq(b'=') {
                Some(Kind::ShiftLeftEq)
            } else {
                Some(Kind::ShiftLeft)
            }
        } else if self.next_ascii_byte_eq(b'=') {
            Some(Kind::LtEq)
        } else if self.peek_byte() == Some(b'!')
            // SingleLineHTMLOpenComment `<!--` in script mode
            && self.source_type.is_script()
            && self.remaining().starts_with("!--")
        {
            None
        } else {
            Some(Kind::LAngle)
        }
    }

    /// returns None for `SingleLineHTMLCloseComment` `-->` in script mode
    pub(super) fn read_minus(&mut self) -> Option<Kind> {
        if self.next_ascii_byte_eq(b'-') {
            // SingleLineHTMLCloseComment `-->` in script mode
            if self.token.is_on_new_line
                && self.source_type.is_script()
                && self.next_ascii_byte_eq(b'>')
            {
                None
            } else {
                Some(Kind::Minus2)
            }
        } else if self.next_ascii_byte_eq(b'=') {
            Some(Kind::MinusEq)
        } else {
            Some(Kind::Minus)
        }
    }

    pub(crate) fn next_right_angle(&mut self) -> Token {
        let kind = self.read_right_angle();
        self.lookahead.clear();
        self.finish_next(kind)
    }

    fn read_right_angle(&mut self) -> Kind {
        if self.next_ascii_byte_eq(b'>') {
            if self.next_ascii_byte_eq(b'>') {
                if self.next_ascii_byte_eq(b'=') {
                    Kind::ShiftRight3Eq
                } else {
                    Kind::ShiftRight3
                }
            } else if self.next_ascii_byte_eq(b'=') {
                Kind::ShiftRightEq
            } else {
                Kind::ShiftRight
            }
        } else if self.next_ascii_byte_eq(b'=') {
            Kind::GtEq
        } else {
            Kind::RAngle
        }
    }
}
