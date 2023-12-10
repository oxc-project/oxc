use std::collections::{HashSet, VecDeque};
use std::iter::Peekable;
use std::ops::Range;
use std::str::{CharIndices, Chars, Matches};

use oxc_allocator::Allocator;
use oxc_diagnostics::Error;

use crate::ast::{Branch, Pattern, RegExpLiteral};
use crate::ecma_version::EcmaVersion;

pub struct Lexer<'a> {
    allocator: &'a Allocator,

    source: &'a str,
    /// Regex usually, use a collected `Vec` could reduce lookahead and other util function implementation complexity
    chars: Vec<char>,

    pub(crate) errors: Vec<Error>,
}

#[allow(clippy::unused_self)]
impl<'a> Lexer<'a> {
    pub fn new(allocator: &'a Allocator, source: &'a str) -> Self {
        Self { source, allocator, errors: vec![], chars: source.chars().collect::<Vec<_>>() }
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,

    /// Source Code
    source_text: &'a str,

    /// All syntax errors from parser and lexer
    /// Note: favor adding to `Diagnostics` instead of raising Err
    errors: Vec<Error>,
    context: ParserContext,
    index: usize,
    group_names: HashSet<String>,
numCapturingParens: usize
}

#[derive(Default, Copy, Clone)]
struct ParserContext {
    source_kind: SourceKind,
    unicode_mode: bool,
    nflag: bool,
    unicode_sets_mode: bool,
    ecma_version: EcmaVersion,
}

impl<'a> Parser<'a> {
    /// Create a new parser
    pub fn new(allocator: &'a Allocator, source_text: &'a str) -> Self {
        Self {
            lexer: Lexer::new(allocator, source_text),
            source_text,
            errors: vec![],
            context: ParserContext::default(),
            index: 0,
            group_names: HashSet::new(),
        }
    }

    pub fn eat(&self, ch: char) -> bool {
        self.lexer.chars.get(self.index) == Some(&ch)
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
    if parser.eat('/') {
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
    let 
    todo!()
}

fn count_capturing_parens<'a>(parser: &mut Parser<'a>) -> usize {
    let start = parser.index;
    let mut in_class = false;
    let mut escaped = false;
    let count = 0;
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
                    || (parser.nth(2) == Some(&'<') && !matches!(parser.nth(3), '=' | '!'))
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
