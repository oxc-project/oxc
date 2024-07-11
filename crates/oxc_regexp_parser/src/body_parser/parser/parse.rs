use oxc_allocator::{Allocator, Box, Vec};
use oxc_diagnostics::{OxcDiagnostic, Result};

use crate::{
    ast,
    body_parser::{reader::Reader, state::State, unicode},
    options::ParserOptions,
    span::SpanFactory,
};

pub struct PatternParser<'a> {
    pub(super) allocator: &'a Allocator,
    pub(super) source_text: &'a str,
    pub(super) span_factory: SpanFactory,
    pub(super) reader: Reader<'a>,
    pub(super) state: State,
}

impl<'a> PatternParser<'a> {
    pub fn new(allocator: &'a Allocator, source_text: &'a str, options: ParserOptions) -> Self {
        let unicode_mode = options.unicode_flag || options.unicode_sets_flag;
        let unicode_sets_mode = options.unicode_sets_flag;

        Self {
            allocator,
            source_text,
            span_factory: SpanFactory::new(options.span_offset),
            reader: Reader::new(source_text, unicode_mode),
            state: State::new(unicode_mode, unicode_sets_mode),
        }
    }

    pub fn parse(&mut self) -> Result<ast::Pattern<'a>> {
        if self.source_text.is_empty() {
            return Err(OxcDiagnostic::error("Empty"));
        }

        self.consume_pattern()
    }

    // ```
    // Pattern[UnicodeMode, UnicodeSetsMode, NamedCaptureGroups] ::
    //   Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    // ```
    // <https://tc39.es/ecma262/#prod-Pattern>
    fn consume_pattern(&mut self) -> Result<ast::Pattern<'a>> {
        // TODO: Define state, use later somewhere
        // this._groupSpecifiers.clear();
        // TODO: Define state, use later here
        // this._backreferenceNames.clear();

        let span_start = self.reader.span_position();
        let disjunction = self.consume_disjunction()?;

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
            span: self.span_factory.create(span_start, self.reader.span_position()),
            alternatives: disjunction,
        };

        // TODO: Implement, finalize backreferences with captured groups
        // this.onPatternLeave(start, this.index);

        Ok(pattern)
    }

    // ```
    // Disjunction[UnicodeMode, UnicodeSetsMode, NamedCaptureGroups] ::
    //   Alternative[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    //   Alternative[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] | Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    // ```
    // <https://tc39.es/ecma262/#prod-Disjunction>
    pub(super) fn consume_disjunction(&mut self) -> Result<Vec<'a, ast::Alternative<'a>>> {
        let mut disjunction = Vec::new_in(self.allocator);

        // TODO: Implement
        // this._groupSpecifiers.enterDisjunction();

        let mut i: usize = 0;
        loop {
            disjunction.push(self.consume_alternative(i)?);

            if self.reader.eat('|') {
                i += 1;
                continue;
            }

            break;
        }

        if self.reader.eat('{') {
            return Err(OxcDiagnostic::error("Lone quantifier brackets"));
        }

        // TODO: Implement
        // this._groupSpecifiers.leaveDisjunction();

        Ok(disjunction)
    }

    // ```
    // Alternative[UnicodeMode, UnicodeSetsMode, NamedCaptureGroups] ::
    //   [empty]
    //   Alternative[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] Term[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    // ```
    // <https://tc39.es/ecma262/#prod-Alternative>
    fn consume_alternative(&mut self, i: usize) -> Result<ast::Alternative<'a>> {
        // TODO: Implement
        let _ = i;
        // this._groupSpecifiers.enterAlternative(i);

        let span_start = self.reader.span_position();

        let mut terms = Vec::new_in(self.allocator);
        while let Some(term) = self.consume_term()? {
            terms.push(term);
        }

        Ok(ast::Alternative {
            span: self.span_factory.create(span_start, self.reader.span_position()),
            terms,
        })
    }

    // ```
    // Term[UnicodeMode, UnicodeSetsMode, NamedCaptureGroups] ::
    //   Assertion[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    //   Atom[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    //   Atom[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] Quantifier
    // ```
    // <https://tc39.es/ecma262/#prod-Term>
    fn consume_term(&mut self) -> Result<Option<ast::Term<'a>>> {
        if let Some(assertion) = self.consume_assertion()? {
            return Ok(Some(ast::Term::Assertion(Box::new_in(assertion, self.allocator))));
        }

        let span_start = self.reader.span_position();
        match (self.consume_atom()?, self.consume_quantifier()?) {
            (Some(atom), None) => Ok(Some(ast::Term::Atom(Box::new_in(atom, self.allocator)))),
            (Some(atom), Some(((min, max), greedy))) => {
                return Ok(Some(ast::Term::AtomWithQuantifier(Box::new_in(
                    ast::Quantifier {
                        span: self.span_factory.create(span_start, self.reader.span_position()),
                        min,
                        max,
                        greedy,
                        atom,
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
        let span_start = self.reader.span_position();

        if self.reader.eat('^') {
            return Ok(Some(ast::Assertion::BoundaryAssertion(Box::new_in(
                ast::BoundaryAssertion::EdgeAssertion(Box::new_in(
                    ast::EdgeAssertion {
                        span: self.span_factory.create(span_start, self.reader.span_position()),
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
                        span: self.span_factory.create(span_start, self.reader.span_position()),
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
                        span: self.span_factory.create(span_start, self.reader.span_position()),
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
                        span: self.span_factory.create(span_start, self.reader.span_position()),
                        negate: true,
                    },
                    self.allocator,
                )),
                self.allocator,
            ))));
        }

        // Lookaround
        let checkpoint = self.reader.checkpoint();
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

                let span = self.span_factory.create(span_start, self.reader.span_position());
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

            self.reader.rewind(checkpoint);
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
    fn consume_atom(&mut self) -> Result<Option<ast::Atom<'a>>> {
        if let Some(atom) = self.consume_pattern_character() {
            return Ok(Some(atom));
        }
        if let Some(atom) = self.consume_dot() {
            return Ok(Some(atom));
        }
        if let Some(atom) = self.consume_atom_escape()? {
            return Ok(Some(atom));
        }
        if let Some(atom) = self.consume_character_class()? {
            return Ok(Some(atom));
        }
        // In the spec, (named) capturing group and non-capturing group are defined in that order.
        // But if `?:` is not valid as a `GroupSpecifier`, why is it not defined in reverse order...?
        if let Some(atom) = self.consume_non_capturing_group()? {
            return Ok(Some(atom));
        }
        if let Some(atom) = self.consume_capturing_group()? {
            return Ok(Some(atom));
        }

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
    /// Returns: `((min, max?), greedy)`
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
    // DecimalDigits[Sep] ::
    //   DecimalDigit
    //   DecimalDigits[?Sep] DecimalDigit
    //   [+Sep] DecimalDigits[+Sep] NumericLiteralSeparator DecimalDigit
    // ```
    // <https://tc39.es/ecma262/#prod-DecimalDigits>
    fn consume_decimal_digits(&mut self) -> Option<usize> {
        let checkpoint = self.reader.checkpoint();

        let mut value = 0;
        while let Some(cp) = self.reader.peek().filter(|&cp| unicode::is_decimal_digits(cp)) {
            // `- '0' as u32`: convert code point to digit
            value = (10 * value) + (cp - '0' as u32) as usize;
            self.reader.advance();
        }

        if self.reader.checkpoint() != checkpoint {
            return Some(value);
        }

        None
    }
}
