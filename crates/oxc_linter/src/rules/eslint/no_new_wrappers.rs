use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_new_wrappers_diagnostic(builtin_name: &str, new_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Do not use `{builtin_name}` as a constructor"))
        .with_help("Remove the `new` operator.")
        .with_label(new_span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNewWrappers;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `new` operators with the `String`, `Number`, and `Boolean` objects
    ///
    /// ### Why is this bad?
    ///
    /// The first problem is that primitive wrapper objects are, in fact,
    /// objects. That means typeof will return `"object"` instead of `"string"`,
    /// `"number"`, or `"boolean"`.  The second problem comes with boolean
    /// objects. Every object is truthy, that means an instance of `Boolean`
    /// always resolves to `true` even when its actual value is `false`.
    ///
    /// https://eslint.org/docs/latest/rules/no-new-wrappers
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var stringObject = new String('Hello world');
    /// var numberObject = new Number(33);
    /// var booleanObject = new Boolean(false);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var stringObject = 'Hello world';
    /// var stringObject2 = String(value);
    /// var numberObject = Number(value);
    /// var booleanObject = Boolean(value);
    /// ```
    NoNewWrappers,
    eslint,
    pedantic,
    pending
);

impl Rule for NoNewWrappers {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(expr) = node.kind() else {
            return;
        };
        let Expression::Identifier(ident) = &expr.callee else {
            return;
        };
        if (ident.name == "String" || ident.name == "Number" || ident.name == "Boolean")
            && ctx.semantic().is_reference_to_global_variable(ident)
        {
            ctx.diagnostic(no_new_wrappers_diagnostic(ident.name.as_str(), expr.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var a = new Object();",
        "var a = String('test'), b = String.fromCharCode(32);",
        "function test(Number) { return new Number; }",
        r#"
            import String from "./string";
            const str = new String(42);
        "#,
        "
            if (foo) {
                result = new Boolean(bar);
            } else {
                var Boolean = CustomBoolean;
            }
        ",
        // Disabled because the eslint-test uses languageOptions: { globals: { String: "off" } }
        // "new String()",

        // Disabled as the global option from the eslint-test does not work
        // "
        //     /* global Boolean:off */
        //     assert(new Boolean);
        // ",
    ];

    let fail = vec![
        "var a = new String('hello');",
        "var a = new Number(10);",
        "var a = new Boolean(false);",
        "
            const a = new String('bar');
            {
                const String = CustomString;
                const b = new String('foo');
            }
        ",
    ];

    Tester::new(NoNewWrappers::NAME, NoNewWrappers::PLUGIN, pass, fail).test_and_snapshot();
}
