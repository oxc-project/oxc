use oxc_allocator::{Allocator, Box, Vec};
use oxc_diagnostics::{OxcDiagnostic, Result};
use oxc_span::Atom as SpanAtom;

use crate::{
    ast,
    body_parser::{reader::Reader, state::State, unicode, unicode_property},
    options::ParserOptions,
    span::SpanFactory,
};

pub struct PatternParser<'a> {
    allocator: &'a Allocator,
    source_text: &'a str,
    span_factory: SpanFactory,
    reader: Reader<'a>,
    state: State,
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
        // For `new RegExp("")` or `new RegExp()` (= empty)
        if self.source_text.is_empty() {
            self.source_text = "(?:)";
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

        // `PatternCharacter`
        if let Some(cp) = self.reader.peek().filter(|&cp| !unicode::is_syntax_character(cp)) {
            self.reader.advance();

            return Ok(Some(ast::RootNode::Value(ast::Value {
                span: self.span_factory.create(span_start, self.reader.span_position()),
                kind: ast::ValueKind::Symbol,
                value: cp,
            })));
        }

        // `.`
        if self.reader.eat('.') {
            return Ok(Some(ast::RootNode::Dot(ast::Dot {
                span: self.span_factory.create(span_start, self.reader.span_position()),
            })));
        }

        // `\ AtomEscape`
        if self.reader.eat('\\') {
            if let Some(atom_escape) = self.parse_atom_escape(span_start)? {
                return Ok(Some(atom_escape));
            }
        }

        // `CharacterClass`
        // TODO
        // `( GroupSpecifieropt Disjunction )`
        // TODO
        // `(?: Disjunction )`
        // TODO

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

        // `.`
        if self.reader.eat('.') {
            return Ok(Some(ast::RootNode::Dot(ast::Dot {
                span: self.span_factory.create(span_start, self.reader.span_position()),
            })));
        }

        // `\ AtomEscape`
        // TODO
        // `\ [lookahead = c]`
        // TODO
        // `CharacterClass`
        // TODO
        // `( GroupSpecifieropt Disjunction )`
        // TODO
        // `(?: Disjunction )`
        // TODO
        // `InvalidBracedQuantifier`
        // TODO
        // `ExtendedPatternCharacter`
        // TODO

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

            return Err(OxcDiagnostic::error("Unterminated quantifier"));
        }

        Ok(None)
    }

    // ```
    // AtomEscape[UnicodeMode, NamedCaptureGroups] ::
    //   [+UnicodeMode] DecimalEscape
    //   [~UnicodeMode] DecimalEscape but only if the CapturingGroupNumber of DecimalEscape is ≤ CountLeftCapturingParensWithin(the Pattern containing DecimalEscape)
    //   CharacterClassEscape[?UnicodeMode]
    //   CharacterEscape[?UnicodeMode, ?NamedCaptureGroups]
    //   [+NamedCaptureGroups] k GroupName[?UnicodeMode]
    // ```
    fn parse_atom_escape(&mut self, span_start: usize) -> Result<Option<ast::RootNode<'a>>> {
        // `DecimalEscape`: \1 means indexed reference
        // TODO

        // `CharacterClassEscape`: \d, \p{...}
        if let Some(character_class_escape) = self.parse_character_class_escape(span_start) {
            return Ok(Some(ast::RootNode::CharacterClassEscape(character_class_escape)));
        }
        if let Some(unicode_property_escape) =
            self.parse_character_class_escape_unicode(span_start)?
        {
            return Ok(Some(ast::RootNode::UnicodePropertyEscape(Box::new_in(
                unicode_property_escape,
                self.allocator,
            ))));
        }

        // `CharacterEscape`: \n, \cM, \0, etc...
        if let Some(character_escape) = self.parse_character_escape(span_start)? {
            return Ok(Some(ast::RootNode::Value(character_escape)));
        }

        // `k GroupName`: \k<name> means named reference
        if self.reader.eat('k') {
            if let Some(name) = self.consume_group_name()? {
                return Ok(Some(ast::RootNode::NamedReference(Box::new_in(
                    ast::NamedReference {
                        span: self.span_factory.create(span_start, self.reader.span_position()),
                        name,
                    },
                    self.allocator,
                ))));
            }

            return Err(OxcDiagnostic::error("Invalid named reference"));
        }

        Err(OxcDiagnostic::error("Invalid atom escape"))
    }

    // ```
    // CharacterClassEscape ::
    //   d
    //   D
    //   s
    //   S
    //   w
    //   W
    // ```
    fn parse_character_class_escape(
        &mut self,
        span_start: usize,
    ) -> Option<ast::CharacterClassEscape> {
        let kind = if self.reader.eat('d') {
            ast::CharacterClassEscapeKind::D
        } else if self.reader.eat('D') {
            ast::CharacterClassEscapeKind::NegativeD
        } else if self.reader.eat('s') {
            ast::CharacterClassEscapeKind::S
        } else if self.reader.eat('S') {
            ast::CharacterClassEscapeKind::NegativeS
        } else if self.reader.eat('w') {
            ast::CharacterClassEscapeKind::W
        } else if self.reader.eat('W') {
            ast::CharacterClassEscapeKind::NegativeW
        } else {
            return None;
        };

        Some(ast::CharacterClassEscape {
            span: self.span_factory.create(span_start, self.reader.span_position()),
            kind,
        })
    }
    // ```
    // CharacterClassEscape[UnicodeMode] ::
    //   [+UnicodeMode] p{ UnicodePropertyValueExpression }
    //   [+UnicodeMode] P{ UnicodePropertyValueExpression }
    // ```
    fn parse_character_class_escape_unicode(
        &mut self,
        span_start: usize,
    ) -> Result<Option<ast::UnicodePropertyEscape<'a>>> {
        if !self.state.unicode_mode {
            return Ok(None);
        }

        let negative = if self.reader.eat('p') {
            true
        } else if self.reader.eat('P') {
            false
        } else {
            return Ok(None);
        };

        if self.reader.eat('{') {
            if let Some((name, value, is_strings_related)) =
                self.consume_unicode_property_value_expression()?
            {
                if negative && is_strings_related {
                    return Err(OxcDiagnostic::error("Invalid property name"));
                }

                if self.reader.eat('}') {
                    return Ok(Some(ast::UnicodePropertyEscape {
                        span: self.span_factory.create(span_start, self.reader.span_position()),
                        negative,
                        name,
                        value,
                    }));
                }
            }
        }

        Err(OxcDiagnostic::error("Unterminated unicode property escape"))
    }

    // ```
    // CharacterEscape[UnicodeMode, NamedCaptureGroups] ::
    //   ControlEscape
    //   c AsciiLetter
    //   0 [lookahead ∉ DecimalDigit]
    //   HexEscapeSequence
    //   RegExpUnicodeEscapeSequence[?UnicodeMode]
    //   [~UnicodeMode] LegacyOctalEscapeSequence
    //   IdentityEscape[?UnicodeMode, ?NamedCaptureGroups]
    // ```
    // (Annex B)
    fn parse_character_escape(&mut self, span_start: usize) -> Result<Option<ast::Value>> {
        // e.g. \n
        if let Some(cp) = self.reader.peek().and_then(unicode::map_control_escape) {
            self.reader.advance();

            return Ok(Some(ast::Value {
                span: self.span_factory.create(span_start, self.reader.span_position()),
                kind: ast::ValueKind::SingleEscape,
                value: cp,
            }));
        }

        // e.g. \cM
        let checkpoint = self.reader.checkpoint();
        if self.reader.eat('c') {
            if let Some(cp) = self.reader.peek().and_then(unicode::map_c_ascii_letter) {
                self.reader.advance();
                return Ok(Some(ast::Value {
                    span: self.span_factory.create(span_start, self.reader.span_position()),
                    kind: ast::ValueKind::ControlLetter,
                    value: cp,
                }));
            }
            self.reader.rewind(checkpoint);
        }

        // e.g. \0
        if self.reader.peek().map_or(false, |cp| cp == '0' as u32)
            && self.reader.peek2().map_or(true, |cp| !unicode::is_decimal_digit(cp))
        {
            self.reader.advance();

            return Ok(Some(ast::Value {
                span: self.span_factory.create(span_start, self.reader.span_position()),
                kind: ast::ValueKind::Null,
                value: 0x0000,
            }));
        }

        // e.g. \x41
        if self.reader.eat('x') {
            if let Some(cp) = self.consume_fixed_hex_digits(2) {
                return Ok(Some(ast::Value {
                    span: self.span_factory.create(span_start, self.reader.span_position()),
                    kind: ast::ValueKind::HexadecimalEscape,
                    value: cp,
                }));
            }

            return Err(OxcDiagnostic::error("Invalid escape"));
        }

        // e.g. \u{1f600}
        if let Some(cp) = self.consume_reg_exp_unicode_escape_sequence()? {
            return Ok(Some(ast::Value {
                span: self.span_factory.create(span_start, self.reader.span_position()),
                kind: ast::ValueKind::UnicodeEscape,
                value: cp,
            }));
        }

        // e.g. \18
        if !self.state.unicode_mode {
            if let Some(cp) = self.consume_legacy_octal_escape_sequence() {
                return Ok(Some(ast::Value {
                    span: self.span_factory.create(span_start, self.reader.span_position()),
                    kind: ast::ValueKind::Octal,
                    value: cp,
                }));
            }
        }

        // e.g. \.
        if let Some(cp) = self.consume_identity_escape() {
            return Ok(Some(ast::Value {
                span: self.span_factory.create(span_start, self.reader.span_position()),
                kind: ast::ValueKind::Identifier,
                value: cp,
            }));
        }

        Ok(None)
    }

    // ---

    // ```
    // DecimalDigits[Sep] ::
    //   DecimalDigit
    //   DecimalDigits[?Sep] DecimalDigit
    //   [+Sep] DecimalDigits[+Sep] NumericLiteralSeparator DecimalDigit
    // ```
    // ([Sep] is disabled for `QuantifierPrefix`, skip it)
    fn consume_decimal_digits(&mut self) -> Option<usize> {
        let checkpoint = self.reader.checkpoint();

        let mut value = 0;
        while let Some(cp) = self.reader.peek().filter(|&cp| unicode::is_decimal_digit(cp)) {
            // `- '0' as u32`: convert code point to digit
            value = (10 * value) + (cp - '0' as u32) as usize;
            self.reader.advance();
        }

        if self.reader.checkpoint() != checkpoint {
            return Some(value);
        }

        None
    }

    // ```
    // UnicodePropertyValueExpression ::
    //   UnicodePropertyName = UnicodePropertyValue
    //   LoneUnicodePropertyNameOrValue
    // ```
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
                if !self.state.unicode_sets_mode {
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
    // GroupName[UnicodeMode] ::
    //   < RegExpIdentifierName[?UnicodeMode] >
    // ```
    fn consume_group_name(&mut self) -> Result<Option<SpanAtom<'a>>> {
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

        if !self.state.unicode_mode {
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
    // RegExpUnicodeEscapeSequence[UnicodeMode] ::
    //   [+UnicodeMode] u HexLeadSurrogate \u HexTrailSurrogate
    //   [+UnicodeMode] u HexLeadSurrogate
    //   [+UnicodeMode] u HexTrailSurrogate
    //   [+UnicodeMode] u HexNonSurrogate
    //   [~UnicodeMode] u Hex4Digits
    //   [+UnicodeMode] u{ CodePoint }
    // ```
    fn consume_reg_exp_unicode_escape_sequence(&mut self) -> Result<Option<u32>> {
        if !self.reader.eat('u') {
            return Ok(None);
        }

        if self.state.unicode_mode {
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
        if self.state.unicode_mode {
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
    // RegExpIdentifierPart[UnicodeMode] ::
    //   IdentifierPartChar
    //   \ RegExpUnicodeEscapeSequence[+UnicodeMode]
    //   [~UnicodeMode] UnicodeLeadSurrogate UnicodeTrailSurrogate
    // ```
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

        if !self.state.unicode_mode {
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
    // LegacyOctalEscapeSequence ::
    //   0 [lookahead ∈ { 8, 9 }]
    //   NonZeroOctalDigit [lookahead ∉ OctalDigit]
    //   ZeroToThree OctalDigit [lookahead ∉ OctalDigit]
    //   FourToSeven OctalDigit
    //   ZeroToThree OctalDigit OctalDigit
    // ```
    fn consume_legacy_octal_escape_sequence(&mut self) -> Option<u32> {
        if let Some(first) = self.consume_octal_digit() {
            // 0 [lookahead ∈ { 8, 9 }]
            if first == 0
                && self.reader.peek().filter(|&cp| cp == '8' as u32 || cp == '9' as u32).is_some()
            {
                return Some(first);
            }

            if let Some(second) = self.consume_octal_digit() {
                if let Some(third) = self.consume_octal_digit() {
                    // ZeroToThree OctalDigit OctalDigit
                    if first <= 3 {
                        return Some(first * 64 + second * 8 + third);
                    }
                }

                // ZeroToThree OctalDigit [lookahead ∉ OctalDigit]
                // FourToSeven OctalDigit
                return Some(first * 8 + second);
            }

            // NonZeroOctalDigit [lookahead ∉ OctalDigit]
            return Some(first);
        }

        None
    }

    fn consume_octal_digit(&mut self) -> Option<u32> {
        let cp = self.reader.peek()?;

        if unicode::is_octal_digit(cp) {
            self.reader.advance();
            // `- '0' as u32`: convert code point to digit
            return Some(cp - '0' as u32);
        }

        None
    }

    // ```
    // IdentityEscape[UnicodeMode] ::
    //   [+UnicodeMode] SyntaxCharacter
    //   [+UnicodeMode] /
    //   [~UnicodeMode] SourceCharacter but not UnicodeIDContinue
    // ```
    fn consume_identity_escape(&mut self) -> Option<u32> {
        let cp = self.reader.peek()?;

        if self.state.unicode_mode {
            if unicode::is_syntax_character(cp) {
                self.reader.advance();
                return Some(cp);
            }

            if cp == '/' as u32 {
                self.reader.advance();
                return Some(cp);
            }
        }

        if !self.state.unicode_mode && !unicode::is_id_continue(cp) {
            self.reader.advance();
            return Some(cp);
        }

        None
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

    fn consume_fixed_hex_digits(&mut self, len: usize) -> Option<u32> {
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
}
