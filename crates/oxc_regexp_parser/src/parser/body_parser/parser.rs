use oxc_allocator::{Allocator, Box, Vec};
use oxc_diagnostics::{OxcDiagnostic, Result};

use crate::{
    ast,
    parser::{
        body_parser::{reader::Reader, state::ParserState, unicode},
        options::ParserOptions,
        span::SpanFactory,
    },
};

pub struct PatternParser<'a> {
    allocator: &'a Allocator,
    source_text: &'a str,
    span_factory: SpanFactory,
    reader: Reader<'a>,
    _state: ParserState,
}

impl<'a> PatternParser<'a> {
    pub fn new(allocator: &'a Allocator, source_text: &'a str, options: ParserOptions) -> Self {
        Self {
            allocator,
            source_text,
            span_factory: SpanFactory::new(options.span_offset),
            reader: Reader::new(source_text, options.is_unicode_mode()),
            _state: ParserState,
        }
    }

    pub fn parse(&mut self) -> Result<ast::Pattern<'a>> {
        if self.source_text.is_empty() {
            return Err(OxcDiagnostic::error("Empty"));
        }

        // TODO: Remove later, just for clippy unused
        self.reader.eat2('a', 'b');
        self.reader.eat3('a', 'b', 'c');
        self.reader.rewind(0);

        self.consume_pattern()
    }

    // ```
    // Pattern[UnicodeMode, UnicodeSetsMode, NamedCaptureGroups] ::
    //   Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    // ```
    // <https://tc39.es/ecma262/#prod-Pattern>
    fn consume_pattern(&mut self) -> Result<ast::Pattern<'a>> {
        let start = self.reader.position();
        // TODO: Read only constants
        // this._numCapturingParens = this.countCapturingParens();
        // TODO: Define state, use later somewhere
        // this._groupSpecifiers.clear();
        // TODO: Define state, use later here
        // this._backreferenceNames.clear();

        // TODO: Maybe useless?
        // this.onPatternEnter(start);
        let alternatives = self.consume_disjunction()?;

        if self.reader.peek().is_some() {
            if self.reader.eat(')') {
                return Err(OxcDiagnostic::error("Unmatched ')'"));
            }
            if self.reader.eat('\\') {
                return Err(OxcDiagnostic::error("'\\' at end of pattern"));
            }
            if self.reader.eat(']') || self.reader.eat('}') {
                return Err(OxcDiagnostic::error("Lone quantifier brackets"));
            }
            return Err(OxcDiagnostic::error("Unexpected character"));
        }

        // TODO: Implement
        // for (const name of this._backreferenceNames) {
        //   if (!this._groupSpecifiers.hasInPattern(name)) {
        //     this.raise("Invalid named capture referenced");
        //   }
        // }

        let pattern = ast::Pattern {
            span: self.span_factory.create(start, self.reader.position()),
            alternatives,
        };

        // TODO: Implement fix up for back references
        // this.onPatternLeave(start, this.index);

        Ok(pattern)
    }

    // ```
    // Disjunction[UnicodeMode, UnicodeSetsMode, NamedCaptureGroups] ::
    //   Alternative[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    //   Alternative[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] | Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    // ```
    // <https://tc39.es/ecma262/#prod-Disjunction>
    fn consume_disjunction(&mut self) -> Result<Vec<'a, ast::Alternative<'a>>> {
        let mut alternatives = Vec::new_in(self.allocator);

        // TODO: Implement
        // this._groupSpecifiers.enterDisjunction();

        let mut i: usize = 0;
        loop {
            alternatives.push(self.consume_alternative(i)?);

            if !self.reader.eat('|') {
                break;
            }

            i += 1;
        }

        if self.reader.eat('{') {
            return Err(OxcDiagnostic::error("Lone quantifier brackets"));
        }

        // TODO: Implement
        // this._groupSpecifiers.leaveDisjunction();

        Ok(alternatives)
    }

    // ```
    // Alternative[UnicodeMode, UnicodeSetsMode, NamedCaptureGroups] ::
    //   [empty]
    //   Alternative[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] Term[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    // ```
    // <https://tc39.es/ecma262/#prod-Alternative>
    fn consume_alternative(&mut self, i: usize) -> Result<ast::Alternative<'a>> {
        let start = self.reader.position();

        // TODO: Implement
        let _ = i;
        // this._groupSpecifiers.enterAlternative(i);

        let mut elements = Vec::new_in(self.allocator);
        loop {
            if self.reader.peek().is_none() {
                break;
            }
            let Some(term) = self.consume_term()? else {
                break;
            };
            elements.push(term);
        }

        Ok(ast::Alternative {
            span: self.span_factory.create(start, self.reader.position()),
            elements,
        })
    }

    // ```
    // Term[UnicodeMode, UnicodeSetsMode, NamedCaptureGroups] ::
    //   Assertion[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    //   Atom[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    //   Atom[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] Quantifier
    // ```
    // <https://tc39.es/ecma262/#prod-Term>
    fn consume_term(&mut self) -> Result<Option<ast::Element<'a>>> {
        if let Some(assertion) = self.consume_assertion()? {
            return Ok(Some(ast::Element::Assertion(Box::new_in(assertion, self.allocator))));
        }

        let start = self.reader.position();
        match (self.consume_atom()?, self.consume_quantifier()?) {
            (Some(atom), None) => {
                Ok(Some(ast::Element::QuantifiableElement(Box::new_in(atom, self.allocator))))
            }
            (Some(atom), Some(((min, max), greedy))) => {
                return Ok(Some(ast::Element::Quantifier(Box::new_in(
                    ast::Quantifier {
                        span: self.span_factory.create(start, self.reader.position()),
                        min,
                        max,
                        greedy,
                        element: atom,
                    },
                    self.allocator,
                ))));
            }
            (None, Some(_)) => Err(OxcDiagnostic::error("Nothing to repeat")),
            (None, None) => Ok(None),
        }
    }

    // TODO: Implement
    fn consume_assertion(&mut self) -> Result<Option<ast::Assertion<'a>>> {
        if self.reader.eat('👻') {
            return Err(OxcDiagnostic::error("TODO"));
        }

        Ok(None)
    }

    // ```
    // Atom[UnicodeMode, UnicodeSetsMode, NamedCaptureGroups] ::
    //   PatternCharacter
    //   .
    //   \ AtomEscape[?UnicodeMode, ?NamedCaptureGroups]
    //   CharacterClass[?UnicodeMode, ?UnicodeSetsMode]
    //   ( GroupSpecifier[?UnicodeMode]opt Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
    //   (?: Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
    // ```
    // <https://tc39.es/ecma262/#prod-Atom>
    fn consume_atom(&mut self) -> Result<Option<ast::QuantifiableElement<'a>>> {
        if let Some(character) = self.consume_pattern_character() {
            return Ok(Some(ast::QuantifiableElement::Character(Box::new_in(
                character,
                self.allocator,
            ))));
        }

        // TODO: Implement
        if self.reader.eat('👻') {
            return Err(OxcDiagnostic::error("TODO"));
        }
        // if self.consume_dot() {}
        // if self.consume_reverse_solidus_atom_escape() {}
        // if self.consume_character_class() {}
        // if self.consume_uncapturing_group() {}
        // if self.consume_capturing_group() {}

        Ok(None)
    }

    // ```
    // Quantifier ::
    //   QuantifierPrefix
    //   QuantifierPrefix ?
    // ```
    // <https://tc39.es/ecma262/#prod-Quantifier>
    // ```
    // QuantifierPrefix ::
    //   *
    //   +
    //   ?
    //   { DecimalDigits }
    //   { DecimalDigits ,}
    //   { DecimalDigits , DecimalDigits }
    // ```
    // <https://tc39.es/ecma262/#prod-QuantifierPrefix>
    #[allow(clippy::type_complexity)]
    fn consume_quantifier(&mut self) -> Result<Option<((usize, Option<usize>), bool)>> {
        let is_greedy = |reader: &mut Reader| !reader.eat('?');

        if self.reader.eat('*') {
            return Ok(Some(((0, None), is_greedy(&mut self.reader))));
        }
        if self.reader.eat('+') {
            return Ok(Some(((1, None), is_greedy(&mut self.reader))));
        }
        if self.reader.eat('?') {
            return Ok(Some(((0, Some(1)), is_greedy(&mut self.reader))));
        }

        if self.reader.eat('{') {
            if let Some(min) = self.consume_decimal_digits() {
                if self.reader.eat('}') {
                    return Ok(Some(((min, Some(min)), is_greedy(&mut self.reader))));
                }

                if self.reader.eat(',') {
                    if self.reader.eat('}') {
                        return Ok(Some(((min, None), is_greedy(&mut self.reader))));
                    }

                    if let Some(max) = self.consume_decimal_digits() {
                        if self.reader.eat('}') {
                            if max < min {
                                return Err(OxcDiagnostic::error(
                                    "Numbers out of order in {} quantifier",
                                ));
                            }

                            return Ok(Some(((min, Some(max)), is_greedy(&mut self.reader))));
                        }
                    }
                }
            }

            return Err(OxcDiagnostic::error("Incomplete quantifier"));
        }

        Ok(None)
    }

    // ```
    // PatternCharacter ::
    //   SourceCharacter but not SyntaxCharacter
    // ```
    // <https://tc39.es/ecma262/#prod-PatternCharacter>
    // ```
    // SyntaxCharacter :: one of
    //   ^ $ \ . * + ? ( ) [ ] { } |
    // ```
    // <https://tc39.es/ecma262/#prod-SyntaxCharacter>
    fn consume_pattern_character(&mut self) -> Option<ast::Character> {
        let start = self.reader.position();

        let cp = self.reader.peek()?;
        if unicode::is_syntax_character(cp) {
            return None;
        }

        self.reader.advance();
        Some(ast::Character {
            span: self.span_factory.create(start, self.reader.position()),
            value: cp,
        })
    }

    fn consume_decimal_digits(&mut self) -> Option<usize> {
        let start = self.reader.position();

        let mut value = 0;
        while let Some(cp) = self.reader.peek() {
            if !unicode::is_decimal_digits(cp) {
                break;
            }

            // `- '0' as u32`: convert code point to digit
            value = (10 * value) + (cp - '0' as u32) as usize;
            self.reader.advance();
        }

        if self.reader.position() != start {
            return Some(value);
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use oxc_allocator::Allocator;

    #[test]
    fn should_handle_unicode() {
        let allocator = Allocator::default();
        let source_text = "Emoji🥹";

        let pattern =
            PatternParser::new(&allocator, source_text, ParserOptions::default()).parse().unwrap();
        assert_eq!(pattern.alternatives[0].elements.len(), 7);

        let pattern = PatternParser::new(
            &allocator,
            source_text,
            ParserOptions::default().with_modes(true, false),
        )
        .parse()
        .unwrap();
        assert_eq!(pattern.alternatives[0].elements.len(), 6);
        let pattern = PatternParser::new(
            &allocator,
            source_text,
            ParserOptions::default().with_modes(false, true),
        )
        .parse()
        .unwrap();
        assert_eq!(pattern.alternatives[0].elements.len(), 6);
    }

    // NOTE: These may be useless when integlation tests are added
    #[test]
    fn should_pass() {
        let allocator = Allocator::default();

        for source_text in &[
            "a",
            "a+",
            "a*",
            "a?",
            "a{1}",
            "a{1,}",
            "a{1,2}",
            "a|b",
            "a|b|c",
            "a|b+?|c",
            "a+b*?c{1}d{2,}e{3,4}",
        ] {
            assert!(
                PatternParser::new(&allocator, source_text, ParserOptions::default())
                    .parse()
                    .is_ok(),
                "{source_text} should be parsed!"
            );
        }
    }

    #[test]
    fn should_fail() {
        let allocator = Allocator::default();

        for source_text in &["", "a)", "b\\", "c]", "d}", "e|+", "f|{", "g{", "g{1", "g{1,", "g{,"]
        {
            assert!(
                PatternParser::new(&allocator, source_text, ParserOptions::default())
                    .parse()
                    .is_err(),
                "{source_text} should fail to parse!"
            );
        }
    }
}
