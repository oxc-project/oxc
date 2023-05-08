//! AST Formatter with whitespace minification
//! code adapted from [esbuild](https://github.com/evanw/esbuild/blob/main/internal/js_formatter/js_formatter.go)

#![feature(let_chains)]

mod gen;

#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

pub use crate::gen::Gen;

#[derive(Debug, Clone, Copy)]
pub struct FormatterOptions {
    pub indentation: u8,
}

impl Default for FormatterOptions {
    fn default() -> Self {
        Self { indentation: 4 }
    }
}

pub struct Formatter {
    options: FormatterOptions,

    /// Output Code
    code: Vec<u8>,

    /// Current indentation tracking
    indentation: u8,

    // states
    needs_semicolon: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Separator {
    Comma,
    Semicolon,
    None,
}

/// Codegen interface for pretty print or minification
impl Formatter {
    #[must_use]
    pub fn new(source_len: usize, options: FormatterOptions) -> Self {
        Self {
            options,
            code: Vec::with_capacity(source_len),
            indentation: 0,
            needs_semicolon: false,
        }
    }

    #[must_use]
    pub fn build(mut self, program: &Program<'_>) -> String {
        program.gen(&mut self);
        self.into_code()
    }

    #[must_use]
    #[inline]
    pub fn into_code(self) -> String {
        unsafe { String::from_utf8_unchecked(self.code) }
    }

    #[must_use]
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
        self.code.push(b'\n');
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
        self.print(b'\n');
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

    #[must_use]
    pub fn last_char(&self) -> Option<&u8> {
        self.code.last()
    }
}
