use oxc_diagnostics::Result;
use oxc_syntax::identifier::is_line_terminator;

use crate::diagnostics;

use super::{Kind, Lexer, RegExpFlags, Token};

impl Lexer<'_> {
    /// Re-tokenize the current `/` or `/=` and return `RegExp`
    /// See Section 12:
    ///   The `InputElementRegExp` goal symbol is used in all syntactic grammar contexts
    ///   where a `RegularExpressionLiteral` is permitted
    /// Which means the parser needs to re-tokenize on `PrimaryExpression`,
    /// `RegularExpressionLiteral` only appear on the right hand side of `PrimaryExpression`
    pub(crate) fn next_regex(&mut self, kind: Kind) -> Result<(Token, u32, RegExpFlags, bool)> {
        self.token.start = self.offset()
            - match kind {
                Kind::Slash => 1,
                Kind::SlashEq => 2,
                _ => unreachable!(),
            };
        let (pattern_end, flags, flags_error) = self.read_regex()?;
        self.lookahead.clear();
        let token = self.finish_next(Kind::RegExp);
        Ok((token, pattern_end, flags, flags_error))
    }

    /// 12.9.5 Regular Expression Literals
    fn read_regex(&mut self) -> Result<(u32, RegExpFlags, bool)> {
        let mut in_escape = false;
        let mut in_character_class = false;
        loop {
            match self.next_char() {
                None => {
                    return Err(diagnostics::unterminated_reg_exp(self.unterminated_range()));
                    // return (self.offset(), RegExpFlags::empty());
                }
                Some(c) if is_line_terminator(c) => {
                    return Err(diagnostics::unterminated_reg_exp(self.unterminated_range()));
                }
                Some(c) => {
                    if in_escape {
                        in_escape = false;
                    } else if c == '/' && !in_character_class {
                        break;
                    } else if c == '[' {
                        in_character_class = true;
                    } else if c == '\\' {
                        in_escape = true;
                    } else if c == ']' {
                        in_character_class = false;
                    }
                }
            }
        }

        let pattern_end = self.offset() - 1; // -1 to exclude `/`
        let mut flags = RegExpFlags::empty();
        // To prevent parsing `oxc_regular_expression` with invalid flags in the parser
        let mut flags_error = false;

        while let Some(b @ (b'$' | b'_' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9')) =
            self.peek_byte()
        {
            self.consume_char();
            let Ok(flag) = RegExpFlags::try_from(b) else {
                self.error(diagnostics::reg_exp_flag(
                    b as char,
                    self.current_offset().expand_left(1),
                ));
                flags_error = true;
                continue;
            };
            if flags.contains(flag) {
                self.error(diagnostics::reg_exp_flag_twice(
                    b as char,
                    self.current_offset().expand_left(1),
                ));
                flags_error = true;
                continue;
            }
            flags |= flag;
        }

        Ok((pattern_end, flags, flags_error))
    }
}
