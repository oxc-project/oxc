//! Tagged Template Transform
//!
//! This plugin transforms tagged template expressions containing `</script` tags
//! to use a helper function call instead, preventing script tag issues in browser environments.
//!
//! ## Background
//!
//! Tagged template literals containing `</script` can break when the code is embedded
//! directly in HTML script tags, because the browser will incorrectly parse `</script` as
//! the end of the script tag.
//!
//! Additionally, String.raw`</script>` !== String.raw`<\/script>` due to how tagged
//! template literals preserve the raw string values.
//!
//! ## Solution
//!
//! This plugin transforms tagged template expressions containing `</script` to use
//! a cached template object with a helper function, matching esbuild's behavior.
//!
//! ## Example
//!
//! Input:
//! ```js
//! foo`</script>`
//! foo`<script>${content}</script>`
//! ```
//!
//! Output:
//! ```js
//! var _foo;
//! var _foo2;
//! foo(_foo || (_foo = babelHelpers.taggedTemplateLiteral(["<\/script>"])));
//! foo(_foo2 || (_foo2 = babelHelpers.taggedTemplateLiteral(["<script>", "<\/script>"])), content);
//! ```
//!
//! ## References
//!
//! - esbuild implementation: <https://github.com/evanw/esbuild/blob/d6427c91edab734da686c4c5d29ed580b08b9fd5/internal/js_parser/js_parser.go#L13894-L13907>
//! - Issue: <https://github.com/oxc-project/oxc/issues/15306>

use std::iter;

use oxc_allocator::{TakeIn, Vec as ArenaVec};
use oxc_ast::ast::*;
use oxc_semantic::SymbolFlags;
use oxc_span::SPAN;
use oxc_traverse::{BoundIdentifier, Traverse};

use crate::{
    common::helper_loader::Helper,
    context::{TransformCtx, TraverseCtx},
    state::TransformState,
};

pub struct TaggedTemplateTransform<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a> Traverse<'a, TransformState<'a>> for TaggedTemplateTransform<'a, '_> {
    // `#[inline]` because it is a hot path and it is directly delegated to `transform_tagged_template`
    #[inline]
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.transform_tagged_template(expr, ctx);
    }
}

impl<'a, 'ctx> TaggedTemplateTransform<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx }
    }

    /// Check if the template literal contains a `</script` tag; note it is case-insensitive.
    fn contains_closing_script_tag(quasi: &TemplateLiteral) -> bool {
        const SCRIPT_TAG: &[u8] = b"</script";

        quasi.quasis.iter().any(|quasi| {
            let raw = &quasi.value.raw;

            // The raw string must be at least as long as the script tag
            if raw.len() < SCRIPT_TAG.len() {
                return false;
            }

            let raw_bytes = raw.as_bytes();
            // Get the bytes up to the last possible starting position of the script tag
            let max_remain_len = raw_bytes.len().saturating_sub(SCRIPT_TAG.len());
            let raw_bytes_iter = raw_bytes[..=max_remain_len].iter().copied().enumerate();
            for (idx, byte) in raw_bytes_iter {
                if byte == b'<'
                    && SCRIPT_TAG
                        .iter()
                        .zip(raw_bytes[idx..].iter())
                        .all(|(a, b)| *a == b.to_ascii_lowercase())
                {
                    return true;
                }
            }

            false
        })
    }

    /// Transform a tagged template expression to use the [`Helper::TaggedTemplateLiteral`] helper function
    // `#[inline]` so that compiler can see `expr` should be a `TaggedTemplateExpression` and reduce redundant checks
    #[inline]
    fn transform_tagged_template(&self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if !matches!(expr, Expression::TaggedTemplateExpression(tagged) if Self::contains_closing_script_tag(&tagged.quasi))
        {
            return;
        }

        let Expression::TaggedTemplateExpression(tagged) = expr.take_in(ctx.ast) else {
            unreachable!();
        };

        *expr = self.transform_tagged_template_impl(tagged.unbox(), ctx);
    }

    fn transform_tagged_template_impl(
        &self,
        expr: TaggedTemplateExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let TaggedTemplateExpression { span, tag, quasi, type_arguments } = expr;

        let binding = self.create_top_level_binding(ctx);
        let arguments = self.transform_template_literal(&binding, quasi, ctx);
        ctx.ast.expression_call(span, tag, type_arguments, arguments, false)
    }

    /// Transform [`TemplateLiteral`] to build the arguments for the tagged template call
    ///
    /// Handle its fields as follows:
    ///   quasis:
    ///     - Create an array expression containing the cooked string literals
    ///     - If cooked differs from raw, create a second array with raw strings
    ///     - Call the helper function with the array expression(s)
    ///     - Create a logical OR expression to cache the result in the binding
    ///     - Wrap the logical OR expression as the first argument
    ///   expressions:
    ///     - Add each expression as the remaining arguments
    ///
    /// Final arguments:
    /// - `(binding || (binding = babelHelpers.taggedTemplateLiteral([<...cooked>])), <...expressions>)` when cooked == raw
    /// - `(binding || (binding = babelHelpers.taggedTemplateLiteral([<...cooked>], [<...raw>])), <...expressions>)` when cooked != raw
    fn transform_template_literal(
        &self,
        binding: &BoundIdentifier<'a>,
        quasi: TemplateLiteral<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> ArenaVec<'a, Argument<'a>> {
        // Check if we need to pass the raw array separately
        let needs_raw_array = quasi.quasis.iter().any(|quasi| match &quasi.value.cooked {
            None => true, // Invalid escape sequence - cooked is None
            Some(cooked) => cooked.as_str() != quasi.value.raw.as_str(),
        });

        // Create cooked array: `[cooked0, cooked1, ...]`
        // Use `void 0` for elements with invalid escape sequences (where cooked is None)
        let cooked_elements = ctx.ast.vec_from_iter(quasi.quasis.iter().map(|quasi| {
            let expr = match &quasi.value.cooked {
                Some(cooked) => ctx.ast.expression_string_literal(SPAN, *cooked, None),
                None => ctx.ast.void_0(SPAN),
            };
            ArrayExpressionElement::from(expr)
        }));
        let cooked_argument = Argument::from(ctx.ast.expression_array(SPAN, cooked_elements));

        // Add raw array if needed: `[raw0, raw1, ...]`
        let raws_argument = needs_raw_array.then(|| {
            let elements = ctx.ast.vec_from_iter(quasi.quasis.iter().map(|quasi| {
                let string = ctx.ast.expression_string_literal(SPAN, quasi.value.raw, None);
                ArrayExpressionElement::from(string)
            }));
            Argument::from(ctx.ast.expression_array(SPAN, elements))
        });

        let template_arguments =
            ctx.ast.vec_from_iter(iter::once(cooked_argument).chain(raws_argument));

        // `babelHelpers.taggedTemplateLiteral([<...cooked>], [<...raw>]?)`
        let template_call =
            self.ctx.helper_call_expr(Helper::TaggedTemplateLiteral, SPAN, template_arguments, ctx);
        // `binding || (binding = babelHelpers.taggedTemplateLiteral([<...cooked>], [<...raw>]?))`
        let template_call =
            Argument::from(Self::create_logical_or_expression(binding, template_call, ctx));

        // `(binding || (binding = babelHelpers.taggedTemplateLiteral([<...cooked>], [<...raw>]?)), <...expressions>)`
        ctx.ast.vec_from_iter(
            // Add the template expressions as the remaining arguments
            iter::once(template_call).chain(quasi.expressions.into_iter().map(Argument::from)),
        )
    }

    /// `binding || (binding = expr)`
    fn create_logical_or_expression(
        binding: &BoundIdentifier<'a>,
        expr: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let left = binding.create_read_expression(ctx);
        let right = ctx.ast.expression_assignment(
            SPAN,
            AssignmentOperator::Assign,
            binding.create_write_target(ctx),
            expr,
        );
        ctx.ast.expression_logical(SPAN, left, LogicalOperator::Or, right)
    }

    /// Creates a `var binding;` variable declaration at the top level and returns the binding
    fn create_top_level_binding(&self, ctx: &mut TraverseCtx<'a>) -> BoundIdentifier<'a> {
        let binding = ctx.generate_uid(
            "templateObject",
            ctx.scoping().root_scope_id(),
            SymbolFlags::FunctionScopedVariable,
        );

        let variable = ctx.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Var,
            binding.create_binding_pattern(ctx),
            None,
            false,
        );

        let stmt = Statement::from(ctx.ast.declaration_variable(
            SPAN,
            VariableDeclarationKind::Var,
            ctx.ast.vec1(variable),
            false,
        ));

        self.ctx.top_level_statements.insert_statement(stmt);

        binding
    }
}
