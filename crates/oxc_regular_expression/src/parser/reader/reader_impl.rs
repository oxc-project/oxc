use oxc_allocator::{Allocator, Vec};
use oxc_diagnostics::Result;
use oxc_span::{Atom, Span};

use crate::parser::reader::string_literal_parser::{
    ast as StringLiteralAst, parse_regexp_literal, ParserOptions, StringLiteralParser,
};

struct Collector<'a> {
    allocator: &'a Allocator,
    unicode_mode: bool,
    parse_string_literal: bool,
}
impl<'a> Collector<'a> {
    fn new(allocator: &'a Allocator, unicode_mode: bool, parse_string_literal: bool) -> Self {
        Self { allocator, unicode_mode, parse_string_literal }
    }

    fn collect(&mut self, source_text: &'a str) -> Result<Vec<'a, StringLiteralAst::CodePoint>> {
        // NOTE: This must be `0`.
        // Since `source_text` here may be a slice of the original source text,
        // using `Span` for `span.source_text(source_text)` will be out of range in some cases.
        let span_offset = 0;

        if self.parse_string_literal {
            let StringLiteralAst::StringLiteral { body, .. } = StringLiteralParser::new(
                self.allocator,
                source_text,
                ParserOptions {
                    strict_mode: false,
                    span_offset,
                    combine_surrogate_pair: self.unicode_mode,
                },
            )
            .parse()?;
            Ok(body)
        } else {
            Ok(parse_regexp_literal(self.allocator, source_text, span_offset, self.unicode_mode))
        }
    }
}

pub struct Reader<'a> {
    source_text: &'a str,
    collector: Collector<'a>,
    units: Vec<'a, StringLiteralAst::CodePoint>,
    index: usize,
}

impl<'a> Reader<'a> {
    pub fn new(
        allocator: &'a Allocator,
        source_text: &'a str,
        unicode_mode: bool,
        parse_string_literal: bool,
    ) -> Self {
        Self {
            source_text,
            collector: Collector::new(allocator, unicode_mode, parse_string_literal),
            units: Vec::new_in(allocator),
            index: 0,
        }
    }

    /// Collects iterating units from the source text.
    /// This method is separated to avoid `new() -> Result<Self>` signature.
    /// This must be called before any other methods.
    pub fn collect_units(&mut self) -> Result<()> {
        self.units = self.collector.collect(self.source_text)?;
        Ok(())
    }

    pub fn start_span(&self) -> Span {
        let unit = self.units.get(self.index);
        // TODO: Why this fails??
        // debug_assert!(
        //     unit.is_some(),
        //     "🦄 INDEX was None: {}/{} in {}",
        //     self.index,
        //     self.units.len(),
        //     self.source_text,
        // );

        let start = unit.map_or(0, |unit| unit.span.start);
        Span::new(start, 0)
    }

    pub fn end_span(&self, mut span: Span) -> Span {
        let unit = self.units.get(self.index - 1);
        debug_assert!(unit.is_some());

        let end = unit.map_or(span.end, |unit| unit.span.end);
        span.end = end;
        debug_assert!(span.end >= span.start);

        span
    }

    pub fn atom(&self, span: Span) -> Atom<'a> {
        Atom::from(span.source_text(self.source_text))
    }

    // NOTE: For now, `usize` is enough for the checkpoint.
    pub fn checkpoint(&self) -> usize {
        self.index
    }

    pub fn rewind(&mut self, checkpoint: usize) {
        self.index = checkpoint;
    }

    pub fn advance(&mut self) {
        self.index += 1;
    }

    fn peek_nth(&self, n: usize) -> Option<u32> {
        let nth = self.index + n;
        self.units.get(nth).map(|cp| cp.value)
    }

    pub fn peek(&self) -> Option<u32> {
        self.peek_nth(0)
    }

    pub fn peek2(&self) -> Option<u32> {
        self.peek_nth(1)
    }

    pub fn eat(&mut self, ch: char) -> bool {
        if self.peek_nth(0) == Some(ch as u32) {
            self.advance();
            return true;
        }
        false
    }

    pub fn eat2(&mut self, ch: char, ch2: char) -> bool {
        if self.peek_nth(0) == Some(ch as u32) && self.peek_nth(1) == Some(ch2 as u32) {
            self.advance();
            self.advance();
            return true;
        }
        false
    }

    pub fn eat3(&mut self, ch: char, ch2: char, ch3: char) -> bool {
        if self.peek_nth(0) == Some(ch as u32)
            && self.peek_nth(1) == Some(ch2 as u32)
            && self.peek_nth(2) == Some(ch3 as u32)
        {
            self.advance();
            self.advance();
            self.advance();
            return true;
        }
        false
    }

    pub fn eat4(&mut self, ch: char, ch2: char, ch3: char, ch4: char) -> bool {
        if self.peek_nth(0) == Some(ch as u32)
            && self.peek_nth(1) == Some(ch2 as u32)
            && self.peek_nth(2) == Some(ch3 as u32)
            && self.peek_nth(3) == Some(ch4 as u32)
        {
            self.advance();
            self.advance();
            self.advance();
            self.advance();
            return true;
        }
        false
    }
}
