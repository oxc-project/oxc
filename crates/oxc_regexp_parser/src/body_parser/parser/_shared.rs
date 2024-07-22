use oxc_diagnostics::{OxcDiagnostic, Result};
use oxc_span::Atom as SpanAtom;

use crate::body_parser::unicode;

impl<'a> super::parse::PatternParser<'a> {
    pub(super) fn consume_fixed_hex_digits(&mut self, len: usize) -> Option<u32> {
        let checkpoint = self.reader.checkpoint();

        let mut value = 0;
        for _ in 0..len {
            let Some(hex) = self.reader.peek().and_then(unicode::map_hex_digit) else {
                self.reader.rewind(checkpoint);
                return None;
            };

            value = (16 * value) + hex;
            self.reader.advance();
        }

        Some(value)
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
    pub(super) fn consume_reg_exp_unicode_escape_sequence(&mut self) -> Result<Option<u32>> {
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

    fn consume_hex_digits(&mut self) -> Option<u32> {
        let checkpoint = self.reader.checkpoint();

        let mut value = 0;
        while let Some(hex) = self.reader.peek().and_then(unicode::map_hex_digit) {
            value = (16 * value) + hex;
            self.reader.advance();
        }

        if self.reader.checkpoint() != checkpoint {
            return Some(value);
        }

        None
    }

    // ```
    // GroupName[UnicodeMode] ::
    //   < RegExpIdentifierName[?UnicodeMode] >
    // ```
    // <https://tc39.es/ecma262/#prod-GroupName>
    pub(super) fn consume_group_name(&mut self) -> Result<Option<SpanAtom<'a>>> {
        if !self.reader.eat('<') {
            return Ok(None);
        }

        if let Some(group_name) = self.consume_reg_exp_idenfigier_name()? {
            if self.reader.eat('>') {
                return Ok(Some(group_name));
            }
        }

        Err(OxcDiagnostic::error("Invalid capture group name"))
    }

    // ```
    // RegExpIdentifierName[UnicodeMode] ::
    //   RegExpIdentifierStart[?UnicodeMode]
    //   RegExpIdentifierName[?UnicodeMode] RegExpIdentifierPart[?UnicodeMode]
    // ```
    // <https://tc39.es/ecma262/#prod-RegExpIdentifierName>
    fn consume_reg_exp_idenfigier_name(&mut self) -> Result<Option<SpanAtom<'a>>> {
        let span_start = self.reader.span_position();

        if self.consume_reg_exp_idenfigier_start()?.is_some() {
            while self.consume_reg_exp_idenfigier_part()?.is_some() {}

            let span_end = self.reader.span_position();
            return Ok(Some(SpanAtom::from(&self.source_text[span_start..span_end])));
        }

        Ok(None)
    }

    // ```
    // RegExpIdentifierStart[UnicodeMode] ::
    //   IdentifierStartChar
    //   \ RegExpUnicodeEscapeSequence[+UnicodeMode]
    //   [~UnicodeMode] UnicodeLeadSurrogate UnicodeTrailSurrogate
    // ```
    // <https://tc39.es/ecma262/#prod-RegExpIdentifierStart>
    fn consume_reg_exp_idenfigier_start(&mut self) -> Result<Option<u32>> {
        if let Some(cp) = self.reader.peek() {
            if unicode::is_identifier_start_char(cp) {
                self.reader.advance();
                return Ok(Some(cp));
            }
        }

        if self.reader.eat('\\') {
            if let Some(cp) = self.consume_reg_exp_unicode_escape_sequence()? {
                return Ok(Some(cp));
            }
        }

        if !self.state.is_unicode_mode() {
            if let Some(lead_surrogate) =
                self.reader.peek().filter(|&cp| unicode::is_lead_surrogate(cp))
            {
                if let Some(trail_surrogate) =
                    self.reader.peek2().filter(|&cp| unicode::is_trail_surrogate(cp))
                {
                    self.reader.advance();
                    self.reader.advance();

                    return Ok(Some(unicode::combine_surrogate_pair(
                        lead_surrogate,
                        trail_surrogate,
                    )));
                }
            }
        }

        Ok(None)
    }

    // ```
    // RegExpIdentifierPart[UnicodeMode] ::
    //   IdentifierPartChar
    //   \ RegExpUnicodeEscapeSequence[+UnicodeMode]
    //   [~UnicodeMode] UnicodeLeadSurrogate UnicodeTrailSurrogate
    // ```
    // <https://tc39.es/ecma262/#prod-RegExpIdentifierPart>
    fn consume_reg_exp_idenfigier_part(&mut self) -> Result<Option<u32>> {
        if let Some(cp) = self.reader.peek() {
            if unicode::is_identifier_part_char(cp) {
                self.reader.advance();
                return Ok(Some(cp));
            }
        }

        if self.reader.eat('\\') {
            if let Some(cp) = self.consume_reg_exp_unicode_escape_sequence()? {
                return Ok(Some(cp));
            }
        }

        if !self.state.is_unicode_mode() {
            if let Some(lead_surrogate) =
                self.reader.peek().filter(|&cp| unicode::is_lead_surrogate(cp))
            {
                if let Some(trail_surrogate) =
                    self.reader.peek2().filter(|&cp| unicode::is_trail_surrogate(cp))
                {
                    self.reader.advance();
                    self.reader.advance();

                    return Ok(Some(unicode::combine_surrogate_pair(
                        lead_surrogate,
                        trail_surrogate,
                    )));
                }
            }
        }

        Ok(None)
    }
}
