use oxc_allocator::{Box, Vec};
use oxc_diagnostics::{OxcDiagnostic, Result};

use crate::ast;

impl<'a> super::parse::PatternParser<'a> {
    // ```
    // NonemptyClassRanges[UnicodeMode] ::
    //   ClassAtom[?UnicodeMode]
    //   ClassAtom[?UnicodeMode] NonemptyClassRangesNoDash[?UnicodeMode]
    //   ClassAtom[?UnicodeMode] - ClassAtom[?UnicodeMode] ClassContents[?UnicodeMode, ~UnicodeSetsMode]
    // ```
    // <https://tc39.es/ecma262/#prod-NonemptyClassRanges>
    pub(super) fn consume_nonempty_class_ranges(
        &mut self,
    ) -> Result<Vec<'a, ast::ClassRangesCharacterClassElement<'a>>> {
        let mut contents = Vec::new_in(self.allocator);

        // NOTE: This implementation may not reflect the spec correctly.
        // `ClassAtom`(= `NonemptyClassRanges`) and `NonemptyClassRangesNoDash` should be distinguished?
        // But `regexpp` also handles them in the same way.
        loop {
            let range_span_start = self.reader.span_position();

            let Some(first_class_atom) = self.consume_class_atom()? else {
                // If there is no more characters, break the loop to return `[empty]`
                break;
            };

            let span_start = self.reader.span_position();
            if self.reader.eat('-') {
                let Some(second_class_atom) = self.consume_class_atom()? else {
                    contents.push(ast::ClassRangesCharacterClassElement::Character(Box::new_in(
                        ast::Character {
                            span: self.span_factory.create(span_start, self.reader.span_position()),
                            value: '-' as u32,
                        },
                        self.allocator,
                    )));
                    // If `-` found but there is no more characters, push `-` as a character and break
                    break;
                };

                match (first_class_atom, second_class_atom) {
                    (
                        ast::ClassRangesCharacterClassElement::Character(min_character),
                        ast::ClassRangesCharacterClassElement::Character(max_character),
                    ) => {
                        contents.push(ast::ClassRangesCharacterClassElement::CharacterClassRange(
                            Box::new_in(
                                ast::CharacterClassRange {
                                    span: self
                                        .span_factory
                                        .create(range_span_start, self.reader.span_position()),
                                    min: min_character.unbox(),
                                    max: max_character.unbox(),
                                },
                                self.allocator,
                            ),
                        ));
                        // If `-` and 2 characters found, push the range and continue
                        continue;
                    }
                    _ => {
                        return Err(OxcDiagnostic::error("Invalid character class range"));
                    }
                }
            }
        }

        Ok(contents)
    }

    // ```
    // ClassAtom[UnicodeMode] ::
    //   -
    //   ClassAtomNoDash[?UnicodeMode]
    // ```
    // <https://tc39.es/ecma262/#prod-ClassAtom>
    fn consume_class_atom(&mut self) -> Result<Option<ast::ClassRangesCharacterClassElement<'a>>> {
        let Some(cp) = self.reader.peek() else {
            return Ok(None);
        };

        let span_start = self.reader.span_position();
        if cp == '-' as u32 {
            self.reader.advance();

            return Ok(Some(ast::ClassRangesCharacterClassElement::Character(Box::new_in(
                ast::Character {
                    span: self.span_factory.create(span_start, self.reader.span_position()),
                    value: cp,
                },
                self.allocator,
            ))));
        }

        if let Some(class_atom_no_dash) = self.consume_class_atom_no_dash()? {
            return Ok(Some(class_atom_no_dash));
        }

        Ok(None)
    }

    // ```
    // ClassAtomNoDash[UnicodeMode] ::
    //   SourceCharacter but not one of \ or ] or -
    //   \ ClassEscape[?UnicodeMode]
    // ```
    // <https://tc39.es/ecma262/#prod-ClassAtomNoDash>
    fn consume_class_atom_no_dash(
        &mut self,
    ) -> Result<Option<ast::ClassRangesCharacterClassElement<'a>>> {
        let Some(cp) = self.reader.peek() else {
            return Ok(None);
        };

        let span_start = self.reader.span_position();
        if cp != '\\' as u32 && cp != ']' as u32 && cp != '-' as u32 {
            self.reader.advance();

            return Ok(Some(ast::ClassRangesCharacterClassElement::Character(Box::new_in(
                ast::Character {
                    span: self.span_factory.create(span_start, self.reader.span_position()),
                    value: cp,
                },
                self.allocator,
            ))));
        }

        let span_start = self.reader.span_position();
        if self.reader.eat('\\') {
            if let Some(class_escape) = self.consume_class_escape(span_start)? {
                return Ok(Some(class_escape));
            }

            return Err(OxcDiagnostic::error("Invalid escape"));
        }

        Ok(None)
    }

    // ```
    // ClassEscape[UnicodeMode] ::
    //   b
    //   [+UnicodeMode] -
    //   CharacterClassEscape[?UnicodeMode]
    //   CharacterEscape[?UnicodeMode]
    // ```
    // <https://tc39.es/ecma262/#prod-ClassEscape>
    fn consume_class_escape(
        &mut self,
        span_start: usize,
    ) -> Result<Option<ast::ClassRangesCharacterClassElement<'a>>> {
        let Some(cp) = self.reader.peek() else {
            return Ok(None);
        };

        if cp == 'b' as u32 {
            self.reader.advance();

            return Ok(Some(ast::ClassRangesCharacterClassElement::Character(Box::new_in(
                ast::Character {
                    span: self.span_factory.create(span_start, self.reader.span_position()),
                    value: cp,
                },
                self.allocator,
            ))));
        }

        if self.state.is_unicode_mode() && cp == '-' as u32 {
            self.reader.advance();

            return Ok(Some(ast::ClassRangesCharacterClassElement::Character(Box::new_in(
                ast::Character {
                    span: self.span_factory.create(span_start, self.reader.span_position()),
                    value: cp,
                },
                self.allocator,
            ))));
        }

        if let Some(escape_character_set) = self.consume_character_class_escape(span_start) {
            return Ok(Some(ast::ClassRangesCharacterClassElement::EscapeCharacterSet(
                Box::new_in(escape_character_set, self.allocator),
            )));
        }
        if self.state.is_unicode_mode() {
            if let Some(unicode_property_character_set) =
                self.consume_character_class_escape_unicode(span_start)?
            {
                match unicode_property_character_set {
                    ast::UnicodePropertyCharacterSet::CharacterUnicodePropertyCharacterSet(
                        character_set,
                    ) => {
                        return Ok(Some(
                            ast::ClassRangesCharacterClassElement::CharacterUnicodePropertyCharacterSet(
                            character_set
                        ),
                ));
                    }
                    // This is `unicode_sets_mode` only pattern.
                    // If `unicode_sets_mode: true`, this function should not be called at all.
                    ast::UnicodePropertyCharacterSet::StringsUnicodePropertyCharacterSet(_) => {
                        return Err(OxcDiagnostic::error(
                            "Unexpected StringsUnicodePropertyCharacterSet",
                        ));
                    }
                }
            }
        }

        if let Some(character) = self.consume_character_escape(span_start)? {
            return Ok(Some(ast::ClassRangesCharacterClassElement::Character(Box::new_in(
                character,
                self.allocator,
            ))));
        }

        Ok(None)
    }
}
