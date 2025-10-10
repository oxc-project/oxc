use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, NewExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn no_new_wrappers_diagnostic(builtin_name: &str, new_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Do not use `{builtin_name}` as a constructor"))
        .with_help("Remove the `new` operator.")
        .with_label(new_span)
}

fn not_a_constructor_diagnostic(builtin_name: &str, new_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{builtin_name}` is not a constructor"))
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
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var stringObject = new String('Hello world');
    /// var numberObject = new Number(33);
    /// var booleanObject = new Boolean(false);
    /// var symbolObject = new Symbol('foo'); // symbol is not a constructor
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var stringObject = 'Hello world';
    /// var stringObject2 = String(value);
    /// var numberObject = Number(value);
    /// var booleanObject = Boolean(value);
    /// var symbolObject = Symbol('foo');
    /// ```
    NoNewWrappers,
    eslint,
    pedantic,
    fix,
);

impl Rule for NoNewWrappers {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(expr) = node.kind() else {
            return;
        };
        let Expression::Identifier(ident) = &expr.callee else {
            return;
        };
        if !ctx.is_reference_to_global_variable(ident) {
            return;
        }
        let name = ident.name.as_str();
        let span = if expr.span.size() > 24 {
            Span::new(expr.span.start, ident.span.end)
        } else {
            expr.span
        };
        match name {
            "String" | "Number" | "Boolean" => {
                ctx.diagnostic_with_fix(no_new_wrappers_diagnostic(name, span), |fixer| {
                    remove_new_operator(fixer, expr, name)
                });
            }
            "Symbol" => {
                ctx.diagnostic_with_fix(not_a_constructor_diagnostic(name, span), |fixer| {
                    remove_new_operator(fixer, expr, name)
                });
            }
            _ => {}
        }
    }
}

fn remove_new_operator<'a>(
    fixer: RuleFixer<'_, 'a>,
    expr: &NewExpression<'a>,
    name: &'a str,
) -> RuleFix {
    debug_assert!(expr.callee.is_identifier_reference());
    let remove_new_fix = fixer.delete_range(Span::new(expr.span.start, expr.callee.span().start));

    let Some(arg) = expr.arguments.first().and_then(Argument::as_expression) else {
        return remove_new_fix;
    };

    match (name, arg.get_inner_expression()) {
        ("Boolean", Expression::BooleanLiteral(lit)) => fixer.replace_with(expr, lit.as_ref()),
        ("String", Expression::StringLiteral(lit)) => fixer.replace_with(expr, lit.as_ref()),
        ("Number", Expression::NumericLiteral(lit)) => fixer.replace_with(expr, lit.as_ref()),
        _ => remove_new_fix,
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
        "
            var a = new String('wow look at me im a really long string, ' +
                               'it sure would be annoying if this whole thing ' +
                               'was underlined instead of just the constructor');
        ",
    ];
    let fix = vec![
        ("var a = new Number(foo);", "var a = Number(foo);"),
        ("var a = new Number(1 + 1);", "var a = Number(1 + 1);"),
        ("var a = new String(foo);", "var a = String(foo);"),
        ("var a = new Boolean(foo);", "var a = Boolean(foo);"),
        ("var a = new Boolean(!!x);", "var a = Boolean(!!x);"),
        ("var a = new Symbol('foo');", "var a = Symbol('foo');"),
        // literals dont need to be wrapped
        ("var a = new Boolean(false);", "var a = false;"),
        ("var a = new Boolean(true);", "var a = true;"),
        ("var a = new String('hello');", "var a = 'hello';"),
        ("var a = new String((((('hello')))));", "var a = 'hello';"),
        ("var a = new Number(10);", "var a = 10;"),
        ("var a = new Number(10 as number);", "var a = 10;"),
        ("(new Number())", "(Number())"),
    ];

    Tester::new(NoNewWrappers::NAME, NoNewWrappers::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
