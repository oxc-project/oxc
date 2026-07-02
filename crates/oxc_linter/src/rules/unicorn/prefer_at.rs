use oxc_syntax::precedence::Precedence;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use oxc_ast::{
    AstKind,
    ast::{
        Argument, AssignmentTarget, BinaryOperator, CallExpression, ChainElement,
        ComputedMemberExpression, Expression, MemberExpression, StaticMemberExpression,
        UnaryOperator, VariableDeclarationKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    utils::{get_precedence, is_same_expression},
};

fn prefer_at_diagnostic(span: Span, method: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `.at()` over `{method}`."))
        .with_help("Use `.at()` for index access.")
        .with_note("https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/at")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferAt(Box<PreferAtConfig>);

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferAtConfig {
    /// Check all index access, not just special patterns like `array.length - 1`.
    /// When enabled, `array[0]`, `array[1]`, etc. will also be flagged.
    check_all_index_access: bool,
    /// List of function names to treat as "get last element" functions.
    /// These functions will be checked for `.at(-1)` usage.
    get_last_element_functions: Vec<String>,
}

impl std::ops::Deref for PreferAt {
    type Target = PreferAtConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer the [`Array#at()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/at) and
    /// [`String#at()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/at)
    /// methods for index access.
    ///
    /// This rule also discourages using [`String#charAt()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/charAt).
    ///
    /// ### Why is this bad?
    ///
    /// The `.at()` method is more readable and consistent for accessing elements by index,
    /// especially for negative indices which access elements from the end of the array or string.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const foo = array[array.length - 1];
    /// const foo = array.slice(-1)[0];
    /// const foo = string.charAt(string.length - 1);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const foo = array.at(-1);
    /// const foo = array.at(-5);
    /// const foo = string.at(-1);
    /// ```
    PreferAt,
    unicorn,
    pedantic,
    dangerous_fix,
    config = PreferAtConfig,
    version = "1.20.0",
    short_description = "Prefer the `Array#at()` and `String#at()` methods for index access.",
);

impl Rule for PreferAt {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        let config = value.as_array().and_then(|arr| arr.first().and_then(|v| v.as_object()));

        Ok(Self(Box::new(PreferAtConfig {
            check_all_index_access: config
                .and_then(|c| c.get("checkAllIndexAccess"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            get_last_element_functions: config
                .and_then(|c| c.get("getLastElementFunctions"))
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default(),
        })))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ComputedMemberExpression(computed) if !is_assignment_target(node, ctx) => {
                self.handle_computed_member(computed, node, ctx);
            }
            AstKind::CallExpression(call) if !is_assignment_target(node, ctx) => {
                self.check_call_expression(call, node, ctx);
            }
            _ => {}
        }
    }
}

impl PreferAt {
    fn handle_computed_member<'a>(
        &self,
        computed: &ComputedMemberExpression<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        // Check if this is accessing [0] on various expression types
        if computed.expression.get_inner_expression().is_number_0()
            && Self::check_slice_with_zero_index(computed, node.id(), ctx)
        {
            return;
        }

        // Check for object.length - N pattern
        if let Some((object, negative_value)) = extract_length_minus_pattern(&computed.expression) {
            if is_same_expression(computed.object.get_inner_expression(), object, ctx) {
                ctx.diagnostic_with_fix(
                    prefer_at_diagnostic(computed.span(), "[index]"),
                    |fixer| {
                        if is_arguments_object(&computed.object) {
                            return fixer.noop();
                        }
                        create_at_fix(
                            &fixer,
                            computed.object.span(),
                            computed.span(),
                            negative_value,
                        )
                    },
                );
            }
        } else if self.check_all_index_access
            && let Some(index) = get_positive_index(&computed.expression)
            && !is_obviously_non_array_receiver(&computed.object, ctx)
        {
            ctx.diagnostic_with_fix(prefer_at_diagnostic(computed.span(), "[index]"), |fixer| {
                if is_arguments_object(&computed.object) {
                    return fixer.noop();
                }

                create_at_fix(&fixer, computed.object.span(), computed.span(), index)
            });
        } else if self.check_all_index_access
            && is_addition_index_expression(&computed.expression)
            && !is_obviously_non_array_receiver(&computed.object, ctx)
        {
            if is_static_positive_index_expression(&computed.expression) {
                ctx.diagnostic_with_fix(
                    prefer_at_diagnostic(computed.span(), "[index]"),
                    |fixer| {
                        if is_arguments_object(&computed.object) {
                            return fixer.noop();
                        }

                        create_at_fix_with_arg(
                            &fixer,
                            computed.object.span(),
                            computed.span(),
                            fixer.source_range(computed.expression.span()),
                        )
                    },
                );
            } else {
                ctx.diagnostic(prefer_at_diagnostic(computed.span(), "[index]"));
            }
        }
    }

    fn check_slice_with_zero_index<'a>(
        computed: &ComputedMemberExpression<'a>,
        node_id: oxc_syntax::node::NodeId,
        ctx: &LintContext<'a>,
    ) -> bool {
        let call_expr = match &computed.object {
            Expression::ChainExpression(chain) => {
                if let ChainElement::CallExpression(call) = &chain.expression {
                    call
                } else {
                    return false;
                }
            }
            expr => {
                if let Expression::CallExpression(call) = expr.get_inner_expression() {
                    call
                } else {
                    return false;
                }
            }
        };

        if let Some(MemberExpression::StaticMemberExpression(static_member)) =
            call_expr.callee.get_member_expr()
            && static_member.property.name == "slice"
        {
            return Self::check_slice_index_access(call_expr, computed, node_id, ctx);
        }

        false
    }

    fn check_call_expression<'a>(
        &self,
        call_expr: &CallExpression<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        let Some(MemberExpression::StaticMemberExpression(static_member)) =
            call_expr.callee.get_member_expr()
        else {
            return;
        };

        match static_member.property.name.as_str() {
            "charAt" => {
                Self::check_char_at(call_expr, static_member, ctx, self.check_all_index_access);
            }
            "pop" | "shift" => Self::check_slice_pop_shift(call_expr, static_member, ctx),
            "last" => {
                check_lodash_last(&self.get_last_element_functions, call_expr, static_member, ctx);
            }
            _ => {}
        }

        // Check for array.slice(-N)[0]
        let parent_kind = ctx.nodes().parent_kind(node.id());
        if let AstKind::ComputedMemberExpression(computed) = parent_kind
            && computed.expression.get_inner_expression().is_number_0()
        {
            let parent_id = ctx.nodes().parent_id(node.id());
            Self::check_slice_index_access(call_expr, computed, parent_id, ctx);
        }
    }

    fn check_char_at<'a>(
        call_expr: &CallExpression<'a>,
        static_member: &StaticMemberExpression<'a>,
        ctx: &LintContext<'a>,
        check_all_index_access: bool,
    ) {
        if call_expr.optional || call_expr.arguments.len() != 1 {
            return;
        }

        let Some(arg) = call_expr.arguments[0].as_expression() else { return };

        if let Some((object, negative_value)) = extract_length_minus_pattern(arg)
            && is_same_expression(static_member.object.get_inner_expression(), object, ctx)
        {
            ctx.diagnostic_with_fix(prefer_at_diagnostic(call_expr.span, "charAt()"), |fixer| {
                create_at_fix(&fixer, static_member.object.span(), call_expr.span, negative_value)
            });
        } else if check_all_index_access {
            if let Some(index) = get_positive_index(arg) {
                ctx.diagnostic_with_fix(
                    prefer_at_diagnostic(call_expr.span, "charAt()"),
                    |fixer| {
                        create_at_fix(&fixer, static_member.object.span(), call_expr.span, index)
                    },
                );
            } else {
                ctx.diagnostic(prefer_at_diagnostic(call_expr.span, "charAt()"));
            }
        }
    }

    fn check_slice_pop_shift<'a>(
        call_expr: &CallExpression<'a>,
        static_member: &StaticMemberExpression<'a>,
        ctx: &LintContext<'a>,
    ) {
        if static_member.optional || call_expr.optional || !call_expr.arguments.is_empty() {
            return;
        }

        let Expression::CallExpression(slice_call) = static_member.object.get_inner_expression()
        else {
            return;
        };

        let Some(MemberExpression::StaticMemberExpression(slice_static)) =
            slice_call.callee.get_member_expr()
        else {
            return;
        };

        if slice_static.optional || slice_call.optional {
            return;
        }

        if slice_static.property.name != "slice" || slice_call.arguments.is_empty() {
            return;
        }

        if slice_call.arguments.iter().any(Argument::is_spread) {
            return;
        }

        let Some(first_arg) = slice_call.arguments[0].as_expression() else { return };

        match slice_call.arguments.len() {
            1 => Self::handle_single_arg_slice(call_expr, slice_static, first_arg, ctx),
            2 => Self::handle_two_arg_slice(
                call_expr,
                static_member,
                slice_static,
                slice_call,
                first_arg,
                ctx,
            ),
            _ => {}
        }
    }

    fn handle_single_arg_slice<'a>(
        call_expr: &CallExpression<'a>,
        slice_static: &StaticMemberExpression<'a>,
        first_arg: &Expression<'a>,
        ctx: &LintContext<'a>,
    ) {
        if let Some(value) = get_negative_integer(first_arg, Some(5))
            && value == -1
        {
            ctx.diagnostic_with_fix(
                prefer_at_diagnostic(call_expr.span, "slice().pop/shift"),
                |fixer| {
                    if is_arguments_object(&slice_static.object) {
                        return fixer.noop();
                    }
                    create_at_fix(
                        &fixer,
                        slice_static.object.span(),
                        Span::new(slice_static.object.span().start, call_expr.span.end),
                        -1,
                    )
                },
            );
        }
    }

    fn handle_two_arg_slice<'a>(
        call_expr: &CallExpression<'a>,
        static_member: &StaticMemberExpression<'a>,
        slice_static: &StaticMemberExpression<'a>,
        slice_call: &CallExpression<'a>,
        first_arg: &Expression<'a>,
        ctx: &LintContext<'a>,
    ) {
        let Some(first_negative) = get_negative_integer(first_arg, None) else { return };
        let Some(second_arg) = slice_call.arguments[1].as_expression() else { return };
        if second_arg.get_inner_expression().is_number_0() {
            return;
        }

        if static_member.property.name == "shift" {
            ctx.diagnostic_with_fix(
                prefer_at_diagnostic(call_expr.span, "slice().pop/shift"),
                |fixer| {
                    if is_arguments_object(&slice_static.object) {
                        return fixer.noop();
                    }
                    create_at_fix(
                        &fixer,
                        slice_static.object.span(),
                        Span::new(slice_static.object.span().start, call_expr.span.end),
                        first_negative,
                    )
                },
            );
        } else if let Some(second_negative) = get_negative_integer(second_arg, None) {
            // `slice(start, end).pop()` returns the slice's LAST element. `.at()` can only
            // express that when the slice is exactly one element (`end == start + 1`), in which
            // case the element is at `start`. For a multi-element slice `.at()` cannot replicate
            // `.pop()` (which takes the last element), so skip reporting in this case.
            if second_negative == first_negative + 1 {
                ctx.diagnostic_with_fix(
                    prefer_at_diagnostic(call_expr.span, "slice().pop/shift"),
                    |fixer| {
                        if is_arguments_object(&slice_static.object) {
                            return fixer.noop();
                        }
                        create_at_fix(
                            &fixer,
                            slice_static.object.span(),
                            Span::new(slice_static.object.span().start, call_expr.span.end),
                            first_negative,
                        )
                    },
                );
            }
        }
    }

    fn check_slice_index_access<'a>(
        call_expr: &CallExpression<'a>,
        computed: &ComputedMemberExpression<'a>,
        computed_node_id: oxc_syntax::node::NodeId,
        ctx: &LintContext<'a>,
    ) -> bool {
        if computed.optional {
            return false;
        }

        // Skip if this computed member is being assigned to or deleted
        let computed_parent_kind = ctx.nodes().parent_kind(computed_node_id);
        if matches!(
            computed_parent_kind,
            AstKind::AssignmentExpression(_)
                | AstKind::UpdateExpression(_)
                | AstKind::ArrayPattern(_)
        ) {
            return false;
        }

        if let AstKind::UnaryExpression(unary) = computed_parent_kind
            && unary.operator == UnaryOperator::Delete
        {
            return false;
        }

        let Some(MemberExpression::StaticMemberExpression(static_member)) =
            call_expr.callee.get_member_expr()
        else {
            return false;
        };

        if call_expr.optional
            || static_member.property.name != "slice"
            || call_expr.arguments.is_empty()
        {
            return false;
        }

        if call_expr.arguments.iter().any(Argument::is_spread) {
            return false;
        }

        let Some(first_arg) = call_expr.arguments[0].as_expression() else { return false };

        let negative_value = match call_expr.arguments.len() {
            1 => get_negative_integer(first_arg, Some(5)),
            2 => {
                // Check if second argument is 0, which means slice returns empty array
                let Some(second_arg) = call_expr.arguments[1].as_expression() else { return false };
                if is_zero_index(second_arg) {
                    // slice(-N, 0) returns empty array, so [0] would be undefined
                    // This is not equivalent to .at(-N)
                    return false;
                }
                get_negative_integer(first_arg, None)
            }
            _ => None,
        };

        if let Some(value) = negative_value {
            ctx.diagnostic_with_fix(prefer_at_diagnostic(computed.span(), "slice()[0]"), |fixer| {
                if is_arguments_object(&static_member.object) {
                    return fixer.noop();
                }
                create_at_fix(
                    &fixer,
                    static_member.object.span(),
                    Span::new(static_member.object.span().start, computed.span().end),
                    value,
                )
            });
            return true;
        }

        false
    }
}

// Helper functions

fn is_arguments_object(expr: &Expression) -> bool {
    expr.get_identifier_reference().is_some_and(|ident| ident.name == "arguments")
}

fn is_assignment_target<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let parent_kind = ctx.nodes().parent_kind(node.id());

    matches!(
        parent_kind,
        AstKind::UpdateExpression(_)
            | AstKind::ArrayPattern(_)
            | AstKind::AssignmentTargetWithDefault(_)
            | AstKind::ArrayAssignmentTarget(_)
    ) || matches!(parent_kind, AstKind::UnaryExpression(unary) if unary.operator == UnaryOperator::Delete)
        || matches!(parent_kind, AstKind::AssignmentExpression(assign_expr) if matches!(&assign_expr.left, AssignmentTarget::ComputedMemberExpression(_)))
}

fn is_positive_number(expr: &Expression) -> bool {
    match expr.get_inner_expression() {
        Expression::NumericLiteral(num) => num.value > 0.0,
        Expression::UnaryExpression(unary) => {
            unary.operator == UnaryOperator::UnaryPlus
                && matches!(unary.argument.get_inner_expression(), Expression::NumericLiteral(_))
        }
        _ => false,
    }
}

fn get_positive_index(expr: &Expression) -> Option<i64> {
    match expr.get_inner_expression() {
        Expression::NumericLiteral(num) if num.value >= 0.0 && num.value.fract() == 0.0 => {
            #[expect(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            if num.value <= i64::MAX as f64 { Some(num.value as i64) } else { None }
        }
        _ => None,
    }
}

/// Returns `true` for an (already unwrapped) expression that cannot be an
/// `Array`, `String`, or `TypedArray`, so calling `.at()` on it would be
/// invalid. Object literals, non-string literals, and function/class
/// expressions have no `.at()` method. String literals are intentionally
/// excluded — strings do have `.at()`.
fn is_unsupported_at_receiver(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::ObjectExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ClassExpression(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::RegExpLiteral(_)
    )
}

/// Returns `true` if `expr` is obviously not an array-like receiver, so
/// `prefer-at` must not rewrite `expr[index]` into `expr.at(index)` (e.g.
/// numeric-key access on a plain object, which has no `.at()` method).
///
/// Mirrors `isObviouslyNonArrayReceiver` from `eslint-plugin-unicorn`
/// (sindresorhus/eslint-plugin-unicorn#2999): the receiver itself is an
/// unsupported `.at()` target, or it is an identifier bound to a `const`
/// whose initializer is one.
fn is_obviously_non_array_receiver(expr: &Expression, ctx: &LintContext) -> bool {
    let inner = expr.get_inner_expression();
    if is_unsupported_at_receiver(inner) {
        return true;
    }

    let Expression::Identifier(ident) = inner else {
        return false;
    };
    let Some(symbol_id) = ctx.scoping().get_reference(ident.reference_id()).symbol_id() else {
        return false;
    };
    let AstKind::VariableDeclarator(declarator) = ctx.symbol_declaration(symbol_id).kind() else {
        return false;
    };
    if declarator.kind != VariableDeclarationKind::Const {
        return false;
    }
    declarator
        .init
        .as_ref()
        .is_some_and(|init| is_unsupported_at_receiver(init.get_inner_expression()))
}

fn is_addition_index_expression(expr: &Expression) -> bool {
    let Expression::BinaryExpression(binary) = expr.get_inner_expression() else { return false };
    binary.operator == BinaryOperator::Addition
        && (get_positive_index(&binary.left).is_some()
            || get_positive_index(&binary.right).is_some())
}

fn is_static_positive_index_expression(expr: &Expression) -> bool {
    let Expression::BinaryExpression(binary) = expr.get_inner_expression() else { return false };
    binary.operator == BinaryOperator::Addition
        && get_positive_index(&binary.left).is_some()
        && get_positive_index(&binary.right).is_some()
}

fn get_negative_integer(expr: &Expression, max_abs_value: Option<u32>) -> Option<i64> {
    let value = match expr.get_inner_expression() {
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::UnaryNegation =>
        {
            #[expect(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            if let Expression::NumericLiteral(num) = unary.argument.get_inner_expression()
                && num.value > 0.0
                && num.value.fract() == 0.0
                && num.value <= i64::MAX as f64
            {
                -(num.value as i64)
            } else {
                return None;
            }
        }
        Expression::NumericLiteral(num) if num.value < 0.0 && num.value.fract() == 0.0 => {
            #[expect(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            #[expect(clippy::collapsible_match)]
            if num.value >= i64::MIN as f64 {
                num.value as i64
            } else {
                return None;
            }
        }
        _ => return None,
    };

    if value == 0 {
        return None;
    }

    if let Some(max) = max_abs_value
        && value.unsigned_abs() > u64::from(max)
    {
        return None;
    }

    Some(value)
}

fn is_zero_index(expr: &Expression) -> bool {
    match expr.get_inner_expression() {
        Expression::NumericLiteral(num) => num.value.abs() < f64::EPSILON,
        _ => false,
    }
}

// Extract pattern: expression.length - N
fn extract_length_minus_pattern<'a>(expr: &'a Expression<'a>) -> Option<(&'a Expression<'a>, i64)> {
    let binary = match expr.get_inner_expression() {
        Expression::BinaryExpression(b) if b.operator == BinaryOperator::Subtraction => b,
        _ => return None,
    };

    let length_member = match binary.left.get_inner_expression() {
        Expression::StaticMemberExpression(m) if m.property.name == "length" => m,
        _ => return None,
    };

    if !is_positive_number(&binary.right) {
        return None;
    }

    // Get the numeric value for the negative index
    let value = match binary.right.get_inner_expression() {
        #[expect(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        Expression::NumericLiteral(num) if num.value > 0.0 && num.value <= i64::MAX as f64 => {
            -(num.value as i64)
        }
        _ => return None,
    };

    Some((&length_member.object, value))
}

// Unified fix creation
fn create_at_fix(
    fixer: &RuleFixer<'_, '_>,
    object_span: Span,
    full_span: Span,
    index: i64,
) -> RuleFix {
    create_at_fix_with_arg(fixer, object_span, full_span, &index.to_string())
}

fn create_at_fix_with_arg(
    fixer: &RuleFixer<'_, '_>,
    object_span: Span,
    full_span: Span,
    argument: &str,
) -> RuleFix {
    let new_code = format!("{}.at({})", fixer.source_range(object_span), argument);
    fixer.replace(full_span, new_code)
}

fn check_lodash_last<'a>(
    get_last_element_functions: &[String],
    call_expr: &CallExpression<'a>,
    static_member: &StaticMemberExpression<'a>,
    ctx: &LintContext<'a>,
) {
    if let Some(ident) = static_member.object.get_identifier_reference() {
        let name = ident.name.as_str();
        if (matches!(name, "_" | "lodash" | "underscore")
            || get_last_element_functions.iter().any(|f| f == name))
            && call_expr.arguments.len() == 1
            && let Some(arg) = call_expr.arguments[0].as_expression()
        {
            ctx.diagnostic_with_fix(
                prefer_at_diagnostic(call_expr.span, &format!("{name}.last()")),
                |fixer| {
                    if is_arguments_object(arg) {
                        return fixer.noop();
                    }

                    let arg_text = fixer.source_range(arg.span());
                    let new_code = if get_precedence(arg)
                        .is_some_and(|precedence| precedence < Precedence::Member)
                    {
                        format!("({arg_text}).at(-1)")
                    } else {
                        format!("{arg_text}.at(-1)")
                    };
                    fixer.replace(call_expr.span, new_code)
                },
            );
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("array.at(-1)", None),
        ("array[array.length - 0];", None),
        ("array[array.length + 1]", None),
        ("array[array.length + -1]", None),
        ("foo[bar.length - 1]", None),
        ("array[array.length - 1] = 1", None),
        ("array[array.length - 1] %= 1", None),
        ("++ array[array.length - 1]", None),
        ("array[array.length - 1] --", None),
        ("delete array[array.length - 1]", None),
        ("class Foo {bar; #bar; baz() {return this.#bar[this.bar.length - 1]}}", None),
        ("([array[array.length - 1]] = [])", None),
        ("({foo: array[array.length - 1] = 9} = {})", None),
        ("string.charAt(string.length - 0);", None),
        ("string.charAt(string.length + 1)", None),
        ("string.charAt(string.length + -1)", None),
        ("foo.charAt(bar.length - 1)", None),
        ("string?.charAt?.(string.length - 1);", None),
        ("string.charAt(9);", None),
        ("array.slice(-1)", None),
        ("new array.slice(-1)", None),
        ("array.slice(-0)[0]", None),
        ("array.slice(-9).pop()", None),
        ("array.slice(-1.1)[0]", None),
        ("array.slice(-1)?.[0]", None),
        ("array.slice?.(-1)[0]", None),
        ("array.notSlice(-1)[0]", None),
        ("array.slice()[0]", None),
        ("array.slice(...[-1])[0]", None),
        ("array.slice(-1).shift?.()", None),
        ("array.slice(-1)?.shift()", None),
        ("array.slice(-1).shift(...[])", None),
        ("new array.slice(-1).shift()", None),
        ("array.slice(-1)[0] += 1", None),
        ("++ array.slice(-1)[0]", None),
        ("array.slice(-1)[0] --", None),
        ("delete array.slice(-1)[0]", None),
        ("array.slice(-9)[0]", None),
        ("array.slice(-9).shift()", None),
        ("array.slice(-0xA)[0b000]", None),
        ("array.slice(-9.1, -8.1)[0]", None),
        ("array.slice(-unknown, -unknown2)[0]", None),
        ("array.slice(-9.1, unknown)[0]", None),
        ("array.slice(-9, unknown).pop()", None),
        ("array.slice(-9, ...unknown)[0]", None),
        ("array.slice(...[-9], unknown)[0]", None),
        ("new _.last(array)", None),
        ("_.last(array, 2)", None),
        ("_.last(...array)", None),
        ("array.slice(-9, 0)[0]", None),
        ("array.slice(-5, 0).pop()", None),
        ("array.slice(-3, 0).shift()", None),
        ("array.slice(-5, -3).pop()", None),
        ("array.slice(-9, -2).pop()", None),
        ("++array[1]", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        (
            "const offset = 5;const extraArgument = 6;string.charAt(offset + 9, extraArgument)",
            Some(serde_json::json!([{ "checkAllIndexAccess": true }])),
        ),
        ("array[unknown]", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        ("array[-1]", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        ("array[1.5]", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        ("array[1n]", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        // https://github.com/oxc-project/oxc/issues/23870
        // Numeric-key access on a plain object must not be reported/fixed:
        // objects have no `.at()` method, so the autofix produces broken code.
        (
            "const tokens = { 60: '#666666', 10: '#F8F8F8' }; tokens[60]",
            Some(serde_json::json!([{ "checkAllIndexAccess": true }])),
        ),
        ("({ 1: 1 })[1]", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        (
            "const object = { 1: 1, a: 2 }; object[1]",
            Some(serde_json::json!([{ "checkAllIndexAccess": true }])),
        ),
        (
            "const object = { 1: 1 } as const; object[1]",
            Some(serde_json::json!([{ "checkAllIndexAccess": true }])),
        ),
        (
            "const object = { 1: 1 }; (object as Record<number, number>)[1]",
            Some(serde_json::json!([{ "checkAllIndexAccess": true }])),
        ),
        (
            "const object = { 1: 1 }; object![1]",
            Some(serde_json::json!([{ "checkAllIndexAccess": true }])),
        ),
        (
            "const object = { 1: 1 }; object[0 + 1]",
            Some(serde_json::json!([{ "checkAllIndexAccess": true }])),
        ),
        ("(5)[0]", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        ("(() => {})[0]", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        ("(class {})[0]", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
    ];

    let fail = vec![
        ("array[array.length - 1];", None),
        ("array?.[array.length - 1];", None),
        ("array[array.length -1];", None),
        ("array[array.length - /* comment */ 1];", None),
        ("array[array.length - 1.];", None),
        ("array[array.length - 0b1];", None),
        ("array[array.length - 9];", None),
        ("array[0][array[0].length - 1];", None),
        ("array[(( array.length )) - 1];", None),
        ("array[array.length - (( 1 ))];", None),
        ("array[(( array.length - 1 ))];", None),
        ("(( array ))[array.length - 1];", None),
        ("(( array[array.length - 1] ));", None),
        ("array[array.length - 1].pop().shift()[0];", None),
        ("a = array[array.length - 1]", None),
        ("const a = array[array.length - 1]", None),
        ("const {a = array[array.length - 1]} = {}", None),
        ("typeof array[array.length - 1]", None),
        ("function foo() {return arguments[arguments.length - 1]}", None),
        ("class Foo {bar; baz() {return this.bar[this.bar.length - 1]}}", None),
        ("class Foo {#bar; baz() {return this.#bar[this.#bar.length - 1]}}", None),
        ("string.charAt(string.length - 1);", None),
        ("string?.charAt(string.length - 1);", None),
        ("string.charAt(string.length - 0o11);", None),
        ("some.string.charAt(some.string.length - 1);", None),
        ("string.charAt((( string.length )) - 0xFF);", None),
        ("string.charAt(string.length - (( 1 )));", None),
        ("string.charAt((( string.length - 1 )));", None),
        ("(( string )).charAt(string.length - 1);", None),
        ("(( string.charAt ))(string.length - 1);", None),
        ("(( string.charAt(string.length - 1) ));", None),
        ("array.slice(-1)[0]", None),
        ("array.slice(-1)[0]", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        ("array?.slice(-1)[0]", None),
        ("array.slice(-1).pop()", None),
        ("array.slice(-1.0).shift()", None),
        ("array.slice(-1)[(( 0 ))];", None),
        ("array.slice(-(( 1 )))[0];", None),
        ("array.slice((( -1 )))[0];", None),
        ("(( array.slice(-1) ))[0];", None),
        ("(( array )).slice(-1)[0];", None),
        ("(( array.slice(-1)[0] ));", None),
        ("(( array.slice(-1) )).pop();", None),
        ("(( array.slice(-1).pop ))();", None),
        ("(( array.slice(-1).pop() ));", None),
        ("array.slice(-1)[0].pop().shift().slice(-1)", None),
        ("array.slice(-9, -8)[0]", None),
        ("array.slice(-9, -0o10)[0]", None),
        ("array.slice(-9, -8).pop()", None),
        ("array.slice(-9, -8).shift()", None),
        ("array.slice((( -9 )), (( -8 )), ).shift()", None),
        ("(( array.slice(-9, -8).shift ))()", None),
        ("array.slice(-9, unknown)[0]", None),
        ("array.slice(-0o11, -7)[0]", None),
        ("array.slice(-9, unknown).shift()", None),
        ("const KNOWN = -8; array.slice(-9, KNOWN).shift()", None),
        ("(( (( array.slice( ((-9)), ((unknown)), ).shift ))() ));", None),
        ("array.slice(-9, (a, really, _really, complicated, second) => argument)[0]", None),
        ("_.last(array)", None),
        ("lodash.last(array)", None),
        ("underscore.last(array)", None),
        ("_.last(new Array)", None),
        (
            "const foo = []
            _.last([bar])",
            None,
        ),
        (
            "const foo = []
            _.last( new Array )",
            None,
        ),
        (
            "const foo = []
            _.last( (( new Array )) )",
            None,
        ),
        ("if (foo) _.last([bar])", None),
        (
            "_.last(getLast(utils.lastOne(array)))",
            Some(
                serde_json::json!([{"getLastElementFunctions": ["getLast", "  utils.lastOne  "]}]),
            ),
        ),
        ("function foo() {return _.last(arguments)}", None),
        // checkAllIndexAccess: true, generated dynamically so not picked up by rulegen.
        ("array[0]", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        ("array[1]", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        ("array[5 + 9]", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        (
            "const offset = 5;array[offset + 9]",
            Some(serde_json::json!([{ "checkAllIndexAccess": true }])),
        ),
        (
            "const offset = -10;array[offset + 9]",
            Some(serde_json::json!([{ "checkAllIndexAccess": true }])),
        ),
        ("array[array.length - 1]", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        // `charAt` doesn't care about value.
        ("string.charAt(9)", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        ("string.charAt(5 + 9)", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        (
            "const offset = 5;string.charAt(offset + 9)",
            Some(serde_json::json!([{ "checkAllIndexAccess": true }])),
        ),
        ("string.charAt(unknown)", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        ("string.charAt(-1)", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        ("string.charAt(1.5)", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        ("string.charAt(1n)", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        (
            "string.charAt(string.length - 1)",
            Some(serde_json::json!([{ "checkAllIndexAccess": true }])),
        ),
        ("foo.charAt(bar.length - 1)", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        // Strings have `.at()`, so a string-literal receiver is still reported (#23870).
        ("\"abc\"[1]", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
    ];

    let fix = vec![
        // array[array.length - N] patterns
        ("array[array.length - 1]", "array.at(-1)", None),
        ("array[array.length - 2]", "array.at(-2)", None),
        // string.charAt patterns
        ("string.charAt(string.length - 1)", "string.at(-1)", None),
        ("string.charAt(string.length - 2)", "string.at(-2)", None),
        // array.slice(-N)[0] patterns
        ("array.slice(-1)[0]", "array.at(-1)", None),
        ("array.slice(-2)[0]", "array.at(-2)", None),
        (
            "array.slice(-1)[0]",
            "array.at(-1)",
            Some(serde_json::json!([{ "checkAllIndexAccess": true }])),
        ),
        // array.slice(-N).pop/shift patterns
        ("array.slice(-1).pop()", "array.at(-1)", None),
        ("array.slice(-1).shift()", "array.at(-1)", None),
        // Two-arg slice patterns
        ("array.slice(-9, -8)[0]", "array.at(-9)", None),
        ("array.slice(-3, -2).shift()", "array.at(-3)", None),
        ("array.slice(-9, -8).pop()", "array.at(-9)", None),
        ("array.slice(-2, -1).pop()", "array.at(-2)", None),
        // Lodash patterns
        ("_.last(array)", "array.at(-1)", None),
        ("lodash.last(array)", "array.at(-1)", None),
        // Edge cases with very large numbers
        ("array[array.length - 9007199254740992]", "array.at(-9007199254740992)", None),
        ("array[0]", "array.at(0)", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        (
            "array[5 + 9]",
            "array.at(5 + 9)",
            Some(serde_json::json!([{ "checkAllIndexAccess": true }])),
        ),
        (
            "string.charAt(9)",
            "string.at(9)",
            Some(serde_json::json!([{ "checkAllIndexAccess": true }])),
        ),
        // Strings have `.at()`, so a string-literal receiver is still fixed (#23870).
        ("\"abc\"[1]", "\"abc\".at(1)", Some(serde_json::json!([{ "checkAllIndexAccess": true }]))),
        ("_.last([] as [])", "([] as []).at(-1)", None),
        ("_.last([1, 2, 3] as const)", "([1, 2, 3] as const).at(-1)", None),
        // `arguments` is not an Array, so `.at()` is not guaranteed to exist.
        ("arguments[arguments.length - 1]", "arguments[arguments.length - 1]", None),
        (
            "arguments[1]",
            "arguments[1]",
            Some(serde_json::json!([{ "checkAllIndexAccess": true }])),
        ),
        ("_.last(arguments)", "_.last(arguments)", None),
        ("arguments.slice(-1)[0]", "arguments.slice(-1)[0]", None),
        ("arguments.slice(-1).pop()", "arguments.slice(-1).pop()", None),
        ("arguments.slice(-1).shift()", "arguments.slice(-1).shift()", None),
    ];

    Tester::new(PreferAt::NAME, PreferAt::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
