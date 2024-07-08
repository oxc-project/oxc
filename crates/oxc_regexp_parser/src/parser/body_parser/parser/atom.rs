use oxc_allocator::Box;
use oxc_diagnostics::{OxcDiagnostic, Result};
use oxc_span::Atom;

use crate::{
    ast,
    parser::body_parser::{unicode, unicode_property},
};

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
    pub(super) fn consume_reverse_solidus_atom_escape(&mut self) -> Result<Option<ast::Atom<'a>>> {
        let span_start = self.reader.span_position();
        if !self.reader.eat('\\') {
            return Ok(None);
        }

        // `DecimalEscape`: \1 means Backreference
        if self.consume_decimal_escape().is_some() {
            let span_end = self.reader.span_position();

            return Ok(Some(ast::Atom::Backreference(Box::new_in(
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

        // TODO: Implement
        // if let Some(_) = self.consume_k_group_name() {}

        Err(OxcDiagnostic::error("Invalid escape"))
    }

    // ```
    // CharacterClassEscape[UnicodeMode] ::
    //   d
    //   D
    //   s
    //   S
    //   w
    //   W
    //   [+UnicodeMode] p{ UnicodePropertyValueExpression }
    //   [+UnicodeMode] P{ UnicodePropertyValueExpression }
    // ```
    // <https://tc39.es/ecma262/#prod-CharacterClassEscape>
    /// Returns: `(kind, negate)`
    fn consume_character_class_escape(&mut self) -> Option<(ast::EscapeCharacterSetKind, bool)> {
        // NOTE: `mayContainStrings`?
        if self.reader.eat('d') {
            return Some((ast::EscapeCharacterSetKind::Digit, false));
        }
        if self.reader.eat('D') {
            return Some((ast::EscapeCharacterSetKind::Digit, true));
        }
        if self.reader.eat('s') {
            return Some((ast::EscapeCharacterSetKind::Space, false));
        }
        if self.reader.eat('S') {
            return Some((ast::EscapeCharacterSetKind::Space, true));
        }
        if self.reader.eat('w') {
            return Some((ast::EscapeCharacterSetKind::Word, false));
        }
        if self.reader.eat('W') {
            return Some((ast::EscapeCharacterSetKind::Word, true));
        }

        None
    }
    /// Returns: `((name, value, is_strings_related_unicode_property), negate)`
    #[allow(clippy::type_complexity)]
    fn consume_character_class_escape_unicode(
        &mut self,
    ) -> Result<Option<((Atom<'a>, Option<Atom<'a>>, bool), bool)>> {
        let negate = if self.reader.eat('p') {
            Some(false)
        } else if self.reader.eat('P') {
            Some(true)
        } else {
            None
        };

        if let Some(negate) = negate {
            if self.reader.eat('{') {
                if let Some((name, value, is_strings_related)) =
                    self.consume_unicode_property_value_expression()?
                {
                    if negate && is_strings_related {
                        return Err(OxcDiagnostic::error("Invalid property name"));
                    }
                    if self.reader.eat('}') {
                        // NOTE: `mayContainStrings`?
                        return Ok(Some(((name, value, negate), is_strings_related)));
                    }
                }
            }

            return Err(OxcDiagnostic::error("Invalid property name"));
        }

        Ok(None)
    }

    // ```
    // UnicodePropertyValueExpression ::
    //   UnicodePropertyName = UnicodePropertyValue
    //   LoneUnicodePropertyNameOrValue
    // ```
    // <https://tc39.es/ecma262/#prod-UnicodePropertyValueExpression>
    /// Returns: `(name, value, is_strings_related_unicode_property)`
    fn consume_unicode_property_value_expression(
        &mut self,
    ) -> Result<Option<(Atom<'a>, Option<Atom<'a>>, bool)>> {
        let checkpoint = self.reader.checkpoint();

        // UnicodePropertyName=UnicodePropertyValue
        if let Some(name) = self.consume_unicode_property_name() {
            if self.reader.eat('=') {
                if let Some(value) = self.consume_unicode_property_value() {
                    if unicode_property::is_valid_unicode_property(&name, &value) {
                        return Ok(Some((name, Some(value), false)));
                    }

                    return Err(OxcDiagnostic::error("Invalid property name"));
                }
            }
        }
        self.reader.rewind(checkpoint);

        // LoneUnicodePropertyNameOrValue
        if let Some(name_or_value) = self.consume_unicode_property_value() {
            if unicode_property::is_valid_unicode_property("General_Category", &name_or_value) {
                return Ok(Some(("General_Category".into(), Some(name_or_value), false)));
            }

            if unicode_property::is_valid_lone_unicode_property(&name_or_value) {
                return Ok(Some((name_or_value, None, false)));
            }

            if unicode_property::is_valid_lone_unicode_property_of_strings(&name_or_value) {
                // Early errors:
                // It is a Syntax Error
                // - if the enclosing Pattern does not have a [UnicodeSetsMode] parameter
                // - and the source text matched by LoneUnicodePropertyNameOrValue is a binary property of strings
                //   - listed in the “Property name” column of Table 68.
                if !self.state.is_unicode_sets_mode() {
                    return Err(OxcDiagnostic::error("Syntax Error"));
                }

                return Ok(Some((name_or_value, None, true)));
            }

            return Err(OxcDiagnostic::error("Invalid property name"));
        }

        Ok(None)
    }

    fn consume_unicode_property_name(&mut self) -> Option<Atom<'a>> {
        let span_start = self.reader.span_position();

        let checkpoint = self.reader.checkpoint();
        while unicode::is_unicode_property_name_character(self.reader.peek()?) {
            self.reader.advance();
        }

        if checkpoint == self.reader.checkpoint() {
            return None;
        }

        Some(Atom::from(&self.source_text[span_start..self.reader.span_position()]))
    }

    fn consume_unicode_property_value(&mut self) -> Option<Atom<'a>> {
        let span_start = self.reader.span_position();

        let checkpoint = self.reader.checkpoint();
        while unicode::is_unicode_property_value_character(self.reader.peek()?) {
            self.reader.advance();
        }

        if checkpoint == self.reader.checkpoint() {
            return None;
        }

        Some(Atom::from(&self.source_text[span_start..self.reader.span_position()]))
    }

    // ```
    // CharacterEscape[UnicodeMode] ::
    //   ControlEscape
    //   c AsciiLetter
    //   0 [lookahead ∉ DecimalDigit]
    //   HexEscapeSequence
    //   RegExpUnicodeEscapeSequence[?UnicodeMode]
    //   IdentityEscape[?UnicodeMode]
    // ```
    // <https://tc39.es/ecma262/#prod-CharacterEscape>
    fn consume_character_escape(&mut self) -> Result<Option<u32>> {
        // e.g. \n
        if let Some(control_escape) = self.reader.peek().and_then(unicode::map_control_escape) {
            self.reader.advance();
            return Ok(Some(control_escape));
        }

        // e.g. \cM
        let checkpoint = self.reader.checkpoint();
        if self.reader.eat('c') {
            if let Some(c_ascii_letter) = self.reader.peek().and_then(unicode::map_c_ascii_letter) {
                self.reader.advance();
                return Ok(Some(c_ascii_letter));
            }
            self.reader.rewind(checkpoint);
        }

        // e.g. \0
        if self.reader.peek().map_or(false, |cp| cp == '0' as u32)
            && self.reader.peek2().map_or(true, |cp| !unicode::is_decimal_digits(cp))
        {
            self.reader.advance();
            return Ok(Some(0x00));
        }

        // e.g. \x41
        if self.reader.eat('x') {
            if let Some(hex_digits) = self.consume_fixed_hex_digits(2) {
                return Ok(Some(hex_digits));
            }
            return Err(OxcDiagnostic::error("Invalid escape"));
        }

        // e.g. \u{1f600}
        if let Some(reg_exp_unicode_escape_sequence) =
            self.consume_reg_exp_unicode_escape_sequence()?
        {
            return Ok(Some(reg_exp_unicode_escape_sequence));
        }

        // e.g. \.
        if let Some(identity_escape) = self.consume_identity_escape() {
            return Ok(Some(identity_escape));
        }

        Ok(None)
    }

    // ```
    // RegExpUnicodeEscapeSequence[UnicodeMode] ::
    //   [+UnicodeMode] u HexLeadSurrogate \u HexTrailSurrogate
    //   [+UnicodeMode] u HexLeadSurrogate
    //   [+UnicodeMode] u HexTrailSurrogate
    //   [+UnicodeMode] u HexNonSurrogate
    //   [~UnicodeMode] u Hex4Digits
    //   [+UnicodeMode] u{ CodePoint }
    // ```
    // <https://tc39.es/ecma262/#prod-RegExpUnicodeEscapeSequence>
    fn consume_reg_exp_unicode_escape_sequence(&mut self) -> Result<Option<u32>> {
        if !self.reader.eat('u') {
            return Ok(None);
        }

        if self.state.is_unicode_mode() {
            let checkpoint = self.reader.checkpoint();

            // HexLeadSurrogate + HexTrailSurrogate
            if let Some(lead_surrogate) =
                self.consume_fixed_hex_digits(4).filter(|&cp| unicode::is_lead_surrogate(cp))
            {
                if self.reader.eat2('\\', 'u') {
                    if let Some(trail_surrogate) = self
                        .consume_fixed_hex_digits(4)
                        .filter(|&cp| unicode::is_trail_surrogate(cp))
                    {
                        return Ok(Some(unicode::combine_surrogate_pair(
                            lead_surrogate,
                            trail_surrogate,
                        )));
                    }
                }
            }
            self.reader.rewind(checkpoint);

            // NOTE: `regexpp` seems not to support these 2 cases, why...?

            // HexLeadSurrogate
            if let Some(lead_surrogate) =
                self.consume_fixed_hex_digits(4).filter(|&cp| unicode::is_lead_surrogate(cp))
            {
                return Ok(Some(lead_surrogate));
            }
            self.reader.rewind(checkpoint);

            // HexTrailSurrogate
            if let Some(trail_surrogate) =
                self.consume_fixed_hex_digits(4).filter(|&cp| unicode::is_trail_surrogate(cp))
            {
                return Ok(Some(trail_surrogate));
            }
            self.reader.rewind(checkpoint);
        }

        // HexNonSurrogate and Hex4Digits are the same
        if let Some(hex_digits) = self.consume_fixed_hex_digits(4) {
            return Ok(Some(hex_digits));
        }

        // {CodePoint}
        if self.state.is_unicode_mode() {
            let checkpoint = self.reader.checkpoint();

            if self.reader.eat('{') {
                if let Some(hex_digits) =
                    self.consume_hex_digits().filter(|&cp| unicode::is_valid_unicode(cp))
                {
                    if self.reader.eat('}') {
                        return Ok(Some(hex_digits));
                    }
                }
            }
            self.reader.rewind(checkpoint);
        }

        Err(OxcDiagnostic::error("Invalid unicode escape"))
    }

    // ```
    // IdentityEscape[UnicodeMode] ::
    //   [+UnicodeMode] SyntaxCharacter
    //   [+UnicodeMode] /
    //   [~UnicodeMode] SourceCharacter but not UnicodeIDContinue
    // ```
    // <https://tc39.es/ecma262/#prod-IdentityEscape>
    fn consume_identity_escape(&mut self) -> Option<u32> {
        let cp = self.reader.peek()?;

        if self.state.is_unicode_mode() {
            if unicode::is_syntax_character(cp) || cp == '/' as u32 {
                self.reader.advance();
                return Some(cp);
            }

            return None;
        }

        if !unicode::is_id_continue(cp) {
            self.reader.advance();
            return Some(cp);
        }

        None
    }
}
