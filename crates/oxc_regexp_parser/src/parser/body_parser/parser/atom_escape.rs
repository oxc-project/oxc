use oxc_diagnostics::{OxcDiagnostic, Result};
use oxc_span::Atom as SpanAtom;

use crate::{
    ast,
    parser::body_parser::{unicode, unicode_property},
};

impl<'a> super::parse::PatternParser<'a> {
    // ```
    // DecimalEscape ::
    //   NonZeroDigit DecimalDigits[~Sep]opt [lookahead ∉ DecimalDigit]
    // ```
    // <https://tc39.es/ecma262/#prod-DecimalEscape>
    pub(super) fn consume_decimal_escape(&mut self) -> Option<usize> {
        if unicode::is_non_zero_digit(self.reader.peek()?) {
            let mut value = 0;

            while let Some(cp) = self.reader.peek() {
                if !unicode::is_decimal_digits(cp) {
                    break;
                }

                // `- '0' as u32`: convert code point to digit
                value = (10 * value) + (cp - '0' as u32) as usize;
                self.reader.advance();
            }

            return Some(value);
        }

        None
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
    pub(super) fn consume_character_class_escape(
        &mut self,
    ) -> Option<(ast::EscapeCharacterSetKind, bool)> {
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
    pub(super) fn consume_character_class_escape_unicode(
        &mut self,
    ) -> Result<Option<((SpanAtom<'a>, Option<SpanAtom<'a>>, bool), bool)>> {
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
    ) -> Result<Option<(SpanAtom<'a>, Option<SpanAtom<'a>>, bool)>> {
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

    fn consume_unicode_property_name(&mut self) -> Option<SpanAtom<'a>> {
        let span_start = self.reader.span_position();

        let checkpoint = self.reader.checkpoint();
        while unicode::is_unicode_property_name_character(self.reader.peek()?) {
            self.reader.advance();
        }

        if checkpoint == self.reader.checkpoint() {
            return None;
        }

        Some(SpanAtom::from(&self.source_text[span_start..self.reader.span_position()]))
    }

    fn consume_unicode_property_value(&mut self) -> Option<SpanAtom<'a>> {
        let span_start = self.reader.span_position();

        let checkpoint = self.reader.checkpoint();
        while unicode::is_unicode_property_value_character(self.reader.peek()?) {
            self.reader.advance();
        }

        if checkpoint == self.reader.checkpoint() {
            return None;
        }

        Some(SpanAtom::from(&self.source_text[span_start..self.reader.span_position()]))
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
    pub(super) fn consume_character_escape(&mut self) -> Result<Option<u32>> {
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

    pub(super) fn consume_k_group_name(&mut self) -> Result<Option<SpanAtom<'a>>> {
        if self.reader.eat('k') {
            if let Some(group_name) = self.consume_group_name()? {
                // TODO: Implement
                // this._backreferenceNames.add(groupName);

                return Ok(Some(group_name));
            }

            return Err(OxcDiagnostic::error("Invalid named reference"));
        }

        Ok(None)
    }
}
