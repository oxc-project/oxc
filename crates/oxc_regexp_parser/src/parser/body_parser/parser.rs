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
        self.reader.eat3('a', 'b', 'c');

        self.consume_pattern()
    }

    // ```
    // Pattern[UnicodeMode, UnicodeSetsMode, NamedCaptureGroups] ::
    //   Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    // ```
    // <https://tc39.es/ecma262/#prod-Pattern>
    fn consume_pattern(&mut self) -> Result<ast::Pattern<'a>> {
        let start = self.reader.span_position();
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
            span: self.span_factory.create(start, self.reader.span_position()),
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
        let start = self.reader.span_position();

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
            span: self.span_factory.create(start, self.reader.span_position()),
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

        let start = self.reader.span_position();
        match (self.consume_atom()?, self.consume_quantifier()?) {
            (Some(atom), None) => {
                Ok(Some(ast::Element::QuantifiableElement(Box::new_in(atom, self.allocator))))
            }
            (Some(atom), Some(((min, max), greedy))) => {
                return Ok(Some(ast::Element::Quantifier(Box::new_in(
                    ast::Quantifier {
                        span: self.span_factory.create(start, self.reader.span_position()),
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

    // Assertion[UnicodeMode, UnicodeSetsMode, NamedCaptureGroups] ::
    //   ^
    //   $
    //   \b
    //   \B
    //   (?= Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
    //   (?! Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
    //   (?<= Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
    //   (?<! Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
    // <https://tc39.es/ecma262/#prod-Assertion>
    fn consume_assertion(&mut self) -> Result<Option<ast::Assertion<'a>>> {
        let start = self.reader.span_position();

        if self.reader.eat('^') {
            return Ok(Some(ast::Assertion::BoundaryAssertion(Box::new_in(
                ast::BoundaryAssertion::EdgeAssertion(Box::new_in(
                    ast::EdgeAssertion {
                        span: self.span_factory.create(start, self.reader.span_position()),
                        kind: ast::EdgeAssertionKind::Start,
                    },
                    self.allocator,
                )),
                self.allocator,
            ))));
        }
        if self.reader.eat('$') {
            return Ok(Some(ast::Assertion::BoundaryAssertion(Box::new_in(
                ast::BoundaryAssertion::EdgeAssertion(Box::new_in(
                    ast::EdgeAssertion {
                        span: self.span_factory.create(start, self.reader.span_position()),
                        kind: ast::EdgeAssertionKind::End,
                    },
                    self.allocator,
                )),
                self.allocator,
            ))));
        }
        if self.reader.eat2('\\', 'B') {
            return Ok(Some(ast::Assertion::BoundaryAssertion(Box::new_in(
                ast::BoundaryAssertion::WordBoundaryAssertion(Box::new_in(
                    ast::WordBoundaryAssertion {
                        span: self.span_factory.create(start, self.reader.span_position()),
                        negate: false,
                    },
                    self.allocator,
                )),
                self.allocator,
            ))));
        }
        if self.reader.eat2('\\', 'b') {
            return Ok(Some(ast::Assertion::BoundaryAssertion(Box::new_in(
                ast::BoundaryAssertion::WordBoundaryAssertion(Box::new_in(
                    ast::WordBoundaryAssertion {
                        span: self.span_factory.create(start, self.reader.span_position()),
                        negate: true,
                    },
                    self.allocator,
                )),
                self.allocator,
            ))));
        }

        // Lookaround
        let rewind_start = self.reader.checkpoint();
        if self.reader.eat2('(', '?') {
            let lookaround = {
                let is_lookbehind = self.reader.eat('<');

                if self.reader.eat('=') {
                    Some((false, is_lookbehind))
                } else if self.reader.eat('!') {
                    Some((true, is_lookbehind))
                } else {
                    None
                }
            };

            if let Some((negate, is_lookbehind)) = lookaround {
                let alternatives = self.consume_disjunction()?;
                if !self.reader.eat(')') {
                    return Err(OxcDiagnostic::error("Unterminated group"));
                }

                let span = self.span_factory.create(start, self.reader.span_position());
                let lookaround_assertion = if is_lookbehind {
                    ast::LookaroundAssertion::LookbehindAssertion(Box::new_in(
                        ast::LookbehindAssertion { span, negate, alternatives },
                        self.allocator,
                    ))
                } else {
                    ast::LookaroundAssertion::LookaheadAssertion(Box::new_in(
                        ast::LookaheadAssertion { span, negate, alternatives },
                        self.allocator,
                    ))
                };

                return Ok(Some(ast::Assertion::LookaroundAssertion(Box::new_in(
                    lookaround_assertion,
                    self.allocator,
                ))));
            }

            self.reader.rewind(rewind_start);
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
        if let Some(any_character_set) = self.consume_dot() {
            return Ok(Some(ast::QuantifiableElement::CharacterSet(Box::new_in(
                ast::CharacterSet::AnyCharacterSet(Box::new_in(any_character_set, self.allocator)),
                self.allocator,
            ))));
        }

        // TODO: Implement
        if self.reader.eat('👻') {
            return Err(OxcDiagnostic::error("TODO"));
        }

        // if self.consume_reverse_solidus_atom_escape() {}
        // if self.consume_character_class() {}
        // if self.consume_capturing_group() {}
        // if self.consume_uncapturing_group() {}

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
                                    "Numbers out of order in braced quantifier",
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
        let start = self.reader.span_position();

        let cp = self.reader.peek()?;
        if unicode::is_syntax_character(cp) {
            return None;
        }

        self.reader.advance();
        Some(ast::Character {
            span: self.span_factory.create(start, self.reader.span_position()),
            value: cp,
        })
    }

    fn consume_dot(&mut self) -> Option<ast::AnyCharacterSet> {
        let start = self.reader.span_position();

        if !self.reader.eat('.') {
            return None;
        }

        Some(ast::AnyCharacterSet {
            span: self.span_factory.create(start, self.reader.span_position()),
        })
    }

    fn consume_decimal_digits(&mut self) -> Option<usize> {
        let start = self.reader.span_position();

        let mut value = 0;
        while let Some(cp) = self.reader.peek() {
            if !unicode::is_decimal_digits(cp) {
                break;
            }

            // `- '0' as u32`: convert code point to digit
            value = (10 * value) + (cp - '0' as u32) as usize;
            self.reader.advance();
        }

        if self.reader.span_position() != start {
            return Some(value);
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use oxc_allocator::Allocator;

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
            "a+b*?c{1}d{2,}e{3,4}?",
            r"^(?=ab)\b(?!cd)(?<=ef)\B(?<!gh)$",
            "a.b..",
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

        for source_text in &[
            "", "a)", r"b\", "c]", "d}", "e|+", "f|{", "g{", "g{1", "g{1,", "g{,", "g{2,1}",
            "(?=h", "(?<!h",
        ] {
            assert!(
                PatternParser::new(&allocator, source_text, ParserOptions::default())
                    .parse()
                    .is_err(),
                "{source_text} should fail to parse!"
            );
        }
    }

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
}
