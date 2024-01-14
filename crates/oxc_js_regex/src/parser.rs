use std::collections::{HashSet, VecDeque};
use std::iter::Peekable;
use std::ops::Range;
use std::os::unix::fs::OpenOptionsExt;
use std::panic;
use std::str::{CharIndices, Chars, Matches};

use oxc_diagnostics::Error;
use oxc_span::Span;

use crate::ast::{
    Alternative, Assertion, Backreference, BackreferenceRef, BoundaryAssertion, Branch,
    CapturingGroup, Character, EdgeAssertion, EdgeAssertionKind, Element, LookaheadAssertion,
    LookaroundAssertion, LookbehindAssertion, Pattern, QuantifiableElement, Quantifier,
    RegExpLiteral, WordBoundaryAssertion,
};
use crate::ast_builder::AstBuilder;
use crate::ecma_version::EcmaVersion;

pub struct Lexer<'a> {
    source: &'a str,
    /// Regex usually, use a collected `Vec` could reduce lookahead and other util function implementation complexity
    chars: Vec<char>,

    pub(crate) errors: Vec<Error>,
}

#[allow(clippy::unused_self)]
impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, errors: vec![], chars: source.chars().collect::<Vec<_>>() }
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    builder: AstBuilder<'a>,

    /// Source Code
    source_text: &'a str,

    /// All syntax errors from parser and lexer
    /// Note: favor adding to `Diagnostics` instead of raising Err
    errors: Vec<Error>,
    context: ParserContext,
    index: usize,
    group_names: HashSet<String>,
    num_capturing_parens: usize,
    last_int_value: usize,
    back_reference_names: HashSet<String>,
    last_assertion_is_quantifiable: bool,
    last_range: Range<usize>,
    last_str_value: Stirng,
}

#[derive(Default, Copy, Clone)]
struct ParserContext {
    source_kind: SourceKind,
    unicode_mode: bool,
    nflag: bool,
    unicode_sets_mode: bool,
    ecma_version: EcmaVersion,
    strict: bool,
}

impl<'a> Parser<'a> {
    /// Create a new parser
    pub fn new(allocator: &'a oxc_allocator::Allocator, source_text: &'a str) -> Self {
        Self {
            lexer: Lexer::new(source_text),
            source_text,
            errors: vec![],
            context: ParserContext::default(),
            index: 0,
            group_names: HashSet::new(),
            num_capturing_parens: 0,
            back_reference_names: HashSet::new(),
            last_int_value: 0,
            last_range: 0..0,
            last_assertion_is_quantifiable: false,
            builder: AstBuilder::new(allocator),
            last_str_value: String::default(),
        }
    }

    pub fn is(&self, ch: char) -> bool {
        self.lexer.chars.get(self.index) == Some(&ch)
    }

    pub fn eat(&mut self, ch: char) -> bool {
        if self.is(ch) {
            self.index += 1;
            true
        } else {
            false
        }
    }
    pub fn span_with_start(&self, start: u32) -> Span {
        Span::new(start, self.index as u32)
    }

    pub fn eat2(&mut self, first: char, second: char) -> bool {
        if self.is(first) && self.nth(1) == Some(&second) {
            self.index += 2;
            true
        } else {
            false
        }
    }

    pub fn eof(&self) -> bool {
        self.index < self.lexer.chars.len()
    }

    pub fn nth(&self, n: usize) -> Option<&char> {
        self.lexer.chars.get(self.index + n)
    }

    /// by default next means `nth(1)`
    pub fn next(&self) -> Option<&char> {
        self.lexer.chars.get(self.index + 1)
    }

    /// get a range chars relative from current cursor
    pub fn nrange(&self, range: Range<usize>) -> Option<&[char]> {
        self.lexer.chars.get(self.index + range.start..(self.index + range.end))
    }

    pub fn current(&self) -> Option<&char> {
        self.lexer.chars.get(self.index)
    }

    pub fn advance(&mut self) -> bool {
        if self.index < self.lexer.chars.len() {
            self.index += 1;
            return true;
        } else {
            false
        }
    }

    pub fn rewind<'a>(parser: &mut Parser<'a>, start: usize) {
        self.index = start;
    }
}

#[derive(Default, Clone, Copy)]
pub enum SourceKind {
    Flags,
    #[default]
    Literal,
    Pattern,
}

pub fn parse_literal<'a>(parser: &mut Parser<'a>) -> RegExpLiteral<'a> {
    if parser.is('/') {
        parser.advance();
        let pattern = parse_pattern(parser);
        todo!()
    } else if parser.source_text.is_empty() {
        panic!("Empty")
    } else {
        match parser.current() {
            Some(ch) => {
                panic!("unexpected character {ch}")
            }
            None => {
                panic!("unexpected eof")
            }
        };
    }
}

fn parse_pattern<'a>(parser: &mut Parser<'a>) -> Pattern<'a> {
    let start = parser.index;
    if let Some(pattern) = parse_pattern_internal(parser) {
        return pattern;
    } else if !parser.context.nflag
        && parser.context.ecma_version >= EcmaVersion::V2018
        && parser.group_names.len() > 0
    {
        parser.rewind(start);
        parser.context.nflag = true;
        return parse_pattern_internal(parser).expect("should have pattern");
    }
    panic!("Invalid pattern")
}

fn parse_pattern_internal<'a>(parser: &mut Parser<'a>) -> Option<Pattern<'a>> {
    let start = parser.index;
    parser.num_capturing_parens = count_capturing_parens(parser);
    parser.group_names.clear();
    parser.back_reference_names.clear();
    todo!()
}

fn parse_disjunction<'a>(parser: &mut Parser<'a>) -> oxc_allocator::Vec<'a, Alternative<'a>> {
    let start = parser.index;
    let mut alternatives = parser.builder.new_vec();
    loop {
        alternatives.push(parse_alternative(parser));
        if !parser.eat('|') {
            break;
        }
    }
    // Only consume the ast when `no_consume` is false
    if parse_quantifier(parser, Some(true)).0 {
        panic!("Nothing to repeat");
    }
    if parser.eat('{') {
        panic!("Lone quantifier brackets")
    }
    alternatives
}

/// Validate the next characters as a RegExp `Alternative` production.
/// ```
///  Alternative[UnicodeMode, UnicodeSetsMode, N]::
///      [empty]
///  Alternative[?UnicodeMode, ?UnicodeSetsMode, ?N] Term[?UnicodeMode, ?UnicodeSetsMode, ?N]
/// ```
fn parse_alternative<'a>(parser: &mut Parser<'a>) -> Alternative<'a> {
    let start = parser.index;
    let mut elements = parser.builder.new_vec();
    while !parser.eof() {
        let (flag, node) = parse_term(parser);
        if let Some(node) = node {
            elements.push(node);
        }
        if !flag {
            break;
        }
    }
    Alternative { span: Span::new(start as u32, parser.index as u32), elements }
}

fn parse_term<'a>(parser: &mut Parser<'a>) -> (bool, Option<Element<'a>>) {
    if parser.context.unicode_mode || parser.context.strict {}
    todo!()
}

fn parse_optional_quantifier<'a>(parser: &mut Parser<'a>) -> (bool, Option<Element<'a>>) {
    let (_, node) = parse_quantifier(parser, None);
    (true, node)
}

fn parse_assertion<'a>(parser: &mut Parser<'a>) -> (bool, Option<Assertion<'a>>) {
    let start = parser.index;
    parser.last_assertion_is_quantifiable = false;

    if parser.eat('^') {
        return (
            true,
            Some(Assertion::BoundaryAssertion(parser.builder.alloc(
                BoundaryAssertion::EdgeAssertion(parser.builder.alloc(EdgeAssertion {
                    span: Span::new(start as u32, parser.index as u32),
                    kind: EdgeAssertionKind::Start,
                })),
            ))),
        );
    }

    if parser.eat('$') {
        return (
            true,
            Some(Assertion::BoundaryAssertion(parser.builder.alloc(
                BoundaryAssertion::EdgeAssertion(parser.builder.alloc(EdgeAssertion {
                    span: Span::new(start as u32, parser.index as u32),
                    kind: EdgeAssertionKind::End,
                })),
            ))),
        );
    }

    if parser.eat2('\\', 'B') {
        return (
            true,
            Some(Assertion::BoundaryAssertion(parser.builder.alloc(
                BoundaryAssertion::WordBoundaryAssertion(parser.builder.alloc(
                    WordBoundaryAssertion {
                        span: Span::new(start as u32, parser.index as u32),
                        negate: true,
                    },
                )),
            ))),
        );
    }

    if parser.eat2('\\', 'b') {
        return (
            true,
            Some(Assertion::BoundaryAssertion(parser.builder.alloc(
                BoundaryAssertion::WordBoundaryAssertion(parser.builder.alloc(
                    WordBoundaryAssertion {
                        span: Span::new(start as u32, parser.index as u32),
                        negate: false,
                    },
                )),
            ))),
        );
    }

    // Lookahead / Lookbehind
    if parser.eat2('(', '?') {
        let lookbeind = parser.context.ecma_version >= EcmaVersion::V2018 && parser.eat('<');
        let mut eq_sign = parser.eat('=');
        let mut negate = if eq_sign { false } else { parser.eat('!') };
        if eq_sign || negate {
            let span = Span::new(start as u32, parser.index as u32);
            let alternatives = parse_disjunction(parser);
            let look_around_assertion =
                if lookbeind {
                    LookaroundAssertion::LookbehindAssertion(
                        parser.builder.alloc(LookbehindAssertion { span, negate, alternatives }),
                    )
                } else {
                    LookaroundAssertion::LookaheadAssertion(
                        parser.builder.alloc(LookaheadAssertion { span, negate, alternatives }),
                    )
                };
            let node = Assertion::LookaroundAssertion(parser.builder.alloc(look_around_assertion));
            if !parser.eat(')') {
                panic!("Unterminated group")
            }
            parser.last_assertion_is_quantifiable = !lookbeind && !parser.context.strict;
        }
        parser.rewind(start);
    }
    (false, None)
}

/// Validate the next characters as a RegExp `Quantifier` production if possible.
/// ```
///  Quantifier::
///        QuantifierPrefix
///        QuantifierPrefix `?`
///   QuantifierPrefix::
///        `*`
///        `+`
///        `?`
///        `{` DecimalDigits `}`
///        `{` DecimalDigits `,}`
///        `{` DecimalDigits `,` DecimalDigits `}`
///   ```
/// returns `true` if it consumed the next characters successfully.
fn parse_quantifier<'a>(
    parser: &mut Parser<'a>,
    no_consume: Option<bool>,
) -> (bool, Option<Element<'a>>) {
    let mut no_consume = no_consume.unwrap_or_default();
    let start = parser.index;
    let mut min = 0;
    let mut max = 0;
    let mut greedy = false;
    let mut element = None;
    match parser.current().cloned() {
        Some('*') => {
            min = 0;
            max = usize::MAX;
            parser.advance();
        }
        Some('+') => {
            min = 1;
            max = usize::MAX;
            parser.advance();
        }
        Some('?') => {
            min = 0;
            max = 1;
            parser.advance();
        }
        Some(_) => {
            if parse_braced_quantifier(parser, no_consume) {
                min = parser.last_range.start;
                max = parser.last_range.end;
            }
        }
        None => return (false, None),
    }
    greedy = !parser.eat('?');

    if !no_consume {
        let quantifier = parser.builder.alloc(Quantifier {
            span: Span { start: start as u32, end: parser.index as u32 },
            min,
            max,
            greedy,
            // https://github.com/eslint-community/regexpp/blob/2e8f1af992fb12eae46a446253e8fa3f6cede92a/src/parser.ts#L269-L275
            // it can't be null, or the program will panic, so we put a dummy element, and parent
            // should replace it
            element: QuantifiableElement::Character(Character {
                span: Span::default(),
                value: ' ',
            }),
        });

        element = Some(Element::Quantifier(quantifier))
    }
    (true, element)
}

fn parse_braced_quantifier<'a>(parser: &mut Parser<'a>, no_error: bool) -> bool {
    let start = parser.index;
    if eat_decimal_digits(parser) {
        let min = parser.last_int_value;
        let mut max = min;
        if parser.eat(',') {
            max = if eat_decimal_digits(parser) { parser.last_int_value } else { usize::MAX };
        }
        if parser.eat('}') {
            if !no_error && max < min {
                panic!("numbers out of order in {{}} quantifier");
            }
            parser.last_range = min..max;
            return true;
        }
    }
    if !no_error && (parser.context.unicode_mode || parser.context.strict) {
        panic!("Incomplete quantifier");
    }
    parser.rewind(start);
    false
}

fn parse_atom<'a>(parser: &mut Parser<'a>) {
    todo!()
}

fn parse_dot<'a>(parser: &mut Parser<'a>) -> (bool, Option<Character>) {
    let start = parser.index;
    if parser.eat('.') {
        (true, Some(Character { span: Span::new(start as u32, parser.index as u32), value: '.' }))
    } else {
        (false, None)
    }
}

fn parse_reverse_solidus_atom_escape<'a>(parser: &mut Parser<'a>) -> bool {
    let start = parser.index;
    if parser.eat('\\') {
        if parse_atom_escape(parser) {
            return true;
        }
        parser.rewind(start);
    }
    false
}

fn parse_atom_escape<'a>(parser: &mut Parser<'a>) -> bool {
    if parse_backreference(parser)
        || parser.consume_character_class_escape()
        || parser.consume_character_escape()
        || (parser.context.nflag && parser.consume_k_group_name())
    {
        true
    } else {
        if parser.strict || parser._unicode_mode {
            parser.raise("Invalid escape");
        }
        false
    }
}

/// TODO: resolve when pattern leave
fn parse_backreference<'a>(parser: &mut Parser<'a>) -> Option<Backreference<'a>> {
    let start = parser.index;
    if parser.eat_decimal_escape() {
        let n = parser.last_int_value;
        if n <= parser.num_capturing_parens {
            Some(Backreference {
                span: Span::new(start as u32, parser.index as u32),
                reference: BackreferenceRef::Number(n as usize),
                resolved: CapturingGroup::default(),
            })
        } else {
            if parser.context.strict || parser.context.unicode_mode {
                panic!("Invalid escape");
            }
            parser.rewind(start);
            None
        }
    } else {
        None
    }
}

struct UnicodeSetsConsumeResult {
    may_contain_strings: Option<bool>,
}

fn consume_character_class_escape<'a>(parser: &mut Parser<'a>) -> Option<UnicodeSetsConsumeResult> {
    let start = parser.index;

    if parser.eat(LATIN_SMALL_LETTER_D) {
        parser.last_int_value = -1;
        parser.on_escape_character_set(start - 1, parser.index, "digit", false);
        return Some(UnicodeSetsConsumeResult { may_contain_strings: None });
    }

    if parser.eat(LATIN_CAPITAL_LETTER_D) {
        parser.last_int_value = -1;
        parser.on_escape_character_set(start - 1, parser.index, "digit", true);
        return Some(UnicodeSetsConsumeResult { may_contain_strings: None });
    }

    if parser.eat(LATIN_SMALL_LETTER_S) {
        parser.last_int_value = -1;
        parser.on_escape_character_set(start - 1, parser.index, "space", false);
        return Some(UnicodeSetsConsumeResult { may_contain_strings: None });
    }

    if parser.eat(LATIN_CAPITAL_LETTER_S) {
        parser.last_int_value = -1;
        parser.on_escape_character_set(start - 1, parser.index, "space", true);
        return Some(UnicodeSetsConsumeResult { may_contain_strings: None });
    }

    if parser.eat(LATIN_SMALL_LETTER_W) {
        parser.last_int_value = -1;
        parser.on_escape_character_set(start - 1, parser.index, "word", false);
        return Some(UnicodeSetsConsumeResult { may_contain_strings: None });
    }

    if parser.eat(LATIN_CAPITAL_LETTER_W) {
        parser.last_int_value = -1;
        parser.on_escape_character_set(start - 1, parser.index, "word", true);
        return Some(UnicodeSetsConsumeResult { may_contain_strings: None });
    }

    let mut negate = false;
    if parser._unicode_mode
        && parser.ecma_version >= 2018
        && (parser.eat(LATIN_SMALL_LETTER_P) || (negate = parser.eat(LATIN_CAPITAL_LETTER_P)))
    {
        parser.last_int_value = -1;
        if parser.eat(LEFT_CURLY_BRACKET) {
            if let Some(result) = parser.eat_unicode_property_value_expression() {
                if parser.eat(RIGHT_CURLY_BRACKET) {
                    if negate && result.strings.is_some() {
                        parser.raise("Invalid property name");
                    }

                    parser.on_unicode_property_character_set(
                        start - 1,
                        parser.index,
                        "property",
                        &result.key,
                        &result.value,
                        negate,
                        result.strings.unwrap_or(false),
                    );

                    return Some(UnicodeSetsConsumeResult {
                        may_contain_strings: result.strings.unwrap_or(false),
                    });
                }
            }
        }
        panic!("Invalid property name");
    }

    None
}

fn consume_k_group_name<'a>(parser: &mut Parser<'a>) -> Option<Backreference<'a>> {
    let start = parser.index;

    if parser.eat('k') {
        if parser.eat_group_name() {
            let group_name: String = parser.last_str_value.clone();
            parser.back_reference_names.insert(group_name.clone());
            return Some(Backreference {
                span: parser.span_with_start(start),
                reference: BackreferenceRef::Atom(group_name.as_str().into()),
                // dummy resolved
                resolved: CapturingGroup::default(),
            });
        }
        panic!("Invalid named reference");
    }

    None
}

fn consume_character_class<'a>(parser: &mut Parser<'a>) -> Option<UnicodeSetsConsumeResult> {
    let start = parser.index;

    if parser.eat(LEFT_SQUARE_BRACKET) {
        let negate = parser.eat(CIRCUMFLEX_ACCENT);
        parser.on_character_class_enter(start, negate, parser._unicode_sets_mode);
        let result = consume_class_contents(parser);
        if !parser.eat(RIGHT_SQUARE_BRACKET) {
            if parser.current_code_point == -1 {
                parser.raise("Unterminated character class");
            }
            parser.raise("Invalid character in character class");
        }
        if negate && result.may_contain_strings {
            parser.raise("Negated character class may contain strings");
        }

        parser.on_character_class_leave(start, parser.index, negate);

        // * Static Semantics: MayContainStrings
        // CharacterClass[UnicodeMode, UnicodeSetsMode] ::
        //         [ ^ ClassContents[?UnicodeMode, ?UnicodeSetsMode] ]
        //     1. Return false.
        // CharacterClass :: [ ClassContents ]
        //     1. Return MayContainStrings of the ClassContents.
        Some(result)
    } else {
        None
    }
}

/**
 * Consume ClassContents in a character class.
 * @returns `UnicodeSetsConsumeResult`.
 */
fn consume_class_contents<'a>(parser: &mut Parser<'a>) -> UnicodeSetsConsumeResult {
    if self._unicode_sets_mode {
        if self.current_code_point == RIGHT_SQUARE_BRACKET {
            // [empty]

            // * Static Semantics: MayContainStrings
            // ClassContents[UnicodeMode, UnicodeSetsMode] ::
            //         [empty]
            //     1. Return false.
            return UnicodeSetsConsumeResult { may_contain_strings: None };
        }
        let result = self.consume_class_set_expression();

        // * Static Semantics: MayContainStrings
        // ClassContents :: ClassSetExpression
        //     1. Return MayContainStrings of the ClassSetExpression.
        return result;
    }

    let strict = self.strict || self._unicode_mode;
    loop {
        // Consume the first ClassAtom
        let range_start = self.index;
        if !self.consume_class_atom() {
            break;
        }
        let min = self._last_int_value;

        // Consume `-`
        if !self.eat(HYPHEN_MINUS) {
            continue;
        }
        self.on_character(range_start - 1, self.index, HYPHEN_MINUS);

        // Consume the second ClassAtom
        if !self.consume_class_atom() {
            break;
        }
        let max = self._last_int_value;

        // Validate
        if min == -1 || max == -1 {
            if strict {
                self.raise("Invalid character class");
            }
            continue;
        }
        if min > max {
            self.raise("Range out of order in character class");
        }

        self.on_character_class_range(range_start, self.index, min, max);
    }

    // * Static Semantics: MayContainStrings
    // ClassContents[UnicodeMode, UnicodeSetsMode] ::
    //         NonemptyClassRanges[?UnicodeMode]
    //     1. Return false.
    return UnicodeSetsConsumeResult { may_contain_strings: false };
}

/**
 * Consume ClassAtom in a character class.
 * @returns `true` if it consumed the next characters successfully.
 */
fn consume_class_atom<'a>(parser: &mut Parser<'a>) -> bool {
    let start = self.index;
    let cp = self.current_code_point;

    if cp != -1 && cp != REVERSE_SOLIDUS && cp != RIGHT_SQUARE_BRACKET {
        self.advance();
        self._last_int_value = cp;
        self.on_character(start, self.index, self._last_int_value);
        return true;
    }

    if self.eat(REVERSE_SOLIDUS) {
        if self.consume_class_escape() {
            return true;
        }
        if !self.strict && self.current_code_point == LATIN_SMALL_LETTER_C {
            self._last_int_value = REVERSE_SOLIDUS;
            self.on_character(start, self.index, self._last_int_value);
            return true;
        }
        if self.strict || self._unicode_mode {
            self.raise("Invalid escape");
        }
        self.rewind(start);
    }

    return false;
}

/**
 * Consume ClassEscape in a character class.
 * @returns `true` if it consumed the next characters successfully.
 */
fn consume_class_escape<'a>(parser: &mut Parser<'a>) -> bool {
    let start = self.index;

    // `b`
    if self.eat(LATIN_SMALL_LETTER_B) {
        self._last_int_value = BACKSPACE;
        self.on_character(start - 1, self.index, self._last_int_value);
        return true;
    }

    // [+UnicodeMode] `-`
    if self._unicode_mode && self.eat(HYPHEN_MINUS) {
        self._last_int_value = HYPHEN_MINUS;
        self.on_character(start - 1, self.index, self._last_int_value);
        return true;
    }

    // [annexB][~UnicodeMode] `c` ClassControlLetter
    let cp = 0;
    if !self.strict
        && !self._unicode_mode
        && self.current_code_point == LATIN_SMALL_LETTER_C
        && (is_decimal_digit((cp = self.next_code_point)) || cp == LOW_LINE)
    {
        self.advance();
        self.advance();
        self._last_int_value = cp % 0x20;
        self.on_character(start - 1, self.index, self._last_int_value);
        return true;
    }

    return self.consume_character_class_escape() || self.consume_character_escape();
}

/**
 * Consume ClassSetExpression in a character class.
 * @returns `UnicodeSetsConsumeResult`.
 */
fn consume_class_set_expression<'a>(parser: &mut Parser<'a>) -> UnicodeSetsConsumeResult {
    let start = self.index;
    let mut may_contain_strings: Option<bool> = None;
    let mut result: Option<UnicodeSetsConsumeResult> = None;

    if self.consume_class_set_character() {
        if self.consume_class_set_range_from_operator(start) {
            // ClassUnion
            self.consume_class_union_right(UnicodeSetsConsumeResult { may_contain_strings: None });
            return UnicodeSetsConsumeResult { may_contain_strings: false };
        }
        // ClassSetOperand

        // * Static Semantics: MayContainStrings
        // ClassSetOperand ::
        //         ClassSetCharacter
        //     1. Return false.
        may_contain_strings = Some(false);
    } else if let Some(res) = self.consume_class_set_operand() {
        may_contain_strings = Some(res.may_contain_strings);
    } else {
        let cp = self.current_code_point;
        if cp == REVERSE_SOLIDUS {
            self.advance();
            self.raise("Invalid escape");
        }
        if cp == self.next_code_point && is_class_set_reserved_double_punctuator_character(cp) {
            self.raise("Invalid set operation in character class");
        }
        self.raise("Invalid character in character class");
    }

    if self.eat2(AMPERSAND, AMPERSAND) {
        // ClassIntersection
        while self.current_code_point != AMPERSAND
            && (result = self.consume_class_set_operand()).is_some()
        {
            self.on_class_intersection(start, self.index);
            if !result.as_ref().unwrap().may_contain_strings.unwrap_or(false) {
                may_contain_strings = Some(false);
            }
            if self.eat2(AMPERSAND, AMPERSAND) {
                continue;
            }

            // * Static Semantics: MayContainStrings
            // ClassSetExpression :: ClassIntersection
            //     1. Return MayContainStrings of the ClassIntersection.
            // ClassIntersection :: ClassSetOperand && ClassSetOperand
            //     1. If MayContainStrings of the first ClassSetOperand is false, return false.
            //     2. If MayContainStrings of the second ClassSetOperand is false, return false.
            //     3. Return true.
            // ClassIntersection :: ClassIntersection && ClassSetOperand
            //     1. If MayContainStrings of the ClassIntersection is false, return false.
            //     2. If MayContainStrings of the ClassSetOperand is false, return false.
            //     3. Return true.
            return UnicodeSetsConsumeResult { may_contain_strings };
        }

        self.raise("Invalid character in character class");
    }
    if self.eat2(HYPHEN_MINUS, HYPHEN_MINUS) {
        // ClassSubtraction
        while self.consume_class_set_operand() {
            self.on_class_subtraction(start, self.index);
            if self.eat2(HYPHEN_MINUS, HYPHEN_MINUS) {
                continue;
            }
            // * Static Semantics: MayContainStrings
            // ClassSetExpression :: ClassSubtraction
            //     1. Return MayContainStrings of the ClassSubtraction.
            // ClassSubtraction :: ClassSetOperand -- ClassSetOperand
            //     1. Return MayContainStrings of the first ClassSetOperand.
            // ClassSubtraction :: ClassSubtraction -- ClassSetOperand
            //     1. Return MayContainStrings of the ClassSubtraction.
            return UnicodeSetsConsumeResult { may_contain_strings };
        }

        self.raise("Invalid character in character class");
    }
    // ClassUnion
    return self.consume_class_union_right(UnicodeSetsConsumeResult { may_contain_strings });
}

/**
 * Consume the right operand of a ClassUnion in a character class.
 * @param left_result The result information for the left ClassSetRange or ClassSetOperand.
 * @returns `UnicodeSetsConsumeResult`.
 */
fn consume_class_union_right<'a>(
    parser: &mut Parser<'a>,
    left_result: UnicodeSetsConsumeResult,
) -> UnicodeSetsConsumeResult {
    // ClassUnion
    let mut may_contain_strings = left_result.may_contain_strings.unwrap_or(false);
    loop {
        let start = self.index;
        if self.consume_class_set_character() {
            self.consume_class_set_range_from_operator(start);
            continue;
        }
        if let Some(result) = self.consume_class_set_operand() {
            if result.may_contain_strings.unwrap_or(false) {
                may_contain_strings = true;
            }
            continue;
        }
        break;
    }

    // * Static Semantics: MayContainStrings
    // ClassSetExpression :: ClassUnion
    //     1. Return MayContainStrings of the ClassUnion.
    // ClassUnion :: ClassSetRange ClassUnion(opt)
    //     1. If the ClassUnion is present, return MayContainStrings of the ClassUnion.
    //     2. Return false.
    // ClassUnion :: ClassSetOperand ClassUnion(opt)
    //     1. If MayContainStrings of the ClassSetOperand is true, return true.
    //     2. If ClassUnion is present, return MayContainStrings of the ClassUnion.
    //     3. Return false.
    return UnicodeSetsConsumeResult { may_contain_strings };
}

fn eat_decimal_digits<'a>(parser: &mut Parser<'a>) -> bool {
    let start = parser.index;
    parser.last_int_value = 0;
    while let Some(ch) = parser.current() {
        let Some(d) = ch.to_digit(10) else {
            break;
        };
        parser.last_int_value = 10 * parser.last_int_value + d as usize;
        parser.advance();
    }
    parser.index != start
}

fn count_capturing_parens<'a>(parser: &mut Parser<'a>) -> usize {
    let start = parser.index;
    let mut in_class = false;
    let mut escaped = false;
    let mut count = 0;
    while let Some(ch) = parser.current() {
        if escaped {
            escaped = false;
        }
        match ch {
            '\\' => {
                escaped = true;
            }
            '[' | ']' => {
                in_class = false;
            }
            '(' if !in_class => {
                if parser.nth(1) != Some(&'?')
                    || (parser.nth(2) == Some(&'<')
                        && !matches!(parser.nth(3), Some(&'=') | Some(&'!')))
                {
                    count += 1;
                }
            }
            _ => {}
        }
        parser.advance();
    }
    parser.rewind(start);
    count
}

/**
 * Consume NestedClass in a character class.
 * @returns `UnicodeSetsConsumeResult`.
 */
fn consume_nested_class<'a>(parser: &mut Parser<'a>) -> Option<UnicodeSetsConsumeResult> {
    let start = self.index;
    if self.eat(LEFT_SQUARE_BRACKET) {
        let negate = self.eat(CIRCUMFLEX_ACCENT);
        self.on_character_class_enter(start, negate, true);
        let result = self.consume_class_contents();
        if !self.eat(RIGHT_SQUARE_BRACKET) {
            self.raise("Unterminated character class");
        }
        if negate && result.may_contain_strings.unwrap_or(false) {
            self.raise("Negated character class may contain strings");
        }
        self.on_character_class_leave(start, self.index, negate);

        // * Static Semantics: MayContainStrings
        // NestedClass ::
        //         [ ^ ClassContents[+UnicodeMode, +UnicodeSetsMode] ]
        //     1. Return false.
        // NestedClass :: [ ClassContents ]
        //     1. Return MayContainStrings of the ClassContents.
        return Some(result);
    }
    if self.eat(REVERSE_SOLIDUS) {
        if let Some(result) = self.consume_character_class_escape() {
            // * Static Semantics: MayContainStrings
            // NestedClass :: \ CharacterClassEscape
            //     1. Return MayContainStrings of the CharacterClassEscape.
            return Some(result);
        }
        self.rewind(start);
    }
    None
}

/**
 * Consume ClassStringDisjunction in a character class.
 * @returns `UnicodeSetsConsumeResult`.
 */
fn consume_class_string_disjunction<'a>(
    parser: &mut Parser<'a>,
) -> Option<UnicodeSetsConsumeResult> {
    let start = self.index;
    if self.eat3(REVERSE_SOLIDUS, LATIN_SMALL_LETTER_Q, LEFT_CURLY_BRACKET) {
        self.on_class_string_disjunction_enter(start);

        let mut i = 0;
        let mut may_contain_strings = false;
        while self.consume_class_string(i).may_contain_strings.unwrap_or(false) {
            may_contain_strings = true;
            i += 1;
            if !self.eat(VERTICAL_LINE) {
                break;
            }
        }

        if self.eat(RIGHT_CURLY_BRACKET) {
            self.on_class_string_disjunction_leave(start, self.index);

            // * Static Semantics: MayContainStrings
            // ClassStringDisjunction :: \q{ ClassStringDisjunctionContents }
            //     1. Return MayContainStrings of the ClassStringDisjunctionContents.
            // ClassStringDisjunctionContents :: ClassString
            //     1. Return MayContainStrings of the ClassString.
            // ClassStringDisjunctionContents :: ClassString | ClassStringDisjunctionContents
            //     1. If MayContainStrings of the ClassString is true, return true.
            //     2. Return MayContainStrings of the ClassStringDisjunctionContents.
            return Some(UnicodeSetsConsumeResult { may_contain_strings });
        }
        self.raise("Unterminated class string disjunction");
    }
    None
}

/**
 * Consume ClassString in a character class.
 * @param i - The index of the string alternative.
 * @returns `UnicodeSetsConsumeResult`.
 */
fn consume_class_string<'a>(parser: &mut Parser<'a>, i: usize) -> UnicodeSetsConsumeResult {
    let start = self.index;

    let mut count = 0;
    self.on_string_alternative_enter(start, i);

    while self.current_code_point != -1 && self.consume_class_set_character() {
        count += 1;
    }

    self.on_string_alternative_leave(start, self.index, i);

    // * Static Semantics: MayContainStrings
    // ClassString :: [empty]
    //     1. Return true.
    // ClassString :: NonEmptyClassString
    //     1. Return MayContainStrings of the NonEmptyClassString.
    // NonEmptyClassString :: ClassSetCharacter NonEmptyClassString(opt)
    //     1. If NonEmptyClassString is present, return true.
    //     2. Return false.
    return UnicodeSetsConsumeResult { may_contain_strings: Some(count != 1) };
}

/**
 * Consume ClassSetCharacter in a character class.
 * Set `self._last_int_value` if it consumed the next characters successfully.
 * @returns `true` if it ate the next characters successfully.
 */
fn consume_class_set_character<'a>(parser: &mut Parser<'a>) -> bool {
    let start = self.index;
    let cp = self.current_code_point;

    if cp != -1 && cp != self.next_code_point
        || !is_class_set_reserved_double_punctuator_character(cp)
    {
        if cp != -1 && !is_class_set_syntax_character(cp) {
            self._last_int_value = cp;
            self.advance();
            self.on_character(start, self.index, self._last_int_value);
            return true;
        }
    }

    if self.eat(REVERSE_SOLIDUS) {
        if self.consume_character_escape() {
            return true;
        }
        if is_class_set_reserved_punctuator(self.current_code_point) {
            self._last_int_value = self.current_code_point;
            self.advance();
            self.on_character(start, self.index, self._last_int_value);
            return true;
        }
        if self.eat(LATIN_SMALL_LETTER_B) {
            self._last_int_value = BACKSPACE;
            self.on_character(start, self.index, self._last_int_value);
            return true;
        }
        self.rewind(start);
    }

    false
}

/**
 * Eat the next characters as a RegExp `GroupName` production if possible.
 * Set `self._last_str_value` if the group name existed.
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_group_name<'a>(parser: &mut Parser<'a>) -> bool {
    if self.eat(LESS_THAN_SIGN) {
        if self.eat_reg_exp_identifier_name() && self.eat(GREATER_THAN_SIGN) {
            return true;
        }
        self.raise("Invalid capture group name");
    }
    false
}

/**
 * Eat the next characters as a RegExp `RegExpIdentifierName` production if
 * possible.
 * Set `self._last_str_value` if the identifier name existed.
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_reg_exp_identifier_name<'a>(parser: &mut Parser<'a>) -> bool {
    if self.eat_reg_exp_identifier_start() {
        self._last_str_value = self._last_int_value.to_string();

        while self.eat_reg_exp_identifier_part() {
            self._last_str_value.push_str(&self._last_int_value.to_string());
        }

        return true;
    }
    false
}

/**
 * Eat the next characters as a RegExp `RegExpIdentifierStart` production if
 * possible.
 * Set `self._last_int_value` if the identifier start existed.
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_reg_exp_identifier_start<'a>(parser: &mut Parser<'a>) -> bool {
    let start = self.index;
    let force_u_flag = !self._unicode_mode && self.ecma_version >= 2020;
    let mut cp = self.current_code_point;
    self.advance();

    if cp == REVERSE_SOLIDUS && self.eat_reg_exp_unicode_escape_sequence(force_u_flag) {
        cp = self._last_int_value;
    } else if force_u_flag && is_lead_surrogate(cp) && is_trail_surrogate(self.current_code_point) {
        cp = combine_surrogate_pair(cp, self.current_code_point);
        self.advance();
    }

    if is_identifier_start_char(cp) {
        self._last_int_value = cp;
        return true;
    }

    if self.index != start {
        self.rewind(start);
    }
    false
}

/**
 * Eat the next characters as a RegExp `RegExpIdentifierPart` production if possible.
 * Set `self._last_int_value` if it ate the next characters successfully.
 * ```
 * RegExpIdentifierPart[UnicodeMode]::
 *      RegExpIdentifierStart[?UnicodeMode]
 *      DecimalDigit
 *      \ UnicodeEscapeSequence[+UnicodeMode]
 * ```
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_reg_exp_identifier_part<'a>(parser: &mut Parser<'a>) -> bool {
    let start = self.index;
    let force_u_flag = !self._unicode_mode && self.ecma_version >= 2020;
    let mut cp = self.current_code_point;
    self.advance();

    if cp == REVERSE_SOLIDUS && self.eat_reg_exp_unicode_escape_sequence(force_u_flag) {
        cp = self._last_int_value;
    } else if force_u_flag && is_lead_surrogate(cp) && is_trail_surrogate(self.current_code_point) {
        cp = combine_surrogate_pair(cp, self.current_code_point);
        self.advance();
    }

    if is_identifier_part_char(cp) {
        self._last_int_value = cp;
        return true;
    }

    if self.index != start {
        self.rewind(start);
    }
    false
}

/**
 * Eat the next characters as the following alternatives if possible.
 * Set `self._last_int_value` if it ate the next characters successfully.
 * ```
 *      `c` ControlLetter
 * ```
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_c_control_letter<'a>(parser: &mut Parser<'a>) -> bool {
    let start = self.index;
    if self.eat(LATIN_SMALL_LETTER_C) {
        if self.eat_control_letter() {
            return true;
        }
        self.rewind(start);
    }
    false
}

/**
 * Eat the next characters as the following alternatives if possible.
 * Set `self._last_int_value` if it ate the next characters successfully.
 * ```
 *      `0` [lookahead ∉ DecimalDigit]
 * ```
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_zero<'a>(parser: &mut Parser<'a>) -> bool {
    if self.current_code_point == DIGIT_ZERO && !is_decimal_digit(self.next_code_point) {
        self._last_int_value = 0;
        self.advance();
        return true;
    }
    false
}

/**
 * Eat the next characters as a RegExp `ControlEscape` production if
 * possible.
 * Set `self._last_int_value` if it ate the next characters successfully.
 * ```
 * ControlEscape:: one of
 *      f n r t v
 * ```
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_control_escape<'a>(parser: &mut Parser<'a>) -> bool {
    if self.eat(LATIN_SMALL_LETTER_F) {
        self._last_int_value = FORM_FEED;
        return true;
    }
    if self.eat(LATIN_SMALL_LETTER_N) {
        self._last_int_value = LINE_FEED;
        return true;
    }
    if self.eat(LATIN_SMALL_LETTER_R) {
        self._last_int_value = CARRIAGE_RETURN;
        return true;
    }
    if self.eat(LATIN_SMALL_LETTER_T) {
        self._last_int_value = CHARACTER_TABULATION;
        return true;
    }
    if self.eat(LATIN_SMALL_LETTER_V) {
        self._last_int_value = LINE_TABULATION;
        return true;
    }
    false
}

/**
 * Eat the next characters as a RegExp `ControlLetter` production if
 * possible.
 * Set `self._last_int_value` if it ate the next characters successfully.
 * ```
 * ControlLetter:: one of
 *      a b c d e f g h i j k l m n o p q r s t u v w x y z
 *      A B C D E F G H I J K L M N O P Q R S T U V W X Y Z
 * ```
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_control_letter<'a>(parser: &mut Parser<'a>) -> bool {
    let cp = self.current_code_point;
    if is_latin_letter(cp) {
        self.advance();
        self._last_int_value = cp % 0x20;
        return true;
    }
    false
}

/**
 * Eat the next characters as a RegExp `RegExpUnicodeEscapeSequence`
 * production if possible.
 * Set `self._last_int_value` if it ate the next characters successfully.
 * ```
 * RegExpUnicodeEscapeSequence[UnicodeMode]::
 *      [+UnicodeMode] `u` HexLeadSurrogate `\u` HexTrailSurrogate
 *      [+UnicodeMode] `u` HexLeadSurrogate
 *      [+UnicodeMode] `u` HexTrailSurrogate
 *      [+UnicodeMode] `u` HexNonSurrogate
 *      [~UnicodeMode] `u` Hex4Digits
 *      [+UnicodeMode] `u{` CodePoint `}`
 * ```
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_reg_exp_unicode_escape_sequence<'a>(parser: &mut Parser<'a>, force_u_flag: bool) -> bool {
    let start = self.index;
    let u_flag = force_u_flag || self._unicode_mode;

    if self.eat(LATIN_SMALL_LETTER_U) {
        if (u_flag && self.eat_reg_exp_unicode_surrogate_pair_escape())
            || self.eat_fixed_hex_digits(4)
            || (u_flag && self.eat_reg_exp_unicode_code_point_escape())
        {
            return true;
        }
        if self.strict || u_flag {
            self.raise("Invalid unicode escape");
        }
        self.rewind(start);
    }

    false
}

/**
 * Eat the next characters as the following alternatives if possible.
 * Set `self._last_int_value` if it ate the next characters successfully.
 * ```
 *      HexLeadSurrogate `\u` HexTrailSurrogate
 * ```
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_reg_exp_unicode_surrogate_pair_escape<'a>(parser: &mut Parser<'a>) -> bool {
    let start = self.index;

    if self.eat_fixed_hex_digits(4) {
        let lead = self._last_int_value;
        if is_lead_surrogate(lead)
            && self.eat(REVERSE_SOLIDUS)
            && self.eat(LATIN_SMALL_LETTER_U)
            && self.eat_fixed_hex_digits(4)
        {
            let trail = self._last_int_value;
            if is_trail_surrogate(trail) {
                self._last_int_value = combine_surrogate_pair(lead, trail);
                return true;
            }
        }

        self.rewind(start);
    }

    false
}

/**
 * Eat the next characters as the following alternatives if possible.
 * Set `self._last_int_value` if it ate the next characters successfully.
 * ```
 *      `{` CodePoint `}`
 * ```
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_reg_exp_unicode_code_point_escape(parser: &mut Parser<'a>) -> bool {
    let start = self.index;

    if self.eat(LEFT_CURLY_BRACKET)
        && self.eat_hex_digits()
        && self.eat(RIGHT_CURLY_BRACKET)
        && is_valid_unicode(self._last_int_value)
    {
        return true;
    }

    self.rewind(start);
    false
}

/**
 * Eat the next characters as a RegExp `IdentityEscape` production if
 * possible.
 * Set `self._last_int_value` if it ate the next characters successfully.
 * ```
 * IdentityEscape[UnicodeMode, N]::
 *      [+UnicodeMode] SyntaxCharacter
 *      [+UnicodeMode] `/`
 *      [strict][~UnicodeMode] SourceCharacter but not UnicodeIDContinue
 *      [annexB][~UnicodeMode] SourceCharacterIdentityEscape[?N]
 * SourceCharacterIdentityEscape[N]::
 *      [~N] SourceCharacter but not c
 *      [+N] SourceCharacter but not one of c k
 * ```
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_identity_escape<'a>(parser: &mut Parser<'a>) -> bool {
    let cp = self.current_code_point;
    if self.is_valid_identity_escape(cp) {
        self._last_int_value = cp;
        self.advance();
        return true;
    }
    false
}

fn is_valid_identity_escape<'a>(parser: &mut Parser<'a>, cp: i32) -> bool {
    if cp == -1 {
        return false;
    }
    if self._unicode_mode {
        return is_syntax_character(cp) || cp == SOLIDUS;
    }
    if self.strict {
        return !is_id_continue(cp);
    }
    if self._n_flag {
        return !(cp == LATIN_SMALL_LETTER_C || cp == LATIN_SMALL_LETTER_K);
    }
    cp != LATIN_SMALL_LETTER_C
}

/**
 * Eat the next characters as a RegExp `DecimalEscape` production if
 * possible.
 * Set `self._last_int_value` if it ate the next characters successfully.
 * ```
 * DecimalEscape::
 *      NonZeroDigit DecimalDigits(opt) [lookahead ∉ DecimalDigit]
 * ```
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_decimal_escape<'a>(parser: &mut Parser<'a>) -> bool {
    parser.last_int_value = 0;
    let mut cp = parser.current();
    if cp >= Some(&'1') && cp <= Some(&'9') {
        while cp >= Some(&'1') && cp <= Some(&'9') {
            parser.last_int_value = 10 * parser.last_int_value + (cp - DIGIT_ZERO);
            parser.advance();
            cp = parser.current();
        }
        return true;
    }
    false
}

/**
 * Eat the next characters as a RegExp `ControlLetter` production if
 * possible.
 * Set `self._last_int_value` if it ate the next characters successfully.
 * ```
 * ControlLetter:: one of
 *      a b c d e f g h i j k l m n o p q r s t u v w x y z
 *      A B C D E F G H I J K L M N O P Q R S T U V W X Y Z
 * ```
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_control_letter<'a>(parser: &mut Parser<'a>) -> bool {
    let cp = parser.current();
    if is_latin_letter(cp) {
        parser.advance();
        parser.last_int_value = cp % 0x20;
        return true;
    }
    false
}

/**
 * Eat the next characters as a RegExp `RegExpUnicodeEscapeSequence`
 * production if possible.
 * Set `self._last_int_value` if it ate the next characters successfully.
 * ```
 * RegExpUnicodeEscapeSequence[UnicodeMode]::
 *      [+UnicodeMode] `u` HexLeadSurrogate `\u` HexTrailSurrogate
 *      [+UnicodeMode] `u` HexLeadSurrogate
 *      [+UnicodeMode] `u` HexTrailSurrogate
 *      [+UnicodeMode] `u` HexNonSurrogate
 *      [~UnicodeMode] `u` Hex4Digits
 *      [+UnicodeMode] `u{` CodePoint `}`
 * ```
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_reg_exp_unicode_escape_sequence<'a>(parser: &mut Parser<'a>, force_u_flag: bool) -> bool {
    let start = self.index;
    let u_flag = force_u_flag || self._unicode_mode;

    if self.eat(LATIN_SMALL_LETTER_U) {
        if (u_flag && self.eat_reg_exp_unicode_surrogate_pair_escape())
            || self.eat_fixed_hex_digits(4)
            || (u_flag && self.eat_reg_exp_unicode_code_point_escape())
        {
            return true;
        }
        if self.strict || u_flag {
            self.raise("Invalid unicode escape");
        }
        self.rewind(start);
    }

    false
}

/**
 * Eat the next characters as the following alternatives if possible.
 * Set `self._last_int_value` if it ate the next characters successfully.
 * ```
 *      HexLeadSurrogate `\u` HexTrailSurrogate
 * ```
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_reg_exp_unicode_surrogate_pair_escape<'a>(parser: &mut Parser<'a>) -> bool {
    let start = self.index;

    if self.eat_fixed_hex_digits(4) {
        let lead = self._last_int_value;
        if is_lead_surrogate(lead)
            && self.eat(REVERSE_SOLIDUS)
            && self.eat(LATIN_SMALL_LETTER_U)
            && self.eat_fixed_hex_digits(4)
        {
            let trail = self._last_int_value;
            if is_trail_surrogate(trail) {
                self._last_int_value = combine_surrogate_pair(lead, trail);
                return true;
            }
        }

        self.rewind(start);
    }

    false
}

/**
 * Eat the next characters as the following alternatives if possible.
 * Set `self._last_int_value` if it ate the next characters successfully.
 * ```
 *      `{` CodePoint `}`
 * ```
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_reg_exp_unicode_code_point_escape<'a>(parser: &mut Parser<'a>) -> bool {
    let start = self.index;

    if self.eat(LEFT_CURLY_BRACKET)
        && self.eat_hex_digits()
        && self.eat(RIGHT_CURLY_BRACKET)
        && is_valid_unicode(self._last_int_value)
    {
        return true;
    }

    self.rewind(start);
    false
}

/**
 * Eat the next characters as a RegExp `IdentityEscape` production if
 * possible.
 * Set `self._last_int_value` if it ate the next characters successfully.
 * ```
 * IdentityEscape[UnicodeMode, N]::
 *      [+UnicodeMode] SyntaxCharacter
 *      [+UnicodeMode] `/`
 *      [strict][~UnicodeMode] SourceCharacter but not UnicodeIDContinue
 *      [annexB][~UnicodeMode] SourceCharacterIdentityEscape[?N]
 * SourceCharacterIdentityEscape[N]::
 *      [~N] SourceCharacter but not c
 *      [+N] SourceCharacter but not one of c k
 * ```
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_identity_escape<'a>(parser: &mut Parser<'a>) -> bool {
    let cp = self.current_code_point;
    if self.is_valid_identity_escape(cp) {
        self._last_int_value = cp;
        self.advance();
        return true;
    }
    false
}

fn is_valid_identity_escape(&self, cp: i32) -> bool {
    if cp == -1 {
        return false;
    }
    if self._unicode_mode {
        return is_syntax_character(cp) || cp == SOLIDUS;
    }
    if self.strict {
        return !is_id_continue(cp);
    }
    if self._n_flag {
        return !(cp == LATIN_SMALL_LETTER_C || cp == LATIN_SMALL_LETTER_K);
    }
    cp != LATIN_SMALL_LETTER_C
}

/**
 * Eat the next characters as a RegExp `DecimalEscape` production if
 * possible.
 * Set `self._last_int_value` if it ate the next characters successfully.
 * ```
 * DecimalEscape::
 *      NonZeroDigit DecimalDigits(opt) [lookahead ∉ DecimalDigit]
 * ```
 * @returns `true` if it ate the next characters successfully.
 */
fn eat_decimal_escape<'a>(parser: &mut Parser<'a>) -> bool {
    self._last_int_value = 0;
    let mut cp = self.current_code_point;
    if cp >= DIGIT_ONE && cp <= DIGIT_NINE {
        while cp >= DIGIT_ZERO && cp <= DIGIT_NINE {
            self._last_int_value = 10 * self._last_int_value + (cp - DIGIT_ZERO);
            self.advance();
            cp = self.current_code_point;
        }
        return true;
    }
    false
}
