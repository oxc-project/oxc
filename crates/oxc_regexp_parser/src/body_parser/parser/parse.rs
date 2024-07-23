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
        // TODO: Use `(?:)` as default?
        if self.source_text.is_empty() {
            return Err(OxcDiagnostic::error("Empty"));
        }

        let result = self.parse_disjunction()?;

        // TODO: Revisit `should_reparse`

        Ok(ast::Pattern { span: self.span_factory.create(0, self.source_text.len()), body: result })
    }

    // ```
    // Disjunction[UnicodeMode, UnicodeSetsMode, NamedCaptureGroups] ::
    //   Alternative[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    //   Alternative[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] | Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    // ```
    fn parse_disjunction(&mut self) -> Result<ast::Disjunction<'a>> {
        let span_start = self.reader.span_position();

        let mut body = Vec::new_in(self.allocator);
        loop {
            body.push(self.parse_alternative()?);

            if !self.reader.eat('|') {
                break;
            }
        }

        Ok(ast::Disjunction {
            span: self.span_factory.create(span_start, self.reader.span_position()),
            body,
        })
    }

    // ```
    // Alternative[UnicodeMode, UnicodeSetsMode, NamedCaptureGroups] ::
    //   [empty]
    //   Alternative[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] Term[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    // ```
    fn parse_alternative(&mut self) -> Result<ast::Alternative<'a>> {
        let span_start = self.reader.span_position();

        let mut body = Vec::new_in(self.allocator);
        while let Some(term) = self.parse_term()? {
            body.push(term);
        }

        Ok(ast::Alternative {
            span: self.span_factory.create(span_start, self.reader.span_position()),
            body,
        })
    }

    // ```
    // Term[UnicodeMode, UnicodeSetsMode, NamedCaptureGroups] ::
    //   [+UnicodeMode] Assertion[+UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    //   [+UnicodeMode] Atom[+UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] Quantifier
    //   [+UnicodeMode] Atom[+UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    //   [~UnicodeMode] QuantifiableAssertion[?NamedCaptureGroups] Quantifier
    //   [~UnicodeMode] Assertion[~UnicodeMode, ~UnicodeSetsMode, ?NamedCaptureGroups]
    //   [~UnicodeMode] ExtendedAtom[?NamedCaptureGroups] Quantifier
    //   [~UnicodeMode] ExtendedAtom[?NamedCaptureGroups]
    // ```
    // (Annex B)
    fn parse_term(&mut self) -> Result<Option<ast::RootNode<'a>>> {
        if self.reader.peek() == Some('|' as u32) || self.reader.peek() == Some(')' as u32) {
            return Ok(None);
        }

        // [+UnicodeMode] Assertion
        // [+UnicodeMode] Atom Quantifier
        // [+UnicodeMode] Atom
        if self.state.unicode_mode {
            if let Some(assertion) = self.parse_assertion()? {
                return Ok(Some(assertion));
            }

            let span_start = self.reader.span_position();
            return match (self.parse_atom()?, self.consume_quantifier()?) {
                (Some(atom), Some(((min, max), greedy))) => {
                    Ok(Some(ast::RootNode::Quantifier(Box::new_in(
                        ast::Quantifier {
                            span: self.span_factory.create(span_start, self.reader.span_position()),
                            greedy,
                            min,
                            max,
                            body: atom,
                        },
                        self.allocator,
                    ))))
                }
                (Some(atom), None) => Ok(Some(atom)),
                (None, Some(_)) => Err(OxcDiagnostic::error("Lone `Quantifier`, expected `Atom`")),
                (None, None) => Ok(None),
            };
        }

        // [~UnicodeMode] QuantifiableAssertion Quantifier
        // [~UnicodeMode] Assertion
        // [~UnicodeMode] ExtendedAtom Quantifier
        // [~UnicodeMode] ExtendedAtom
        let span_start = self.reader.span_position();
        if let Some(assertion) = self.parse_assertion()? {
            // `QuantifiableAssertion` = (Negative)Lookahead: `(?=...)` or `(?!...)`
            if let ast::RootNode::LookAroundGroup(look_around) = &assertion {
                if matches!(
                    look_around.kind,
                    ast::LookAroundGroupKind::Lookahead
                        | ast::LookAroundGroupKind::NegativeLookahead
                ) {
                    if let Some(((min, max), greedy)) = self.consume_quantifier()? {
                        return Ok(Some(ast::RootNode::Quantifier(Box::new_in(
                            ast::Quantifier {
                                span: self
                                    .span_factory
                                    .create(span_start, self.reader.span_position()),
                                greedy,
                                min,
                                max,
                                body: assertion,
                            },
                            self.allocator,
                        ))));
                    }
                }
            }

            return Ok(Some(assertion));
        }

        match (self.parse_extended_atom()?, self.consume_quantifier()?) {
            (Some(atom), Some(((min, max), greedy))) => {
                Ok(Some(ast::RootNode::Quantifier(Box::new_in(
                    ast::Quantifier {
                        span: self.span_factory.create(span_start, self.reader.span_position()),
                        min,
                        max,
                        greedy,
                        body: atom,
                    },
                    self.allocator,
                ))))
            }
            (Some(atom), None) => Ok(Some(atom)),
            (None, Some(_)) => {
                Err(OxcDiagnostic::error("Lone `Quantifier`, expected `ExtendedAtom`"))
            }
            (None, None) => Ok(None),
        }
    }

    // ```
    // Assertion[UnicodeMode, UnicodeSetsMode, NamedCaptureGroups] ::
    //   ^
    //   $
    //   \b
    //   \B
    //   [+UnicodeMode] (?= Disjunction[+UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
    //   [+UnicodeMode] (?! Disjunction[+UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
    //   [~UnicodeMode] QuantifiableAssertion[?NamedCaptureGroups]
    //   (?<= Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
    //   (?<! Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
    //
    // QuantifiableAssertion[NamedCaptureGroups] ::
    //   (?= Disjunction[~UnicodeMode, ~UnicodeSetsMode, ?NamedCaptureGroups] )
    //   (?! Disjunction[~UnicodeMode, ~UnicodeSetsMode, ?NamedCaptureGroups] )
    // ```
    // (Annex B)
    fn parse_assertion(&mut self) -> Result<Option<ast::RootNode<'a>>> {
        let span_start = self.reader.span_position();

        let kind = if self.reader.eat('^') {
            Some(ast::AssertionKind::Start)
        } else if self.reader.eat('$') {
            Some(ast::AssertionKind::End)
        } else if self.reader.eat2('\\', 'b') {
            Some(ast::AssertionKind::Boundary)
        } else if self.reader.eat2('\\', 'B') {
            Some(ast::AssertionKind::NegativeBoundary)
        } else {
            None
        };

        if let Some(kind) = kind {
            return Ok(Some(ast::RootNode::Assertion(ast::Assertion {
                span: self.span_factory.create(span_start, self.reader.span_position()),
                kind,
            })));
        }

        let kind = if self.reader.eat3('(', '?', '=') {
            Some(ast::LookAroundGroupKind::Lookahead)
        } else if self.reader.eat3('(', '?', '!') {
            Some(ast::LookAroundGroupKind::NegativeLookahead)
        } else if self.reader.eat3('(', '<', '=') {
            Some(ast::LookAroundGroupKind::Lookbehind)
        } else if self.reader.eat3('(', '<', '!') {
            Some(ast::LookAroundGroupKind::NegativeLookbehind)
        } else {
            None
        };

        if let Some(kind) = kind {
            let disjunction = self.parse_disjunction()?;

            if !self.reader.eat(')') {
                return Err(OxcDiagnostic::error("Unterminated lookaround group"));
            }

            return Ok(Some(ast::RootNode::LookAroundGroup(Box::new_in(
                ast::LookAroundGroup {
                    span: self.span_factory.create(span_start, self.reader.span_position()),
                    kind,
                    body: disjunction,
                },
                self.allocator,
            ))));
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
    fn parse_atom(&mut self) -> Result<Option<ast::RootNode<'a>>> {
        let span_start = self.reader.span_position();

        if let Some(cp) = self.reader.peek().filter(|&cp| !unicode::is_syntax_character(cp)) {
            self.reader.advance();

            return Ok(Some(ast::RootNode::Value(ast::Value {
                span: self.span_factory.create(span_start, self.reader.span_position()),
                kind: ast::ValueKind::Symbol,
                code_point: cp,
            })));
        }

        if self.reader.eat('.') {
            return Ok(Some(ast::RootNode::Dot(ast::Dot {
                span: self.span_factory.create(span_start, self.reader.span_position()),
            })));
        }

        // TODO: \ AtomEscape[?UnicodeMode, ?NamedCaptureGroups]
        // TODO: CharacterClass[?UnicodeMode, ?UnicodeSetsMode]
        // TODO: ( GroupSpecifier[?UnicodeMode]opt Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
        // TODO: (?: Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )

        if self.reader.eat('👻') {
            return Err(OxcDiagnostic::error("WIP..."));
        }
        Ok(None)
    }

    // ```
    // ExtendedAtom[NamedCaptureGroups] ::
    //   .
    //   \ AtomEscape[~UnicodeMode, ?NamedCaptureGroups]
    //   \ [lookahead = c]
    //   CharacterClass[~UnicodeMode, ~UnicodeSetsMode]
    //   ( GroupSpecifier[~UnicodeMode]opt Disjunction[~UnicodeMode, ~UnicodeSetsMode, ?NamedCaptureGroups] )
    //   (?: Disjunction[~UnicodeMode, ~UnicodeSetsMode, ?NamedCaptureGroups] )
    //   InvalidBracedQuantifier
    //   ExtendedPatternCharacter
    // ```
    fn parse_extended_atom(&mut self) -> Result<Option<ast::RootNode<'a>>> {
        let span_start = self.reader.span_position();

        if self.reader.eat('.') {
            return Ok(Some(ast::RootNode::Dot(ast::Dot {
                span: self.span_factory.create(span_start, self.reader.span_position()),
            })));
        }

        // TODO: \ AtomEscape[~UnicodeMode, ?NamedCaptureGroups]
        // TODO: \ [lookahead = c]
        // TODO: CharacterClass[~UnicodeMode, ~UnicodeSetsMode]
        // TODO: ( GroupSpecifier[~UnicodeMode]opt Disjunction[~UnicodeMode, ~UnicodeSetsMode, ?NamedCaptureGroups] )
        // TODO: (?: Disjunction[~UnicodeMode, ~UnicodeSetsMode, ?NamedCaptureGroups] )
        // TODO: InvalidBracedQuantifier
        // TODO: ExtendedPatternCharacter

        if self.reader.eat('👻') {
            return Err(OxcDiagnostic::error("WIP..."));
        }
        Ok(None)
    }

    // ```
    // Quantifier ::
    //   QuantifierPrefix
    //   QuantifierPrefix ?
    //
    // QuantifierPrefix ::
    //   *
    //   +
    //   ?
    //   { DecimalDigits[~Sep] }
    //   { DecimalDigits[~Sep] ,}
    //   { DecimalDigits[~Sep] , DecimalDigits[~Sep] }
    // ```
    /// Returns: ((min, max), greedy)
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
    // ([Sep] is disabled for `QuantifierPrefix`)
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
