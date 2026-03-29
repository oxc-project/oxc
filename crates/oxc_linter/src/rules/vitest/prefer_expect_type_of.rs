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
    OxcDiagnostic::warn("Type assertions should be done using `expectTypeOf`.")
        .with_help(format!("Substitute the assertion with `{help}`."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferExpectTypeOf;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce using `expectTypeOf` instead of `expect(typeof ...)`
    ///
    /// ### Why is this bad?
    ///
    /// Vitest provide a more expressive type-safe way to test type than using `expect(typeof ...)`
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
    ///   expect(typeof () => {}).toBe('function')
    ///   expect(typeof Symbol()).toBe('symbol')
    ///   expect(typeof 123n).toBe('bigint')
    ///   expect(typeof undefined).toBe('undefined')
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// test('type checking', () => {
    ///   expectTypeOf('hello').toBeString()
    ///   expectTypeOf(42).toBeNumber()
    ///   expectTypeOf(true).toBeBoolean()
    ///   expectTypeOf({}).toBeObject()
    ///   expectTypeOf(() => {}).toBeFunction()
    ///   expectTypeOf(Symbol()).toBeSymbol()
    ///   expectTypeOf(123n).toBeBigInt()
    ///   expectTypeOf(undefined).toBeUndefined()
    /// })
    /// ```
    PreferExpectTypeOf,
    vitest,
    style,
    fix,
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

        if !expect_call
            .members
            .iter()
            .any(|member| member.is_name_equal("toBe") || member.is_name_equal("toEqual"))
        {
            return;
        }

        let Some(Argument::StringLiteral(type_expected)) =
            expect_call.matcher_arguments.and_then(|arguments| arguments.first())
        else {
            return;
        };

        let method = {
            match type_expected.value.as_ref() {
                "string" => "toBeString",
                "number" => "toBeNumber",
                "boolean" => "toBeBoolean",
                "object" => "toBeObject",
                "function" => "toBeFunction",
                "symbol" => "toBeSymbol",
                "bigint" => "toBeBigInt",
                "undefined" => "toBeUndefined",
                _ => return,
            }
        };

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

        let code = format!("expectTypeOf({param}){modifier_text}.{method}()");

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
        r#"expectTypeOf("name").toBeString()"#,
        r#"expectTypeOf("name").not.toBeString()"#,
        "expectTypeOf(12).toBeNumber()",
        "expectTypeOf(12).not.toBeNumber()",
        "expectTypeOf(true).toBeBoolean()",
        "expectTypeOf({a: 1}).toBeObject()",
        "expectTypeOf(() => {}).toBeFunction()",
        "expectTypeOf(sym).toBeSymbol()",
        "expectTypeOf(BigInt(123)).toBeBigInt()",
        "expectTypeOf(undefined).toBeUndefined()",
        "expect(value).not.toBe(42)",
        "expect(value).not.toEqual(42)",
    ];

    let fail = vec![
        r#"expect(typeof 12).toBe("number")"#,
        r#"expect(typeof "name").toBe("string")"#,
        r#"expect(typeof true).toBe("boolean")"#,
        r#"expect(typeof variable).toBe("object")"#,
        r#"expect(typeof fn).toBe("function")"#,
        r#"expect(typeof value).toEqual("string")"#,
        r#"expect(typeof value).not.toBe("string")"#,
        r#"expect(typeof value)["not"].toBe("string")"#,
    ];

    let fix = vec![
        (r#"expect(typeof 12).toBe("number")"#, "expectTypeOf(12).toBeNumber()"),
        (r#"expect(typeof "name").toBe("string")"#, r#"expectTypeOf("name").toBeString()"#),
        (r#"expect(typeof true).toBe("boolean")"#, "expectTypeOf(true).toBeBoolean()"),
        (r#"expect(typeof variable).toBe("object")"#, "expectTypeOf(variable).toBeObject()"),
        (r#"expect(typeof fn).toBe("function")"#, "expectTypeOf(fn).toBeFunction()"),
        (r#"expect(typeof value).toEqual("string")"#, "expectTypeOf(value).toBeString()"),
        (r#"expect(typeof value).not.toBe("string")"#, "expectTypeOf(value).not.toBeString()"),
        (
            r#"expect(typeof value)["not"].toBe("string")"#,
            r#"expectTypeOf(value)["not"].toBeString()"#,
        ),
    ];

    Tester::new(PreferExpectTypeOf::NAME, PreferExpectTypeOf::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
