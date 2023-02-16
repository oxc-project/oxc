//! AST Printer with whitespace minification
//! code adapted from [esbuild](https://github.com/evanw/esbuild/blob/main/internal/js_printer/js_printer.go)

#![feature(let_chains)]

mod gen;

#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::gen::Gen;

#[derive(Debug, Clone, Copy)]
pub struct PrinterOptions {
    pub minify_whitespace: bool,
    pub indentation: u8,
}

impl Default for PrinterOptions {
    fn default() -> Self {
        Self { minify_whitespace: false, indentation: 4 }
    }
}

pub struct Printer {
    options: PrinterOptions,

    /// Output Code
    code: Vec<u8>,

    /// Current indentation tracking
    indentation: u8,

    // states
    needs_semicolon: bool,
    prev_op_end: usize,
    prev_op: Option<Operator>,
}

#[derive(Debug, Clone, Copy)]
pub enum Separator {
    Comma,
    Semicolon,
    None,
}

/// Codegen interface for pretty print or minification
impl Printer {
    #[must_use]
    pub fn new(source_len: usize, options: PrinterOptions) -> Self {
        // Initialize the output code buffer to reduce memory reallocation.
        // Minification will reduce by at least half the original size,
        // so in fact no reallocation should happen at all.
        let capacity = if options.minify_whitespace { source_len / 2 } else { source_len };
        Self {
            options,
            code: Vec::with_capacity(capacity),
            indentation: 0,
            needs_semicolon: false,
            prev_op_end: 0,
            prev_op: None,
        }
    }

    #[must_use]
    pub fn build(mut self, program: &Program<'_>) -> String {
        program.gen(&mut self);
        unsafe { String::from_utf8_unchecked(self.code) }
    }

    #[must_use]
    pub const fn code(&self) -> &Vec<u8> {
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
        if !self.options.minify_whitespace {
            self.code.push(b' ');
        }
    }

    #[inline]
    pub fn print_newline(&mut self) {
        if !self.options.minify_whitespace {
            self.code.push(b'\n');
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

    fn print_space_before_operator(&mut self, next: Operator) {
        if self.prev_op_end != self.code.len() {
            return;
        }
        let Some(prev) = self.prev_op else { return };
        // "+ + y" => "+ +y"
        // "+ ++ y" => "+ ++y"
        // "x + + y" => "x+ +y"
        // "x ++ + y" => "x+++y"
        // "x + ++ y" => "x+ ++y"
        // "-- >" => "-- >"
        // "< ! --" => "<! --"
        let bin_op_add = Operator::BinaryOperator(BinaryOperator::Addition);
        let bin_op_sub = Operator::BinaryOperator(BinaryOperator::Subtraction);
        let un_op_pos = Operator::UnaryOperator(UnaryOperator::UnaryPlus);
        let un_op_pre_inc = Operator::UpdateOperator(UpdateOperator::Increment);
        let un_op_neg = Operator::UnaryOperator(UnaryOperator::UnaryNegation);
        let un_op_pre_dec = Operator::UpdateOperator(UpdateOperator::Decrement);
        let un_op_post_dec = Operator::UpdateOperator(UpdateOperator::Decrement);
        let bin_op_gt = Operator::BinaryOperator(BinaryOperator::GreaterThan);
        let un_op_not = Operator::UnaryOperator(UnaryOperator::LogicalNot);
        if ((prev == bin_op_add || prev == un_op_pos)
            && (next == bin_op_add || next == un_op_pos || next == un_op_pre_inc))
            || ((prev == bin_op_sub || prev == un_op_neg)
                && (next == bin_op_sub || next == un_op_neg || next == un_op_pre_dec))
            || (prev == un_op_post_dec && next == bin_op_gt)
            || (prev == un_op_not && next == un_op_pre_dec && self.code().len() > 1/*&& p.js[len(p.js)-2] == '<'*/)
        {
            self.print(b' ');
        }
    }

    fn print_semicolon_after_statement(&mut self) {
        if self.options.minify_whitespace {
            self.needs_semicolon = true;
        } else {
            self.print_semicolon();
            self.print(b'\n');
        }
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
        if !self.options.minify_whitespace {
            for _ in 0..self.indentation {
                self.print(b' ');
            }
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

    pub fn print_identifier(&mut self, name: &[u8]) {
        self.print_str(name);
    }

    #[must_use]
    pub fn last_char(&self) -> Option<&u8> {
        self.code.last()
    }
}
