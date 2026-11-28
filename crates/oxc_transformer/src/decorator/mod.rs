mod accessor;
mod legacy;
mod options;

use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::GetSpan;
use oxc_traverse::Traverse;

use crate::{context::TraverseCtx, state::TransformState};

use legacy::LegacyDecorator;
pub use options::{DecoratorMode, DecoratorOptions};

pub struct Decorator<'a> {
    mode: DecoratorMode,

    // Plugins
    legacy_decorator: LegacyDecorator<'a>,
}

impl Decorator<'_> {
    pub fn new(options: DecoratorOptions, mode: DecoratorMode) -> Self {
        Self { legacy_decorator: LegacyDecorator::new(options), mode }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for Decorator<'a> {
    #[inline]
    fn exit_program(
        &mut self,
        node: &mut Program<'a>,
        ctx: &mut oxc_traverse::TraverseCtx<'a, TransformState<'a>>,
    ) {
        if self.mode.is_legacy() {
            self.legacy_decorator.exit_program(node, ctx);
        }
    }

    #[inline]
    fn enter_statements(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.mode.is_legacy() {
            self.legacy_decorator.enter_statements(stmts, ctx);
        }
    }

    #[inline]
    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.mode.is_legacy() {
            self.legacy_decorator.exit_statement(stmt, ctx);
        }
    }

    #[inline]
    fn enter_class(&mut self, node: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.mode.is_legacy() {
            self.legacy_decorator.enter_class(node, ctx);
        }
    }

    #[inline]
    fn exit_class(&mut self, node: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.mode.is_legacy() {
            self.legacy_decorator.exit_class(node, ctx);
        }
    }

    #[inline]
    fn enter_method_definition(
        &mut self,
        node: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.mode.is_legacy() {
            self.legacy_decorator.enter_method_definition(node, ctx);
        }
    }

    #[inline]
    fn exit_method_definition(
        &mut self,
        node: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.mode.is_legacy() {
            self.legacy_decorator.exit_method_definition(node, ctx);
        }
    }

    #[inline]
    fn enter_accessor_property(
        &mut self,
        node: &mut AccessorProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.mode.is_legacy() {
            self.legacy_decorator.enter_accessor_property(node, ctx);
        }
    }

    #[inline]
    fn exit_accessor_property(
        &mut self,
        node: &mut AccessorProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.mode.is_legacy() {
            self.legacy_decorator.exit_accessor_property(node, ctx);
        }
    }

    #[inline]
    fn enter_property_definition(
        &mut self,
        node: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.mode.is_legacy() {
            self.legacy_decorator.enter_property_definition(node, ctx);
        }
    }

    #[inline]
    fn exit_property_definition(
        &mut self,
        node: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.mode.is_legacy() {
            self.legacy_decorator.exit_property_definition(node, ctx);
        }
    }

    #[inline]
    fn enter_decorator(
        &mut self,
        node: &mut oxc_ast::ast::Decorator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.mode.is_legacy() {
            self.legacy_decorator.enter_decorator(node, ctx);
        } else {
            Self::report_invalid_expression(node, ctx);
        }
    }
}

impl<'a> Decorator<'a> {
    /// Report a decorator expression that the standard decorator grammar does not allow, such as
    /// `@a.b().c` (TS1497), in every non-legacy mode.
    ///
    /// The parser reports this for JavaScript files. It cannot for TypeScript files, because
    /// `experimentalDecorators` (legacy mode) code relies on any `LeftHandSideExpression` being
    /// accepted, e.g. `@http.controller('hooks').middleware(M)`.
    fn report_invalid_expression(
        decorator: &oxc_ast::ast::Decorator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !ctx.state.source_type.is_typescript() || decorator.span.is_unspanned() {
            return;
        }
        // The parser knows whether the first token after `@` is `(`. Recover that from the source.
        let source_text = ctx.state.source_text;
        let Some(start) = skip_trivia(source_text, decorator.span.start + 1) else { return };
        let is_parenthesized = source_text.as_bytes().get(start as usize) == Some(&b'(');
        let expression = &decorator.expression;
        if let Err(error) = expression.check_decorator_expression(start, is_parenthesized) {
            ctx.state.error(
                OxcDiagnostic::error(error.message())
                    .with_help("This decorator expression is only valid with `experimentalDecorators` (legacy decorators).")
                    .with_label(expression.span()),
            );
        }
    }

    #[inline]
    pub fn exit_class_at_end(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.mode.is_legacy() {
            self.legacy_decorator.exit_class_at_end(class, ctx);
        }
    }
}

/// Returns the position of the first character at or after `pos` that is not whitespace or a
/// comment, or `None` if `pos` is out of bounds.
fn skip_trivia(source_text: &str, pos: u32) -> Option<u32> {
    let mut rest = source_text.get(pos as usize..)?;
    loop {
        let trimmed = rest.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{FEFF}');
        rest = if let Some(comment) = trimmed.strip_prefix("//") {
            comment.find(['\n', '\r', '\u{2028}', '\u{2029}']).map_or("", |end| &comment[end..])
        } else if let Some(comment) = trimmed.strip_prefix("/*") {
            comment.find("*/").map_or("", |end| &comment[end + 2..])
        } else {
            #[expect(clippy::cast_possible_truncation)]
            return Some((source_text.len() - trimmed.len()) as u32);
        };
    }
}
