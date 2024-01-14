use std::collections::{HashSet, VecDeque};
use std::iter::Peekable;
use std::ops::Range;
use std::os::unix::fs::OpenOptionsExt;
use std::panic;
use std::str::{CharIndices, Chars, Matches};

use oxc_diagnostics::Error;
use oxc_span::Span;

use crate::ast::{
    Alternative, Assertion, Branch, Character, Element, Pattern, QuantifiableElement, Quantifier,
    RegExpLiteral,
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

    pub fn eof(&self) -> bool {
        self.index < self.lexer.chars.len()
    }

    pub fn nth(&self, n: usize) -> Option<&char> {
        self.lexer.chars.get(self.index + n)
    }

    /// by default next means `next_1`
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

    pub fn rewind(&mut self, start: usize) {
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

fn parse_disjunction<'a>(parser: &mut Parser<'a>) {
    let start = parser.index;
    let mut i = 0;
    loop {}
}

/// Validate the next characters as a RegExp `Alternative` production.
/// ```
///  Alternative[UnicodeMode, UnicodeSetsMode, N]::
///      [empty]
///  Alternative[?UnicodeMode, ?UnicodeSetsMode, ?N] Term[?UnicodeMode, ?UnicodeSetsMode, ?N]
/// ```
fn parser_alternative<'a>(parser: &mut Parser<'a>) -> Alternative<'a> {
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

fn parse_assertion<'a>(parser: &mut Parser<'a>) -> (bool, Option<Assertion<'a>>) {
    let start = parser.index;
    parser.last_assertion_is_quantifiable = false;

    todo!()
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
            element: QuantifiableElement::Character(Character { span: Span::default(), value: 0 }),
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
                if parser.next() != Some(&'?')
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
