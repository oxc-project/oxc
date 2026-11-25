use oxc_ast::{
    AstKind,
    ast::{
        Argument, ArrayExpressionElement, BindingPattern, CallExpression, ChainElement, Expression,
        IdentifierReference, MemberExpression, NewExpression,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};
use oxc_syntax::precedence::Precedence;

use crate::{
    AstNode,
    ast_util::{
        could_be_asi_hazard, get_symbol_id_of_variable, is_method_call, variable_declaration_kind,
    },
    context::LintContext,
    rule::Rule,
    utils::{
        get_first_parameter_name, get_precedence, get_return_identifier_name,
        is_empty_array_expression, is_prototype_property, pad_fix_with_token_boundary,
    },
};

fn prefer_array_flat_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer Array#flat() over legacy techniques to flatten arrays.")
        .with_help(r"Call `.flat()` on the array instead.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferArrayFlat;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers `Array#flat()` over legacy techniques to flatten arrays.
    ///
    /// ### Why is this bad?
    ///
    /// ES2019 introduced a new method [`Array#flat()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/flat) that flatten arrays.
    ///
    /// This rule aims to standardize the use of `Array#flat()` over legacy techniques to flatten arrays.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = array.flatMap(x => x);
    /// const foo = array.reduce((a, b) => a.concat(b), []);
    /// const foo = array.reduce((a, b) => [...a, ...b], []);
    /// const foo = [].concat(...array);
    /// const foo = [].concat.apply([], array);
    /// const foo = Array.prototype.concat.apply([], array);
    /// const foo = Array.prototype.concat.call([], ...array);
    /// const foo = _.flatten(array);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const foo = array.flat();
    /// const foo = [maybeArray].flat();
    /// ```
    PreferArrayFlat,
    unicorn,
    pedantic,
    recommended,
    conditional_dangerous_fix,
    version = "0.0.20",
    short_description = "Prefers `Array#flat()` over legacy techniques to flatten arrays.",
);

impl Rule for PreferArrayFlat {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        check_array_flat_map_case(call_expr, ctx);
        check_array_reduce_case(call_expr, ctx);
        check_array_concat_case(call_expr, ctx);
        check_array_prototype_concat_case(call_expr, ctx);
        check_lodash_flatten_case(node, call_expr, ctx);
    }
}

// `array.flatMap(x => x)`
fn check_array_flat_map_case<'a>(call_expr: &CallExpression<'a>, ctx: &LintContext<'a>) {
    if !is_method_call(call_expr, None, Some(&["flatMap"]), Some(1), Some(1)) {
        return;
    }

    let Argument::ArrowFunctionExpression(first_argument) = &call_expr.arguments[0] else {
        return;
    };

    if first_argument.r#async || first_argument.params.parameters_count() != 1 {
        return;
    }

    let Some(first_param_name) = get_first_parameter_name(&first_argument.params) else {
        return;
    };

    let return_param_name = first_argument
        .get_expression()
        .and_then(|expression| {
            expression.get_identifier_reference().map(|ident| ident.name.as_str())
        })
        .or_else(|| first_argument.get_function_body().and_then(get_return_identifier_name));
    let Some(return_param_name) = return_param_name else {
        return;
    };

    if first_param_name != return_param_name {
        return;
    }

    if let Some(member_expr) = call_expr.callee.as_member_expression()
        && is_obviously_non_array_flat_map_receiver(member_expr.object(), ctx)
    {
        return;
    }

    let target_fix_span = call_expr
        .callee
        .as_member_expression()
        .and_then(MemberExpression::static_property_info)
        .map(|v| Span::new(v.0.start, call_expr.span.end));

    if let Some(span) = target_fix_span {
        ctx.diagnostic_with_fix(prefer_array_flat_diagnostic(call_expr.span), |fixer| {
            fixer.replace(span, "flat()")
        });
    } else {
        ctx.diagnostic(prefer_array_flat_diagnostic(call_expr.span));
    }
}

enum ConstInitKind {
    Array,
    NonArray,
}

fn is_obviously_non_array_flat_map_receiver(object: &Expression, ctx: &LintContext) -> bool {
    let Expression::Identifier(ident) = object.without_parentheses() else {
        return false;
    };
    let is_pascal_case = ident.name.chars().next().is_some_and(char::is_uppercase);
    match const_variable_initializer_kind(ident, ctx) {
        Some(ConstInitKind::Array) => false,
        Some(ConstInitKind::NonArray) => true,
        None => is_pascal_case,
    }
}

fn const_variable_initializer_kind(
    ident: &IdentifierReference,
    ctx: &LintContext,
) -> Option<ConstInitKind> {
    let symbol_id = get_symbol_id_of_variable(ident, ctx)?;
    let node = ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id));
    let AstKind::VariableDeclarator(declarator) = node.kind() else {
        return None;
    };
    if !variable_declaration_kind(declarator, ctx.nodes())?.is_const() {
        return None;
    }
    let init = declarator.init.as_ref()?.get_inner_expression();
    if is_definitely_array_expression(init) {
        Some(ConstInitKind::Array)
    } else if is_definitely_non_array_expression(init) {
        Some(ConstInitKind::NonArray)
    } else {
        None
    }
}

fn is_definitely_array_expression(expr: &Expression) -> bool {
    match expr {
        Expression::ArrayExpression(_) => true,
        Expression::NewExpression(new_expr) => {
            matches!(new_expr.callee.without_parentheses(), Expression::Identifier(ident) if ident.name == "Array")
        }
        _ => false,
    }
}

fn is_definitely_non_array_expression(expr: &Expression) -> bool {
    match expr {
        Expression::ObjectExpression(_)
        | Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::TemplateLiteral(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ClassExpression(_)
        | Expression::PrivateInExpression(_)
        | Expression::JSXElement(_)
        | Expression::JSXFragment(_) => true,
        Expression::NewExpression(new_expr) => {
            !matches!(new_expr.callee.without_parentheses(), Expression::Identifier(ident) if ident.name == "Array")
        }
        _ => false,
    }
}

// `array.reduce((a, b) => a.concat(b), [])`
// `array.reduce((a, b) => [...a, ...b], [])`
fn check_array_reduce_case<'a>(call_expr: &CallExpression<'a>, ctx: &LintContext<'a>) {
    if !is_method_call(call_expr, None, Some(&["reduce"]), Some(2), Some(2)) {
        return;
    }
    let Argument::ArrowFunctionExpression(first_argument) = &call_expr.arguments[0] else {
        return;
    };
    let Some(second_argument) = call_expr.arguments[1].as_expression() else {
        return;
    };

    if first_argument.r#async
        || first_argument.params.rest.is_some()
        || !is_empty_array_expression(second_argument)
    {
        return;
    }

    let [first_parameter, second_parameter] = first_argument.params.items.as_slice() else {
        return;
    };

    let Some((first_parameter, second_parameter)) = ({
        match (&first_parameter.pattern, &second_parameter.pattern) {
            (
                BindingPattern::BindingIdentifier(first_param),
                BindingPattern::BindingIdentifier(second_param),
            ) => Some((&first_param.name, &second_param.name)),

            _ => None,
        }
    }) else {
        return;
    };

    let Some(expression) = first_argument.get_expression() else { return };

    // `array.reduce((a, b) => a.concat(b), [])`
    if let Expression::CallExpression(concat_call_expr) = expression
        && is_method_call(concat_call_expr, None, Some(&["concat"]), Some(1), Some(1))
        && let Argument::Identifier(first_argument_ident) = &concat_call_expr.arguments[0]
    {
        if first_argument_ident.name != second_parameter {
            return;
        }

        let Expression::Identifier(second_argument_ident) =
            concat_call_expr.callee.get_member_expr().unwrap().object()
        else {
            return;
        };

        if second_argument_ident.name != first_parameter {
            return;
        }

        ctx.diagnostic_with_fix(prefer_array_flat_diagnostic(call_expr.span), |fixer| {
            let target_fix_span = call_expr
                .callee
                .as_member_expression()
                .and_then(MemberExpression::static_property_info)
                .map(|v| Span::new(v.0.start, call_expr.span.end));

            debug_assert!(target_fix_span.is_some());

            if let Some(span) = target_fix_span {
                fixer.replace(span, "flat()")
            } else {
                fixer.noop()
            }
        });
    }

    // `array.reduce((a, b) => [...a, ...b], [])`
    if let Expression::ArrayExpression(array_expr) = expression {
        if array_expr.elements.len() != 2 {
            return;
        }

        let Some((first_element, second_element)) = ({
            match (&array_expr.elements[0], &array_expr.elements[1]) {
                (
                    ArrayExpressionElement::SpreadElement(first_element),
                    ArrayExpressionElement::SpreadElement(second_element),
                ) => match (&first_element.argument, &second_element.argument) {
                    (
                        Expression::Identifier(first_element),
                        Expression::Identifier(second_element),
                    ) => Some((first_element, second_element)),
                    _ => None,
                },
                _ => None,
            }
        }) else {
            return;
        };

        if first_element.name != first_parameter || second_element.name != second_parameter {
            return;
        }

        ctx.diagnostic_with_fix(prefer_array_flat_diagnostic(call_expr.span), |fixer| {
            let target_fix_span = call_expr
                .callee
                .as_member_expression()
                .and_then(MemberExpression::static_property_info)
                .map(|v| Span::new(v.0.start, call_expr.span.end));

            debug_assert!(target_fix_span.is_some());

            if let Some(target_fix_span) = target_fix_span {
                fixer.replace(target_fix_span, "flat()")
            } else {
                fixer.noop()
            }
        });
    }
}

// `[].concat(...array)`
fn check_array_concat_case<'a>(call_expr: &CallExpression<'a>, ctx: &LintContext<'a>) {
    if call_expr.optional || !is_method_call(call_expr, None, Some(&["concat"]), Some(1), Some(1)) {
        return;
    }

    let Some(member_expr) = call_expr.callee.as_member_expression() else {
        return;
    };

    if member_expr.optional() {
        return;
    }

    // `[].concat(...array)`
    //
    // Only the spread form flattens, matching eslint-plugin-unicorn's `emptyArrayConcat`:
    // `[].concat(value)` is plain normalization of a single value into an array, so it is not
    // reported.
    if let Expression::ArrayExpression(array_expr) = member_expr.object() {
        if !array_expr.elements.is_empty()
            || !matches!(call_expr.arguments.first(), Some(Argument::SpreadElement(_)))
        {
            return;
        }
        ctx.diagnostic(prefer_array_flat_diagnostic(call_expr.span));
    }
}

// - `[].concat.apply([], array)` and `Array.prototype.concat.apply([], array)`
// - `[].concat.call([], ...array)` and `Array.prototype.concat.call([], ...array)`
fn check_array_prototype_concat_case<'a>(call_expr: &CallExpression<'a>, ctx: &LintContext<'a>) {
    let Some(member_expr) = call_expr.callee.get_member_expr() else {
        return;
    };

    if call_expr.optional || member_expr.optional() {
        return;
    }

    if let Some(member_expr_obj) = member_expr.object().as_member_expression() {
        let is_call_call = is_method_call(call_expr, None, Some(&["call"]), Some(2), Some(2));
        let is_apply_call = is_method_call(call_expr, None, Some(&["apply"]), Some(2), Some(2));

        // `call` only consumes an array of concat arguments when the second argument is spread,
        // while `apply` passes the array as-is, so a spread there is not flattening.
        // Plain normalization such as `[].concat.call([], value)` is not reported, matching
        // eslint-plugin-unicorn.
        let is_spread_second_argument =
            matches!(call_expr.arguments.get(1), Some(Argument::SpreadElement(_)));

        if (is_call_call || is_apply_call)
            && is_prototype_property(member_expr_obj, "concat", Some("Array"))
            && let Some(first_argument) = call_expr.arguments[0].as_expression()
            && is_empty_array_expression(first_argument)
            && (if is_call_call { is_spread_second_argument } else { !is_spread_second_argument })
        {
            if is_apply_call
                && let Some(Argument::Identifier(array)) = call_expr.arguments.get(1)
                && array.name != "arguments"
            {
                let replacement = format!("{}.flat()", ctx.source_range(array.span));
                ctx.diagnostic_with_dangerous_fix(
                    prefer_array_flat_diagnostic(call_expr.span),
                    |fixer| fixer.replace(call_expr.span, replacement),
                );
            } else {
                ctx.diagnostic(prefer_array_flat_diagnostic(call_expr.span));
            }
        }
    }
}

const TYPED_ARRAY_NAMES: &[&str] = &[
    "Int8Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "Int16Array",
    "Uint16Array",
    "Int32Array",
    "Uint32Array",
    "Float16Array",
    "Float32Array",
    "Float64Array",
    "BigInt64Array",
    "BigUint64Array",
];

fn is_known_non_array(expr: &Expression, ctx: &LintContext) -> bool {
    match expr.get_inner_expression() {
        Expression::Identifier(ident) => {
            (matches!(ident.name.as_str(), "arguments" | "undefined")
                && ident.is_global_reference(ctx.scoping()))
                || matches!(
                    const_variable_initializer_kind(ident, ctx),
                    Some(ConstInitKind::NonArray)
                )
        }
        Expression::CallExpression(call_expr) => {
            is_method_call(call_expr, Some(TYPED_ARRAY_NAMES), Some(&["from", "of"]), None, None)
        }
        Expression::UnaryExpression(_) => true,
        expr => is_definitely_non_array_expression(expr),
    }
}

fn new_expression_has_parens(new_expr: &NewExpression) -> bool {
    if !new_expr.arguments.is_empty() {
        return true;
    }
    let after_callee =
        new_expr.type_arguments.as_ref().map_or(new_expr.callee.span().end, |ta| ta.span.end);
    new_expr.span.end > after_callee
}

fn starts_with_declaration_like_prefix(mut expr: &Expression) -> bool {
    loop {
        match expr {
            Expression::ComputedMemberExpression(member) => {
                if matches!(&member.object, Expression::Identifier(ident) if ident.name == "let") {
                    return true;
                }
                expr = &member.object;
            }
            Expression::StaticMemberExpression(member) => expr = &member.object,
            Expression::PrivateFieldExpression(member) => expr = &member.object,
            Expression::CallExpression(call) => expr = &call.callee,
            Expression::TaggedTemplateExpression(tagged) => expr = &tagged.tag,
            Expression::TSNonNullExpression(non_null) => expr = &non_null.expression,
            Expression::TSInstantiationExpression(inst) => expr = &inst.expression,
            Expression::ChainExpression(chain) => match &chain.expression {
                ChainElement::CallExpression(call) => expr = &call.callee,
                ChainElement::StaticMemberExpression(member) => expr = &member.object,
                ChainElement::ComputedMemberExpression(member) => {
                    if matches!(
                        &member.object,
                        Expression::Identifier(ident) if ident.name == "let"
                    ) {
                        return true;
                    }
                    expr = &member.object;
                }
                ChainElement::PrivateFieldExpression(member) => expr = &member.object,
                ChainElement::TSNonNullExpression(non_null) => expr = &non_null.expression,
            },
            Expression::ObjectExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ClassExpression(_) => return true,
            _ => return false,
        }
    }
}

fn needs_member_object_parens(expr: &Expression) -> bool {
    if starts_with_declaration_like_prefix(expr) {
        return true;
    }
    match expr {
        Expression::ParenthesizedExpression(_)
        | Expression::CallExpression(_)
        | Expression::ImportExpression(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::PrivateFieldExpression(_)
        | Expression::TaggedTemplateExpression(_)
        | Expression::TSNonNullExpression(_) => false,
        Expression::NewExpression(new_expr) => !new_expression_has_parens(new_expr),
        Expression::ChainExpression(_)
        | Expression::TSInstantiationExpression(_)
        | Expression::JSXElement(_)
        | Expression::JSXFragment(_)
        | Expression::PrivateInExpression(_) => true,
        _ => get_precedence(expr).is_some_and(|p| p < Precedence::Call),
    }
}

// `_.flatten(array)`, `lodash.flatten(array)`, `underscore.flatten(array)`
fn check_lodash_flatten_case<'a>(
    node: &AstNode<'a>,
    call_expr: &CallExpression<'a>,
    ctx: &LintContext<'a>,
) {
    if call_expr.optional || call_expr.type_arguments.is_some() || call_expr.arguments.len() != 1 {
        return;
    }

    let Some(arg_expr) = call_expr.arguments[0].as_expression() else {
        return;
    };

    let Expression::StaticMemberExpression(member_expr) = call_expr.callee.without_parentheses()
    else {
        return;
    };

    if member_expr.optional || member_expr.property.name != "flatten" {
        return;
    }

    let Expression::Identifier(ident) = member_expr.object.without_parentheses() else {
        return;
    };

    if !matches!(ident.name.as_str(), "_" | "lodash" | "underscore") {
        return;
    }

    if is_known_non_array(arg_expr, ctx) {
        return;
    }

    let arg_span = arg_expr.span();
    let has_comments = ctx.has_comments_between(Span::new(call_expr.span.start, arg_span.start))
        || ctx.has_comments_between(Span::new(arg_span.end, call_expr.span.end));

    let diagnostic = prefer_array_flat_diagnostic(call_expr.span);
    if has_comments {
        ctx.diagnostic(diagnostic);
        return;
    }

    let mut replacement = {
        let arg_text = ctx.source_range(arg_span);
        if needs_member_object_parens(arg_expr) {
            format!("({arg_text}).flat()")
        } else {
            format!("{arg_text}.flat()")
        }
    };

    if matches!(replacement.as_bytes().first(), Some(b'(' | b'[' | b'`' | b'+' | b'-' | b'/'))
        && could_be_asi_hazard(node, ctx)
    {
        replacement.insert(0, ';');
    }

    pad_fix_with_token_boundary(ctx.source_text(), call_expr.span, &mut replacement);

    ctx.diagnostic_with_dangerous_fix(diagnostic, |fixer| {
        fixer.replace(call_expr.span, replacement)
    });
}

#[test]
fn test() {
    use crate::{fixer::FixKind, tester::Tester};

    let pass = vec![
        "array.flatMap",
        "new array.flatMap(x => x)",
        "flatMap(x => x)",
        "array.notFlatMap(x => x)",
        "array[flatMap](x => x)",
        "array.flatMap(x => x, thisArgument)",
        "array.flatMap(...[x => x])",
        "array.flatMap(function (x) { return x; })",
        "array.flatMap(async x => x)",
        "array.flatMap(function * (x) { return x;})",
        "array.flatMap(() => x)",
        "array.flatMap((x, y) => x)",
        // "array.flatMap((x) => { return x; })",
        "array.flatMap(x => y)",
        "const randomObject = {
                flatMap(function_) {
                    function_();
                },
            };
            randomObject.flatMap(x => x);",
        "Effects.flatMap(x => x)",
        "const effects = {
                flatMap(function_) {
                    function_();
                },
            };
            effects.flatMap(x => x);",
        "const effects = new Set(); effects.flatMap(x => x);",
        "const mapping = new Map(); mapping.flatMap(x => x);",
        r#"const text = ""; text.flatMap(x => x);"#,
        "const handler = () => {}; handler.flatMap(x => x);",
        "const collection = new Foo(); collection.flatMap(x => x);",
        "new array.reduce((a, b) => a.concat(b), [])",
        "array.reduce",
        "reduce((a, b) => a.concat(b), [])",
        "array[reduce]((a, b) => a.concat(b), [])",
        "array.notReduce((a, b) => a.concat(b), [])",
        "array.reduce((a, b) => a.concat(b), [], EXTRA_ARGUMENT)",
        "array.reduce((a, b) => a.concat(b), NOT_EMPTY_ARRAY)",
        "array.reduce((a, b, extraParameter) => a.concat(b), [])",
        "array.reduce((a,) => a.concat(b), [])",
        "array.reduce(() => a.concat(b), [])",
        "array.reduce((a, b) => {return a.concat(b); }, [])",
        "array.reduce(function (a, b) { return a.concat(b); }, [])",
        "array.reduce((a, b) => b.concat(b), [])",
        "array.reduce((a, b) => a.concat(a), [])",
        "array.reduce((a, b) => b.concat(a), [])",
        "array.reduce((a, b) => a.notConcat(b), [])",
        "array.reduce((a, b) => a.concat, [])",
        "new array.reduce((a, b) => [...a, ...b], [])",
        "array[reduce]((a, b) => [...a, ...b], [])",
        "reduce((a, b) => [...a, ...b], [])",
        "array.notReduce((a, b) => [...a, ...b], [])",
        "array.reduce((a, b) => [...a, ...b], [], EXTRA_ARGUMENT)",
        "array.reduce((a, b) => [...a, ...b], NOT_EMPTY_ARRAY)",
        "array.reduce((a, b, extraParameter) => [...a, ...b], [])",
        "array.reduce((a,) => [...a, ...b], [])",
        "array.reduce(() => [...a, ...b], [])",
        "array.reduce((a, b) => {return [...a, ...b]; }, [])",
        "array.reduce(function (a, b) { return [...a, ...b]; }, [])",
        "array.reduce((a, b) => [...b, ...b], [])",
        "array.reduce((a, b) => [...a, ...a], [])",
        "array.reduce((a, b) => [...b, ...a], [])",
        "array.reduce((a, b) => [a, ...b], [])",
        "array.reduce((a, b) => [...a, b], [])",
        "array.reduce((a, b) => [a, b], [])",
        "array.reduce((a, b) => [...a, ...b, c], [])",
        "array.reduce((a, b) => [...a, ...b,,], [])",
        "array.reduce((a, b) => [,...a, ...b], [])",
        "array.reduce((a, b) => [, ], [])",
        "array.reduce((a, b) => [, ,], [])",
        "[].concat",
        "new [].concat(array)",
        "[][concat](array)",
        "[].notConcat(array)",
        "[,].concat(array)",
        "({}).concat(array)",
        "[].concat()",
        "[].concat(array, EXTRA_ARGUMENT)",
        "[]?.concat(array)",
        "[].concat?.(array)",
        // Plain `[].concat(value)` normalization: only the spread form flattens
        "[].concat(array)",
        "[].concat(maybeArray)",
        "[].concat( ((0, maybeArray)) )",
        "[].concat( ((maybeArray)) )",
        "[].concat( [foo] )",
        "[].concat( [[foo]] )",
        "function foo(){return[].concat(maybeArray)}",
        "async function a() { return [].concat(await getArray()); }",
        "[].concat(some./**/array)",
        "[/**/].concat(some./**/array)",
        "[/**/].concat(some.array)",
        // `call` only flattens when the second argument is spread, so the plain normalization
        // forms below are not reported either
        "[].concat.call([], maybeArray)",
        "[].concat.call([], ((0, maybeArray)))",
        "[].concat.call([], ((maybeArray)))",
        "[].concat.call([], [foo])",
        "[].concat.call([], [[foo]])",
        "Array.prototype.concat.call([], maybeArray)",
        "Array.prototype.concat.call([], ((0, maybeArray)))",
        "Array.prototype.concat.call([], ((maybeArray)))",
        "Array.prototype.concat.call([], [foo])",
        "Array.prototype.concat.call([], [[foo]])",
        "Array.prototype.concat.call([], (0, array))",
        "before()
            Array.prototype.concat.call([], +1)",
        "before()
            Array.prototype.concat.call([], 1)",
        "before()
            Array.prototype.concat.call([], 1.)",
        "before()
            Array.prototype.concat.call([], .1)",
        "before()
            Array.prototype.concat.call([], 1.0)",
        "new [].concat(...array)",
        "[][concat](...array)",
        "[].notConcat(...array)",
        "[,].concat(...array)",
        "({}).concat(...array)",
        "[].concat()",
        "[].concat(...array, EXTRA_ARGUMENT)",
        "[]?.concat(...array)",
        "[].concat?.(...array)",
        "new [].concat.apply([], array)",
        "[].concat.apply",
        "[].concat.apply([], ...array)",
        "[].concat.apply([], array, EXTRA_ARGUMENT)",
        "[].concat.apply([])",
        "[].concat.apply(NOT_EMPTY_ARRAY, array)",
        "[].concat.apply([,], array)",
        "[,].concat.apply([], array)",
        "[].concat[apply]([], array)",
        "[][concat].apply([], array)",
        "[].concat.notApply([], array)",
        "[].notConcat.apply([], array)",
        "[].concat.apply?.([], array)",
        "[].concat?.apply([], array)",
        "[]?.concat.apply([], array)",
        "new Array.prototype.concat.apply([], array)",
        "Array.prototype.concat.apply",
        "Array.prototype.concat.apply([], ...array)",
        "Array.prototype.concat.apply([], array, EXTRA_ARGUMENT)",
        "Array.prototype.concat.apply([])",
        "Array.prototype.concat.apply(NOT_EMPTY_ARRAY, array)",
        "Array.prototype.concat.apply([,], array)",
        "Array.prototype.concat[apply]([], array)",
        "Array.prototype[concat].apply([], array)",
        "Array[prototype].concat.apply([], array)",
        "Array.prototype.concat.notApply([], array)",
        "Array.prototype.notConcat.apply([], array)",
        "Array.notPrototype.concat.apply([], array)",
        "NotArray.prototype.concat.apply([], array)",
        "Array.prototype.concat.apply?.([], array)",
        "Array.prototype.concat?.apply([], array)",
        "Array.prototype?.concat.apply([], array)",
        "Array?.prototype.concat.apply([], array)",
        "object.Array.prototype.concat.apply([], array)",
        "new _.flatten(array)",
        "_.flatten",
        "_.flatten(array, EXTRA_ARGUMENT)",
        "_.flatten(...array)",
        "_[flatten](array)",
        "_.notFlatten(array)",
        "NOT_LODASH.flatten(array)",
        "_.flatten?.(array)",
        "_?.flatten(array)",
        "object._.flatten(array)",
        "array.flat()",
        "array.flat(1)",
        "const effects = new Set() as Set<unknown>; effects.flatMap(x => x);",
        r#"const text = "" as string; text.flatMap(x => x);"#,
        "const handler = (() => {}) as Function; handler.flatMap(x => x);",
        "const collection = new Foo() satisfies Foo; collection.flatMap(x => x);",
        "array.reduce((a, ...b) => a.concat(b), [])",
        "array.reduce((a, ...b) => [...a, ...b], [])",
        "array.reduce((a, b, ...rest) => a.concat(b), [])",
        "array.reduce((a, b, ...rest) => [...a, ...b], [])",
        "_.flatten()",
        "lodash['flatten'](array)",
        "_.flatten<number[]>(array)",
        "_.flatten(1)",
        "_.flatten(+1)",
        "_.flatten(-1)",
        "_.flatten(1n)",
        r#"_.flatten("a")"#,
        "_.flatten(`a`)",
        "_.flatten({})",
        "_.flatten(undefined)",
        "_.flatten(null)",
        "_.flatten(new Uint8Array())",
        "_.flatten(new Set())",
        "_.flatten(new Map())",
        "_.flatten(Uint8Array.from(x))",
        "_.flatten(function () {})",
        "_.flatten(() => {})",
        "_.flatten(class {})",
        "_.flatten(new (class {})())",
        "function unicornRun() { return _.flatten(arguments); }",
        "function unicornRun() { return _.flatten((arguments)); }",
        "const set = new Set(); _.flatten(set);",
        "const Items = new lib.Array(); _.flatten(Items);",
        "const set = new Set(); _.flatten(set as (any));",
        "_.flatten(<div />);",
        "_.flatten(<></>);",
        "class Foo { #x; m(obj) { return _.flatten(#x in obj); } }",
    ];

    let fail = vec![
        "array.flatMap(x => x)",
        "array?.flatMap(x => x)",
        "function foo(){return[].flatMap(x => x)}",
        "foo.flatMap(x => x)instanceof Array",
        "array.flatMap((x) => x)",
        "Foo.bar.flatMap(x => x)",
        "const values = getValues(); values.flatMap(x => x);",
        "const values = []; values.flatMap(x => x);",
        "const Items = []; Items.flatMap(x => x);",
        "for (const value of values) {
                value.flatMap(x => x);
            }",
        "array.reduce((a, b) => a.concat(b), [])",
        "array?.reduce((a, b) => a.concat(b), [])",
        "function foo(){return[].reduce((a, b) => a.concat(b), [])}",
        "function foo(){return[]?.reduce((a, b) => a.concat(b), [])}",
        "array.reduce((a, b) => [...a, ...b], [])",
        "array.reduce((a, b) => [...a, ...b,], [])",
        "function foo(){return[].reduce((a, b) => [...a, ...b,], [])}",
        "[].concat(...array)",
        "[].concat(...(( array )))",
        "[].concat(...(( [foo] )))",
        "[].concat(...(( [[foo]] )))",
        "function foo(){return[].concat(...array)}",
        "class A extends[].concat(...array){}",
        "const A = class extends[].concat(...array){}",
        "[].concat.apply([], array)",
        "[].concat.apply([], ((0, array)))",
        "[].concat.apply([], ((array)))",
        "[].concat.apply([], [foo])",
        "[].concat.apply([], [[foo]])",
        "function flatten() { return [].concat.apply([], arguments); }",
        "[].concat.call([], ...array)",
        "[].concat.call([], ...((0, array)))",
        "[].concat.call([], ...((array)))",
        "[].concat.call([], ...[foo])",
        "[].concat.call([], ...[[foo]])",
        "function foo(){return[].concat.call([], ...array)}",
        "Array.prototype.concat.apply([], array)",
        "Array.prototype.concat.apply([], ((0, array)))",
        "Array.prototype.concat.apply([], ((array)))",
        "Array.prototype.concat.apply([], [foo])",
        "Array.prototype.concat.apply([], [[foo]])",
        "function flatten() { return Array.prototype.concat.apply([], arguments); }",
        "Array.prototype.concat.call([], ...array)",
        "Array.prototype.concat.call([], ...((0, array)))",
        "Array.prototype.concat.call([], ...((array)))",
        "Array.prototype.concat.call([], ...[foo])",
        "Array.prototype.concat.call([], ...[[foo]])",
        "/**/[].concat.apply([], array)",
        "Array.prototype.concat.apply([], array)",
        "_.flatten(array)",
        "lodash.flatten(array)",
        "underscore.flatten(array)",
        "before()
            Array.prototype.concat.apply([], [array].concat(array))",
        "before()
            Array.prototype.concat.apply([], +1)",
        "Array.prototype.concat.apply([], (0, array))",
        "_.flatten((0, array))",
        "async function a() { return _.flatten(await getArray()); }",
        "async function a() { return _.flatten((await getArray())); }",
        "before()
            Array.prototype.concat.apply([], 1)",
        "before()
            Array.prototype.concat.apply([], 1.)",
        "before()
            Array.prototype.concat.apply([], .1)",
        "before()
            Array.prototype.concat.apply([], 1.0)",
        "const Items = [] as unknown[]; Items.flatMap(x => x);",
        "_.flatten(array,)",
        "const Items = new Array(); _.flatten(Items);",
        "_.flatten(new Array)",
        "_.flatten(new Array())",
        "_.flatten(getArray())",
        "_.flatten(Array.from(x))",
        "before()\n_.flatten(getArray())",
        "_.flatten(foo?.bar)",
        "_.flatten(getArrays<number>)",
        "before()\n_.flatten([foo])",
        "function foo() { return(_.flatten)(array); }",
        "_.flatten(let[0]);",
        "_.flatten(let[0]());",
        "_.flatten(let[0]`a`);",
        "_.flatten(let[0]?.a);",
        "_.flatten({ a: [1] }.a);",
        "_.flatten({ a: [1] }['a']);",
        "const f = () => _.flatten({ a: [1] }.a);",
        "_.flatten(function () { return [[1]]; }());",
        "export default _.flatten(function () { return [[1]]; }());",
        "_.flatten(class { static a = [[1]]; }.a);",
        "before()\n_.flatten({ a: [1] }.a);",
        "before()\n_.flatten(({ a: [1] }).a);",
        "_.flatten(/* keep */ array);",
        "_.flatten(array /* keep */,);",
        "_.flatten(values satisfies { length: number });",
    ];

    let fix = vec![
        ("array.flatMap(x => x)", "array.flat()", None, FixKind::SafeFix),
        ("array.reduce((a, b) => a.concat(b), [])", "array.flat()", None, FixKind::SafeFix),
        ("array.reduce((a, b) => [...a, ...b], [])", "array.flat()", None, FixKind::SafeFix),
        ("Foo.bar.flatMap(x => x)", "Foo.bar.flat()", None, FixKind::SafeFix),
        (
            "const values = getValues(); values.flatMap(x => x);",
            "const values = getValues(); values.flat();",
            None,
            FixKind::SafeFix,
        ),
        (
            "const values = []; values.flatMap(x => x);",
            "const values = []; values.flat();",
            None,
            FixKind::SafeFix,
        ),
        (
            "const Items = []; Items.flatMap(x => x);",
            "const Items = []; Items.flat();",
            None,
            FixKind::SafeFix,
        ),
        (
            "for (const value of values) {
                value.flatMap(x => x);
            }",
            "for (const value of values) {
                value.flat();
            }",
            None,
            FixKind::SafeFix,
        ),
        (
            "const Items = [] as unknown[]; Items.flatMap(x => x);",
            "const Items = [] as unknown[]; Items.flat();",
            None,
            FixKind::SafeFix,
        ),
        ("/**/[].concat.apply([], array)", "/**/array.flat()", None, FixKind::DangerousFix),
        ("Array.prototype.concat.apply([], array)", "array.flat()", None, FixKind::DangerousFix),
        ("_.flatten(array)", "array.flat()", None, FixKind::DangerousFix),
        ("_.flatten(array,)", "array.flat()", None, FixKind::DangerousFix),
        ("lodash.flatten(array)", "array.flat()", None, FixKind::DangerousFix),
        ("underscore.flatten(array)", "array.flat()", None, FixKind::DangerousFix),
        (
            "const Items = new Array(); _.flatten(Items);",
            "const Items = new Array(); Items.flat();",
            None,
            FixKind::DangerousFix,
        ),
        ("_.flatten((0, array))", "(0, array).flat()", None, FixKind::DangerousFix),
        (
            "async function a() { return _.flatten(await getArray()); }",
            "async function a() { return (await getArray()).flat(); }",
            None,
            FixKind::DangerousFix,
        ),
        (
            "async function a() { return _.flatten((await getArray())); }",
            "async function a() { return (await getArray()).flat(); }",
            None,
            FixKind::DangerousFix,
        ),
        ("_.flatten(new Array)", "(new Array).flat()", None, FixKind::DangerousFix),
        ("_.flatten(new Array())", "new Array().flat()", None, FixKind::DangerousFix),
        ("_.flatten(getArray())", "getArray().flat()", None, FixKind::DangerousFix),
        ("_.flatten(Array.from(x))", "Array.from(x).flat()", None, FixKind::DangerousFix),
        (
            "before()\n_.flatten(getArray())",
            "before()\ngetArray().flat()",
            None,
            FixKind::DangerousFix,
        ),
        ("_.flatten(foo?.bar)", "(foo?.bar).flat()", None, FixKind::DangerousFix),
        ("_.flatten(getArrays<number>)", "(getArrays<number>).flat()", None, FixKind::DangerousFix),
        ("before()\n_.flatten([foo])", "before()\n;[foo].flat()", None, FixKind::DangerousFix),
        (
            "function foo() { return(_.flatten)(array); }",
            "function foo() { return array.flat(); }",
            None,
            FixKind::DangerousFix,
        ),
        ("_.flatten(let[0]);", "(let[0]).flat();", None, FixKind::DangerousFix),
        ("_.flatten(let[0]());", "(let[0]()).flat();", None, FixKind::DangerousFix),
        ("_.flatten(let[0]`a`);", "(let[0]`a`).flat();", None, FixKind::DangerousFix),
        ("_.flatten(let[0]?.a);", "(let[0]?.a).flat();", None, FixKind::DangerousFix),
        ("_.flatten({ a: [1] }.a);", "({ a: [1] }.a).flat();", None, FixKind::DangerousFix),
        ("_.flatten({ a: [1] }['a']);", "({ a: [1] }['a']).flat();", None, FixKind::DangerousFix),
        (
            "const f = () => _.flatten({ a: [1] }.a);",
            "const f = () => ({ a: [1] }.a).flat();",
            None,
            FixKind::DangerousFix,
        ),
        (
            "_.flatten(function () { return [[1]]; }());",
            "(function () { return [[1]]; }()).flat();",
            None,
            FixKind::DangerousFix,
        ),
        (
            "export default _.flatten(function () { return [[1]]; }());",
            "export default (function () { return [[1]]; }()).flat();",
            None,
            FixKind::DangerousFix,
        ),
        (
            "_.flatten(class { static a = [[1]]; }.a);",
            "(class { static a = [[1]]; }.a).flat();",
            None,
            FixKind::DangerousFix,
        ),
        (
            "before()\n_.flatten({ a: [1] }.a);",
            "before()\n;({ a: [1] }.a).flat();",
            None,
            FixKind::DangerousFix,
        ),
        (
            "before()\n_.flatten(({ a: [1] }).a);",
            "before()\n;({ a: [1] }).a.flat();",
            None,
            FixKind::DangerousFix,
        ),
        (
            "_.flatten(/* keep */ array);",
            "_.flatten(/* keep */ array);",
            None,
            FixKind::DangerousFix,
        ),
        (
            "_.flatten(array /* keep */,);",
            "_.flatten(array /* keep */,);",
            None,
            FixKind::DangerousFix,
        ),
        (
            "_.flatten(values satisfies { length: number });",
            "(values satisfies { length: number }).flat();",
            None,
            FixKind::DangerousFix,
        ),
    ];

    Tester::new(PreferArrayFlat::NAME, PreferArrayFlat::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
