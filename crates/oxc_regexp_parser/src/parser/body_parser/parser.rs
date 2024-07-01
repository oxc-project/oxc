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
            reader: Reader::new(source_text, options.unicode_mode),
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
            (Some(atom), Some(quantifier)) => {
                return Ok(Some(ast::Element::Quantifier(Box::new_in(
                    ast::Quantifier {
                        span: self.span_factory.create(start, self.reader.position()),
                        min: quantifier.0 .0,
                        max: quantifier.0 .1,
                        greedy: quantifier.1,
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
    fn consume_quantifier(&mut self) -> Result<Option<((usize, usize), bool)>> {
        let is_greedy = |reader: &mut Reader| !reader.eat('?');

        if self.reader.eat('*') {
            return Ok(Some((
                (0, usize::MAX), // TODO: POSITIVE_INFINITY?
                is_greedy(&mut self.reader),
            )));
        }
        if self.reader.eat('+') {
            return Ok(Some((
                (1, usize::MAX), // TODO: POSITIVE_INFINITY?
                is_greedy(&mut self.reader),
            )));
        }
        if self.reader.eat('?') {
            return Ok(Some(((0, 1), is_greedy(&mut self.reader))));
        }

        // TODO: Implement
        if self.reader.eat('{') {
            // if (this.eatDecimalDigits()) {
            //   const min = this._lastIntValue;
            //   let max = min;
            //   if (this.eat(COMMA)) {
            //     max = this.eatDecimalDigits()
            //       ? this._lastIntValue
            //       : Number.POSITIVE_INFINITY;
            //   }
            //   if (this.eat(RIGHT_CURLY_BRACKET)) {
            //     if (!noError && max < min) {
            //       this.raise("numbers out of order in {} quantifier");
            //     }
            //     return { min, max } + greedy;
            //     return true;
            //   }
            // }

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
}

#[cfg(test)]
mod test {
    use super::*;
    use oxc_allocator::Allocator;

    // NOTE: These may be useless when integlation tests are added
    #[test]
    fn should_pass() {
        let allocator = Allocator::default();

        let pattern =
            PatternParser::new(&allocator, "abc", ParserOptions::default()).parse().unwrap();
        assert_eq!(pattern.alternatives.len(), 1);
        assert_eq!(pattern.alternatives[0].elements.len(), 3);

        let pattern =
            PatternParser::new(&allocator, "a|b|", ParserOptions::default()).parse().unwrap();
        assert_eq!(pattern.alternatives.len(), 3);
        let pattern =
            PatternParser::new(&allocator, "a|b+?|c", ParserOptions::default()).parse().unwrap();
        assert_eq!(pattern.alternatives.len(), 3);

        let pattern =
            PatternParser::new(&allocator, "Emoji🥹", ParserOptions::default()).parse().unwrap();
        assert_eq!(pattern.alternatives[0].elements.len(), 7);
        let pattern = PatternParser::new(
            &allocator,
            "Emoji🥹",
            ParserOptions::default().with_modes(true, false),
        )
        .parse()
        .unwrap();
        assert_eq!(pattern.alternatives[0].elements.len(), 6);
    }

    #[test]
    fn should_fail() {
        let allocator = Allocator::default();

        assert!(PatternParser::new(&allocator, "", ParserOptions::default()).parse().is_err());
        assert!(PatternParser::new(&allocator, "a)", ParserOptions::default()).parse().is_err());
        assert!(PatternParser::new(&allocator, "b\\", ParserOptions::default()).parse().is_err());
        assert!(PatternParser::new(&allocator, "c]", ParserOptions::default()).parse().is_err());
        assert!(PatternParser::new(&allocator, "d}", ParserOptions::default()).parse().is_err());
        assert!(PatternParser::new(&allocator, "e|+", ParserOptions::default()).parse().is_err());
        assert!(PatternParser::new(&allocator, "e|{", ParserOptions::default()).parse().is_err());
    }
}
