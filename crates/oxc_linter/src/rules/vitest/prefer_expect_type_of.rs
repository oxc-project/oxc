use oxc_ast::{
    AstKind,
    ast::{Argument, UnaryOperator},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{MemberExpressionElement, PossibleJestNode, parse_expect_jest_fn_call},
};

fn prefer_expect_type_of_diagnostic(span: Span, help: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Type assertions should be done using `toBeTypeOf`.")
        .with_help(format!("Substitute the assertion with `{help}`."))
        .with_label(span)
        .with_note("https://vitest.dev/api/expect#tobetypeof")
}

#[derive(Debug, Default, Clone)]
pub struct PreferExpectTypeOf;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce using [`toBeTypeOf`](https://vitest.dev/api/expect#tobetypeof) instead of `expect(typeof ...).toBe(...)`.
    ///
    /// ### Why is this bad?
    ///
    /// `expect(typeof value).toBe(type)` works but is awkward and produces poor failure messages.
    /// Vitest's built-in `toBeTypeOf` matcher performs the same `typeof` comparison with a clearer API and better error output.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// test('type checking', () => {
    ///   expect(typeof 'hello').toBe('string')
    ///   expect(typeof 42).toBe('number')
    ///   expect(typeof true).toBe('boolean')
    ///   expect(typeof {}).toBe('object')
    ///   expect(typeof (() => {})).toBe('function')
    ///   expect(typeof Symbol()).toBe('symbol')
    ///   expect(typeof 123n).toBe('bigint')
    ///   expect(typeof undefined).toBe('undefined')
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// test('type checking', () => {
    ///   expect('hello').toBeTypeOf('string')
    ///   expect(42).toBeTypeOf('number')
    ///   expect(true).toBeTypeOf('boolean')
    ///   expect({}).toBeTypeOf('object')
    ///   expect(() => {}).toBeTypeOf('function')
    ///   expect(Symbol()).toBeTypeOf('symbol')
    ///   expect(123n).toBeTypeOf('bigint')
    ///   expect(undefined).toBeTypeOf('undefined')
    /// })
    /// ```
    PreferExpectTypeOf,
    vitest,
    style,
    fix,
    version = "1.44.0",
);

impl Rule for PreferExpectTypeOf {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        Self::run(jest_node, ctx);
    }
}

impl PreferExpectTypeOf {
    fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;

        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(expect_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        let Some(Argument::UnaryExpression(typeof_expression)) =
            expect_call.expect_arguments.and_then(|arguments| arguments.first())
        else {
            return;
        };

        if !matches!(typeof_expression.operator, UnaryOperator::Typeof) {
            return;
        }

        let Some(matcher) = expect_call.matcher() else {
            return;
        };

        if !(matcher.is_name_equal("toBe") || matcher.is_name_equal("toEqual")) {
            return;
        }

        let Some(type_expected) =
            expect_call.matcher_arguments.and_then(|arguments| arguments.first())
        else {
            return;
        };

        if matches!(type_expected, Argument::SpreadElement(_)) {
            return;
        }

        let modifier_text =
            expect_call.modifiers().iter().fold(String::new(), |mut acc, modifier| {
                use std::fmt::Write;
                match modifier.element {
                    // `.not`
                    MemberExpressionElement::IdentName(_) => {
                        write!(&mut acc, ".{}", ctx.source_range(modifier.span)).unwrap();
                    }
                    // `["not"]`, `[not]`, `[`not`]`, etc.
                    MemberExpressionElement::Expression(_) => {
                        write!(&mut acc, "[{}]", ctx.source_range(modifier.span)).unwrap();
                    }
                }
                acc
            });

        let param = ctx.source_range(GetSpan::span(&typeof_expression.argument));
        let type_text = ctx.source_range(type_expected.span());

        let code = format!("expect({param}){modifier_text}.toBeTypeOf({type_text})");

        ctx.diagnostic_with_fix(
            prefer_expect_type_of_diagnostic(call_expr.span, code.as_ref()),
            |fixer| fixer.replace(call_expr.span, code),
        );
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"expect("name").toBeTypeOf("string")"#,
        r#"expect("name").not.toBeTypeOf("string")"#,
        r#"expect(12).toBeTypeOf("number")"#,
        r#"expect(true).toBeTypeOf("boolean")"#,
        r#"expect({a: 1}).toBeTypeOf("object")"#,
        r#"expect(() => {}).toBeTypeOf("function")"#,
        r#"expect(sym).toBeTypeOf("symbol")"#,
        r#"expect(BigInt(123)).toBeTypeOf("bigint")"#,
        r#"expect(undefined).toBeTypeOf("undefined")"#,
        "expect(value).not.toBe(42)",
        "expect(value).not.toEqual(42)",
        "expect(typeof value).toBe(...typeNames)",
    ];

    let fail = vec![
        r#"expect(typeof 12).toBe("number")"#,
        r#"expect(typeof "name").toBe("string")"#,
        r#"expect(typeof true).toBe("boolean")"#,
        r#"expect(typeof variable).toBe("object")"#,
        r#"expect(typeof fn).toBe("function")"#,
        r#"expect(typeof sym).toBe("symbol")"#,
        r#"expect(typeof big).toBe("bigint")"#,
        r#"expect(typeof value).toBe("undefined")"#,
        r#"expect(typeof value).toEqual("string")"#,
        r#"expect(typeof value).not.toBe("string")"#,
        r#"expect(typeof value)["not"].toBe("string")"#,
        r#"expect(typeof value).toBe("unknown")"#,
        "expect(typeof value).toBe(typeName)",
    ];

    let fix = vec![
        (r#"expect(typeof 12).toBe("number")"#, r#"expect(12).toBeTypeOf("number")"#),
        (r#"expect(typeof "name").toBe("string")"#, r#"expect("name").toBeTypeOf("string")"#),
        (r#"expect(typeof true).toBe("boolean")"#, r#"expect(true).toBeTypeOf("boolean")"#),
        (r#"expect(typeof variable).toBe("object")"#, r#"expect(variable).toBeTypeOf("object")"#),
        (r#"expect(typeof fn).toBe("function")"#, r#"expect(fn).toBeTypeOf("function")"#),
        (r#"expect(typeof sym).toBe("symbol")"#, r#"expect(sym).toBeTypeOf("symbol")"#),
        (r#"expect(typeof big).toBe("bigint")"#, r#"expect(big).toBeTypeOf("bigint")"#),
        (r#"expect(typeof value).toBe("undefined")"#, r#"expect(value).toBeTypeOf("undefined")"#),
        (r#"expect(typeof value).toEqual("string")"#, r#"expect(value).toBeTypeOf("string")"#),
        (r#"expect(typeof value).not.toBe("string")"#, r#"expect(value).not.toBeTypeOf("string")"#),
        (
            r#"expect(typeof value)["not"].toBe("string")"#,
            r#"expect(value)["not"].toBeTypeOf("string")"#,
        ),
        (r#"expect(typeof value).toBe("unknown")"#, r#"expect(value).toBeTypeOf("unknown")"#),
        ("expect(typeof value).toBe(typeName)", "expect(value).toBeTypeOf(typeName)"),
    ];

    Tester::new(PreferExpectTypeOf::NAME, PreferExpectTypeOf::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
