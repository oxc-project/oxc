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
    state: State<'a>,
}

impl<'a> PatternParser<'a> {
    pub fn new(allocator: &'a Allocator, source_text: &'a str, options: ParserOptions) -> Self {
        // `RegExp` can not be empty.
        // - Literal `//` means just a single line comment
        // - For `new RegExp("")` or `new RegExp()` (= empty), use a placeholder
        let source_text = if source_text.is_empty() { "(?:)" } else { source_text };

        Self {
            allocator,
            source_text,
            span_factory: SpanFactory::new(options.span_offset),
            reader: Reader::new(source_text, options.unicode_mode),
            state: State::new(options.unicode_mode, options.unicode_sets_mode),
        }
    }

    pub fn parse(&mut self) -> Result<ast::Pattern<'a>> {
        // Pre parse whole pattern to collect:
        // - the number of (named|unnamed) capturing groups
        //   - For `\1` in `\1()` to be handled as indexed reference
        // - names of named capturing groups
        //   - For `\k<a>`, `\k<a>(?<b>)` to be handled as early error in `+NamedCaptureGroups`
        //
        // NOTE: It means that this perform 2 loops for every cases.
        // - Pros: Code is simple enough and easy to understand
        // - Cons: 1st pass is completely useless if the pattern does not contain any capturing groups
        // We may re-consider this if we need more performance rather than simplicity.
        let duplicated_named_capturing_groups =
            self.state.initialize_with_parsing(self.source_text);

        // [SS:EE] Pattern :: Disjunction
        // It is a Syntax Error if CountLeftCapturingParensWithin(Pattern) ≥ 2**32 - 1.
        //
        // If this is greater than `u32::MAX`, it is memory overflow, though.
        // But I never seen such a gigantic pattern with 4,294,967,295 parens!
        if u32::MAX == self.state.num_of_capturing_groups {
            return Err(OxcDiagnostic::error(
                "Invalid regular expression: Too many capturing groups",
            )
            .with_label(self.span_factory.create(0, 0)));
        }
        // [SS:EE] Pattern :: Disjunction
        // It is a Syntax Error if Pattern contains two or more GroupSpecifiers for which the CapturingGroupName of GroupSpecifier is the same.
        if !duplicated_named_capturing_groups.is_empty() {
            return Err(OxcDiagnostic::error("Invalid regular expression: Duplicated group name")
                .with_labels(
                    duplicated_named_capturing_groups
                        .iter()
                        .map(|(start, end)| self.span_factory.create(*start, *end)),
                ));
        }

        // Let's start parsing!
        let disjunction = self.parse_disjunction()?;

        if self.reader.peek().is_some() {
            let span_start = self.reader.offset();
            return Err(OxcDiagnostic::error(
                "Invalid regular expression: Could not parse the entire pattern",
            )
            .with_label(self.span_factory.create(span_start, span_start)));
        }

        Ok(ast::Pattern {
            span: self.span_factory.create(0, self.source_text.len()),
            body: disjunction,
        })
    }

    // ```
    // Disjunction[UnicodeMode, UnicodeSetsMode, NamedCaptureGroups] ::
    //   Alternative[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    //   Alternative[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] | Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    // ```
    fn parse_disjunction(&mut self) -> Result<ast::Disjunction<'a>> {
        let span_start = self.reader.offset();

        let mut body = Vec::new_in(self.allocator);
        loop {
            body.push(self.parse_alternative()?);

            if !self.reader.eat('|') {
                break;
            }
        }

        Ok(ast::Disjunction {
            span: self.span_factory.create(span_start, self.reader.offset()),
            body,
        })
    }

    // ```
    // Alternative[UnicodeMode, UnicodeSetsMode, NamedCaptureGroups] ::
    //   [empty]
    //   Alternative[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] Term[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups]
    // ```
    fn parse_alternative(&mut self) -> Result<ast::Alternative<'a>> {
        let span_start = self.reader.offset();

        let mut body = Vec::new_in(self.allocator);
        while let Some(term) = self.parse_term()? {
            body.push(term);
        }

        Ok(ast::Alternative {
            span: self.span_factory.create(span_start, self.reader.offset()),
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
    fn parse_term(&mut self) -> Result<Option<ast::Term<'a>>> {
        // [+UnicodeMode] Assertion
        // [+UnicodeMode] Atom Quantifier
        // [+UnicodeMode] Atom
        if self.state.unicode_mode {
            if let Some(assertion) = self.parse_assertion()? {
                return Ok(Some(assertion));
            }

            let span_start = self.reader.offset();
            return match (self.parse_atom()?, self.consume_quantifier()?) {
                (Some(atom), Some(((min, max), greedy))) => {
                    Ok(Some(ast::Term::Quantifier(Box::new_in(
                        ast::Quantifier {
                            span: self.span_factory.create(span_start, self.reader.offset()),
                            greedy,
                            min,
                            max,
                            body: atom,
                        },
                        self.allocator,
                    ))))
                }
                (Some(atom), None) => Ok(Some(atom)),
                (None, Some(_)) => Err(OxcDiagnostic::error(
                    "Invalid regular expression: Lone `Quantifier` found, expected with `Atom`",
                )
                .with_label(self.span_factory.create(span_start, self.reader.offset()))),
                (None, None) => Ok(None),
            };
        }

        // [~UnicodeMode] QuantifiableAssertion Quantifier
        // [~UnicodeMode] Assertion
        // [~UnicodeMode] ExtendedAtom Quantifier
        // [~UnicodeMode] ExtendedAtom
        let span_start = self.reader.offset();
        if let Some(assertion) = self.parse_assertion()? {
            // `QuantifiableAssertion` = (Negative)Lookahead: `(?=...)` or `(?!...)`
            if let ast::Term::LookAroundAssertion(look_around) = &assertion {
                if matches!(
                    look_around.kind,
                    ast::LookAroundAssertionKind::Lookahead
                        | ast::LookAroundAssertionKind::NegativeLookahead
                ) {
                    if let Some(((min, max), greedy)) = self.consume_quantifier()? {
                        return Ok(Some(ast::Term::Quantifier(Box::new_in(
                            ast::Quantifier {
                                span: self.span_factory.create(span_start, self.reader.offset()),
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
            (Some(extended_atom), Some(((min, max), greedy))) => {
                Ok(Some(ast::Term::Quantifier(Box::new_in(
                    ast::Quantifier {
                        span: self.span_factory.create(span_start, self.reader.offset()),
                        min,
                        max,
                        greedy,
                        body: extended_atom,
                    },
                    self.allocator,
                ))))
            }
            (Some(extended_atom), None) => Ok(Some(extended_atom)),
            (None, Some(_)) => Err(OxcDiagnostic::error(
                "Invalid regular expression: Lone `Quantifier` found, expected with `ExtendedAtom`",
            )
            .with_label(self.span_factory.create(span_start, self.reader.offset()))),
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
    fn parse_assertion(&mut self) -> Result<Option<ast::Term<'a>>> {
        let span_start = self.reader.offset();

        let kind = if self.reader.eat('^') {
            Some(ast::BoundaryAssertionKind::Start)
        } else if self.reader.eat('$') {
            Some(ast::BoundaryAssertionKind::End)
        } else if self.reader.eat2('\\', 'b') {
            Some(ast::BoundaryAssertionKind::Boundary)
        } else if self.reader.eat2('\\', 'B') {
            Some(ast::BoundaryAssertionKind::NegativeBoundary)
        } else {
            None
        };

        if let Some(kind) = kind {
            return Ok(Some(ast::Term::BoundaryAssertion(ast::BoundaryAssertion {
                span: self.span_factory.create(span_start, self.reader.offset()),
                kind,
            })));
        }

        let kind = if self.reader.eat3('(', '?', '=') {
            Some(ast::LookAroundAssertionKind::Lookahead)
        } else if self.reader.eat3('(', '?', '!') {
            Some(ast::LookAroundAssertionKind::NegativeLookahead)
        } else if self.reader.eat4('(', '?', '<', '=') {
            Some(ast::LookAroundAssertionKind::Lookbehind)
        } else if self.reader.eat4('(', '?', '<', '!') {
            Some(ast::LookAroundAssertionKind::NegativeLookbehind)
        } else {
            None
        };

        if let Some(kind) = kind {
            let disjunction = self.parse_disjunction()?;

            if !self.reader.eat(')') {
                return Err(OxcDiagnostic::error(
                    "Invalid regular expression: Unterminated lookaround assertion",
                )
                .with_label(self.span_factory.create(span_start, self.reader.offset())));
            }

            return Ok(Some(ast::Term::LookAroundAssertion(Box::new_in(
                ast::LookAroundAssertion {
                    span: self.span_factory.create(span_start, self.reader.offset()),
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
    //   ( GroupSpecifier[?UnicodeMode][opt] Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
    //   (?: Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
    // ```
    fn parse_atom(&mut self) -> Result<Option<ast::Term<'a>>> {
        let span_start = self.reader.offset();

        // PatternCharacter
        if let Some(cp) = self.reader.peek().filter(|&cp| !unicode::is_syntax_character(cp)) {
            self.reader.advance();

            return Ok(Some(ast::Term::Character(ast::Character {
                span: self.span_factory.create(span_start, self.reader.offset()),
                kind: ast::CharacterKind::Symbol,
                value: cp,
            })));
        }

        // .
        if self.reader.eat('.') {
            return Ok(Some(ast::Term::Dot(ast::Dot {
                span: self.span_factory.create(span_start, self.reader.offset()),
            })));
        }

        // \ AtomEscape[?UnicodeMode, ?NamedCaptureGroups]
        if self.reader.eat('\\') {
            if let Some(atom_escape) = self.parse_atom_escape(span_start)? {
                return Ok(Some(atom_escape));
            }
        }

        // CharacterClass[?UnicodeMode, ?UnicodeSetsMode]
        if let Some(character_class) = self.parse_character_class()? {
            return Ok(Some(ast::Term::CharacterClass(Box::new_in(
                character_class,
                self.allocator,
            ))));
        }

        // (?: Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
        if let Some(ignore_group) = self.parse_ignore_group()? {
            return Ok(Some(ast::Term::IgnoreGroup(Box::new_in(ignore_group, self.allocator))));
        }

        // ( GroupSpecifier[?UnicodeMode][opt] Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
        // ( Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
        if let Some(capturing_group) = self.parse_capturing_group()? {
            return Ok(Some(ast::Term::CapturingGroup(Box::new_in(
                capturing_group,
                self.allocator,
            ))));
        }

        Ok(None)
    }

    // ```
    // ExtendedAtom[NamedCaptureGroups] ::
    //   .
    //   \ AtomEscape[~UnicodeMode, ?NamedCaptureGroups]
    //   \ [lookahead = c]
    //   CharacterClass[~UnicodeMode, ~UnicodeSetsMode]
    //   ( GroupSpecifier[~UnicodeMode][opt] Disjunction[~UnicodeMode, ~UnicodeSetsMode, ?NamedCaptureGroups] )
    //   (?: Disjunction[~UnicodeMode, ~UnicodeSetsMode, ?NamedCaptureGroups] )
    //   InvalidBracedQuantifier
    //   ExtendedPatternCharacter
    // ```
    fn parse_extended_atom(&mut self) -> Result<Option<ast::Term<'a>>> {
        let span_start = self.reader.offset();

        // .
        if self.reader.eat('.') {
            return Ok(Some(ast::Term::Dot(ast::Dot {
                span: self.span_factory.create(span_start, self.reader.offset()),
            })));
        }

        if self.reader.eat('\\') {
            // \ AtomEscape[~UnicodeMode, ?NamedCaptureGroups]
            if let Some(atom_escape) = self.parse_atom_escape(span_start)? {
                return Ok(Some(atom_escape));
            }

            // \ [lookahead = c]
            if self.reader.peek().filter(|&cp| cp == 'c' as u32).is_some() {
                return Ok(Some(ast::Term::Character(ast::Character {
                    span: self.span_factory.create(span_start, self.reader.offset()),
                    kind: ast::CharacterKind::Symbol,
                    value: '\\' as u32,
                })));
            }

            return Err(OxcDiagnostic::error("Invalid regular expression: Invalid escape")
                .with_label(self.span_factory.create(span_start, self.reader.offset())));
        }

        // CharacterClass[~UnicodeMode, ~UnicodeSetsMode]
        if let Some(character_class) = self.parse_character_class()? {
            return Ok(Some(ast::Term::CharacterClass(Box::new_in(
                character_class,
                self.allocator,
            ))));
        }

        // (?: Disjunction[~UnicodeMode, ~UnicodeSetsMode, ?NamedCaptureGroups] )
        if let Some(ignore_group) = self.parse_ignore_group()? {
            return Ok(Some(ast::Term::IgnoreGroup(Box::new_in(ignore_group, self.allocator))));
        }

        // ( GroupSpecifier[~UnicodeMode][opt] Disjunction[~UnicodeMode, ~UnicodeSetsMode, ?NamedCaptureGroups] )
        // ( Disjunction[~UnicodeMode, ~UnicodeSetsMode, ?NamedCaptureGroups] )
        if let Some(capturing_group) = self.parse_capturing_group()? {
            return Ok(Some(ast::Term::CapturingGroup(Box::new_in(
                capturing_group,
                self.allocator,
            ))));
        }

        // InvalidBracedQuantifier
        let span_start = self.reader.offset();
        if self.consume_quantifier()?.is_some() {
            // [SS:EE] ExtendedAtom :: InvalidBracedQuantifier
            // It is a Syntax Error if any source text is matched by this production.
            // (Annex B)
            return Err(OxcDiagnostic::error(
                "Invalid regular expression: Invalid braced quantifier",
            )
            .with_label(self.span_factory.create(span_start, self.reader.offset())));
        }

        // ExtendedPatternCharacter
        if let Some(cp) = self.consume_extended_pattern_character() {
            return Ok(Some(ast::Term::Character(ast::Character {
                span: self.span_factory.create(span_start, self.reader.offset()),
                kind: ast::CharacterKind::Symbol,
                value: cp,
            })));
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
    // (Annex B)
    fn parse_atom_escape(&mut self, span_start: usize) -> Result<Option<ast::Term<'a>>> {
        let checkpoint = self.reader.checkpoint();

        // DecimalEscape: \1 means indexed reference
        if let Some(index) = self.consume_decimal_escape() {
            if self.state.unicode_mode {
                // [SS:EE] AtomEscape :: DecimalEscape
                // It is a Syntax Error if the CapturingGroupNumber of DecimalEscape is strictly greater than CountLeftCapturingParensWithin(the Pattern containing AtomEscape).
                if self.state.num_of_capturing_groups < index {
                    return Err(OxcDiagnostic::error(
                        "Invalid regular expression: Invalid indexed reference",
                    )
                    .with_label(self.span_factory.create(span_start, self.reader.offset())));
                }

                return Ok(Some(ast::Term::IndexedReference(ast::IndexedReference {
                    span: self.span_factory.create(span_start, self.reader.offset()),
                    index,
                })));
            }

            if index <= self.state.num_of_capturing_groups {
                return Ok(Some(ast::Term::IndexedReference(ast::IndexedReference {
                    span: self.span_factory.create(span_start, self.reader.offset()),
                    index,
                })));
            }

            self.reader.rewind(checkpoint);
        }

        // CharacterClassEscape: \d, \p{...}
        if let Some(character_class_escape) = self.parse_character_class_escape(span_start) {
            return Ok(Some(ast::Term::CharacterClassEscape(character_class_escape)));
        }
        if let Some(unicode_property_escape) =
            self.parse_character_class_escape_unicode(span_start)?
        {
            return Ok(Some(ast::Term::UnicodePropertyEscape(Box::new_in(
                unicode_property_escape,
                self.allocator,
            ))));
        }

        // CharacterEscape: \n, \cM, \0, etc...
        if let Some(character_escape) = self.parse_character_escape(span_start)? {
            return Ok(Some(ast::Term::Character(character_escape)));
        }

        // k GroupName: \k<name> means named reference
        if self.state.named_capture_groups && self.reader.eat('k') {
            if let Some(name) = self.consume_group_name()? {
                // [SS:EE] AtomEscape :: k GroupName
                // It is a Syntax Error if GroupSpecifiersThatMatch(GroupName) is empty.
                if !self.state.capturing_group_names.contains(name.as_str()) {
                    return Err(OxcDiagnostic::error(
                        "Invalid regular expression: Group specifier is empty",
                    )
                    .with_label(self.span_factory.create(span_start, self.reader.offset())));
                }

                return Ok(Some(ast::Term::NamedReference(Box::new_in(
                    ast::NamedReference {
                        span: self.span_factory.create(span_start, self.reader.offset()),
                        name,
                    },
                    self.allocator,
                ))));
            }

            return Err(OxcDiagnostic::error(
                "Invalid regular expression: Invalid named reference",
            )
            .with_label(self.span_factory.create(span_start, self.reader.offset())));
        }

        Ok(None)
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
            span: self.span_factory.create(span_start, self.reader.offset()),
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
            false
        } else if self.reader.eat('P') {
            true
        } else {
            return Ok(None);
        };

        if self.reader.eat('{') {
            if let Some((name, value, is_strings_related)) =
                self.consume_unicode_property_value_expression()?
            {
                if self.reader.eat('}') {
                    // [SS:EE] CharacterClassEscape :: P{ UnicodePropertyValueExpression }
                    // It is a Syntax Error if MayContainStrings of the UnicodePropertyValueExpression is true.
                    // MayContainStrings is true
                    // - if the UnicodePropertyValueExpression is LoneUnicodePropertyNameOrValue
                    //   - and it is binary property of strings(can be true only with `UnicodeSetsMode`)
                    if negative && is_strings_related {
                        return Err(OxcDiagnostic::error(
                            "Invalid property name(negative + property of strings)",
                        )
                        .with_label(self.span_factory.create(span_start, self.reader.offset())));
                    }

                    return Ok(Some(ast::UnicodePropertyEscape {
                        span: self.span_factory.create(span_start, self.reader.offset()),
                        negative,
                        strings: is_strings_related,
                        name,
                        value,
                    }));
                }
            }
        }

        Err(OxcDiagnostic::error(
            "Invalid regular expression: Unterminated unicode property escape",
        )
        .with_label(self.span_factory.create(span_start, self.reader.offset())))
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
    fn parse_character_escape(&mut self, span_start: usize) -> Result<Option<ast::Character>> {
        // e.g. \n
        if let Some(cp) = self.reader.peek().and_then(unicode::map_control_escape) {
            self.reader.advance();

            return Ok(Some(ast::Character {
                span: self.span_factory.create(span_start, self.reader.offset()),
                kind: ast::CharacterKind::SingleEscape,
                value: cp,
            }));
        }

        // e.g. \cM
        let checkpoint = self.reader.checkpoint();
        if self.reader.eat('c') {
            if let Some(cp) = self.reader.peek().and_then(unicode::map_c_ascii_letter) {
                self.reader.advance();

                return Ok(Some(ast::Character {
                    span: self.span_factory.create(span_start, self.reader.offset()),
                    kind: ast::CharacterKind::ControlLetter,
                    value: cp,
                }));
            }
            self.reader.rewind(checkpoint);
        }

        // e.g. \0
        if self.reader.peek().filter(|&cp| cp == '0' as u32).is_some()
            && self.reader.peek2().filter(|&cp| unicode::is_decimal_digit(cp)).is_none()
        {
            self.reader.advance();

            return Ok(Some(ast::Character {
                span: self.span_factory.create(span_start, self.reader.offset()),
                kind: ast::CharacterKind::Null,
                value: 0x00,
            }));
        }

        // e.g. \x41
        if self.reader.eat('x') {
            if let Some(cp) = self.consume_fixed_hex_digits(2) {
                return Ok(Some(ast::Character {
                    span: self.span_factory.create(span_start, self.reader.offset()),
                    kind: ast::CharacterKind::HexadecimalEscape,
                    value: cp,
                }));
            }
            self.reader.rewind(checkpoint);
        }

        // e.g. \u{1f600}
        if let Some(cp) = self.consume_reg_exp_unicode_escape_sequence(self.state.unicode_mode)? {
            return Ok(Some(ast::Character {
                span: self.span_factory.create(span_start, self.reader.offset()),
                kind: ast::CharacterKind::UnicodeEscape,
                value: cp,
            }));
        }

        // e.g. \18
        if !self.state.unicode_mode {
            if let Some(cp) = self.consume_legacy_octal_escape_sequence() {
                return Ok(Some(ast::Character {
                    span: self.span_factory.create(span_start, self.reader.offset()),
                    kind: ast::CharacterKind::Octal,
                    value: cp,
                }));
            }
        }

        // e.g. \.
        if let Some(cp) = self.consume_identity_escape() {
            return Ok(Some(ast::Character {
                span: self.span_factory.create(span_start, self.reader.offset()),
                kind: ast::CharacterKind::Identifier,
                value: cp,
            }));
        }

        Ok(None)
    }

    // ```
    // CharacterClass[UnicodeMode, UnicodeSetsMode] ::
    //   [ [lookahead ≠ ^] ClassContents[?UnicodeMode, ?UnicodeSetsMode] ]
    //   [^ ClassContents[?UnicodeMode, ?UnicodeSetsMode] ]
    // ```
    fn parse_character_class(&mut self) -> Result<Option<ast::CharacterClass<'a>>> {
        let span_start = self.reader.offset();

        if self.reader.eat('[') {
            let negative = self.reader.eat('^');
            let (kind, body) = self.parse_class_contents()?;

            if self.reader.eat(']') {
                // [SS:EE] CharacterClass :: [^ ClassContents ]
                // It is a Syntax Error if MayContainStrings of the ClassContents is true.
                if negative
                    && body.iter().any(|item| match item {
                        // MayContainStrings is true
                        // - if ClassContents contains UnicodePropertyValueExpression
                        //   - and the UnicodePropertyValueExpression is LoneUnicodePropertyNameOrValue
                        //     - and it is binary property of strings(can be true only with `UnicodeSetsMode`)
                        ast::CharacterClassContents::UnicodePropertyEscape(
                            unicode_property_escape,
                        ) => unicode_property_escape.strings,
                        _ => false,
                    })
                {
                    return Err(OxcDiagnostic::error(
                        "Invalid regular expression: Invalid character class",
                    )
                    .with_label(self.span_factory.create(span_start, self.reader.offset())));
                }

                return Ok(Some(ast::CharacterClass {
                    span: self.span_factory.create(span_start, self.reader.offset()),
                    negative,
                    kind,
                    body,
                }));
            }

            return Err(OxcDiagnostic::error(
                "Invalid regular expression: Unterminated character class",
            )
            .with_label(self.span_factory.create(span_start, self.reader.offset())));
        }

        Ok(None)
    }

    // ```
    // ClassContents[UnicodeMode, UnicodeSetsMode] ::
    //   [empty]
    //   [~UnicodeSetsMode] NonemptyClassRanges[?UnicodeMode]
    //   [+UnicodeSetsMode] ClassSetExpression
    // ```
    fn parse_class_contents(
        &mut self,
    ) -> Result<(ast::CharacterClassContentsKind, Vec<'a, ast::CharacterClassContents<'a>>)> {
        // [empty]
        if self.reader.peek().filter(|&cp| cp == ']' as u32).is_some() {
            return Ok((ast::CharacterClassContentsKind::Union, Vec::new_in(self.allocator)));
        }

        // [+UnicodeSetsMode] ClassSetExpression
        if self.state.unicode_sets_mode {
            return self.parse_class_set_expression();
        }

        // [~UnicodeSetsMode] NonemptyClassRanges[?UnicodeMode]
        self.parse_nonempty_class_ranges()
    }

    // ```
    // NonemptyClassRanges[UnicodeMode] ::
    //   ClassAtom[?UnicodeMode]
    //   ClassAtom[?UnicodeMode] NonemptyClassRangesNoDash[?UnicodeMode]
    //   ClassAtom[?UnicodeMode] - ClassAtom[?UnicodeMode] ClassContents[?UnicodeMode, ~UnicodeSetsMode]
    //
    // NonemptyClassRangesNoDash[UnicodeMode] ::
    //   ClassAtom[?UnicodeMode]
    //   ClassAtomNoDash[?UnicodeMode] NonemptyClassRangesNoDash[?UnicodeMode]
    //   ClassAtomNoDash[?UnicodeMode] - ClassAtom[?UnicodeMode] ClassContents[?UnicodeMode, ~UnicodeSetsMode]
    // ```
    fn parse_nonempty_class_ranges(
        &mut self,
    ) -> Result<(ast::CharacterClassContentsKind, Vec<'a, ast::CharacterClassContents<'a>>)> {
        let mut body = Vec::new_in(self.allocator);

        loop {
            let range_span_start = self.reader.offset();

            let Some(class_atom) = self.parse_class_atom()? else {
                break;
            };

            let span_start = self.reader.offset();
            if !self.reader.eat('-') {
                // ClassAtom[?UnicodeMode]
                body.push(class_atom);
                continue;
            }

            let dash = ast::CharacterClassContents::Character(ast::Character {
                span: self.span_factory.create(span_start, self.reader.offset()),
                kind: ast::CharacterKind::Symbol,
                value: '-' as u32,
            });

            let Some(class_atom_to) = self.parse_class_atom()? else {
                // ClassAtom[?UnicodeMode] NonemptyClassRangesNoDash[?UnicodeMode]
                // => ClassAtom[?UnicodeMode] ClassAtom[?UnicodeMode]
                // => ClassAtom[?UnicodeMode] -
                body.push(class_atom);
                body.push(dash);
                continue;
            };

            // ClassAtom[?UnicodeMode] - ClassAtom[?UnicodeMode] ClassContents[?UnicodeMode, ~UnicodeSetsMode]
            // If both sides are characters, it is a range.
            if let (
                ast::CharacterClassContents::Character(from),
                ast::CharacterClassContents::Character(to),
            ) = (&class_atom, &class_atom_to)
            {
                // [SS:EE] NonemptyClassRanges :: ClassAtom - ClassAtom ClassContents
                // [SS:EE] NonemptyClassRangesNoDash :: ClassAtomNoDash - ClassAtom ClassContents
                // It is a Syntax Error if IsCharacterClass of the first ClassAtom is false, IsCharacterClass of the second ClassAtom is false, and the CharacterValue of the first ClassAtom is strictly greater than the CharacterValue of the second ClassAtom.
                if to.value < from.value {
                    return Err(OxcDiagnostic::error(
                        "Invalid regular expression: Character class range out of order",
                    )
                    .with_label(self.span_factory.create(span_start, self.reader.offset())));
                }

                body.push(ast::CharacterClassContents::CharacterClassRange(Box::new_in(
                    ast::CharacterClassRange {
                        span: from.span.merge(&to.span),
                        min: *from,
                        max: *to,
                    },
                    self.allocator,
                )));
                continue;
            }

            // If not, it is just a union of characters.

            // [SS:EE] NonemptyClassRanges :: ClassAtom - ClassAtom ClassContents
            // [SS:EE] NonemptyClassRangesNoDash :: ClassAtomNoDash - ClassAtom ClassContents
            // It is a Syntax Error if IsCharacterClass of the first ClassAtom is true or IsCharacterClass of the second ClassAtom is true and this production has a [UnicodeMode] parameter.
            // (Annex B)
            if self.state.unicode_mode {
                return Err(OxcDiagnostic::error(
                    "Invalid regular expression: Invalid character class range",
                )
                .with_label(self.span_factory.create(range_span_start, self.reader.offset())));
            }

            body.push(class_atom);
            body.push(dash);
            body.push(class_atom_to);
        }

        // [empty] is already covered by the caller, but for sure
        debug_assert!(!body.is_empty());

        Ok((ast::CharacterClassContentsKind::Union, body))
    }

    // ```
    // ClassAtom[UnicodeMode] ::
    //   -
    //   ClassAtomNoDash[?UnicodeMode]
    // ```
    fn parse_class_atom(&mut self) -> Result<Option<ast::CharacterClassContents<'a>>> {
        let span_start = self.reader.offset();

        if self.reader.eat('-') {
            return Ok(Some(ast::CharacterClassContents::Character(ast::Character {
                span: self.span_factory.create(span_start, self.reader.offset()),
                kind: ast::CharacterKind::Symbol,
                value: '-' as u32,
            })));
        }

        self.parse_class_atom_no_dash()
    }

    // ```
    // ClassAtomNoDash[UnicodeMode, NamedCaptureGroups] ::
    //   SourceCharacter but not one of \ or ] or -
    //   \ ClassEscape[?UnicodeMode, ?NamedCaptureGroups]
    //   \ [lookahead = c]
    // ```
    // (Annex B)
    fn parse_class_atom_no_dash(&mut self) -> Result<Option<ast::CharacterClassContents<'a>>> {
        let span_start = self.reader.offset();

        if let Some(cp) = self
            .reader
            .peek()
            .filter(|&cp| cp != '\\' as u32 && cp != ']' as u32 && cp != '-' as u32)
        {
            self.reader.advance();

            return Ok(Some(ast::CharacterClassContents::Character(ast::Character {
                span: self.span_factory.create(span_start, self.reader.offset()),
                kind: ast::CharacterKind::Symbol,
                value: cp,
            })));
        }

        if self.reader.eat('\\') {
            if self.reader.peek().filter(|&cp| cp == 'c' as u32).is_some() {
                return Ok(Some(ast::CharacterClassContents::Character(ast::Character {
                    span: self.span_factory.create(span_start, self.reader.offset()),
                    kind: ast::CharacterKind::Symbol,
                    value: '\\' as u32,
                })));
            }

            if let Some(class_escape) = self.parse_class_escape(span_start)? {
                return Ok(Some(class_escape));
            }

            return Err(OxcDiagnostic::error("Invalid regular expression: Invalid class atom")
                .with_label(self.span_factory.create(span_start, self.reader.offset())));
        }

        Ok(None)
    }

    // ```
    // ClassEscape[UnicodeMode, NamedCaptureGroups] ::
    //   b
    //   [+UnicodeMode] -
    //   [~UnicodeMode] c ClassControlLetter
    //   CharacterClassEscape[?UnicodeMode]
    //   CharacterEscape[?UnicodeMode, ?NamedCaptureGroups]
    //
    // ClassControlLetter ::
    //   DecimalDigit
    //   _
    // ```
    // (Annex B)
    fn parse_class_escape(
        &mut self,
        span_start: usize,
    ) -> Result<Option<ast::CharacterClassContents<'a>>> {
        // b
        if self.reader.eat('b') {
            return Ok(Some(ast::CharacterClassContents::Character(ast::Character {
                span: self.span_factory.create(span_start, self.reader.offset()),
                kind: ast::CharacterKind::SingleEscape,
                value: 0x08,
            })));
        }

        // [+UnicodeMode] -
        if self.state.unicode_mode && self.reader.eat('-') {
            return Ok(Some(ast::CharacterClassContents::Character(ast::Character {
                span: self.span_factory.create(span_start, self.reader.offset()),
                kind: ast::CharacterKind::Symbol,
                value: '-' as u32,
            })));
        }

        // [~UnicodeMode] c ClassControlLetter
        if !self.state.unicode_mode {
            let checkpoint = self.reader.checkpoint();

            if self.reader.eat('c') {
                if let Some(cp) = self
                    .reader
                    .peek()
                    .filter(|&cp| unicode::is_decimal_digit(cp) || cp == '-' as u32)
                {
                    self.reader.advance();

                    return Ok(Some(ast::CharacterClassContents::Character(ast::Character {
                        span: self.span_factory.create(span_start, self.reader.offset()),
                        kind: ast::CharacterKind::ControlLetter,
                        value: cp,
                    })));
                }

                self.reader.rewind(checkpoint);
            }
        }

        // CharacterClassEscape[?UnicodeMode]
        if let Some(character_class_escape) = self.parse_character_class_escape(span_start) {
            return Ok(Some(ast::CharacterClassContents::CharacterClassEscape(
                character_class_escape,
            )));
        }
        if let Some(unicode_property_escape) =
            self.parse_character_class_escape_unicode(span_start)?
        {
            return Ok(Some(ast::CharacterClassContents::UnicodePropertyEscape(Box::new_in(
                unicode_property_escape,
                self.allocator,
            ))));
        }

        // CharacterEscape[?UnicodeMode, ?NamedCaptureGroups]
        if let Some(character_escape) = self.parse_character_escape(span_start)? {
            return Ok(Some(ast::CharacterClassContents::Character(character_escape)));
        }

        Ok(None)
    }

    // ```
    // ClassSetExpression ::
    //   ClassUnion
    //   ClassIntersection
    //   ClassSubtraction
    // ```
    fn parse_class_set_expression(
        &mut self,
    ) -> Result<(ast::CharacterClassContentsKind, Vec<'a, ast::CharacterClassContents<'a>>)> {
        // ClassUnion :: ClassSetRange ClassUnion[opt]
        if let Some(class_set_range) = self.parse_class_set_range()? {
            return self.parse_class_set_union(class_set_range);
        }

        if let Some(class_set_operand) = self.parse_class_set_operand()? {
            // ClassIntersection
            if self.reader.peek().filter(|&cp| cp == '&' as u32).is_some()
                && self.reader.peek2().filter(|&cp| cp == '&' as u32).is_some()
            {
                return self.parse_class_set_intersection(class_set_operand);
            }
            // ClassSubtraction
            if self.reader.peek().filter(|&cp| cp == '-' as u32).is_some()
                && self.reader.peek2().filter(|&cp| cp == '-' as u32).is_some()
            {
                return self.parse_class_set_subtraction(class_set_operand);
            }

            // ClassUnion :: ClassSetOperand ClassUnion[opt]
            return self.parse_class_set_union(class_set_operand);
        }

        let span_start = self.reader.offset();
        Err(OxcDiagnostic::error(
            "Invalid regular expression: Expected nonempty class set expression",
        )
        .with_label(self.span_factory.create(span_start, self.reader.offset())))
    }

    // ```
    // ClassUnion ::
    //   ClassSetRange ClassUnion[opt]
    //   ClassSetOperand ClassUnion[opt]
    // ```
    fn parse_class_set_union(
        &mut self,
        class_set_range_or_class_set_operand: ast::CharacterClassContents<'a>,
    ) -> Result<(ast::CharacterClassContentsKind, Vec<'a, ast::CharacterClassContents<'a>>)> {
        let mut body = Vec::new_in(self.allocator);
        body.push(class_set_range_or_class_set_operand);

        loop {
            if let Some(class_set_range) = self.parse_class_set_range()? {
                body.push(class_set_range);
                continue;
            }
            if let Some(class_set_operand) = self.parse_class_set_operand()? {
                body.push(class_set_operand);
                continue;
            }

            break;
        }

        Ok((ast::CharacterClassContentsKind::Union, body))
    }

    // ```
    // ClassIntersection ::
    //   ClassSetOperand && [lookahead ≠ &] ClassSetOperand
    //   ClassIntersection && [lookahead ≠ &] ClassSetOperand
    // ```
    fn parse_class_set_intersection(
        &mut self,
        class_set_operand: ast::CharacterClassContents<'a>,
    ) -> Result<(ast::CharacterClassContentsKind, Vec<'a, ast::CharacterClassContents<'a>>)> {
        let mut body = Vec::new_in(self.allocator);
        body.push(class_set_operand);

        loop {
            if self.reader.peek().filter(|&cp| cp == ']' as u32).is_some() {
                break;
            }

            if self.reader.eat2('&', '&') {
                let span_start = self.reader.offset();
                if self.reader.eat('&') {
                    return Err(OxcDiagnostic::error(
                        "Unexpected `&` inside of class interseciton", // spellchecker:disable-line
                    )
                    .with_label(self.span_factory.create(span_start, self.reader.offset())));
                }

                if let Some(class_set_operand) = self.parse_class_set_operand()? {
                    body.push(class_set_operand);
                    continue;
                }
            }

            let span_start = self.reader.offset();
            return Err(OxcDiagnostic::error(
                "Invalid character in character class set interseciton", // spellchecker:disable-line
            )
            .with_label(self.span_factory.create(span_start, self.reader.offset())));
        }

        Ok((ast::CharacterClassContentsKind::Intersection, body))
    }

    // ```
    // ClassSubtraction ::
    //   ClassSetOperand -- ClassSetOperand
    //   ClassSubtraction -- ClassSetOperand
    // ```
    fn parse_class_set_subtraction(
        &mut self,
        class_set_operand: ast::CharacterClassContents<'a>,
    ) -> Result<(ast::CharacterClassContentsKind, Vec<'a, ast::CharacterClassContents<'a>>)> {
        let mut body = Vec::new_in(self.allocator);
        body.push(class_set_operand);

        loop {
            if self.reader.peek().filter(|&cp| cp == ']' as u32).is_some() {
                break;
            }

            if self.reader.eat2('-', '-') {
                if let Some(class_set_operand) = self.parse_class_set_operand()? {
                    body.push(class_set_operand);
                    continue;
                }
            }

            let span_start = self.reader.offset();
            return Err(OxcDiagnostic::error(
                "Invalid character in character class set subtraction",
            )
            .with_label(self.span_factory.create(span_start, self.reader.offset())));
        }

        Ok((ast::CharacterClassContentsKind::Subtraction, body))
    }

    // ```
    // ClassSetRange ::
    //   ClassSetCharacter - ClassSetCharacter
    // ```
    fn parse_class_set_range(&mut self) -> Result<Option<ast::CharacterClassContents<'a>>> {
        let checkpoint = self.reader.checkpoint();

        if let Some(class_set_character) = self.parse_class_set_character()? {
            if self.reader.eat('-') {
                if let Some(class_set_character_to) = self.parse_class_set_character()? {
                    // [SS:EE] ClassSetRange :: ClassSetCharacter - ClassSetCharacter
                    // It is a Syntax Error if the CharacterValue of the first ClassSetCharacter is strictly greater than the CharacterValue of the second ClassSetCharacter.
                    if class_set_character_to.value < class_set_character.value {
                        return Err(OxcDiagnostic::error(
                            "Invalid regular expression: Character set class range out of order",
                        )
                        .with_label(class_set_character.span.merge(&class_set_character_to.span)));
                    }

                    return Ok(Some(ast::CharacterClassContents::CharacterClassRange(
                        Box::new_in(
                            ast::CharacterClassRange {
                                span: class_set_character.span.merge(&class_set_character_to.span),
                                min: class_set_character,
                                max: class_set_character_to,
                            },
                            self.allocator,
                        ),
                    )));
                }
            }
        }
        self.reader.rewind(checkpoint);

        Ok(None)
    }

    // ```
    // ClassSetOperand ::
    //   NestedClass
    //   ClassStringDisjunction
    //   ClassSetCharacter
    //
    // ClassStringDisjunction ::
    //   \q{ ClassStringDisjunctionContents }
    // ```
    fn parse_class_set_operand(&mut self) -> Result<Option<ast::CharacterClassContents<'a>>> {
        if let Some(nested_class) = self.parse_nested_class()? {
            return Ok(Some(nested_class));
        }

        let span_start = self.reader.offset();
        if self.reader.eat3('\\', 'q', '{') {
            let (class_string_disjunction_contents, strings) =
                self.parse_class_string_disjunction_contents()?;

            if self.reader.eat('}') {
                return Ok(Some(ast::CharacterClassContents::ClassStringDisjunction(Box::new_in(
                    ast::ClassStringDisjunction {
                        span: self.span_factory.create(span_start, self.reader.offset()),
                        strings,
                        body: class_string_disjunction_contents,
                    },
                    self.allocator,
                ))));
            }

            return Err(OxcDiagnostic::error(
                "Invalid regular expression: Unterminated class string disjunction",
            )
            .with_label(self.span_factory.create(span_start, self.reader.offset())));
        }

        if let Some(class_set_character) = self.parse_class_set_character()? {
            return Ok(Some(ast::CharacterClassContents::Character(class_set_character)));
        }

        Ok(None)
    }

    // ```
    // NestedClass ::
    //   [ [lookahead ≠ ^] ClassContents[+UnicodeMode, +UnicodeSetsMode] ]
    //   [^ ClassContents[+UnicodeMode, +UnicodeSetsMode] ]
    //   \ CharacterClassEscape[+UnicodeMode]
    // ```
    fn parse_nested_class(&mut self) -> Result<Option<ast::CharacterClassContents<'a>>> {
        let span_start = self.reader.offset();

        // [ [lookahead ≠ ^] ClassContents[+UnicodeMode, +UnicodeSetsMode] ]
        // [^ ClassContents[+UnicodeMode, +UnicodeSetsMode] ]
        if self.reader.eat('[') {
            let negative = self.reader.eat('^');
            let (kind, body) = self.parse_class_contents()?;

            if self.reader.eat(']') {
                // [SS:EE] NestedClass :: [^ ClassContents ]
                // It is a Syntax Error if MayContainStrings of the ClassContents is true.
                if negative {
                    let may_contain_strings = |item: &ast::CharacterClassContents| match item {
                        // MayContainStrings is true
                        // - if ClassContents contains UnicodePropertyValueExpression
                        //   - && UnicodePropertyValueExpression is LoneUnicodePropertyNameOrValue
                        //     - && it is binary property of strings(can be true only with `UnicodeSetsMode`)
                        ast::CharacterClassContents::UnicodePropertyEscape(
                            unicode_property_escape,
                        ) => unicode_property_escape.strings,
                        // MayContainStrings is true
                        // - if ClassStringDisjunction is [empty]
                        // - || if ClassStringDisjunction contains ClassString
                        //   - && ClassString is [empty]
                        //   - || ClassString contains 2 more ClassSetCharacters
                        ast::CharacterClassContents::ClassStringDisjunction(
                            class_string_disjunction,
                        ) => class_string_disjunction.strings,
                        _ => false,
                    };

                    if match kind {
                        // MayContainStrings is true
                        // - if ClassContents is ClassUnion
                        //   - && ClassUnion has ClassOperands
                        //     - && at least 1 ClassOperand has MayContainStrings: true
                        ast::CharacterClassContentsKind::Union => {
                            body.iter().any(|item| may_contain_strings(item))
                        }
                        // MayContainStrings is true
                        // - if ClassContents is ClassIntersection
                        //   - && ClassIntersection has ClassOperands
                        //     - && all ClassOperands have MayContainStrings: true
                        ast::CharacterClassContentsKind::Intersection => {
                            body.iter().all(|item| may_contain_strings(item))
                        }
                        // MayContainStrings is true
                        // - if ClassContents is ClassSubtraction
                        //   - && ClassSubtraction has ClassOperands
                        //     - && the first ClassOperand has MayContainStrings: true
                        ast::CharacterClassContentsKind::Subtraction => {
                            body.iter().next().map_or(false, |item| may_contain_strings(item))
                        }
                    } {
                        return Err(OxcDiagnostic::error(
                            "Invalid regular expression: Invalid character class",
                        )
                        .with_label(self.span_factory.create(span_start, self.reader.offset())));
                    }
                }

                return Ok(Some(ast::CharacterClassContents::NestedCharacterClass(Box::new_in(
                    ast::CharacterClass {
                        span: self.span_factory.create(span_start, self.reader.offset()),
                        negative,
                        kind,
                        body,
                    },
                    self.allocator,
                ))));
            }

            return Err(OxcDiagnostic::error(
                "Invalid regular expression: Unterminated nested class",
            )
            .with_label(self.span_factory.create(span_start, self.reader.offset())));
        }

        // \ CharacterClassEscape[+UnicodeMode]
        let span_start = self.reader.offset();
        let checkpoint = self.reader.checkpoint();
        if self.reader.eat('\\') {
            if let Some(character_class_escape) = self.parse_character_class_escape(span_start) {
                return Ok(Some(ast::CharacterClassContents::CharacterClassEscape(
                    character_class_escape,
                )));
            }
            if let Some(unicode_property_escape) =
                self.parse_character_class_escape_unicode(span_start)?
            {
                return Ok(Some(ast::CharacterClassContents::UnicodePropertyEscape(Box::new_in(
                    unicode_property_escape,
                    self.allocator,
                ))));
            }

            self.reader.rewind(checkpoint);
        }

        Ok(None)
    }

    // ```
    // ClassStringDisjunctionContents ::
    //   ClassString
    //   ClassString | ClassStringDisjunctionContents
    // ```
    // Returns: (ClassStringDisjunctionContents, contain_strings)
    fn parse_class_string_disjunction_contents(
        &mut self,
    ) -> Result<(Vec<'a, ast::ClassString<'a>>, bool)> {
        let mut body = Vec::new_in(self.allocator);
        let mut strings = false;

        loop {
            let (class_string, contain_strings) = self.parse_class_string()?;
            body.push(class_string);
            if contain_strings {
                strings = true;
            }

            if !self.reader.eat('|') {
                break;
            }
        }

        if body.is_empty() {
            strings = true;
        }

        Ok((body, strings))
    }

    // ```
    // ClassString ::
    //   [empty]
    //   NonEmptyClassString
    //
    // NonEmptyClassString ::
    //   ClassSetCharacter NonEmptyClassString[opt]
    // ```
    // Returns (ClassString, contain_strings)
    fn parse_class_string(&mut self) -> Result<(ast::ClassString<'a>, bool)> {
        let span_start = self.reader.offset();

        let mut body = Vec::new_in(self.allocator);
        while let Some(class_set_character) = self.parse_class_set_character()? {
            body.push(class_set_character);
        }

        // True if empty or contains 2 or more characters
        let contain_strings = body.len() != 1;

        Ok((
            ast::ClassString {
                span: self.span_factory.create(span_start, self.reader.offset()),
                body,
            },
            contain_strings,
        ))
    }

    // ```
    // ClassSetCharacter ::
    //   [lookahead ∉ ClassSetReservedDoublePunctuator] SourceCharacter but not ClassSetSyntaxCharacter
    //   \ CharacterEscape[+UnicodeMode]
    //   \ ClassSetReservedPunctuator
    //   \b
    // ```
    fn parse_class_set_character(&mut self) -> Result<Option<ast::Character>> {
        let span_start = self.reader.offset();

        if let (Some(cp1), Some(cp2)) = (self.reader.peek(), self.reader.peek2()) {
            if !unicode::is_class_set_reserved_double_punctuator(cp1, cp2)
                && !unicode::is_class_set_syntax_character(cp1)
            {
                self.reader.advance();

                return Ok(Some(ast::Character {
                    span: self.span_factory.create(span_start, self.reader.offset()),
                    kind: ast::CharacterKind::Symbol,
                    value: cp1,
                }));
            }
        }

        let checkpoint = self.reader.checkpoint();
        if self.reader.eat('\\') {
            if let Some(character_escape) = self.parse_character_escape(span_start)? {
                return Ok(Some(character_escape));
            }

            if let Some(cp) =
                self.reader.peek().filter(|&cp| unicode::is_class_set_reserved_punctuator(cp))
            {
                self.reader.advance();
                return Ok(Some(ast::Character {
                    span: self.span_factory.create(span_start, self.reader.offset()),
                    kind: ast::CharacterKind::Identifier,
                    value: cp,
                }));
            }

            if self.reader.eat('b') {
                return Ok(Some(ast::Character {
                    span: self.span_factory.create(span_start, self.reader.offset()),
                    kind: ast::CharacterKind::SingleEscape,
                    value: 0x08,
                }));
            }

            self.reader.rewind(checkpoint);
        }

        Ok(None)
    }

    // ```
    // ( GroupSpecifier[?UnicodeMode][opt] Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
    //
    // GroupSpecifier[UnicodeMode] ::
    //   ? GroupName[?UnicodeMode]
    // ```
    fn parse_capturing_group(&mut self) -> Result<Option<ast::CapturingGroup<'a>>> {
        let span_start = self.reader.offset();

        if self.reader.eat('(') {
            let mut group_name = None;

            // GroupSpecifier is optional, but if it exists, `?` is also required
            if self.reader.eat('?') {
                let Some(name) = self.consume_group_name()? else {
                    return Err(OxcDiagnostic::error(
                        "Invalid regular expression: Capturing group name is missing",
                    )
                    .with_label(self.span_factory.create(span_start, self.reader.offset())));
                };
                group_name = Some(name);
            }

            let disjunction = self.parse_disjunction()?;
            if self.reader.eat(')') {
                return Ok(Some(ast::CapturingGroup {
                    span: self.span_factory.create(span_start, self.reader.offset()),
                    name: group_name,
                    body: disjunction,
                }));
            }

            return Err(OxcDiagnostic::error(
                "Invalid regular expression: Unterminated capturing group",
            )
            .with_label(self.span_factory.create(span_start, self.reader.offset())));
        }

        Ok(None)
    }

    // ```
    // (?: Disjunction[?UnicodeMode, ?UnicodeSetsMode, ?NamedCaptureGroups] )
    // ```
    fn parse_ignore_group(&mut self) -> Result<Option<ast::IgnoreGroup<'a>>> {
        let span_start = self.reader.offset();

        if self.reader.eat3('(', '?', ':') {
            let disjunction = self.parse_disjunction()?;

            if !self.reader.eat(')') {
                return Err(OxcDiagnostic::error(
                    "Invalid regular expression: Unterminated ignore group",
                )
                .with_label(self.span_factory.create(span_start, self.reader.offset())));
            }

            return Ok(Some(ast::IgnoreGroup {
                span: self.span_factory.create(span_start, self.reader.offset()),
                // TODO: Stage3 ModifierFlags
                enabling_modifiers: None,
                disabling_modifiers: None,
                body: disjunction,
            }));
        }

        Ok(None)
    }

    // ---

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
    fn consume_quantifier(&mut self) -> Result<Option<((u32, Option<u32>), bool)>> {
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

        let span_start = self.reader.offset();
        let checkpoint = self.reader.checkpoint();
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
                                // [SS:EE] QuantifierPrefix :: { DecimalDigits , DecimalDigits }
                                // It is a Syntax Error if the MV of the first DecimalDigits is strictly greater than the MV of the second DecimalDigits.
                                return Err(OxcDiagnostic::error(
                                    "Numbers out of order in braced quantifier",
                                )
                                .with_label(
                                    self.span_factory.create(span_start, self.reader.offset()),
                                ));
                            }

                            return Ok(Some(((min, Some(max)), is_greedy(&mut self.reader))));
                        }
                    }
                }
            }

            self.reader.rewind(checkpoint);
        }

        Ok(None)
    }

    // ```
    // DecimalEscape ::
    //   NonZeroDigit DecimalDigits[~Sep][opt] [lookahead ∉ DecimalDigit]
    // ```
    fn consume_decimal_escape(&mut self) -> Option<u32> {
        let checkpoint = self.reader.checkpoint();

        if let Some(index) = self.consume_decimal_digits() {
            // \0 is CharacterEscape, not DecimalEscape
            if index != 0 {
                return Some(index);
            }

            self.reader.rewind(checkpoint);
        }

        None
    }

    // ```
    // DecimalDigits[Sep] ::
    //   DecimalDigit
    //   DecimalDigits[?Sep] DecimalDigit
    //   [+Sep] DecimalDigits[+Sep] NumericLiteralSeparator DecimalDigit
    // ```
    // ([Sep] is disabled for `QuantifierPrefix` and `DecimalEscape`, skip it)
    fn consume_decimal_digits(&mut self) -> Option<u32> {
        let checkpoint = self.reader.checkpoint();

        let mut value = 0;
        while let Some(cp) = self.reader.peek().filter(|&cp| unicode::is_decimal_digit(cp)) {
            // `- '0' as u32`: convert code point to digit
            value = (10 * value) + (cp - '0' as u32);
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
    /// Returns: `(name, Option<value>, is_strings_related_unicode_property)`
    fn consume_unicode_property_value_expression(
        &mut self,
    ) -> Result<Option<(SpanAtom<'a>, Option<SpanAtom<'a>>, bool)>> {
        let checkpoint = self.reader.checkpoint();

        // UnicodePropertyName=UnicodePropertyValue
        if let Some(name) = self.consume_unicode_property_name() {
            if self.reader.eat('=') {
                let span_start = self.reader.offset();

                if let Some(value) = self.consume_unicode_property_value() {
                    // [SS:EE] UnicodePropertyValueExpression :: UnicodePropertyName = UnicodePropertyValue
                    // It is a Syntax Error if the source text matched by UnicodePropertyName is not a Unicode property name or property alias listed in the “Property name and aliases” column of Table 65.
                    // [SS:EE] UnicodePropertyValueExpression :: UnicodePropertyName = UnicodePropertyValue
                    // It is a Syntax Error if the source text matched by UnicodePropertyValue is not a property value or property value alias for the Unicode property or property alias given by the source text matched by UnicodePropertyName listed in PropertyValueAliases.txt.
                    if !unicode_property::is_valid_unicode_property(&name, &value) {
                        return Err(OxcDiagnostic::error(
                            "Invalid regular expression: Invalid unicode property name",
                        )
                        .with_label(self.span_factory.create(span_start, self.reader.offset())));
                    }

                    return Ok(Some((name, Some(value), false)));
                }
            }
        }
        self.reader.rewind(checkpoint);

        let span_start = self.reader.offset();
        // LoneUnicodePropertyNameOrValue
        if let Some(name_or_value) = self.consume_unicode_property_value() {
            // [SS:EE] UnicodePropertyValueExpression :: LoneUnicodePropertyNameOrValue
            // It is a Syntax Error if the source text matched by LoneUnicodePropertyNameOrValue is not a Unicode property value or property value alias for the General_Category (gc) property listed in PropertyValueAliases.txt, nor a binary property or binary property alias listed in the “Property name and aliases” column of Table 66, nor a binary property of strings listed in the “Property name” column of Table 67.
            if unicode_property::is_valid_unicode_property("General_Category", &name_or_value) {
                return Ok(Some(("General_Category".into(), Some(name_or_value), false)));
            }
            if unicode_property::is_valid_lone_unicode_property(&name_or_value) {
                return Ok(Some((name_or_value, None, false)));
            }
            // [SS:EE] UnicodePropertyValueExpression :: LoneUnicodePropertyNameOrValue
            // It is a Syntax Error if the enclosing Pattern does not have a [UnicodeSetsMode] parameter and the source text matched by LoneUnicodePropertyNameOrValue is a binary property of strings listed in the “Property name” column of Table 67.
            if unicode_property::is_valid_lone_unicode_property_of_strings(&name_or_value) {
                if !self.state.unicode_sets_mode {
                    return Err(OxcDiagnostic::error(
                        "`UnicodeSetsMode` is required for binary property of strings",
                    )
                    .with_label(self.span_factory.create(span_start, self.reader.offset())));
                }

                return Ok(Some((name_or_value, None, true)));
            }

            return Err(OxcDiagnostic::error(
                "Invalid regular expression: Invalid unicode property name or value",
            )
            .with_label(self.span_factory.create(span_start, self.reader.offset())));
        }

        Ok(None)
    }

    fn consume_unicode_property_name(&mut self) -> Option<SpanAtom<'a>> {
        let span_start = self.reader.offset();

        let checkpoint = self.reader.checkpoint();
        while unicode::is_unicode_property_name_character(self.reader.peek()?) {
            self.reader.advance();
        }

        if checkpoint == self.reader.checkpoint() {
            return None;
        }

        Some(SpanAtom::from(&self.source_text[span_start..self.reader.offset()]))
    }

    fn consume_unicode_property_value(&mut self) -> Option<SpanAtom<'a>> {
        let span_start = self.reader.offset();

        let checkpoint = self.reader.checkpoint();
        while unicode::is_unicode_property_value_character(self.reader.peek()?) {
            self.reader.advance();
        }

        if checkpoint == self.reader.checkpoint() {
            return None;
        }

        Some(SpanAtom::from(&self.source_text[span_start..self.reader.offset()]))
    }

    // ```
    // GroupName[UnicodeMode] ::
    //   < RegExpIdentifierName[?UnicodeMode] >
    // ```
    fn consume_group_name(&mut self) -> Result<Option<SpanAtom<'a>>> {
        let span_start = self.reader.offset();

        if !self.reader.eat('<') {
            return Ok(None);
        }

        if let Some(group_name) = self.consume_reg_exp_idenfigier_name()? {
            if self.reader.eat('>') {
                return Ok(Some(group_name));
            }
        }

        Err(OxcDiagnostic::error("Invalid regular expression: Unterminated capturing group name")
            .with_label(self.span_factory.create(span_start, self.reader.offset())))
    }

    // ```
    // RegExpIdentifierName[UnicodeMode] ::
    //   RegExpIdentifierStart[?UnicodeMode]
    //   RegExpIdentifierName[?UnicodeMode] RegExpIdentifierPart[?UnicodeMode]
    // ```
    fn consume_reg_exp_idenfigier_name(&mut self) -> Result<Option<SpanAtom<'a>>> {
        let span_start = self.reader.offset();

        if self.consume_reg_exp_idenfigier_start()?.is_some() {
            while self.consume_reg_exp_idenfigier_part()?.is_some() {}

            return Ok(Some(SpanAtom::from(&self.source_text[span_start..self.reader.offset()])));
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
        if let Some(cp) = self.reader.peek().filter(|&cp| unicode::is_identifier_start_char(cp)) {
            self.reader.advance();
            return Ok(Some(cp));
        }

        let span_start = self.reader.offset();
        if self.reader.eat('\\') {
            if let Some(cp) = self.consume_reg_exp_unicode_escape_sequence(true)? {
                // [SS:EE] RegExpIdentifierStart :: \ RegExpUnicodeEscapeSequence
                // It is a Syntax Error if the CharacterValue of RegExpUnicodeEscapeSequence is not the numeric value of some code point matched by the IdentifierStartChar lexical grammar production.
                if !unicode::is_identifier_start_char(cp) {
                    return Err(OxcDiagnostic::error(
                        "Invalid regular expression: Invalid unicode escape sequence",
                    )
                    .with_label(self.span_factory.create(span_start, self.reader.offset())));
                }

                return Ok(Some(cp));
            }

            return Err(OxcDiagnostic::error(
                "Invalid regular expression: Invalid unicode escape sequence",
            )
            .with_label(self.span_factory.create(span_start, self.reader.offset())));
        }

        if !self.state.unicode_mode {
            let span_start = self.reader.offset();

            if let Some(lead_surrogate) =
                self.reader.peek().filter(|&cp| unicode::is_lead_surrogate(cp))
            {
                if let Some(trail_surrogate) =
                    self.reader.peek2().filter(|&cp| unicode::is_trail_surrogate(cp))
                {
                    self.reader.advance();
                    self.reader.advance();
                    let cp = unicode::combine_surrogate_pair(lead_surrogate, trail_surrogate);

                    // [SS:EE] RegExpIdentifierStart :: UnicodeLeadSurrogate UnicodeTrailSurrogate
                    // It is a Syntax Error if the RegExpIdentifierCodePoint of RegExpIdentifierStart is not matched by the UnicodeIDStart lexical grammar production.
                    if !unicode::is_unicode_id_start(cp) {
                        return Err(OxcDiagnostic::error(
                            "Invalid regular expression: Invalid surrogate pair",
                        )
                        .with_label(self.span_factory.create(span_start, self.reader.offset())));
                    }

                    return Ok(Some(cp));
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
    fn consume_reg_exp_idenfigier_part(&mut self) -> Result<Option<u32>> {
        if let Some(cp) = self.reader.peek() {
            if unicode::is_identifier_part_char(cp) {
                self.reader.advance();
                return Ok(Some(cp));
            }
        }

        let span_start = self.reader.offset();
        if self.reader.eat('\\') {
            if let Some(cp) = self.consume_reg_exp_unicode_escape_sequence(true)? {
                // [SS:EE] RegExpIdentifierPart :: \ RegExpUnicodeEscapeSequence
                // It is a Syntax Error if the CharacterValue of RegExpUnicodeEscapeSequence is not the numeric value of some code point matched by the IdentifierPartChar lexical grammar production.
                if !unicode::is_identifier_part_char(cp) {
                    return Err(OxcDiagnostic::error(
                        "Invalid regular expression: Invalid unicode escape sequence",
                    )
                    .with_label(self.span_factory.create(span_start, self.reader.offset())));
                }

                return Ok(Some(cp));
            }

            return Err(OxcDiagnostic::error(
                "Invalid regular expression: Invalid unicode escape sequence",
            )
            .with_label(self.span_factory.create(span_start, self.reader.offset())));
        }

        if !self.state.unicode_mode {
            let span_start = self.reader.offset();

            if let Some(lead_surrogate) =
                self.reader.peek().filter(|&cp| unicode::is_lead_surrogate(cp))
            {
                if let Some(trail_surrogate) =
                    self.reader.peek2().filter(|&cp| unicode::is_trail_surrogate(cp))
                {
                    self.reader.advance();
                    self.reader.advance();

                    let cp = unicode::combine_surrogate_pair(lead_surrogate, trail_surrogate);
                    // [SS:EE] RegExpIdentifierPart :: UnicodeLeadSurrogate UnicodeTrailSurrogate
                    // It is a Syntax Error if the RegExpIdentifierCodePoint of RegExpIdentifierPart is not matched by the UnicodeIDContinue lexical grammar production.
                    if !unicode::is_unicode_id_continue(cp) {
                        return Err(OxcDiagnostic::error(
                            "Invalid regular expression: Invalid surrogate pair",
                        )
                        .with_label(self.span_factory.create(span_start, self.reader.offset())));
                    }

                    return Ok(Some(cp));
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
    fn consume_reg_exp_unicode_escape_sequence(
        &mut self,
        unicode_mode: bool,
    ) -> Result<Option<u32>> {
        let span_start = self.reader.offset();
        let checkpoint = self.reader.checkpoint();

        if self.reader.eat('u') {
            if unicode_mode {
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
            if unicode_mode {
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

            if self.state.unicode_mode {
                return Err(OxcDiagnostic::error(
                    "Invalid regular expression: Invalid unicode escape sequence",
                )
                .with_label(self.span_factory.create(span_start, self.reader.offset())));
            }
            self.reader.rewind(checkpoint);
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
    // IdentityEscape[UnicodeMode, NamedCaptureGroups] ::
    //   [+UnicodeMode] SyntaxCharacter
    //   [+UnicodeMode] /
    //   [~UnicodeMode] SourceCharacterIdentityEscape[?NamedCaptureGroups]
    //
    // SourceCharacterIdentityEscape[NamedCaptureGroups] ::
    //   [~NamedCaptureGroups] SourceCharacter but not c
    //   [+NamedCaptureGroups] SourceCharacter but not one of c or k
    // ```
    // (Annex B)
    fn consume_identity_escape(&mut self) -> Option<u32> {
        let cp = self.reader.peek()?;

        if self.state.unicode_mode {
            if unicode::is_syntax_character(cp) || cp == '/' as u32 {
                self.reader.advance();
                return Some(cp);
            }
            return None;
        }

        if self.state.named_capture_groups {
            if cp != 'c' as u32 && cp != 'k' as u32 {
                self.reader.advance();
                return Some(cp);
            }
            return None;
        }

        if cp != 'c' as u32 {
            self.reader.advance();
            return Some(cp);
        }

        None
    }

    // ```
    // ExtendedPatternCharacter ::
    //   SourceCharacter but not one of ^ $ \ . * + ? ( ) [ |
    // ```
    fn consume_extended_pattern_character(&mut self) -> Option<u32> {
        let cp = self.reader.peek()?;

        if cp == '^' as u32
            || cp == '$' as u32
            || cp == '\\' as u32
            || cp == '.' as u32
            || cp == '*' as u32
            || cp == '+' as u32
            || cp == '?' as u32
            || cp == '(' as u32
            || cp == ')' as u32
            || cp == '[' as u32
            || cp == '|' as u32
        {
            return None;
        }

        self.reader.advance();
        Some(cp)
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
