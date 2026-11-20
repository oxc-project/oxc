use oxc_ast::{
    AstKind,
    ast::{
        CallExpression, Expression, IdentifierReference, MemberExpression, StaticMemberExpression,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::BinaryOperator;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{is_same_expression, pad_fix_with_token_boundary},
};

fn prefer_number_is_safe_integer_diagnostic(span: Span, check: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `Number.isSafeInteger()` over {check}."))
        .with_help("Review the value's type and range before using `Number.isSafeInteger()`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferNumberIsSafeInteger;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers `Number.isSafeInteger()` over `Number.isInteger()`, modulo checks,
    /// comparisons with `Math.floor()` or `Math.trunc()`, and Lodash or Underscore
    /// `isInteger()` and `isSafeInteger()` calls.
    ///
    /// ### Why is this bad?
    ///
    /// Integers outside `[-(2 ** 53 - 1), 2 ** 53 - 1]` can lose precision.
    /// `Number.isInteger(2 ** 53)` returns `true`, while `Number.isSafeInteger()`
    /// rejects that value. Modulo checks also coerce values: `[['1']] % 1 === 0`
    /// is `true`, while `Number.isSafeInteger([['1']])` is `false`.
    ///
    /// This rule only provides suggestions because changing the check can change
    /// program behavior, including widening negated checks such as `!Number.isInteger(x)`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// Number.isInteger(value);
    /// value % 1 === 0;
    /// Math.trunc(value) === value;
    /// _.isInteger(value);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// Number.isSafeInteger(value);
    /// ```
    PreferNumberIsSafeInteger,
    unicorn,
    pedantic,
    conditional_suggestion,
    version = "next",
    short_description = "Prefer `Number.isSafeInteger()` over integer checks.",
);

impl Rule for PreferNumberIsSafeInteger {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let argument = match node.kind() {
            AstKind::CallExpression(call) => {
                let Some((object, member)) = static_method(call) else { return };
                if object.name == "Number"
                    && member.property.name == "isInteger"
                    && ctx.is_reference_to_global_variable(object)
                {
                    ctx.diagnostic_with_suggestion(
                        prefer_number_is_safe_integer_diagnostic(
                            member.property.span,
                            "`Number.isInteger()`",
                        ),
                        |fixer| {
                            fixer.replace(member.property.span, "isSafeInteger").with_message(
                                "Replace `Number.isInteger()` with `Number.isSafeInteger()`.",
                            )
                        },
                    );
                    return;
                }
                if !matches!(object.name.as_str(), "_" | "lodash" | "underscore")
                    || !matches!(member.property.name.as_str(), "isInteger" | "isSafeInteger")
                    || call.arguments.len() != 1
                {
                    return;
                }
                call.arguments[0].as_expression()
            }
            AstKind::BinaryExpression(binary)
                if binary.operator == BinaryOperator::StrictEquality =>
            {
                [(&binary.left, &binary.right), (&binary.right, &binary.left)].into_iter().find_map(
                    |(check, value)| {
                        if value.without_parentheses().is_number_0()
                            && let Expression::BinaryExpression(modulo) =
                                check.without_parentheses()
                            && modulo.operator == BinaryOperator::Remainder
                            && let Expression::NumericLiteral(divisor) =
                                modulo.right.without_parentheses()
                            && divisor.value.to_bits() == 1.0_f64.to_bits()
                        {
                            return Some(&modulo.left);
                        }
                        let Expression::CallExpression(call) = check.without_parentheses() else {
                            return None;
                        };
                        let (object, member) = static_method(call)?;
                        if object.name != "Math"
                            || !matches!(member.property.name.as_str(), "floor" | "trunc")
                            || call.arguments.len() != 1
                            || !ctx.is_reference_to_global_variable(object)
                        {
                            return None;
                        }
                        let argument = call.arguments[0].as_expression()?;
                        same_reference(argument, value, ctx).then_some(argument)
                    },
                )
            }
            _ => return,
        };
        let Some(argument) = argument else { return };
        if ctx.scoping().find_binding(node.scope_id(), "Number".into()).is_some() {
            return;
        }
        let argument = argument.without_parentheses();
        let span = node.kind().span();
        ctx.diagnostic_with_suggestion(
            prefer_number_is_safe_integer_diagnostic(span, "this integer check"),
            |fixer| {
                if ctx.has_comments_between(Span::new(span.start, argument.span().start))
                    || ctx.has_comments_between(Span::new(argument.span().end, span.end))
                {
                    return fixer.noop();
                }
                let text = ctx.source_range(argument.span());
                let mut replacement = if matches!(argument, Expression::SequenceExpression(_)) {
                    format!("Number.isSafeInteger(({text}))")
                } else {
                    format!("Number.isSafeInteger({text})")
                };
                pad_fix_with_token_boundary(ctx.source_text(), span, &mut replacement);
                fixer
                    .replace(span, replacement)
                    .with_message("Replace this integer check with `Number.isSafeInteger()`.")
            },
        );
    }
}

fn static_method<'a>(
    call: &'a CallExpression<'a>,
) -> Option<(&'a IdentifierReference<'a>, &'a StaticMemberExpression<'a>)> {
    let Expression::StaticMemberExpression(member) = call.callee.without_parentheses() else {
        return None;
    };
    let Expression::Identifier(object) = member.object.without_parentheses() else { return None };
    (!call.optional && !member.optional).then_some((object, member))
}

fn same_reference(left: &Expression, right: &Expression, ctx: &LintContext) -> bool {
    let left = left.get_inner_expression();
    let right = right.get_inner_expression();
    if let (Some(left), Some(right)) = (left.get_member_expr(), right.get_member_expr()) {
        return same_reference(left.object(), right.object(), ctx)
            && match (left.static_property_name(), right.static_property_name()) {
                (Some(left), Some(right)) => left == right,
                (None, None) => match (left, right) {
                    (
                        MemberExpression::ComputedMemberExpression(left),
                        MemberExpression::ComputedMemberExpression(right),
                    ) => same_reference(&left.expression, &right.expression, ctx),
                    (
                        MemberExpression::PrivateFieldExpression(left),
                        MemberExpression::PrivateFieldExpression(right),
                    ) => left.field.name == right.field.name,
                    _ => false,
                },
                _ => false,
            };
    }
    match (left, right) {
        (Expression::NumericLiteral(left), Expression::NumericLiteral(right)) => {
            left.value.partial_cmp(&right.value) == Some(std::cmp::Ordering::Equal)
        }
        (Expression::BigIntLiteral(left), Expression::BigIntLiteral(right)) => {
            left.value == right.value
        }
        (Expression::Identifier(_), Expression::Identifier(_))
        | (Expression::ThisExpression(_), Expression::ThisExpression(_))
        | (Expression::Super(_), Expression::Super(_))
        | (Expression::NullLiteral(_), Expression::NullLiteral(_))
        | (Expression::BooleanLiteral(_), Expression::BooleanLiteral(_))
        | (Expression::StringLiteral(_), Expression::StringLiteral(_))
        | (Expression::RegExpLiteral(_), Expression::RegExpLiteral(_)) => {
            is_same_expression(left, right, ctx)
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Number.isSafeInteger(x)",
        "Number.isInteger",
        "new Number.isInteger(x)",
        "isInteger(x)",
        "Number.isInteger?.(x)",
        "Number?.isInteger(x)",
        "Number[isInteger](x)",
        r#"Number["isInteger"](x)"#,
        "Number.isinteger(x)",
        "NotNumber.isInteger(x)",
        "window.Number.isInteger(x)",
        "Number.isInteger.bind(Number)",
        "const x = Number.isInteger",
        "globalThis.Number.isInteger(x)",
        "const Number = {isInteger() {}}; Number.isInteger(x)",
        r#"import Number from "number"; Number.isInteger(x)"#,
        "function foo(Number) { return Number.isInteger(x); }",
        "const Number = {}; value % 1 === 0",
        r#"import Number from "number"; Math.trunc(value) === value"#,
        "function foo(Number) { return _.isInteger(value); }",
        "value % 1 == 0",
        "value % 1 !== 0",
        "0 !== value % 1",
        "(value | 0) === value",
        "Number.parseInt(value, 10) === value",
        "Math.round(value) === value",
        "Math.floor(value) === otherValue",
        "Math.trunc(value) === otherValue",
        "Math.floor(value) == value",
        "Math.trunc(value) !== value",
        "const Math = {trunc() {}}; Math.trunc(value) === value",
        "Math.trunc?.(value) === value",
        "Math?.trunc(value) === value",
        "Math[trunc](value) === value",
        r#"Math["trunc"](value) === value"#,
        "Math.floor(...value) === value",
        "Math.trunc(...value) === value",
        "Math.floor(value, extra) === value",
        "Math.trunc(value, extra) === value",
        "_.isInteger?.(value)",
        "_?.isInteger(value)",
        "_[isInteger](value)",
        r#"_["isInteger"](value)"#,
        "_.isInteger(...value)",
        "_.isInteger(value, extra)",
        "lodash.isSafeInteger(...value)",
        "underscore.isSafeInteger(value, extra)",
        "value % 2 === 0",
        "value % 0.9999999999999999 === 0",
        "value * 1 === 0",
        "0 === value",
        "function f() { value % 1 === 0; var Number; }",
        "function f(Math) { return Math.floor(value) === value; }",
        "Math.trunc(getValue()) === getValue()",
        "Math.trunc(value + 1) === value + 1",
        "Math.floor(1) === 1.0000000000000002",
        "Math.floor(1n) === 2n",
        "Math.trunc(object[index + 1]) === object[index + 1]",
        "Math.trunc(value) === (value, other)",
        "value % 1 === -0",
        "_.isInteger()",
    ];

    let fix = vec![
        ("Number.isInteger(x)", "Number.isSafeInteger(x)"),
        ("!Number.isInteger(x)", "!Number.isSafeInteger(x)"),
        ("if (Number.isInteger(x)) {}", "if (Number.isSafeInteger(x)) {}"),
        ("if (!Number.isInteger(x)) {}", "if (!Number.isSafeInteger(x)) {}"),
        ("Number.isInteger(0.5)", "Number.isSafeInteger(0.5)"),
        ("(( (( Number )).isInteger( ((x)), )))", "(( (( Number )).isSafeInteger( ((x)), )))"),
        ("Number.isInteger(x) ? a : b", "Number.isSafeInteger(x) ? a : b"),
        ("Number.isInteger(/* comment */ x)", "Number.isSafeInteger(/* comment */ x)"),
        ("Number.isInteger(...x)", "Number.isSafeInteger(...x)"),
        ("Number.isInteger()", "Number.isSafeInteger()"),
        ("value % 1 === 0", "Number.isSafeInteger(value)"),
        ("0 === value % 1", "Number.isSafeInteger(value)"),
        ("object.value % 1 === 0", "Number.isSafeInteger(object.value)"),
        ("(foo, bar) % 1 === 0", "Number.isSafeInteger((foo, bar))"),
        ("Math.floor(value) === value", "Number.isSafeInteger(value)"),
        ("Math.floor(1) === 1.0", "Number.isSafeInteger(1)"),
        ("Math.floor(1n) === 0x1n", "Number.isSafeInteger(1n)"),
        ("value === Math.floor(value)", "Number.isSafeInteger(value)"),
        ("Math.trunc(value) === value", "Number.isSafeInteger(value)"),
        ("value === Math.trunc(value)", "Number.isSafeInteger(value)"),
        ("Math.trunc(object.value) === object.value", "Number.isSafeInteger(object.value)"),
        (
            r#"Math.trunc(object["value"]) === object.value"#,
            r#"Number.isSafeInteger(object["value"])"#,
        ),
        ("_.isInteger(value)", "Number.isSafeInteger(value)"),
        ("lodash.isInteger(value)", "Number.isSafeInteger(value)"),
        ("underscore.isInteger(value)", "Number.isSafeInteger(value)"),
        ("_.isSafeInteger(value)", "Number.isSafeInteger(value)"),
        ("lodash.isSafeInteger(value)", "Number.isSafeInteger(value)"),
        ("underscore.isSafeInteger(value)", "Number.isSafeInteger(value)"),
        ("value /* comment */ % 1 === 0", "value /* comment */ % 1 === 0"),
        ("0 === value /* comment */ % 1", "0 === value /* comment */ % 1"),
        ("_.isInteger(/* comment */ value)", "_.isInteger(/* comment */ value)"),
        ("Math.trunc(/* comment */ value) === value", "Math.trunc(/* comment */ value) === value"),
        ("Number.isInteger(x as number)", "Number.isSafeInteger(x as number)"),
        ("Number.isInteger(x!)", "Number.isSafeInteger(x!)"),
        ("Math.trunc(value as number) === value", "Number.isSafeInteger(value as number)"),
        ("Math.trunc(value!) === value", "Number.isSafeInteger(value!)"),
        ("Math.trunc(value) === value as number", "Number.isSafeInteger(value)"),
        ("Number.isInteger(2 ** 53)", "Number.isSafeInteger(2 ** 53)"),
        ("!Number.isInteger(2 ** 53)", "!Number.isSafeInteger(2 ** 53)"),
        ("[['1']] % 1 === 0", "Number.isSafeInteger([['1']])"),
        ("Number.isInteger(x, y)", "Number.isSafeInteger(x, y)"),
        ("Math.trunc(object[index]) === object[index]", "Number.isSafeInteger(object[index])"),
        ("_.isInteger(object./* keep */value)", "Number.isSafeInteger(object./* keep */value)"),
        ("Math.trunc(value) === value /* keep */", "Number.isSafeInteger(value) /* keep */"),
        ("function f(){return.5 % 1 === 0}", "function f(){return Number.isSafeInteger(.5)}"),
    ];
    let fail = fix.iter().map(|(source, _)| *source).collect();

    Tester::new(PreferNumberIsSafeInteger::NAME, PreferNumberIsSafeInteger::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
