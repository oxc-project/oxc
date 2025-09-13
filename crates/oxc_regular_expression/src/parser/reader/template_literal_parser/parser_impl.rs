use oxc_diagnostics::Result;
use oxc_span::Span;

use crate::parser::reader::{
    Options,
    ast::CodePoint,
    characters::{
        CR, LF, LS, PS, is_line_terminator, is_non_escape_character, is_single_escape_character,
    },
    template_literal_parser::{ast, diagnostics},
};

// Internal representation of escape sequence resolved unit in a string literal.
type OffsetsAndCp = ((u32, u32), u32);

pub struct Parser {
    // NOTE: In JavaScript, template literals are UTF-16 encoded,
    // so we need to be aware of surrogate pairs, while collecting offsets for `Span`.
    // Rather than using `encode_utf16()`, split surrogate pairs manually is easier
    // to detect the start and end of each code point.
    chars: Vec<char>,
    index: usize,
    offset: u32,
    options: Options,
}

impl Parser {
    fn handle_code_point(
        body: &mut Vec<CodePoint>,
        (offsets, cp): OffsetsAndCp,
        span_offset: u32,
        combine_surrogate_pair: bool,
    ) {
        let span = Span::new(span_offset + offsets.0, span_offset + offsets.1);

        if combine_surrogate_pair || (0..=0xffff).contains(&cp) {
            // If the code point is in the BMP or if forced, just push it
            body.push(CodePoint { span, value: cp });
        } else {
            // Otherwise, split the code point into a surrogate pair, sharing the same span
            let (lead, trail) =
                (0xd800 + ((cp - 0x10000) >> 10), 0xdc00 + ((cp - 0x10000) & 0x3ff));
            body.push(CodePoint { span, value: lead });
            body.push(CodePoint { span, value: trail });
        }
    }

    // ---

    pub fn new(source_text: &str, options: Options) -> Self {
        Self { chars: source_text.chars().collect::<Vec<_>>(), index: 0, offset: 0, options }
    }

    // We do not parse TemplateHead, TemplateTail, TemplateSubstitutionTail, TemplateMiddle
    // ```
    // Template ::
    //   NoSubstitutionTemplate
    //   TemplateHead
    //
    // NoSubstitutionTemplate ::
    //   ` TemplateCharacters[opt] `
    //
    // ```
    pub fn parse(mut self) -> Result<ast::TemplateLiteral> {
        if !self.eat('`') {
            return Err(diagnostics::invalid_input(Span::empty(self.options.span_offset)));
        }

        let body = self.parse_template()?;

        if self.eat('`') {
            if self.peek().is_some() {
                return Err(diagnostics::invalid_input(Span::empty(
                    self.options.span_offset + self.offset(),
                )));
            }

            let span = Span::sized(self.options.span_offset, self.offset());
            return Ok(ast::TemplateLiteral { span, body });
        }

        Err(diagnostics::invalid_input(Span::empty(self.options.span_offset + self.offset())))
    }

    // ---

    // ```
    // Template ::
    //   NoSubstitutionTemplate
    //   TemplateHead
    // ```
    fn parse_template(&mut self) -> Result<Vec<CodePoint>> {
        // ToDo: diagnostic when TemplateHead is found
        self.parse_template_characters()
    }

    // ```
    //  TemplateCharacters ::
    //   TemplateCharacter TemplateCharacters[opt]
    // ```
    fn parse_template_characters(&mut self) -> Result<Vec<CodePoint>> {
        let mut body = vec![];
        while let Some(code_point) = self.parse_template_character()? {
            Parser::handle_code_point(
                &mut body,
                code_point,
                self.options.span_offset,
                self.options.combine_surrogate_pair,
            );
        }
        Ok(body)
    }

    // ```
    //  TemplateCharacter ::
    //    $ [lookahead ≠ {]
    //    \ TemplateEscapeSequence
    //    \ NotEscapeSequence
    //    LineContinuation
    //    LineTerminatorSequence
    //    SourceCharacter but not one of ` or \ or $ or LineTerminator
    // ```
    fn parse_template_character(&mut self) -> Result<Option<OffsetsAndCp>> {
        let offset_start = self.offset();

        // $ [lookahead ≠ {]
        if self.peek() == Some('$') && self.peek2() != Some('{') {
            self.advance();
            return Ok(Some(((offset_start, self.offset()), '$' as u32)));
        }

        if self.eat('\\') {
            if let Some(cp) = self.parse_template_escape_sequence(offset_start)? {
                return Ok(Some(((offset_start, self.offset()), cp)));
            }
            if let Some(cp) = self.parse_not_escape_sequence()? {
                return Ok(Some(((offset_start, self.offset()), cp)));
            }
        }
        if let Some(cp) = self.parse_line_continuation() {
            return Ok(Some(((offset_start, self.offset()), cp)));
        }
        if let Some(cp) = self.parse_line_terminator_sequence() {
            return Ok(Some(((offset_start, self.offset()), cp)));
        }

        if let Some(ch) = self.peek() {
            if ch == '$' {
                //  Skip it too, but we do not support `TemplateHead ` or `TemplateMiddle`.
                return Err(diagnostics::template_substitution(Span::new(
                    self.options.span_offset + offset_start,
                    self.options.span_offset + self.offset(),
                )));
            }

            if ch == '\\' || ch == '`' || is_line_terminator(ch) {
                return Ok(None);
            }

            self.advance();

            return Ok(Some(((offset_start, self.offset()), ch as u32)));
        }

        Ok(None)
    }

    // ```
    //  TemplateEscapeSequence ::
    //     CharacterEscapeSequence
    //     0 [lookahead ∉ DecimalDigit]
    //     HexEscapeSequence
    //     UnicodeEscapeSequence
    // ```
    fn parse_template_escape_sequence(&mut self, offset_start: u32) -> Result<Option<u32>> {
        if let Some(cp) = self.parse_character_escape_sequence() {
            return Ok(Some(cp));
        }
        if self.peek() == Some('0') && self.peek2().is_none_or(|ch| !ch.is_ascii_digit()) {
            self.advance();
            return Ok(Some(0x00));
        }
        if let Some(cp) = self.parse_hex_escape_sequence()? {
            return Ok(Some(cp));
        }
        if let Some(cp) = self.parse_unicode_escape_sequence(offset_start)? {
            return Ok(Some(cp));
        }

        Ok(None)
    }

    // ```
    // CharacterEscapeSequence ::
    //   SingleEscapeCharacter
    //   NonEscapeCharacter
    // ```
    fn parse_character_escape_sequence(&mut self) -> Option<u32> {
        if let Some(ch) = self.peek().filter(|&ch| is_single_escape_character(ch)) {
            self.advance();
            return Some(ch as u32);
        }
        if let Some(ch) = self.peek().filter(|&ch| is_non_escape_character(ch)) {
            self.advance();
            return Some(ch as u32);
        }

        None
    }

    // ```
    // NotEscapeSequence ::
    //   0 DecimalDigit
    //   DecimalDigit but not 0
    //   x [lookahead ∉ HexDigit]
    //   x HexDigit [lookahead ∉ HexDigit]
    //   u [lookahead ∉ HexDigit] [lookahead ≠ {]
    //   u HexDigit [lookahead ∉ HexDigit]
    //   u HexDigit HexDigit [lookahead ∉ HexDigit]
    //   u HexDigit HexDigit HexDigit [lookahead ∉ HexDigit]
    //   u { [lookahead ∉ HexDigit]
    //   u { NotCodePoint [lookahead ∉ HexDigit]
    //   u { CodePoint [lookahead ∉ HexDigit] [lookahead ≠ }]
    // ```
    fn parse_not_escape_sequence(&mut self) -> Result<Option<u32>> {
        let checkpoint = self.checkpoint();

        // 0 DecimalDigit
        if self.eat('0') {
            if let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    self.advance();
                    return Ok(Some(ch as u32));
                }
            }
            self.rewind(checkpoint);
        }

        // DecimalDigit but not 0
        if let Some(ch) = self.peek() {
            if ch.is_ascii_digit() && ch != '0' {
                self.advance();
                return Ok(Some(ch as u32));
            }
        }

        // x [lookahead ∉ HexDigit] or x HexDigit [lookahead ∉ HexDigit]
        if self.eat('x') {
            let offset_start = self.offset();
            match self.consume_hex_digits(offset_start) {
                Ok(Some(_)) => {
                    self.rewind(checkpoint);
                    return Ok(None);
                }
                Ok(None) => {
                    return Ok(Some('x' as u32));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        // u [lookahead ∉ HexDigit] [lookahead ≠ {]
        // u HexDigit [lookahead ∉ HexDigit]
        // u HexDigit HexDigit [lookahead ∉ HexDigit]
        // u HexDigit HexDigit HexDigit [lookahead ∉ HexDigit]
        // u { [lookahead ∉ HexDigit]
        // u { NotCodePoint [lookahead ∉ HexDigit]
        // u { CodePoint [lookahead ∉ HexDigit] [lookahead ≠ }]
        if self.eat('u') {
            let offset_start = self.offset();
            if self.eat('{') {
                match self.consume_hex_digits(offset_start) {
                    Ok(Some(_)) => {
                        if !self.eat('}') {
                            return Ok(Some('u' as u32));
                        }
                        self.rewind(checkpoint);
                        return Ok(None);
                    }
                    Ok(None) => {
                        return Ok(Some('u' as u32));
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            let mut hex_count = 0;
            for _ in 0..4 {
                if let Some(ch) = self.peek() {
                    if ch.is_ascii_hexdigit() {
                        self.advance();
                        hex_count += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            if hex_count == 0 || hex_count < 4 {
                return Ok(Some('u' as u32));
            }
            if let Some(ch) = self.peek() {
                if ch.is_ascii_hexdigit() {
                    self.rewind(checkpoint);
                    return Ok(None);
                }
            }
            self.rewind(checkpoint);
            return Ok(None);
        }

        Ok(None)
    }

    // ```
    // HexEscapeSequence ::
    //   x HexDigit HexDigit
    // ```
    fn parse_hex_escape_sequence(&mut self) -> Result<Option<u32>> {
        if self.eat('x') {
            let first = self.consume_hex_digit();
            let second = self.consume_hex_digit();
            if let (Some(first), Some(second)) = (first, second) {
                return Ok(Some(first * 16 + second));
            }

            // Invalid hex escape: \x not followed by two hex digits
            return Err(diagnostics::invalid_hex_escape(Span::new(
                self.options.span_offset + self.offset(),
                self.options.span_offset + self.offset(),
            )));
        }

        Ok(None)
    }

    // ```
    // UnicodeEscapeSequence ::
    //   u Hex4Digits
    //   u{ CodePoint }
    // ```
    fn parse_unicode_escape_sequence(&mut self, offset_start: u32) -> Result<Option<u32>> {
        let checkpoint = self.checkpoint();

        if self.eat('u') {
            if let Some(cp) = self.consume_hex4_digits() {
                return Ok(Some(cp));
            } else if self.peek() != Some('{') {
                // If not followed by 4 hex digits or a code point escape, error
                return Err(diagnostics::invalid_unicode_escape(Span::new(
                    self.options.span_offset + offset_start,
                    self.options.span_offset + self.offset(),
                )));
            }
            self.rewind(checkpoint);
        }

        if self.eat('u') {
            if self.eat('{') {
                // Try to parse hex digits, error if not valid
                match self.consume_hex_digits(offset_start) {
                    Ok(Some(hex_digits)) if hex_digits <= 0x10_ffff => {
                        if self.eat('}') {
                            return Ok(Some(hex_digits));
                        }

                        // Missing closing '}'
                        return Err(diagnostics::invalid_unicode_escape(Span::new(
                            self.options.span_offset + offset_start,
                            self.options.span_offset + self.offset(),
                        )));
                    }
                    Ok(_) => {
                        // No valid hex digits or out of range
                        return Err(diagnostics::invalid_unicode_escape(Span::new(
                            self.options.span_offset + offset_start,
                            self.options.span_offset + self.offset(),
                        )));
                    }
                    Err(e) => return Err(e),
                }
            }
            self.rewind(checkpoint);
        }

        Ok(None)
    }

    // ```
    // LineContinuation ::
    //   \ LineTerminatorSequence
    //
    // ```
    fn parse_line_continuation(&mut self) -> Option<u32> {
        let checkpoint = self.checkpoint();

        if self.eat('\\') {
            if let Some(terminator) = self.parse_line_terminator_sequence() {
                return Some(terminator);
            }
        }

        self.rewind(checkpoint);
        None
    }

    // ```
    // LineTerminatorSequence ::
    //   <LF>
    //   <CR> [lookahead ≠ <LF>]
    //   <LS>
    //   <PS>
    //   <CR> <LF>
    // ```
    fn parse_line_terminator_sequence(&mut self) -> Option<u32> {
        let checkpoint = self.checkpoint();

        if self.peek() == Some(LF) {
            self.advance();
            return Some(LF as u32);
        }
        if self.peek() == Some(CR) && self.peek2() != Some(LF) {
            self.advance();
            return Some(CR as u32);
        }
        if self.peek() == Some(LS) {
            self.advance();
            return Some(LS as u32);
        }
        if self.peek() == Some(PS) {
            self.advance();
            return Some(PS as u32);
        }
        // NOTE: CR+LF can not represent as a single code point.
        // I don't know the best way to handle this.
        // To distinguish this from CR and LF, structural change is needed...
        if self.peek() == Some(CR) && self.peek2() == Some(LF) {
            self.advance();
            self.advance();
            return Some(LF as u32);
        }

        self.rewind(checkpoint);
        None
    }

    // ---

    fn consume_hex_digit(&mut self) -> Option<u32> {
        if let Some(ch) = self.peek().filter(char::is_ascii_hexdigit) {
            self.advance();
            return ch.to_digit(16);
        }

        None
    }

    // ```
    // Hex4Digits ::
    //   HexDigit HexDigit HexDigit HexDigit
    // ```
    fn consume_hex4_digits(&mut self) -> Option<u32> {
        let checkpoint = self.checkpoint();

        let mut value = 0;
        for _ in 0..4 {
            let Some(hex) =
                self.peek().filter(char::is_ascii_hexdigit).and_then(|ch| ch.to_digit(16))
            else {
                self.rewind(checkpoint);
                return None;
            };

            value = (16 * value) + hex;
            self.advance();
        }

        Some(value)
    }

    fn consume_hex_digits(&mut self, offset_start: u32) -> Result<Option<u32>> {
        let checkpoint = self.checkpoint();

        let mut value: u32 = 0;
        while let Some(hex) =
            self.peek().filter(char::is_ascii_hexdigit).and_then(|ch| ch.to_digit(16))
        {
            // To prevent panic on overflow cases like `\u{FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF}`
            if let Some(v) = value.checked_mul(16).and_then(|v| v.checked_add(hex)) {
                value = v;
                self.advance();
            } else {
                return Err(diagnostics::too_large_unicode_escape_sequence(Span::new(
                    self.options.span_offset + offset_start,
                    self.options.span_offset + self.offset(),
                )));
            }
        }

        if self.checkpoint() != checkpoint {
            return Ok(Some(value));
        }

        Ok(None)
    }

    // ---

    fn checkpoint(&self) -> (usize, u32) {
        (self.index, self.offset)
    }

    fn rewind(&mut self, checkpoint: (usize, u32)) {
        self.index = checkpoint.0;
        self.offset = checkpoint.1;
    }

    fn advance(&mut self) {
        if let Some(ch) = self.chars.get(self.index) {
            #[expect(clippy::cast_possible_truncation)]
            let len = ch.len_utf8() as u32;
            self.offset += len;
            self.index += 1;
        }
    }

    fn eat(&mut self, ch: char) -> bool {
        if self.peek() == Some(ch) {
            self.advance();
            return true;
        }
        false
    }

    fn offset(&self) -> u32 {
        self.offset
    }

    fn peek_nth(&self, n: usize) -> Option<char> {
        let nth = self.index + n;
        self.chars.get(nth).copied()
    }

    fn peek(&self) -> Option<char> {
        self.peek_nth(0)
    }

    fn peek2(&self) -> Option<char> {
        self.peek_nth(1)
    }
}
