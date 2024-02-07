use super::simd::{string_literal_lookup, Position};
use super::{AutoCow, Kind, Lexer, Span, Token};
use crate::diagnostics;

impl<'a> Lexer<'a> {
    /// 12.9.4 String Literals
    pub(super) fn read_string_literal(&mut self, delimiter: char) -> Kind {
        let mut builder = AutoCow::new(self);
        while !self.source.is_eof() {
            let Position { offset, alignment } = string_literal_lookup(&self.source);

            if offset == alignment {
                // no delimiter found in this 32 bytes
                self.source.advance(offset);
                continue;
            }
            let matched = self.source.nth(offset);
            self.source.advance(offset);
            match matched {
                b'\r' | b'\n' => {
                    self.error(diagnostics::UnterminatedString(self.unterminated_range()));
                    return Kind::Undetermined;
                }
                c @ (b'"' | b'\'') => {
                    if c as char == delimiter {
                        self.save_string(builder.has_escape(), builder.finish_without_push(self));
                        return Kind::Str;
                    }
                    builder.push_matching(c as char);
                }
                b'\\' => {
                    let start = self.offset() - 1;
                    let text = builder.get_mut_string_without_current_ascii_char(self);
                    let mut is_valid_escape_sequence = true;
                    self.read_string_escape_sequence(text, false, &mut is_valid_escape_sequence);
                    if !is_valid_escape_sequence {
                        let range = Span::new(start, self.offset());
                        self.error(diagnostics::InvalidEscapeSequence(range));
                    }
                }
                c => {
                    builder.push_matching(c as char);
                }
            }
        }
        // add padding to the end of the source
        todo!()
    }

    /// Save the string if it is escaped
    /// This reduces the overall memory consumption while keeping the `Token` size small
    /// Strings without escaped values can be retrieved as is from the token span
    pub(super) fn save_string(&mut self, has_escape: bool, s: &'a str) {
        if !has_escape {
            return;
        }
        self.escaped_strings.insert(self.token.start, s);
        self.token.escaped = true;
    }

    pub(crate) fn get_string(&self, token: Token) -> &'a str {
        if token.escaped {
            return self.escaped_strings[&token.start];
        }

        let raw = &self.source.whole()[token.start as usize..token.end as usize];
        match token.kind {
            Kind::Str => {
                &raw[1..raw.len() - 1] // omit surrounding quotes
            }
            Kind::PrivateIdentifier => {
                &raw[1..] // omit leading `#`
            }
            _ => raw,
        }
    }
}
