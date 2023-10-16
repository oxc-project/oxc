//! Prettier
//!
//! This crate is intended to be [prettier](https://prettier.io).
//! Please use the `oxc_codegen ` for code generation.

mod gen;

#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

pub use crate::gen::Gen;

#[derive(Debug, PartialEq, Eq, Clone)]
/// @see [prettier](https://prettier.io/docs/en/options.html#end-of-line)
pub enum EndOfLine {
    /// Line Feed only (`\n`), common on Linux and macOS as well as inside git repos
    LF,
    /// Carriage Return + Line Feed characters (`\r\n`), common on Windows
    CRLF,
    /// Carriage Return character only (`\r`), used very rarely
    CR,
    /// Maintain existing line endings (mixed values within one file are normalised by looking at what’s used after the first line)
    Auto(String),
}

impl EndOfLine {
    pub fn get_final_end_of_line(&self) -> FinalEndOfLine {
        match self {
            Self::Auto(raw_input) => Self::auto_detect_end_of_line(raw_input),
            Self::LF => FinalEndOfLine::LF,
            Self::CRLF => FinalEndOfLine::CRLF,
            Self::CR => FinalEndOfLine::CR,
        }
    }

    pub fn auto_detect_end_of_line(raw_input_text: &str) -> FinalEndOfLine {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalEndOfLine {
    LF,
    CRLF,
    CR,
}

#[derive(Debug, Clone)]
pub struct FormatterOptions {
    pub indentation: u8,
    // <https://prettier.io/docs/en/options#quotes>
    pub single_quote: bool,
    pub end_of_line: EndOfLine,
}

impl Default for FormatterOptions {
    fn default() -> Self {
        Self { indentation: 4, single_quote: false, end_of_line: EndOfLine::LF }
    }
}

#[derive(Debug)]
/// processed and reserved for internal use
pub struct InnerOptions {
    pub indentation: u8,
    pub end_of_line: FinalEndOfLine,
    pub single_quote: bool,
}

impl From<FormatterOptions> for InnerOptions {
    fn from(options: FormatterOptions) -> Self {
        Self {
            indentation: options.indentation,
            single_quote: options.single_quote,
            end_of_line: options.end_of_line.get_final_end_of_line(),
        }
    }
}

pub struct Formatter {
    options: InnerOptions,

    /// Output Code
    code: Vec<u8>,

    /// Current indentation tracking
    indentation: u8,

    // states
    needs_semicolon: bool,

    // Quote property with double quotes
    quote_property_with_double_quotes: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Separator {
    Comma,
    Semicolon,
    None,
}

/// Codegen interface for pretty print or minification
impl Formatter {
    pub fn new(source_len: usize, options: FormatterOptions) -> Self {
        Self {
            options: options.into(),
            code: Vec::with_capacity(source_len),
            indentation: 0,
            needs_semicolon: false,
            quote_property_with_double_quotes: false,
        }
    }

    pub fn build(mut self, program: &Program<'_>) -> String {
        program.gen(&mut self);
        self.into_code()
    }

    #[inline]
    pub fn into_code(self) -> String {
        // SAFETY: criteria of `from_utf8_unchecked`.are met.
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
        match self.options.end_of_line {
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

    #[inline]
    pub fn print_quote(&mut self) {
        self.print(if self.options.single_quote { b'\'' } else { b'"' });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_detects_lf() {
        assert_eq!(FinalEndOfLine::LF, EndOfLine::auto_detect_end_of_line("One\nTwo\nThree"));
    }

    #[test]
    fn auto_detects_crlf() {
        assert_eq!(FinalEndOfLine::CRLF, EndOfLine::auto_detect_end_of_line("One\r\nTwo\r\nThree"));
    }

    #[test]
    fn auto_detects_cr() {
        assert_eq!(FinalEndOfLine::CR, EndOfLine::auto_detect_end_of_line("One\rTwo\rThree"));
    }
}
