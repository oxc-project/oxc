use oxc_allocator::Box;
use oxc_diagnostics::{OxcDiagnostic, Result};
use oxc_span::Atom;

use crate::{ast, parser::body_parser::unicode};

impl<'a> super::parse::PatternParser<'a> {
    // ```
    // PatternCharacter ::
    //   SourceCharacter but not SyntaxCharacter
    // ```
    // <https://tc39.es/ecma262/#prod-PatternCharacter>
    pub(super) fn consume_pattern_character(&mut self) -> Option<ast::QuantifiableElement<'a>> {
        let cp = self.reader.peek()?;
        if unicode::is_syntax_character(cp) {
            return None;
        }

        let span_start = self.reader.span_position();
        self.reader.advance();

        Some(ast::QuantifiableElement::Character(Box::new_in(
            ast::Character {
                span: self.span_factory.create(span_start, self.reader.span_position()),
                value: cp,
            },
            self.allocator,
        )))
    }

    pub(super) fn consume_dot(&mut self) -> Option<ast::QuantifiableElement<'a>> {
        let span_start = self.reader.span_position();
        if !self.reader.eat('.') {
            return None;
        }

        Some(ast::QuantifiableElement::CharacterSet(Box::new_in(
            ast::CharacterSet::AnyCharacterSet(Box::new_in(
                ast::AnyCharacterSet {
                    span: self.span_factory.create(span_start, self.reader.span_position()),
                },
                self.allocator,
            )),
            self.allocator,
        )))
    }

    // ```
    // AtomEscape[UnicodeMode, NamedCaptureGroups] ::
    //   DecimalEscape
    //   CharacterClassEscape[?UnicodeMode]
    //   CharacterEscape[?UnicodeMode]
    //   [+NamedCaptureGroups] k GroupName[?UnicodeMode]
    // ```
    // <https://tc39.es/ecma262/#prod-AtomEscape>
    pub(super) fn consume_reverse_solidus_atom_escape(
        &mut self,
    ) -> Result<Option<ast::QuantifiableElement<'a>>> {
        let span_start = self.reader.span_position();
        if !self.reader.eat('\\') {
            return Ok(None);
        }

        // `DecimalEscape`: `\1` means Backreference
        if self.consume_decimal_escape().is_some() {
            let span_end = self.reader.span_position();

            return Ok(Some(ast::QuantifiableElement::Backreference(Box::new_in(
                ast::Backreference::TemporaryBackreference(Box::new_in(
                    ast::TemporaryBackreference {
                        span: self.span_factory.create(span_start, span_end),
                        r#ref: Atom::from(&self.source_text[span_start..span_end]),
                    },
                    self.allocator,
                )),
                self.allocator,
            ))));
        }

        // TODO: Implement
        // if let Some(_) = self.consume_character_class_escape() {}
        // if let Some(_) = self.consume_character_escape() {}
        // if let Some(_) = self.consume_k_group_name() {}

        Err(OxcDiagnostic::error("Invalid escape"))
    }
}
