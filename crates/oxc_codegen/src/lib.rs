//! Oxc Codegen
//!
//! Code adapted from
//! * [esbuild](https://github.com/evanw/esbuild/blob/v0.24.0/internal/js_printer/js_printer.go)

#![warn(missing_docs)]

use std::{borrow::Cow, cmp, slice};

use cow_utils::CowUtils;

use oxc_ast::ast::*;
use oxc_data_structures::{code_buffer::CodeBuffer, stack::Stack};
use oxc_semantic::Scoping;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    identifier::{is_identifier_part, is_identifier_part_ascii},
    operator::{BinaryOperator, UnaryOperator, UpdateOperator},
    precedence::Precedence,
};

mod binary_expr_visitor;
mod comment;
mod context;
mod r#gen;
mod operator;
mod options;
mod sourcemap_builder;
mod str;

use binary_expr_visitor::BinaryExpressionVisitor;
use comment::CommentsMap;
use operator::Operator;
use sourcemap_builder::SourcemapBuilder;
use str::{Quote, cold_branch, is_script_close_tag};

pub use context::Context;
pub use r#gen::{Gen, GenExpr};
pub use options::{CodegenOptions, CommentOptions, LegalComment};

// Re-export `IndentChar` from `oxc_data_structures`
pub use oxc_data_structures::code_buffer::IndentChar;

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
    source_text: Option<&'a str>,

    scoping: Option<Scoping>,

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
    quote: Quote,

    // Builders
    comments: CommentsMap,

    sourcemap_builder: Option<SourcemapBuilder<'a>>,
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
        Self {
            options,
            source_text: None,
            scoping: None,
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
            quote: Quote::Double,
            comments: CommentsMap::default(),
            sourcemap_builder: None,
        }
    }

    /// Pass options to the code generator.
    #[must_use]
    pub fn with_options(mut self, options: CodegenOptions) -> Self {
        self.quote = if options.single_quote { Quote::Single } else { Quote::Double };
        self.code = CodeBuffer::with_indent(options.indent_char, options.indent_width);
        self.options = options;
        self
    }

    /// Sets the source text for the code generator.
    #[must_use]
    pub fn with_source_text(mut self, source_text: &'a str) -> Self {
        self.source_text = Some(source_text);
        self
    }

    /// Set the symbol table used for identifier renaming.
    ///
    /// Can be used for easy renaming of variables (based on semantic analysis).
    #[must_use]
    pub fn with_scoping(mut self, scoping: Option<Scoping>) -> Self {
        self.scoping = scoping;
        self
    }

    /// Print a [`Program`] into a string of source code.
    ///
    /// A source map will be generated if [`CodegenOptions::source_map_path`] is set.
    #[must_use]
    pub fn build(mut self, program: &Program<'a>) -> CodegenReturn {
        self.quote = if self.options.single_quote { Quote::Single } else { Quote::Double };
        self.source_text = Some(program.source_text);
        self.code.reserve(program.source_text.len());
        self.build_comments(&program.comments);
        if let Some(path) = &self.options.source_map_path {
            self.sourcemap_builder = Some(SourcemapBuilder::new(path, program.source_text));
        }
        program.print(&mut self, Context::default());
        let legal_comments = self.handle_eof_linked_or_external_comments(program);
        let code = self.code.into_string();
        let map = self.sourcemap_builder.map(SourcemapBuilder::into_sourcemap);
        CodegenReturn { code, map, legal_comments }
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

    /// Push str into the buffer, escaping `</script` to `<\/script`.
    #[inline]
    pub fn print_str_escaping_script_close_tag(&mut self, s: &str) {
        // `</script` will be very rare. So we try to make the search as quick as possible by:
        // 1. Searching for `<` first, and only checking if followed by `/script` once `<` is found.
        // 2. Searching longer strings for `<` in chunks of 16 bytes using SIMD, and only doing the
        //    more expensive byte-by-byte search once a `<` is found.

        let bytes = s.as_bytes();
        let mut consumed = 0;

        #[expect(clippy::unnecessary_safety_comment)]
        // Search range of bytes for `</script`, byte by byte.
        //
        // Bytes between `ptr` and `last_ptr` (inclusive) are searched for `<`.
        // If `<` is found, the following 7 bytes are checked to see if they're `/script`.
        //
        // SAFETY:
        // * `ptr` and `last_ptr` must be within bounds of `bytes`.
        // * `last_ptr` must be greater or equal to `ptr`.
        // * `last_ptr` must be no later than 8 bytes before end of string.
        //   i.e. safe to read 8 bytes at `end_ptr`.
        let mut search_bytes = |mut ptr: *const u8, last_ptr| {
            loop {
                // SAFETY: `ptr` is always less than or equal to `last_ptr`.
                // `last_ptr` is within bounds of `bytes`, so safe to read a byte at `ptr`.
                let byte = unsafe { *ptr.as_ref().unwrap_unchecked() };
                if byte == b'<' {
                    // SAFETY: `ptr <= last_ptr`, and `last_ptr` points to no later than
                    // 8 bytes before end of string, so safe to read 8 bytes from `ptr`
                    let slice = unsafe { slice::from_raw_parts(ptr, 8) };
                    if is_script_close_tag(slice) {
                        // Push str up to and including `<`. Skip `/`. Write `\/` instead.
                        // SAFETY:
                        // `consumed` is initially 0, and only updated below to be after `/`,
                        // so in bounds, and on a UTF-8 char boundary.
                        // `index` is on `<`, so `index + 1` is in bounds and a UTF-8 char boundary.
                        // `consumed` is always less than `index + 1` as it's set on a previous round.
                        unsafe {
                            let index = ptr.offset_from_unsigned(bytes.as_ptr());
                            let before = bytes.get_unchecked(consumed..=index);
                            self.code.print_bytes_unchecked(before);

                            // Set `consumed` to after `/`
                            consumed = index + 2;
                        }
                        self.print_str("\\/");
                        // Note: We could advance `ptr` by 8 bytes here to skip over `</script`,
                        // but this branch will be very rarely taken, so it's better to keep it simple
                    }
                }

                if ptr == last_ptr {
                    break;
                }
                // SAFETY: `ptr` is less than `last_ptr`, which is in bounds, so safe to increment `ptr`
                ptr = unsafe { ptr.add(1) };
            }
        };

        // Search string in chunks of 16 bytes
        let mut chunks = bytes.chunks_exact(16);
        for (chunk_index, chunk) in chunks.by_ref().enumerate() {
            #[expect(clippy::missing_panics_doc, reason = "infallible")]
            let chunk: &[u8; 16] = chunk.try_into().unwrap();

            // Compiler vectorizes this loop to a few SIMD ops
            let mut contains_lt = false;
            for &byte in chunk {
                if byte == b'<' {
                    contains_lt = true;
                }
            }

            if contains_lt {
                // Chunk contains at least one `<`.
                // Find them, and check if they're the start of `</script`.
                //
                // SAFETY: `index` is byte index of start of chunk.
                // We search bytes starting with first byte of chunk, and ending with last byte of chunk.
                // i.e. `index` to `index + 15` (inclusive).
                // If this chunk is towards the end of the string, reduce the range of bytes searched
                // so the last byte searched has at least 7 further bytes after it.
                // i.e. safe to read 8 bytes at `last_ptr`.
                cold_branch(|| unsafe {
                    let index = chunk_index * 16;
                    let remaining_bytes = bytes.len() - index;
                    let last_offset = cmp::min(remaining_bytes - 8, 15);
                    let ptr = bytes.as_ptr().add(index);
                    let last_ptr = ptr.add(last_offset);
                    search_bytes(ptr, last_ptr);
                });
            }
        }

        // Search last chunk byte-by-byte.
        // Skip this if less than 8 bytes remaining, because less than 8 bytes can't contain `</script`.
        let last_chunk = chunks.remainder();
        if last_chunk.len() >= 8 {
            let ptr = last_chunk.as_ptr();
            // SAFETY: `last_chunk.len() >= 8`, so `- 8` cannot wrap.
            // `last_chunk.as_ptr().add(last_chunk.len() - 8)` is in bounds of `last_chunk`.
            let last_ptr = unsafe { ptr.add(last_chunk.len() - 8) };
            search_bytes(ptr, last_ptr);
        }

        // SAFETY: `consumed` is either 0, or after `/`, so on a UTF-8 char boundary, and in bounds
        unsafe {
            let remaining = bytes.get_unchecked(consumed..);
            self.code.print_bytes_unchecked(remaining);
        }
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

    fn print_directives_and_statements(
        &mut self,
        directives: &[Directive<'_>],
        stmts: &[Statement<'_>],
        ctx: Context,
    ) {
        for directive in directives {
            directive.print(self, ctx);
        }
        let Some((first, rest)) = stmts.split_first() else {
            return;
        };

        // Ensure first string literal is not a directive.
        let mut first_needs_parens = false;
        if directives.is_empty() && !self.options.minify {
            if let Statement::ExpressionStatement(s) = first {
                let s = s.expression.without_parentheses();
                if matches!(s, Expression::StringLiteral(_)) {
                    first_needs_parens = true;
                    self.print_ascii_byte(b'(');
                    s.print_expr(self, Precedence::Lowest, ctx);
                    self.print_ascii_byte(b')');
                    self.print_semicolon_after_statement();
                }
            }
        }

        if !first_needs_parens {
            first.print(self, ctx);
        }

        for stmt in rest {
            self.print_semicolon_if_needed();
            stmt.print(self, ctx);
        }
    }

    #[inline]
    fn print_list<T: Gen>(&mut self, items: &[T], ctx: Context) {
        let Some((first, rest)) = items.split_first() else {
            return;
        };
        first.print(self, ctx);
        for item in rest {
            self.print_comma();
            self.print_soft_space();
            item.print(self, ctx);
        }
    }

    #[inline]
    fn print_expressions<T: GenExpr>(&mut self, items: &[T], precedence: Precedence, ctx: Context) {
        let Some((first, rest)) = items.split_first() else {
            return;
        };
        first.print_expr(self, precedence, ctx);
        for item in rest {
            self.print_comma();
            self.print_soft_space();
            item.print_expr(self, precedence, ctx);
        }
    }

    fn print_arguments(&mut self, span: Span, arguments: &[Argument<'_>], ctx: Context) {
        self.print_ascii_byte(b'(');

        let has_comment_before_right_paren = span.end > 0 && self.has_comment(span.end - 1);

        let has_comment = has_comment_before_right_paren
            || arguments.iter().any(|item| self.has_comment(item.span().start));

        if has_comment {
            self.indent();
            self.print_list_with_comments(arguments, ctx);
            // Handle `/* comment */);`
            if !has_comment_before_right_paren
                || (span.end > 0 && !self.print_expr_comments(span.end - 1))
            {
                self.print_soft_newline();
            }
            self.dedent();
            self.print_indent();
        } else {
            self.print_list(arguments, ctx);
        }
        self.print_ascii_byte(b')');
    }

    fn print_list_with_comments(&mut self, items: &[Argument<'_>], ctx: Context) {
        let Some((first, rest)) = items.split_first() else {
            return;
        };
        if self.print_expr_comments(first.span().start) {
            self.print_indent();
        } else {
            self.print_soft_newline();
            self.print_indent();
        }
        first.print(self, ctx);
        for item in rest {
            self.print_comma();
            if self.print_expr_comments(item.span().start) {
                self.print_indent();
            } else {
                self.print_soft_newline();
                self.print_indent();
            }
            item.print(self, ctx);
        }
    }

    fn get_identifier_reference_name(&self, reference: &IdentifierReference<'a>) -> &'a str {
        if let Some(scoping) = &self.scoping {
            if let Some(reference_id) = reference.reference_id.get() {
                if let Some(name) = scoping.get_reference_name(reference_id) {
                    // SAFETY: Hack the lifetime to be part of the allocator.
                    return unsafe { std::mem::transmute_copy(&name) };
                }
            }
        }
        reference.name.as_str()
    }

    fn get_binding_identifier_name(&self, ident: &BindingIdentifier<'a>) -> &'a str {
        if let Some(scoping) = &self.scoping {
            if let Some(symbol_id) = ident.symbol_id.get() {
                let name = scoping.symbol_name(symbol_id);
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
        let mut buffer = dragonbox_ecma::Buffer::new();
        if num < 1000.0 && num.fract() == 0.0 {
            self.print_str(buffer.format(num));
            self.need_space_before_dot = self.code_len();
        } else {
            self.print_minified_number(num, &mut buffer);
        }
    }

    fn print_decorators(&mut self, decorators: &[Decorator<'_>], ctx: Context) {
        for decorator in decorators {
            decorator.print(self, ctx);
            self.print_hard_space();
        }
    }

    // Optimized version of `get_minified_number` from terser
    // https://github.com/terser/terser/blob/c5315c3fd6321d6b2e076af35a70ef532f498505/lib/output.js#L2418
    // Instead of building all candidates and finding the shortest, we track the shortest as we go
    // and use self.print_str directly instead of returning intermediate strings
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_possible_wrap)]
    fn print_minified_number(&mut self, num: f64, buffer: &mut dragonbox_ecma::Buffer) {
        if num < 1000.0 && num.fract() == 0.0 {
            self.print_str(buffer.format(num));
            self.need_space_before_dot = self.code_len();
            return;
        }

        let mut s = buffer.format(num);

        if s.starts_with("0.") {
            s = &s[1..];
        }

        let mut best_candidate = s.cow_replacen("e+", "e", 1);
        let mut best_len = best_candidate.len();
        let mut is_hex = false;

        // Track the best candidate found so far
        if num.fract() == 0.0 {
            // For integers, check hex format and other optimizations
            let hex_candidate = format!("0x{:x}", num as u128);
            if hex_candidate.len() < best_len {
                is_hex = true;
                best_candidate = hex_candidate.into();
                best_len = best_candidate.len();
            }
        }
        // Check for scientific notation optimizations for numbers starting with ".0"
        else if best_candidate.starts_with(".0") {
            // Skip the first '0' since we know it's there from the starts_with check
            if let Some(i) = best_candidate.bytes().skip(2).position(|c| c != b'0') {
                let len = i + 2; // `+2` to include the dot and first zero.
                let digits = &best_candidate[len..];
                let exp = digits.len() + len - 1;
                let exp_str_len = itoa::Buffer::new().format(exp).len();
                // Calculate expected length: digits + 'e-' + exp_length
                let expected_len = digits.len() + 2 + exp_str_len;
                if expected_len < best_len {
                    best_candidate = format!("{digits}e-{exp}").into();
                    debug_assert_eq!(best_candidate.len(), expected_len);
                    best_len = best_candidate.len();
                }
            }
        }

        // Check for numbers ending with zeros (but not hex numbers)
        // The `!is_hex` check is necessary to prevent hex numbers like `0x8000000000000000`
        // from being incorrectly converted to scientific notation
        if !is_hex && best_candidate.ends_with('0') {
            if let Some(len) = best_candidate.bytes().rev().position(|c| c != b'0') {
                let base = &best_candidate[0..best_candidate.len() - len];
                let exp_str_len = itoa::Buffer::new().format(len).len();
                // Calculate expected length: base + 'e' + len
                let expected_len = base.len() + 1 + exp_str_len;
                if expected_len < best_len {
                    best_candidate = format!("{base}e{len}").into();
                    debug_assert_eq!(best_candidate.len(), expected_len);
                    best_len = expected_len;
                }
            }
        }

        // Check for scientific notation optimization: `1.2e101` -> `12e100`
        if let Some((integer, point, exponent)) = best_candidate
            .split_once('.')
            .and_then(|(a, b)| b.split_once('e').map(|e| (a, e.0, e.1)))
        {
            let new_expr = exponent.parse::<isize>().unwrap() - point.len() as isize;
            let new_exp_str_len = itoa::Buffer::new().format(new_expr).len();
            // Calculate expected length: integer + point + 'e' + new_exp_str_len
            let expected_len = integer.len() + point.len() + 1 + new_exp_str_len;
            if expected_len < best_len {
                best_candidate = format!("{integer}{point}e{new_expr}").into();
                debug_assert_eq!(best_candidate.len(), expected_len);
            }
        }

        // Print the best candidate and update need_space_before_dot
        self.print_str(&best_candidate);
        if !best_candidate.bytes().any(|b| matches!(b, b'.' | b'e' | b'x')) {
            self.need_space_before_dot = self.code_len();
        }
    }

    fn add_source_mapping(&mut self, span: Span) {
        if let Some(sourcemap_builder) = self.sourcemap_builder.as_mut() {
            if !span.is_empty() {
                sourcemap_builder.add_source_mapping(self.code.as_bytes(), span.start, None);
            }
        }
    }

    fn add_source_mapping_end(&mut self, span: Span) {
        if let Some(sourcemap_builder) = self.sourcemap_builder.as_mut() {
            if !span.is_empty() {
                sourcemap_builder.add_source_mapping(self.code.as_bytes(), span.end, None);
            }
        }
    }

    fn add_source_mapping_for_name(&mut self, span: Span, name: &str) {
        if let Some(sourcemap_builder) = self.sourcemap_builder.as_mut() {
            if !span.is_empty() {
                sourcemap_builder.add_source_mapping_for_name(self.code.as_bytes(), span, name);
            }
        }
    }
}
