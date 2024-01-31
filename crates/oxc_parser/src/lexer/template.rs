use super::{AutoCow, Kind, Lexer, Token};
use crate::diagnostics;

use oxc_syntax::identifier::{CR, LF};

impl<'a> Lexer<'a> {
    /// 12.8.6 Template Literal Lexical Components
    pub(super) fn read_template_literal(&mut self, substitute: Kind, tail: Kind) -> Kind {
        let mut builder = AutoCow::new(self);
        let mut is_valid_escape_sequence = true;
        while let Some(c) = self.next_char() {
            match c {
                '$' if self.peek() == Some('{') => {
                    self.save_template_string(
                        is_valid_escape_sequence,
                        builder.has_escape(),
                        builder.finish_without_push(self),
                    );
                    self.consume_char();
                    return substitute;
                }
                '`' => {
                    self.save_template_string(
                        is_valid_escape_sequence,
                        builder.has_escape(),
                        builder.finish_without_push(self),
                    );
                    return tail;
                }
                CR => {
                    builder.force_allocation_without_current_ascii_char(self);
                    if self.next_eq(LF) {
                        builder.push_different(LF);
                    }
                }
                '\\' => {
                    let text = builder.get_mut_string_without_current_ascii_char(self);
                    self.read_string_escape_sequence(text, true, &mut is_valid_escape_sequence);
                }
                _ => builder.push_matching(c),
            }
        }
        self.error(diagnostics::UnterminatedString(self.unterminated_range()));
        Kind::Undetermined
    }

    /// Re-tokenize the current `}` token for `TemplateSubstitutionTail`
    /// See Section 12, the parser needs to re-tokenize on `TemplateSubstitutionTail`,
    pub(crate) fn next_template_substitution_tail(&mut self) -> Token {
        self.current.token.start = self.offset() - 1;
        let kind = self.read_template_literal(Kind::TemplateMiddle, Kind::TemplateTail);
        self.lookahead.clear();
        self.finish_next(kind)
    }

    /// Save the template if it is escaped
    fn save_template_string(
        &mut self,
        is_valid_escape_sequence: bool,
        has_escape: bool,
        s: &'a str,
    ) {
        if !has_escape {
            return;
        }
        self.escaped_templates
            .insert(self.current.token.start, is_valid_escape_sequence.then(|| s));
        self.current.token.escaped = true;
    }

    pub(crate) fn get_template_string(&self, token: Token) -> Option<&'a str> {
        if token.escaped {
            return self.escaped_templates[&token.start];
        }
        let raw = &self.source[token.start as usize..token.end as usize];
        Some(match token.kind {
            Kind::NoSubstitutionTemplate | Kind::TemplateTail => {
                &raw[1..raw.len() - 1] // omit surrounding quotes or leading "}" and trailing "`"
            }
            Kind::TemplateHead | Kind::TemplateMiddle => {
                &raw[1..raw.len() - 2] // omit leading "`" or "}" and trailing "${"
            }
            _ => raw,
        })
    }
}
