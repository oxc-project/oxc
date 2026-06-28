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

use oxc_allocator::{ArenaVec, TakeIn};
use oxc_ast::{ast::*, builder::NONE};
use oxc_semantic::SymbolFlags;
use oxc_span::SPAN;
use oxc_traverse::{BoundIdentifier, Traverse};

use crate::{
    common::helper_loader::{Helper, helper_call_expr},
    context::TraverseCtx,
    state::TransformState,
};

const SCRIPT_TAG: &[u8; 8] = b"</script";
const SCRIPT_TAG_LEN: usize = SCRIPT_TAG.len();

pub struct TaggedTemplateTransform;

impl<'a> Traverse<'a, TransformState<'a>> for TaggedTemplateTransform {
    // `#[inline]` because this is a hot path and most `Expression`s are not `TaggedTemplateExpression`s,
    // so we want this inlined to handle the common case without a function call
    #[inline]
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if matches!(expr, Expression::TaggedTemplateExpression(_)) {
            self.transform_tagged_template(expr, ctx);
        }
    }
}

impl<'a> TaggedTemplateTransform {
    pub fn new() -> Self {
        Self {}
    }

    /// Transform a tagged template expression to use the [`Helper::TaggedTemplateLiteral`] helper function.
    #[cold] // Tagged template expressions are rare
    fn transform_tagged_template(&self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        debug_assert!(matches!(expr, Expression::TaggedTemplateExpression(_)));

        if !matches!(expr, Expression::TaggedTemplateExpression(tagged) if Self::contains_closing_script_tag(&tagged.quasi))
        {
            return;
        }

        let Expression::TaggedTemplateExpression(tagged) = expr.take_in(ctx) else {
            unreachable!();
        };

        let TaggedTemplateExpression { span, tag, quasi: template_lit, type_arguments, .. } =
            tagged.unbox();

        let binding = self.create_top_level_binding(ctx);
        let arguments = self.transform_template_literal(&binding, template_lit, ctx);
        *expr = Expression::new_call_expression(span, tag, type_arguments, arguments, false, ctx);
    }

    /// Check if the template literal contains a `</script` tag; note it is case-insensitive.
    fn contains_closing_script_tag(quasi: &TemplateLiteral) -> bool {
        quasi.quasis.iter().any(|quasi| {
            let raw = &quasi.value.raw;

            // The raw string must be at least as long as the script tag
            if raw.len() < SCRIPT_TAG_LEN {
                return false;
            }

            let raw_bytes = raw.as_bytes();
            // Get the bytes up to the last possible starting position of the script tag

            let max_start_pos = raw_bytes.len() - SCRIPT_TAG_LEN;
            for (i, byte) in raw_bytes[..=max_start_pos].iter().copied().enumerate() {
                // The first character must be a `<`
                if byte != b'<' {
                    continue;
                }

                // Check if this position contains "</script"
                let slice = &raw_bytes[i..i + SCRIPT_TAG_LEN];
                if is_script_close_tag(slice) {
                    return true;
                }
            }

            false
        })
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
    #[expect(clippy::unused_self)]
    fn transform_template_literal(
        &self,
        binding: &BoundIdentifier<'a>,
        template_lit: TemplateLiteral<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> ArenaVec<'a, Argument<'a>> {
        // Create cooked array: `[cooked0, cooked1, ...]`.
        // Use `void 0` for elements with invalid escape sequences (where `cooked` is `None`).
        // Also check if we need to pass the raw array separately.
        let mut needs_raw_array = false;
        let cooked_elements = ArenaVec::from_iter_in(
            template_lit.quasis.iter().map(|quasi| {
                let expr = if let Some(cooked) = &quasi.value.cooked {
                    if cooked.as_str() != quasi.value.raw.as_str() {
                        needs_raw_array = true;
                    }
                    Expression::new_string_literal(SPAN, *cooked, None, ctx)
                } else {
                    // Invalid escape sequence - cooked is None
                    needs_raw_array = true;
                    Expression::new_void_0(SPAN, ctx)
                };
                ArrayExpressionElement::from(expr)
            }),
            ctx,
        );
        let cooked_argument = Argument::new_array_expression(SPAN, cooked_elements, ctx);

        // Add raw array if needed: `[raw0, raw1, ...]`
        let template_arguments = if needs_raw_array {
            let elements = ArenaVec::from_iter_in(
                template_lit.quasis.iter().map(|quasi| {
                    let string = Expression::new_string_literal(SPAN, quasi.value.raw, None, ctx);
                    ArrayExpressionElement::from(string)
                }),
                ctx,
            );
            let raws_argument = Argument::new_array_expression(SPAN, elements, ctx);
            ArenaVec::from_array_in([cooked_argument, raws_argument], ctx)
        } else {
            ArenaVec::from_value_in(cooked_argument, ctx)
        };

        // `babelHelpers.taggedTemplateLiteral([<...cooked>], [<...raw>]?)`
        let template_call =
            helper_call_expr(Helper::TaggedTemplateLiteral, template_arguments, ctx);
        // `binding || (binding = babelHelpers.taggedTemplateLiteral([<...cooked>], [<...raw>]?))`
        let template_call =
            Argument::from(Self::create_logical_or_expression(binding, template_call, ctx));

        // `(binding || (binding = babelHelpers.taggedTemplateLiteral([<...cooked>], [<...raw>]?)), <...expressions>)`
        ArenaVec::from_iter_in(
            // Add the template expressions as the remaining arguments
            iter::once(template_call)
                .chain(template_lit.expressions.into_iter().map(Argument::from)),
            ctx,
        )
    }

    /// `binding || (binding = expr)`
    fn create_logical_or_expression(
        binding: &BoundIdentifier<'a>,
        expr: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let left = binding.create_read_expression(ctx);
        let right = Expression::new_assignment_expression(
            SPAN,
            AssignmentOperator::Assign,
            binding.create_write_target(ctx),
            expr,
            ctx,
        );
        Expression::new_logical_expression(SPAN, left, LogicalOperator::Or, right, ctx)
    }

    /// Creates a `var binding;` variable declaration at the top level and returns the binding
    #[expect(clippy::unused_self)]
    fn create_top_level_binding(&self, ctx: &mut TraverseCtx<'a>) -> BoundIdentifier<'a> {
        let binding = ctx.generate_uid(
            "templateObject",
            ctx.scoping().root_scope_id(),
            SymbolFlags::FunctionScopedVariable,
        );

        let variable = VariableDeclarator::new(
            SPAN,
            VariableDeclarationKind::Var,
            binding.create_binding_pattern(ctx),
            NONE,
            None,
            false,
            ctx,
        );

        let stmt = Statement::new_variable_declaration(
            SPAN,
            VariableDeclarationKind::Var,
            ArenaVec::from_value_in(variable, ctx),
            false,
            ctx,
        );

        ctx.state.top_level_statements.insert_statement(stmt);

        binding
    }
}

/// Check if `slice` is `</script`, regardless of case.
///
/// `slice.len()` must be 8.
//
//  NOTE: This function is copied from `oxc_codegen/src/str.rs`.
//
// `#[inline(always)]` so that compiler can see from caller that `slice.len() == 8`
// and so `slice.try_into().unwrap()` cannot fail. This function is only 4 instructions.
#[expect(clippy::inline_always)]
#[inline(always)]
pub fn is_script_close_tag(slice: &[u8]) -> bool {
    // Compiler condenses these operations to an 8-byte read, u64 AND, and u64 compare.
    // https://godbolt.org/z/K8q68WGn6
    let mut bytes: [u8; 8] = slice.try_into().unwrap();
    for byte in bytes.iter_mut().skip(2) {
        // `| 32` converts ASCII upper case letters to lower case.
        *byte |= 32;
    }
    bytes == *SCRIPT_TAG
}
