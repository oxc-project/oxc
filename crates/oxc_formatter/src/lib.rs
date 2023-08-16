//! AST Formatter with whitespace minification
//! code adapted from [esbuild](https://github.com/evanw/esbuild/blob/main/internal/js_formatter/js_formatter.go)

mod gen;

#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

pub use crate::gen::Gen;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndOfLine<'a> {
    /// Line Feed only (`\n`), common on Linux and macOS as well as inside git repos
    LF,
    /// Carriage Return + Line Feed characters (`\r\n`), common on Windows
    CRLF,
    /// Carriage Return character only (`\r`), used very rarely
    CR,
    /// Maintain existing line endings (mixed values within one file are normalised by looking at what’s used after the first line)
    Auto(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalEndOfLine {
    LF,
    CRLF,
    CR,
}

#[derive(Debug, Clone, Copy)]
pub struct FormatterOptions<'a> {
    pub indentation: u8,
    pub end_of_line: EndOfLine<'a>,
}

impl Default for FormatterOptions<'_> {
    fn default() -> Self {
        Self { indentation: 4, end_of_line: EndOfLine::LF }
    }
}

pub struct Formatter<'a> {
    options: FormatterOptions<'a>,

    /// Output Code
    code: Vec<u8>,

    /// Current indentation tracking
    indentation: u8,

    // states
    needs_semicolon: bool,

    // Quote property with double quotes
    quote_property_with_double_quotes: bool,

    final_end_of_line: FinalEndOfLine,
}

#[derive(Debug, Clone, Copy)]
pub enum Separator {
    Comma,
    Semicolon,
    None,
}

/// Codegen interface for pretty print or minification
impl<'a> Formatter<'a> {
    pub fn new(source_len: usize, options: FormatterOptions<'a>) -> Self {
        Self {
            options,
            code: Vec::with_capacity(source_len),
            indentation: 0,
            needs_semicolon: false,
            quote_property_with_double_quotes: false,
            final_end_of_line: get_final_end_of_line(options.end_of_line),
        }
    }

    pub fn build(mut self, program: &Program<'_>) -> String {
        program.gen(&mut self);
        self.into_code()
    }

    #[inline]
    pub fn into_code(self) -> String {
        unsafe { String::from_utf8_unchecked(self.code) }
    }

    pub fn code(&self) -> &Vec<u8> {
        &self.code
    }

    /// Push a single character into the buffer
    #[inline]
    pub fn print(&mut self, ch: u8) {
        self.code.push(ch);
    }

    /// Push a string into the buffer
    #[inline]
    pub fn print_str(&mut self, s: &[u8]) {
        self.code.extend_from_slice(s);
    }

    #[inline]
    pub fn print_space(&mut self) {
        self.code.push(b' ');
    }

    #[inline]
    pub fn print_newline(&mut self) {
        match self.final_end_of_line {
            FinalEndOfLine::LF => {
                self.code.push(b'\n');
            }
            FinalEndOfLine::CRLF => {
                self.code.push(b'\r');
                self.code.push(b'\n');
            }
            FinalEndOfLine::CR => {
                self.code.push(b'\r');
            }
        }
    }

    #[inline]
    pub fn indent(&mut self) {
        self.indentation += self.options.indentation;
    }

    #[inline]
    pub fn dedent(&mut self) {
        self.indentation -= self.options.indentation;
    }

    #[inline]
    pub fn print_semicolon(&mut self) {
        self.print(b';');
    }

    #[inline]
    pub fn print_comma(&mut self) {
        self.print(b',');
    }

    fn print_semicolon_after_statement(&mut self) {
        self.print_semicolon();
        self.print_newline();
    }

    fn print_semicolon_if_needed(&mut self) {
        if self.needs_semicolon {
            self.print_semicolon();
            self.needs_semicolon = false;
        }
    }

    #[inline]
    pub fn print_ellipsis(&mut self) {
        self.print_str(b"...");
    }

    #[inline]
    pub fn print_colon(&mut self) {
        self.print(b':');
    }

    #[inline]
    pub fn print_equal(&mut self) {
        self.print(b'=');
    }

    pub fn print_indent(&mut self) {
        for _ in 0..self.indentation {
            self.print(b' ');
        }
    }

    #[inline]
    pub fn print_sequence<T: Gen>(&mut self, items: &[T], separator: Separator) {
        let len = items.len();
        for (index, item) in items.iter().enumerate() {
            item.gen(self);
            match separator {
                Separator::Semicolon => self.print_semicolon(),
                Separator::Comma => self.print(b','),
                Separator::None => {}
            }
            if index != len - 1 {
                self.print_newline();
            }
        }
    }

    #[inline]
    pub fn print_body(&mut self, stmt: &Statement<'_>) {
        if let Statement::BlockStatement(block) = stmt {
            self.print_space();
            self.print_block1(block);
            self.print_newline();
        } else {
            self.print_newline();
            self.indent();
            stmt.gen(self);
            self.dedent();
        }
    }

    #[inline]
    pub fn print_block1(&mut self, stmt: &BlockStatement<'_>) {
        self.print(b'{');
        self.print_newline();
        self.indent();
        for item in &stmt.body {
            self.print_semicolon_if_needed();
            item.gen(self);
        }
        self.dedent();
        self.needs_semicolon = false;
        self.print_indent();
        self.print(b'}');
    }

    #[inline]
    pub fn print_block<T: Gen>(&mut self, items: &[T], separator: Separator) {
        self.print(b'{');
        self.indent();
        if !items.is_empty() {
            self.print_newline();
        }
        self.print_sequence(items, separator);
        self.dedent();
        if !items.is_empty() {
            self.print_newline();
        }
        self.print(b'}');
    }

    #[inline]
    pub fn print_list<T: Gen>(&mut self, items: &[T]) {
        for (index, item) in items.iter().enumerate() {
            if index != 0 {
                self.print_comma();
                self.print_space();
            }
            item.gen(self);
        }
    }

    pub fn last_char(&self) -> Option<&u8> {
        self.code.last()
    }
}

fn auto_detect_end_of_line(raw_input_text: &str) -> FinalEndOfLine {
    let first_line_feed_pos = raw_input_text.chars().position(|ch| ch == '\n');
    first_line_feed_pos.map_or(FinalEndOfLine::CR, |first_line_feed_pos| {
        let char_before_line_feed_pos = first_line_feed_pos.saturating_sub(1);
        let char_before_line_feed = raw_input_text.chars().nth(char_before_line_feed_pos);
        match char_before_line_feed {
            Some('\r') => FinalEndOfLine::CRLF,
            _ => FinalEndOfLine::LF,
        }
    })
}

fn get_final_end_of_line(eol: EndOfLine) -> FinalEndOfLine {
    match eol {
        EndOfLine::Auto(raw_input) => auto_detect_end_of_line(raw_input),
        EndOfLine::LF => FinalEndOfLine::LF,
        EndOfLine::CRLF => FinalEndOfLine::CRLF,
        EndOfLine::CR => FinalEndOfLine::CR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_detects_lf() {
        assert_eq!(FinalEndOfLine::LF, auto_detect_end_of_line("One\nTwo\nThree"));
    }

    #[test]
    fn auto_detects_crlf() {
        assert_eq!(FinalEndOfLine::CRLF, auto_detect_end_of_line("One\r\nTwo\r\nThree"));
    }

    #[test]
    fn auto_detects_cr() {
        assert_eq!(FinalEndOfLine::CR, auto_detect_end_of_line("One\rTwo\rThree"));
    }
}
