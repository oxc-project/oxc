use oxc_allocator::Box;
use oxc_diagnostics::{OxcDiagnostic, Result};

use crate::{ast, parser::body_parser::unicode};

impl<'a> super::parse::PatternParser<'a> {
    // ```
    // PatternCharacter ::
    //   SourceCharacter but not SyntaxCharacter
    // ```
    // <https://tc39.es/ecma262/#prod-PatternCharacter>
    pub(super) fn consume_pattern_character(&mut self) -> Option<ast::Atom<'a>> {
        let span_start = self.reader.span_position();

        let cp = self.reader.peek()?;
        if unicode::is_syntax_character(cp) {
            return None;
        }
        self.reader.advance();

        Some(ast::Atom::Character(Box::new_in(
            ast::Character {
                span: self.span_factory.create(span_start, self.reader.span_position()),
                value: cp,
            },
            self.allocator,
        )))
    }

    pub(super) fn consume_dot(&mut self) -> Option<ast::Atom<'a>> {
        let span_start = self.reader.span_position();
        if !self.reader.eat('.') {
            return None;
        }

        Some(ast::Atom::CharacterSet(Box::new_in(
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
    pub(super) fn consume_atom_escape(&mut self) -> Result<Option<ast::Atom<'a>>> {
        let span_start = self.reader.span_position();
        if !self.reader.eat('\\') {
            return Ok(None);
        }

        // `DecimalEscape`: \1 means Backreference
        if let Some(decimal) = self.consume_decimal_escape() {
            let span_end = self.reader.span_position();

            return Ok(Some(ast::Atom::Backreference(Box::new_in(
                ast::Backreference::NormalBackreference(Box::new_in(
                    ast::NormalBackreference {
                        span: self.span_factory.create(span_start, span_end),
                        r#ref: decimal,
                    },
                    self.allocator,
                )),
                self.allocator,
            ))));
        }

        if let Some((kind, negate)) = self.consume_character_class_escape() {
            return Ok(Some(ast::Atom::CharacterSet(Box::new_in(
                ast::CharacterSet::EscapeCharacterSet(Box::new_in(
                    ast::EscapeCharacterSet {
                        span: self.span_factory.create(span_start, self.reader.span_position()),
                        kind,
                        negate,
                    },
                    self.allocator,
                )),
                self.allocator,
            ))));
        }
        if self.state.is_unicode_mode() {
            if let Some(((name, value, negate), is_strings_related)) =
                self.consume_character_class_escape_unicode()?
            {
                let span = self.span_factory.create(span_start, self.reader.span_position());
                return Ok(Some(ast::Atom::CharacterSet(Box::new_in(
                    ast::CharacterSet::UnicodePropertyCharacterSet(Box::new_in(
                        if is_strings_related {
                            ast::UnicodePropertyCharacterSet::StringsUnicodePropertyCharacterSet(
                                Box::new_in(
                                    ast::StringsUnicodePropertyCharacterSet { span, key: name },
                                    self.allocator,
                                ),
                            )
                        } else {
                            ast::UnicodePropertyCharacterSet::CharacterUnicodePropertyCharacterSet(
                                Box::new_in(
                                    ast::CharacterUnicodePropertyCharacterSet {
                                        span,
                                        key: name,
                                        value,
                                        negate,
                                    },
                                    self.allocator,
                                ),
                            )
                        },
                        self.allocator,
                    )),
                    self.allocator,
                ))));
            }
        }

        if let Some(cp) = self.consume_character_escape()? {
            return Ok(Some(ast::Atom::Character(Box::new_in(
                ast::Character {
                    span: self.span_factory.create(span_start, self.reader.span_position()),
                    value: cp,
                },
                self.allocator,
            ))));
        }

        // `k<GroupName>`: \k<name> means Backreference
        if let Some(r#ref) = self.consume_k_group_name()? {
            return Ok(Some(ast::Atom::Backreference(Box::new_in(
                ast::Backreference::NamedBackreference(Box::new_in(
                    ast::NamedBackreference {
                        span: self.span_factory.create(span_start, self.reader.span_position()),
                        r#ref,
                    },
                    self.allocator,
                )),
                self.allocator,
            ))));
        }

        Err(OxcDiagnostic::error("Invalid escape"))
    }

    // ```
    // (?: Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
    // ```
    pub(super) fn consume_non_capturing_group(&mut self) -> Result<Option<ast::Atom<'a>>> {
        let span_start = self.reader.span_position();
        if self.reader.eat3('(', '?', ':') {
            let alternatives = self.consume_disjunction()?;

            if self.reader.eat(')') {
                return Ok(Some(ast::Atom::NonCapturingGroup(Box::new_in(
                    ast::NonCapturingGroup {
                        span: self.span_factory.create(span_start, self.reader.span_position()),
                        alternatives,
                    },
                    self.allocator,
                ))));
            }

            return Err(OxcDiagnostic::error("Unterminated group"));
        }

        Ok(None)
    }
}
