//! Oxc Codegen
//!
//! Code adapted from
//! * [esbuild](https://github.com/evanw/esbuild/blob/v0.24.0/internal/js_printer/js_printer.go)
#![warn(missing_docs)]

mod binary_expr_visitor;
mod code_buffer;
mod comment;
mod context;
mod gen;
mod operator;
mod options;
mod sourcemap_builder;

use std::borrow::Cow;

use oxc_ast::ast::{
    BindingIdentifier, BlockStatement, Comment, Expression, IdentifierReference, Program, Statement,
};
use oxc_data_structures::stack::Stack;
use oxc_semantic::SymbolTable;
use oxc_span::{GetSpan, Span, SPAN};
use oxc_syntax::{
    identifier::{is_identifier_part, is_identifier_part_ascii, LS, PS},
    operator::{BinaryOperator, UnaryOperator, UpdateOperator},
    precedence::Precedence,
};

use crate::{
    binary_expr_visitor::BinaryExpressionVisitor, code_buffer::CodeBuffer, comment::CommentsMap,
    operator::Operator, sourcemap_builder::SourcemapBuilder,
};
pub use crate::{
    context::Context,
    gen::{Gen, GenExpr},
    options::{CodegenOptions, LegalComment},
};

/// Code generator without whitespace removal.
pub type CodeGenerator<'a> = Codegen<'a>;

/// Output from [`Codegen::build`]
#[non_exhaustive]
pub struct CodegenReturn {
    /// The generated source code.
    pub code: String,

    /// The source map from the input source code to the generated source code.
    ///
    /// You must set [`CodegenOptions::source_map_path`] for this to be [`Some`].
    pub map: Option<oxc_sourcemap::SourceMap>,

    /// All the legal comments returned from [LegalComment::Linked] or [LegalComment::External].
    pub legal_comments: Vec<Comment>,
}

/// A code generator for printing JavaScript and TypeScript code.
///
/// ## Example
/// ```rust
/// use oxc_codegen::{Codegen, CodegenOptions};
/// use oxc_ast::ast::Program;
/// use oxc_parser::Parser;
/// use oxc_allocator::Allocator;
/// use oxc_span::SourceType;
///
/// let allocator = Allocator::default();
/// let source = "const a = 1 + 2;";
/// let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
/// assert!(parsed.errors.is_empty());
///
/// let js = Codegen::new().build(&parsed.program);
/// assert_eq!(js.code, "const a = 1 + 2;\n");
/// ```
pub struct Codegen<'a> {
    pub(crate) options: CodegenOptions,

    /// Original source code of the AST
    source_text: &'a str,

    symbol_table: Option<SymbolTable>,

    /// Output Code
    code: CodeBuffer,

    // states
    prev_op_end: usize,
    prev_reg_exp_end: usize,
    need_space_before_dot: usize,
    print_next_indent_as_space: bool,
    binary_expr_stack: Stack<BinaryExpressionVisitor<'a>>,
    /// Indicates the output is JSX type, it is set in [`Program::gen`] and the result
    /// is obtained by [`oxc_span::SourceType::is_jsx`]
    is_jsx: bool,

    /// For avoiding `;` if the previous statement ends with `}`.
    needs_semicolon: bool,

    prev_op: Option<Operator>,

    start_of_stmt: usize,
    start_of_arrow_expr: usize,
    start_of_default_export: usize,

    /// Track the current indentation level
    indent: u32,

    /// Fast path for [CodegenOptions::single_quote]
    quote: u8,
    /// Fast path for if print comments
    print_comments: bool,

    // Builders
    comments: CommentsMap,

    legal_comments: Vec<Comment>,
    /// Start of comment that needs to be moved to the before VariableDeclarator
    ///
    /// For example:
    /// ```js
    ///  /* @__NO_SIDE_EFFECTS__ */ export const a = function() {
    ///  }, b = 10000;
    /// ```
    /// Should be generated as:
    /// ```js
    ///   export const /* @__NO_SIDE_EFFECTS__ */ a = function() {
    ///  }, b = 10000;
    /// ```
    start_of_annotation_comment: Option<u32>,

    sourcemap_builder: Option<SourcemapBuilder>,
}

impl Default for Codegen<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<Codegen<'a>> for String {
    fn from(val: Codegen<'a>) -> Self {
        val.into_source_text()
    }
}

impl<'a> From<Codegen<'a>> for Cow<'a, str> {
    fn from(val: Codegen<'a>) -> Self {
        Cow::Owned(val.into_source_text())
    }
}

// Public APIs
impl<'a> Codegen<'a> {
    /// Create a new code generator.
    ///
    /// This is equivalent to [`Codegen::default`].
    #[must_use]
    pub fn new() -> Self {
        let options = CodegenOptions::default();
        let print_comments = options.print_comments();
        Self {
            options,
            source_text: "",
            symbol_table: None,
            code: CodeBuffer::default(),
            needs_semicolon: false,
            need_space_before_dot: 0,
            print_next_indent_as_space: false,
            binary_expr_stack: Stack::with_capacity(12),
            prev_op_end: 0,
            prev_reg_exp_end: 0,
            prev_op: None,
            start_of_stmt: 0,
            start_of_arrow_expr: 0,
            start_of_default_export: 0,
            is_jsx: false,
            indent: 0,
            quote: b'"',
            print_comments,
            comments: CommentsMap::default(),
            start_of_annotation_comment: None,
            legal_comments: vec![],
            sourcemap_builder: None,
        }
    }

    /// Pass options to the code generator.
    #[must_use]
    pub fn with_options(mut self, options: CodegenOptions) -> Self {
        self.quote = if options.single_quote { b'\'' } else { b'"' };
        self.print_comments = options.print_comments();
        self.options = options;
        self
    }

    /// Set the symbol table used for identifier renaming.
    ///
    /// Can be used for easy renaming of variables (based on semantic analysis).
    #[must_use]
    pub fn with_symbol_table(mut self, symbol_table: Option<SymbolTable>) -> Self {
        self.symbol_table = symbol_table;
        self
    }

    /// Print a [`Program`] into a string of source code.
    ///
    /// A source map will be generated if [`CodegenOptions::source_map_path`] is set.
    #[must_use]
    pub fn build(mut self, program: &Program<'a>) -> CodegenReturn {
        self.quote = if self.options.single_quote { b'\'' } else { b'"' };
        self.source_text = program.source_text;
        self.code.reserve(program.source_text.len());
        if self.print_comments {
            if program.comments.is_empty() {
                self.print_comments = false;
            } else {
                self.build_comments(&program.comments);
            }
        }
        if let Some(path) = &self.options.source_map_path {
            self.sourcemap_builder = Some(SourcemapBuilder::new(path, program.source_text));
        }
        program.print(&mut self, Context::default());
        self.try_print_eof_legal_comments();
        let code = self.code.into_string();
        let map = self.sourcemap_builder.map(SourcemapBuilder::into_sourcemap);
        CodegenReturn { code, map, legal_comments: self.legal_comments }
    }

    /// Turn what's been built so far into a string. Like [`build`],
    /// this fininishes a print and returns the generated source code. Unlike
    /// [`build`], no source map is generated.
    ///
    /// This is more useful for cases that progressively build code using [`print_expression`].
    ///
    /// [`build`]: Codegen::build
    /// [`print_expression`]: Codegen::print_expression
    #[must_use]
    pub fn into_source_text(self) -> String {
        self.code.into_string()
    }

    /// Push a single ASCII byte into the buffer.
    ///
    /// # Panics
    /// Panics if `byte` is not an ASCII byte (`0 - 0x7F`).
    #[inline]
    pub fn print_ascii_byte(&mut self, byte: u8) {
        self.code.print_ascii_byte(byte);
    }

    /// Push str into the buffer
    #[inline]
    pub fn print_str(&mut self, s: &str) {
        self.code.print_str(s);
    }

    /// Print a single [`Expression`], adding it to the code generator's
    /// internal buffer. Unlike [`Codegen::build`], this does not consume `self`.
    #[inline]
    pub fn print_expression(&mut self, expr: &Expression<'_>) {
        expr.print_expr(self, Precedence::Lowest, Context::empty());
    }
}

// Private APIs
impl<'a> Codegen<'a> {
    fn code(&self) -> &CodeBuffer {
        &self.code
    }

    fn code_len(&self) -> usize {
        self.code().len()
    }

    #[inline]
    fn print_soft_space(&mut self) {
        if !self.options.minify {
            self.print_ascii_byte(b' ');
        }
    }

    #[inline]
    fn print_hard_space(&mut self) {
        self.print_ascii_byte(b' ');
    }

    #[inline]
    fn print_soft_newline(&mut self) {
        if !self.options.minify {
            self.print_ascii_byte(b'\n');
        }
    }

    #[inline]
    fn print_hard_newline(&mut self) {
        self.print_ascii_byte(b'\n');
    }

    #[inline]
    fn print_semicolon(&mut self) {
        self.print_ascii_byte(b';');
    }

    #[inline]
    fn print_comma(&mut self) {
        self.print_ascii_byte(b',');
    }

    #[inline]
    fn print_space_before_identifier(&mut self) {
        let Some(byte) = self.last_byte() else { return };

        if self.prev_reg_exp_end != self.code.len() {
            let is_identifier = if byte.is_ascii() {
                // Fast path for ASCII (very common case)
                is_identifier_part_ascii(byte as char)
            } else {
                is_identifier_part(self.last_char().unwrap())
            };
            if !is_identifier {
                return;
            }
        }

        self.print_hard_space();
    }

    #[inline]
    fn last_byte(&self) -> Option<u8> {
        self.code.last_byte()
    }

    #[inline]
    fn last_char(&self) -> Option<char> {
        self.code.last_char()
    }

    #[inline]
    fn indent(&mut self) {
        if !self.options.minify {
            self.indent += 1;
        }
    }

    #[inline]
    fn dedent(&mut self) {
        if !self.options.minify {
            self.indent -= 1;
        }
    }

    #[inline]
    fn wrap<F: FnMut(&mut Self)>(&mut self, wrap: bool, mut f: F) {
        if wrap {
            self.print_ascii_byte(b'(');
        }
        f(self);
        if wrap {
            self.print_ascii_byte(b')');
        }
    }

    #[inline]
    fn print_indent(&mut self) {
        if self.options.minify {
            return;
        }
        if self.print_next_indent_as_space {
            self.print_hard_space();
            self.print_next_indent_as_space = false;
            return;
        }
        self.code.print_indent(self.indent as usize);
    }

    #[inline]
    fn print_semicolon_after_statement(&mut self) {
        if self.options.minify {
            self.needs_semicolon = true;
        } else {
            self.print_str(";\n");
        }
    }

    #[inline]
    fn print_semicolon_if_needed(&mut self) {
        if self.needs_semicolon {
            self.print_semicolon();
            self.needs_semicolon = false;
        }
    }

    #[inline]
    fn print_ellipsis(&mut self) {
        self.print_str("...");
    }

    #[inline]
    fn print_colon(&mut self) {
        self.print_ascii_byte(b':');
    }

    #[inline]
    fn print_equal(&mut self) {
        self.print_ascii_byte(b'=');
    }

    fn print_curly_braces<F: FnOnce(&mut Self)>(&mut self, span: Span, single_line: bool, op: F) {
        self.add_source_mapping(span);
        self.print_ascii_byte(b'{');
        if !single_line {
            self.print_soft_newline();
            self.indent();
        }
        op(self);
        if !single_line {
            self.dedent();
            self.print_indent();
        }
        self.add_source_mapping_end(span);
        self.print_ascii_byte(b'}');
    }

    fn print_block_start(&mut self, span: Span) {
        self.add_source_mapping(span);
        self.print_ascii_byte(b'{');
        self.print_soft_newline();
        self.indent();
    }

    fn print_block_end(&mut self, span: Span) {
        self.dedent();
        self.print_indent();
        self.add_source_mapping_end(span);
        self.print_ascii_byte(b'}');
    }

    fn print_body(&mut self, stmt: &Statement<'_>, need_space: bool, ctx: Context) {
        match stmt {
            Statement::BlockStatement(stmt) => {
                self.print_soft_space();
                self.print_block_statement(stmt, ctx);
                self.print_soft_newline();
            }
            Statement::EmptyStatement(_) => {
                self.print_semicolon();
                self.print_soft_newline();
            }
            stmt => {
                if need_space && self.options.minify {
                    self.print_hard_space();
                }
                self.print_next_indent_as_space = true;
                stmt.print(self, ctx);
            }
        }
    }

    fn print_block_statement(&mut self, stmt: &BlockStatement<'_>, ctx: Context) {
        self.print_curly_braces(stmt.span, stmt.body.is_empty(), |p| {
            for stmt in &stmt.body {
                p.print_semicolon_if_needed();
                stmt.print(p, ctx);
            }
        });
        self.needs_semicolon = false;
    }

    // We tried optimizing this to move the `index != 0` check out of the loop:
    // ```
    // let mut iter = items.iter();
    // let Some(item) = iter.next() else { return };
    // item.print(self, ctx);
    // for item in iter {
    //     self.print_comma();
    //     self.print_soft_space();
    //     item.print(self, ctx);
    // }
    // ```
    // But it turned out this was actually a bit slower.
    // <https://github.com/oxc-project/oxc/pull/5221>
    fn print_list<T: Gen>(&mut self, items: &[T], ctx: Context) {
        for (index, item) in items.iter().enumerate() {
            if index != 0 {
                self.print_comma();
                self.print_soft_space();
            }
            item.print(self, ctx);
        }
    }

    fn print_list_with_comments<T: Gen + GetSpan>(&mut self, items: &[T], ctx: Context) {
        for (index, item) in items.iter().enumerate() {
            if index != 0 {
                self.print_comma();
            }
            if self.print_comments && self.has_non_annotation_comment(item.span().start) {
                self.print_expr_comments(item.span().start);
                self.print_indent();
            } else {
                self.print_soft_newline();
                self.print_indent();
            }
            item.print(self, ctx);
        }
    }

    fn print_expressions<T: GenExpr>(&mut self, items: &[T], precedence: Precedence, ctx: Context) {
        for (index, item) in items.iter().enumerate() {
            if index != 0 {
                self.print_comma();
                self.print_soft_space();
            }
            item.print_expr(self, precedence, ctx);
        }
    }

    fn get_identifier_reference_name(&self, reference: &IdentifierReference<'a>) -> &'a str {
        if let Some(symbol_table) = &self.symbol_table {
            if let Some(reference_id) = reference.reference_id.get() {
                if let Some(name) = symbol_table.get_reference_name(reference_id) {
                    // SAFETY: Hack the lifetime to be part of the allocator.
                    return unsafe { std::mem::transmute_copy(&name) };
                }
            }
        }
        reference.name.as_str()
    }

    fn get_binding_identifier_name(&self, ident: &BindingIdentifier<'a>) -> &'a str {
        if let Some(symbol_table) = &self.symbol_table {
            if let Some(symbol_id) = ident.symbol_id.get() {
                let name = symbol_table.get_name(symbol_id);
                // SAFETY: Hack the lifetime to be part of the allocator.
                return unsafe { std::mem::transmute_copy(&name) };
            }
        }
        ident.name.as_str()
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
        let bin_op_add = Operator::Binary(BinaryOperator::Addition);
        let bin_op_sub = Operator::Binary(BinaryOperator::Subtraction);
        let un_op_pos = Operator::Unary(UnaryOperator::UnaryPlus);
        let un_op_pre_inc = Operator::Update(UpdateOperator::Increment);
        let un_op_neg = Operator::Unary(UnaryOperator::UnaryNegation);
        let un_op_pre_dec = Operator::Update(UpdateOperator::Decrement);
        let un_op_post_dec = Operator::Update(UpdateOperator::Decrement);
        let bin_op_gt = Operator::Binary(BinaryOperator::GreaterThan);
        let un_op_not = Operator::Unary(UnaryOperator::LogicalNot);
        if ((prev == bin_op_add || prev == un_op_pos)
            && (next == bin_op_add || next == un_op_pos || next == un_op_pre_inc))
            || ((prev == bin_op_sub || prev == un_op_neg)
                && (next == bin_op_sub || next == un_op_neg || next == un_op_pre_dec))
            || (prev == un_op_post_dec && next == bin_op_gt)
            || (prev == un_op_not
                && next == un_op_pre_dec
                // `prev == UnaryOperator::LogicalNot` which means last byte is ASCII,
                // and therefore previous character is 1 byte from end of buffer
                && self.code.peek_nth_byte_back(1) == Some(b'<'))
        {
            self.print_hard_space();
        }
    }

    fn print_non_negative_float(&mut self, num: f64) {
        // Inline the buffer here to avoid heap allocation on `buffer.format(*self).to_string()`.
        let mut buffer = ryu_js::Buffer::new();
        if num < 1000.0 && num.fract() == 0.0 {
            self.print_str(buffer.format(num));
            self.need_space_before_dot = self.code_len();
        } else {
            let s = Self::get_minified_number(num, &mut buffer);
            self.print_str(&s);
            if !s.bytes().any(|b| matches!(b, b'.' | b'e' | b'x')) {
                self.need_space_before_dot = self.code_len();
            }
        }
    }

    fn print_quoted_utf16(&mut self, s: &str, allow_backtick: bool) {
        let quote = if self.options.minify {
            let mut single_cost: i32 = 0;
            let mut double_cost: i32 = 0;
            let mut backtick_cost: i32 = 0;
            let mut bytes = s.as_bytes().iter().peekable();
            while let Some(b) = bytes.next() {
                match b {
                    b'\n' if self.options.minify => backtick_cost = backtick_cost.saturating_sub(1),
                    b'\'' => single_cost += 1,
                    b'"' => double_cost += 1,
                    b'`' => backtick_cost += 1,
                    b'$' => {
                        if bytes.peek() == Some(&&b'{') {
                            backtick_cost += 1;
                        }
                    }
                    _ => {}
                }
            }
            let mut quote = b'"';
            if double_cost > single_cost {
                quote = b'\'';
                if single_cost > backtick_cost && allow_backtick {
                    quote = b'`';
                }
            } else if double_cost > backtick_cost && allow_backtick {
                quote = b'`';
            }
            quote
        } else {
            self.quote
        };

        self.print_ascii_byte(quote);
        self.print_unquoted_utf16(s, quote);
        self.print_ascii_byte(quote);
    }

    fn print_unquoted_utf16(&mut self, s: &str, quote: u8) {
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '\x00' => {
                    if chars.peek().is_some_and(|&next| next.is_ascii_digit()) {
                        self.print_str("\\x00");
                    } else {
                        self.print_str("\\0");
                    }
                }
                '\x07' => self.print_str("\\x07"),
                '\u{8}' => self.print_str("\\b"), // \b
                '\u{b}' => self.print_str("\\v"), // \v
                '\u{c}' => self.print_str("\\f"), // \f
                '\n' => {
                    if quote == b'`' {
                        self.print_ascii_byte(b'\n');
                    } else {
                        self.print_str("\\n");
                    }
                }
                '\r' => self.print_str("\\r"),
                '\x1B' => self.print_str("\\x1B"),
                '\\' => self.print_str("\\\\"),
                // Allow `U+2028` and `U+2029` in string literals
                // <https://tc39.es/proposal-json-superset>
                // <https://github.com/tc39/proposal-json-superset>
                LS => self.print_str("\\u2028"),
                PS => self.print_str("\\u2029"),
                '\u{a0}' => self.print_str("\\xA0"),
                '\'' => {
                    if quote == b'\'' {
                        self.print_ascii_byte(b'\\');
                    }
                    self.print_ascii_byte(b'\'');
                }
                '\"' => {
                    if quote == b'"' {
                        self.print_ascii_byte(b'\\');
                    }
                    self.print_ascii_byte(b'"');
                }
                '`' => {
                    if quote == b'`' {
                        self.print_ascii_byte(b'\\');
                    }
                    self.print_ascii_byte(b'`');
                }
                '$' => {
                    if chars.peek() == Some(&'{') {
                        self.print_ascii_byte(b'\\');
                    }
                    self.print_ascii_byte(b'$');
                }
                _ => self.print_str(c.encode_utf8([0; 4].as_mut())),
            }
        }
    }

    // `get_minified_number` from terser
    // https://github.com/terser/terser/blob/c5315c3fd6321d6b2e076af35a70ef532f498505/lib/output.js#L2418
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_possible_wrap)]
    fn get_minified_number(num: f64, buffer: &mut ryu_js::Buffer) -> Cow<'_, str> {
        use cow_utils::CowUtils;

        if num < 1000.0 && num.fract() == 0.0 {
            return Cow::Borrowed(buffer.format(num));
        }

        let mut s = buffer.format(num);

        if s.starts_with("0.") {
            s = &s[1..];
        }

        let s = s.cow_replacen("e+", "e", 1);

        let mut candidates = vec![s.clone()];

        if num.fract() == 0.0 {
            candidates.push(Cow::Owned(format!("0x{:x}", num as u128)));
        }

        // create `1e-2`
        if s.starts_with(".0") {
            if let Some((i, _)) = s[1..].bytes().enumerate().find(|(_, c)| *c != b'0') {
                let len = i + 1; // `+1` to include the dot.
                let digits = &s[len..];
                candidates.push(Cow::Owned(format!("{digits}e-{}", digits.len() + len - 1)));
            }
        }

        // create 1e2
        if s.ends_with('0') {
            if let Some((len, _)) = s.bytes().rev().enumerate().find(|(_, c)| *c != b'0') {
                candidates.push(Cow::Owned(format!("{}e{len}", &s[0..s.len() - len])));
            }
        }

        // `1.2e101` -> ("1", "2", "101")
        // `1.3415205933077406e300` -> `13415205933077406e284;`
        if let Some((integer, point, exponent)) =
            s.split_once('.').and_then(|(a, b)| b.split_once('e').map(|e| (a, e.0, e.1)))
        {
            candidates.push(Cow::Owned(format!(
                "{integer}{point}e{}",
                exponent.parse::<isize>().unwrap() - point.len() as isize
            )));
        }

        candidates.into_iter().min_by_key(|c| c.len()).unwrap()
    }

    fn add_source_mapping(&mut self, span: Span) {
        if span == SPAN {
            return;
        }
        if let Some(sourcemap_builder) = self.sourcemap_builder.as_mut() {
            sourcemap_builder.add_source_mapping(self.code.as_bytes(), span.start, None);
        }
    }

    fn add_source_mapping_end(&mut self, span: Span) {
        if span == SPAN {
            return;
        }
        if let Some(sourcemap_builder) = self.sourcemap_builder.as_mut() {
            sourcemap_builder.add_source_mapping(self.code.as_bytes(), span.end, None);
        }
    }

    fn add_source_mapping_for_name(&mut self, span: Span, name: &str) {
        if span == SPAN {
            return;
        }
        if let Some(sourcemap_builder) = self.sourcemap_builder.as_mut() {
            sourcemap_builder.add_source_mapping_for_name(self.code.as_bytes(), span, name);
        }
    }
}
