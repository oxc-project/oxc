//! Oxc Codegen
//!
//! Code adapted from
//! * [esbuild](https://github.com/evanw/esbuild/blob/main/internal/js_printer/js_printer.go)

mod annotation_comment;
mod context;
mod gen;
mod operator;
mod sourcemap_builder;

use std::{borrow::Cow, ops::Range};

use rustc_hash::FxHashMap;

use oxc_ast::{
    ast::{BlockStatement, Directive, Expression, Program, Statement},
    Comment, Trivias,
};
use oxc_mangler::Mangler;
use oxc_span::{CompactStr, Span};
use oxc_syntax::{
    identifier::is_identifier_part,
    operator::{BinaryOperator, UnaryOperator, UpdateOperator},
    precedence::Precedence,
    symbol::SymbolId,
};

pub use crate::{
    context::Context,
    gen::{Gen, GenExpr},
};
use crate::{operator::Operator, sourcemap_builder::SourcemapBuilder};

/// Code generator without whitespace removal.
pub type CodeGenerator<'a> = Codegen<'a, false>;

/// Code generator with whitespace removal.
pub type WhitespaceRemover<'a> = Codegen<'a, true>;

#[derive(Default, Clone, Copy)]
pub struct CodegenOptions {
    /// Use single quotes instead of double quotes.
    pub single_quote: bool,
}

#[derive(Default, Clone, Copy)]
pub struct CommentOptions {
    /// Enable preserve annotate comments, like `/* #__PURE__ */` and `/* #__NO_SIDE_EFFECTS__ */`.
    pub preserve_annotate_comments: bool,
}

pub struct CodegenReturn {
    pub source_text: String,
    pub source_map: Option<oxc_sourcemap::SourceMap>,
}

pub struct Codegen<'a, const MINIFY: bool> {
    options: CodegenOptions,
    comment_options: CommentOptions,

    source_text: &'a str,

    trivias: Trivias,

    mangler: Option<Mangler>,

    /// Output Code
    code: Vec<u8>,

    // states
    prev_op_end: usize,
    prev_reg_exp_end: usize,
    need_space_before_dot: usize,
    print_next_indent_as_space: bool,

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

    // Builders
    sourcemap_builder: Option<SourcemapBuilder>,

    /// The key of map is the node start position,
    /// the first element of value is the start of the comment
    /// the second element of value includes the end of the comment and comment kind.
    move_comment_map: MoveCommentMap,
    latest_consumed_comment_end: u32,
}

impl<'a, const MINIFY: bool> Default for Codegen<'a, MINIFY> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, const MINIFY: bool> From<Codegen<'a, MINIFY>> for String {
    fn from(mut val: Codegen<'a, MINIFY>) -> Self {
        val.into_source_text()
    }
}

impl<'a, const MINIFY: bool> From<Codegen<'a, MINIFY>> for Cow<'a, str> {
    fn from(mut val: Codegen<'a, MINIFY>) -> Self {
        Cow::Owned(val.into_source_text())
    }
}

// Public APIs
impl<'a, const MINIFY: bool> Codegen<'a, MINIFY> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            options: CodegenOptions::default(),
            comment_options: CommentOptions::default(),
            source_text: "",
            trivias: Trivias::default(),
            mangler: None,
            code: vec![],
            needs_semicolon: false,
            need_space_before_dot: 0,
            print_next_indent_as_space: false,
            prev_op_end: 0,
            prev_reg_exp_end: 0,
            prev_op: None,
            start_of_stmt: 0,
            start_of_arrow_expr: 0,
            start_of_default_export: 0,
            indent: 0,
            quote: b'"',
            sourcemap_builder: None,
            move_comment_map: MoveCommentMap::default(),
            latest_consumed_comment_end: 0,
        }
    }

    /// Initialize the output code buffer to reduce memory reallocation.
    /// Minification will reduce by at least half of the original size.
    #[must_use]
    pub fn with_capacity(mut self, source_text_len: usize) -> Self {
        let capacity = if MINIFY { source_text_len / 2 } else { source_text_len };
        self.code = Vec::with_capacity(capacity);
        self
    }

    #[must_use]
    pub fn with_options(mut self, options: CodegenOptions) -> Self {
        self.options = options;
        self.quote = if options.single_quote { b'\'' } else { b'"' };
        self
    }

    #[must_use]
    pub fn enable_comment(
        mut self,
        source_text: &'a str,
        trivias: Trivias,
        options: CommentOptions,
    ) -> Self {
        self.source_text = source_text;
        self.trivias = trivias;
        self.comment_options = options;
        self
    }

    #[must_use]
    pub fn enable_source_map(mut self, source_name: &str, source_text: &str) -> Self {
        let mut sourcemap_builder = SourcemapBuilder::default();
        sourcemap_builder.with_name_and_source(source_name, source_text);
        self.sourcemap_builder = Some(sourcemap_builder);
        self
    }

    #[must_use]
    pub fn with_mangler(mut self, mangler: Option<Mangler>) -> Self {
        self.mangler = mangler;
        self
    }

    #[must_use]
    pub fn build(mut self, program: &Program<'_>) -> CodegenReturn {
        program.gen(&mut self, Context::default());
        let source_text = self.into_source_text();
        let source_map = self.sourcemap_builder.map(SourcemapBuilder::into_sourcemap);
        CodegenReturn { source_text, source_map }
    }

    #[must_use]
    pub fn into_source_text(&mut self) -> String {
        // SAFETY: criteria of `from_utf8_unchecked` are met.

        unsafe { String::from_utf8_unchecked(std::mem::take(&mut self.code)) }
    }

    /// Push a single character into the buffer
    #[inline]
    pub fn print_char(&mut self, ch: u8) {
        self.code.push(ch);
    }

    /// Push str into the buffer
    #[inline]
    pub fn print_str(&mut self, s: &str) {
        self.code.extend(s.as_bytes());
    }
}

// Private APIs
impl<'a, const MINIFY: bool> Codegen<'a, MINIFY> {
    fn code(&self) -> &Vec<u8> {
        &self.code
    }

    fn code_len(&self) -> usize {
        self.code().len()
    }

    #[inline]
    fn print_soft_space(&mut self) {
        if !MINIFY {
            self.print_char(b' ');
        }
    }

    #[inline]
    pub fn print_hard_space(&mut self) {
        self.print_char(b' ');
    }

    #[inline]
    fn print_soft_newline(&mut self) {
        if !MINIFY {
            self.print_char(b'\n');
        }
    }

    #[inline]
    fn print_semicolon(&mut self) {
        self.print_char(b';');
    }

    #[inline]
    fn print_comma(&mut self) {
        self.print_char(b',');
    }

    #[inline]
    fn print_space_before_identifier(&mut self) {
        if self
            .peek_nth(0)
            .is_some_and(|ch| is_identifier_part(ch) || self.prev_reg_exp_end == self.code.len())
        {
            self.print_hard_space();
        }
    }

    #[inline]
    fn peek_nth(&self, n: usize) -> Option<char> {
        // SAFETY: criteria of `from_utf8_unchecked` are met.
        unsafe { std::str::from_utf8_unchecked(self.code()) }.chars().nth_back(n)
    }

    #[inline]
    fn indent(&mut self) {
        if !MINIFY {
            self.indent += 1;
        }
    }

    #[inline]
    fn dedent(&mut self) {
        if !MINIFY {
            self.indent -= 1;
        }
    }

    #[inline]
    fn print_indent(&mut self) {
        if MINIFY {
            return;
        }
        if self.print_next_indent_as_space {
            self.print_hard_space();
            self.print_next_indent_as_space = false;
            return;
        }
        self.code.extend(std::iter::repeat(b'\t').take(self.indent as usize));
    }

    #[inline]
    fn print_semicolon_after_statement(&mut self) {
        if MINIFY {
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
    pub fn print_colon(&mut self) {
        self.print_char(b':');
    }

    #[inline]
    fn print_equal(&mut self) {
        self.print_char(b'=');
    }

    fn print_sequence<T: Gen<MINIFY>>(&mut self, items: &[T], ctx: Context) {
        for item in items {
            item.gen(self, ctx);
            self.print_comma();
        }
    }

    fn print_curly_braces<F: FnOnce(&mut Self)>(&mut self, span: Span, single_line: bool, op: F) {
        self.add_source_mapping(span.start);
        self.print_char(b'{');
        if !single_line {
            self.print_soft_newline();
            self.indent();
        }
        op(self);
        if !single_line {
            self.dedent();
            self.print_indent();
        }
        self.add_source_mapping(span.end);
        self.print_char(b'}');
    }

    fn print_block_start(&mut self, position: u32) {
        self.add_source_mapping(position);
        self.print_char(b'{');
        self.print_soft_newline();
        self.indent();
    }

    fn print_block_end(&mut self, position: u32) {
        self.dedent();
        self.print_indent();
        self.add_source_mapping(position);
        self.print_char(b'}');
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
                if need_space && MINIFY {
                    self.print_hard_space();
                }
                self.print_next_indent_as_space = true;
                stmt.gen(self, ctx);
            }
        }
    }

    fn print_block_statement(&mut self, stmt: &BlockStatement<'_>, ctx: Context) {
        self.print_curly_braces(stmt.span, stmt.body.is_empty(), |p| {
            p.print_directives_and_statements(None, &stmt.body, ctx);
        });
        self.needs_semicolon = false;
    }

    fn print_list<T: Gen<MINIFY>>(&mut self, items: &[T], ctx: Context) {
        for (index, item) in items.iter().enumerate() {
            if index != 0 {
                self.print_comma();
                self.print_soft_space();
            }
            item.gen(self, ctx);
        }
    }

    #[inline]
    pub fn print_expression(&mut self, expr: &Expression<'_>) {
        expr.gen_expr(self, Precedence::lowest(), Context::default());
    }

    fn print_expressions<T: GenExpr<MINIFY>>(
        &mut self,
        items: &[T],
        precedence: Precedence,
        ctx: Context,
    ) {
        for (index, item) in items.iter().enumerate() {
            if index != 0 {
                self.print_comma();
                self.print_soft_space();
            }
            item.gen_expr(self, precedence, ctx);
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn print_symbol(&mut self, span: Span, symbol_id: Option<SymbolId>, fallback: &str) {
        if let Some(mangler) = &self.mangler {
            if let Some(symbol_id) = symbol_id {
                let name = mangler.get_symbol_name(symbol_id);
                let name = CompactStr::new(name);
                self.add_source_mapping_for_name(span, &name);
                self.print_str(&name);
                return;
            }
        }
        self.add_source_mapping_for_name(span, fallback);
        self.print_str(fallback);
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
            || (prev == un_op_not && next == un_op_pre_dec && self.peek_nth(1) == Some('<'))
        {
            self.print_hard_space();
        }
    }

    #[inline]
    fn wrap<F: FnMut(&mut Self)>(&mut self, wrap: bool, mut f: F) {
        if wrap {
            self.print_char(b'(');
        }
        f(self);
        if wrap {
            self.print_char(b')');
        }
    }

    #[inline]
    fn wrap_quote<F: FnMut(&mut Self, u8)>(&mut self, mut f: F) {
        self.print_char(self.quote);
        f(self, self.quote);
        self.print_char(self.quote);
    }

    fn print_directives_and_statements(
        &mut self,
        directives: Option<&[Directive]>,
        statements: &[Statement<'_>],
        ctx: Context,
    ) {
        if let Some(directives) = directives {
            if directives.is_empty() {
                if let Some(Statement::ExpressionStatement(s)) = statements.first() {
                    if matches!(s.expression.get_inner_expression(), Expression::StringLiteral(_)) {
                        self.print_semicolon();
                        self.print_soft_newline();
                    }
                }
            } else {
                for directive in directives {
                    directive.gen(self, ctx);
                    self.print_semicolon_if_needed();
                }
            }
        }
        for stmt in statements {
            self.print_semicolon_if_needed();
            stmt.gen(self, ctx);
        }
    }

    fn add_source_mapping(&mut self, position: u32) {
        if let Some(sourcemap_builder) = self.sourcemap_builder.as_mut() {
            sourcemap_builder.add_source_mapping(&self.code, position, None);
        }
    }

    fn add_source_mapping_for_name(&mut self, span: Span, name: &str) {
        if let Some(sourcemap_builder) = self.sourcemap_builder.as_mut() {
            sourcemap_builder.add_source_mapping_for_name(&self.code, span, name);
        }
    }
}

pub(crate) type MoveCommentMap = FxHashMap<u32, Comment>;

// Comment related
impl<'a, const MINIFY: bool> Codegen<'a, MINIFY> {
    /// Avoid issue related to rustc borrow checker .
    /// Since if you want to print a range of source code, you need to borrow the source code
    /// as immutable first, and call the [Self::print_str] which is a mutable borrow.
    fn print_range_of_source_code(&mut self, range: Range<usize>) {
        self.code.extend_from_slice(self.source_text[range].as_bytes());
    }

    /// In some scenario, we want to move the comment that should be codegened to another position.
    /// ```js
    ///  /* @__NO_SIDE_EFFECTS__ */ export const a = function() {
    ///
    ///  }, b = 10000;
    ///
    /// ```
    /// should generate such output:
    /// ```js
    ///   export const /* @__NO_SIDE_EFFECTS__ */ a = function() {
    ///
    ///  }, b = 10000;
    /// ```
    fn move_comment(&mut self, position: u32, full_comment_info: Comment) {
        self.move_comment_map.insert(position, full_comment_info);
    }

    fn try_get_leading_comment(&self, start: u32) -> Option<&Comment> {
        self.trivias.comments_range(0..start).next_back()
    }

    fn try_take_moved_comment(&mut self, node_start: u32) -> Option<Comment> {
        self.move_comment_map.remove(&node_start)
    }
}
