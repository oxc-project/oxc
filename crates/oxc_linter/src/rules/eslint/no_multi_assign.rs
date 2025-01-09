use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_multi_assign_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use chained assignment")
        .with_help("Separate each assignment into its own statement")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoMultiAssign {
    ignore_non_declaration: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow use of chained assignment expressions.
    ///
    /// ### Why is this bad?
    ///
    /// Chaining the assignment of variables can lead to unexpected results and be difficult to read.
    /// ```js
    /// (function() {
    ///     const foo = bar = 0; // Did you mean `foo = bar == 0`?
    ///     bar = 1;             // This will not fail since `bar` is not constant.
    /// })();
    /// console.log(bar);        // This will output 1 since `bar` is not scoped.
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var a = b = c = 5;
    ///
    /// const foo = bar = "baz";
    ///
    /// let d =
    ///     e =
    ///     f;
    ///
    /// class Foo {
    ///     a = b = 10;
    /// }
    ///
    /// a = b = "quux";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var a = 5;
    /// var b = 5;
    /// var c = 5;
    ///
    /// const foo = "baz";
    /// const bar = "baz";
    ///
    /// let d = c;
    /// let e = c;
    ///
    /// class Foo {
    ///     a = 10;
    ///     b = 10;
    /// }
    ///
    /// a = "quux";
    /// b = "quux";
    /// ```
    ///
    /// ### Options
    ///
    /// This rule has an object option:
    /// * `"ignoreNonDeclaration"`: When set to `true`, the rule allows chains that don't include initializing a variable in a declaration or initializing a class field. Default is `false`.
    ///
    /// #### ignoreNonDeclaration
    ///
    /// Examples of **correct** code for the `{ "ignoreNonDeclaration": true }` option:
    /// ```js
    /// let a;
    /// let b;
    /// a = b = "baz";
    ///
    /// const x = {};
    /// const y = {};
    /// x.one = y.one = 1;
    /// ```
    ///
    /// Examples of **incorrect** code for the `{ "ignoreNonDeclaration": true }` option:
    /// ```js
    /// let a = b = "baz";
    ///
    /// const foo = bar = 1;
    ///
    /// class Foo {
    ///     a = b = 10;
    /// }
    /// ```
    NoMultiAssign,
    eslint,
    style,
);

impl Rule for NoMultiAssign {
    fn from_configuration(value: serde_json::Value) -> Self {
        let ignore_non_declaration = value
            .get(0)
            .and_then(|config| config.get("ignoreNonDeclaration"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Self { ignore_non_declaration }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // e.g. `var a = b = c;`
        if let AstKind::VariableDeclarator(declarator) = node.kind() {
            let Some(Expression::AssignmentExpression(assign_expr)) = &declarator.init else {
                return;
            };
            ctx.diagnostic(no_multi_assign_diagnostic(assign_expr.span));
        }

        // e.g. `class A { a = b = 1; }`
        if let AstKind::PropertyDefinition(prop_def) = node.kind() {
            let Some(Expression::AssignmentExpression(assign_expr)) = &prop_def.value else {
                return;
            };
            ctx.diagnostic(no_multi_assign_diagnostic(assign_expr.span));
        }

        // e.g. `let a; let b; a = b = 1;`
        if !self.ignore_non_declaration {
            if let AstKind::AssignmentExpression(parent_expr) = node.kind() {
                let Expression::AssignmentExpression(expr) = &parent_expr.right else {
                    return;
                };
                ctx.diagnostic(no_multi_assign_diagnostic(expr.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "var a, b, c,
			d = 0;",
            None,
        ),
        (
            "var a = 1; var b = 2; var c = 3;
			var d = 0;",
            None,
        ),
        ("var a = 1 + (b === 10 ? 5 : 4);", None),
        ("const a = 1, b = 2, c = 3;", None), // { "ecmaVersion": 6 },
        (
            "const a = 1;
			const b = 2;
			 const c = 3;",
            None,
        ), // { "ecmaVersion": 6 },
        ("for(var a = 0, b = 0;;){}", None),
        ("for(let a = 0, b = 0;;){}", None), // { "ecmaVersion": 6 },
        ("for(const a = 0, b = 0;;){}", None), // { "ecmaVersion": 6 },
        ("export let a, b;", None),          // { "ecmaVersion": 6, "sourceType": "module" },
        (
            "export let a,
			 b = 0;",
            None,
        ), // { "ecmaVersion": 6, "sourceType": "module" },
        (
            "const x = {};const y = {};x.one = y.one = 1;",
            Some(serde_json::json!([{ "ignoreNonDeclaration": true }])),
        ), // { "ecmaVersion": 6 },
        ("let a, b;a = b = 1", Some(serde_json::json!([{ "ignoreNonDeclaration": true }]))), // { "ecmaVersion": 6 },
        ("class C { [foo = 0] = 0 }", None), // { "ecmaVersion": 2022 }
    ];

    let fail = vec![
        ("var a = b = c;", None),
        ("var a = b = c = d;", None),
        ("let foo = bar = cee = 100;", None), // { "ecmaVersion": 6 },
        ("a=b=c=d=e", None),
        ("a=b=c", None),
        (
            "a
			=b
			=c",
            None,
        ),
        ("var a = (b) = (((c)))", None),
        ("var a = ((b)) = (c)", None),
        ("var a = b = ( (c * 12) + 2)", None),
        (
            "var a =
			((b))
			 = (c)",
            None,
        ),
        ("a = b = '=' + c + 'foo';", None),
        ("a = b = 7 * 12 + 5;", None),
        (
            "const x = {};
			const y = x.one = 1;",
            Some(serde_json::json!([{ "ignoreNonDeclaration": true }])),
        ), // { "ecmaVersion": 6 },
        ("let a, b;a = b = 1", Some(serde_json::json!([{}]))), // { "ecmaVersion": 6 },
        ("let x, y;x = y = 'baz'", Some(serde_json::json!([{ "ignoreNonDeclaration": false }]))), // { "ecmaVersion": 6 },
        ("const a = b = 1", Some(serde_json::json!([{ "ignoreNonDeclaration": true }]))), // { "ecmaVersion": 6 },
        ("class C { field = foo = 0 }", None), // { "ecmaVersion": 2022 },
        (
            "class C { field = foo = 0 }",
            Some(serde_json::json!([{ "ignoreNonDeclaration": true }])),
        ), // { "ecmaVersion": 2022 }
    ];

    Tester::new(NoMultiAssign::NAME, NoMultiAssign::PLUGIN, pass, fail).test_and_snapshot();
}
