use std::{borrow::Cow, cmp::Ordering};

use cow_utils::CowUtils;
use oxc_ast::{
    ast::{BindingPatternKind, VariableDeclarator},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn sort_vars_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Variable declarations should be sorted").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct SortVars {
    ignore_case: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// When declaring multiple variables within the same block, sorting variable names make it
    /// easier to find necessary variable easier at a later time.
    ///
    /// ### Why is this bad?
    ///
    /// Unsorted variable declarations can make the code harder to read and maintain.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var b, a;
    /// var a, B, c;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var a, b, c, d;
    /// var B, a, c;
    /// ```
    SortVars,
    eslint,
    pedantic,
    pending
);

impl Rule for SortVars {
    fn from_configuration(value: serde_json::Value) -> Self {
        let ignore_case = value
            .get(0)
            .and_then(|v| v.get("ignoreCase"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Self { ignore_case }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::VariableDeclaration(var_decl) = node.kind() else {
            return;
        };

        if var_decl.declarations.len() <= 1 {
            return;
        };

        let mut previous: Option<&VariableDeclarator> = None;
        for current in var_decl
            .declarations
            .iter()
            .filter(|decl| matches!(decl.id.kind, BindingPatternKind::BindingIdentifier(_)))
        {
            if let Some(previous) = previous {
                if self.get_sortable_name(previous).cmp(&self.get_sortable_name(current))
                    == Ordering::Greater
                {
                    ctx.diagnostic(sort_vars_diagnostic(current.span));
                }
            }

            previous = Some(current);
        }
    }
}

impl SortVars {
    fn get_sortable_name<'a>(&self, decl: &VariableDeclarator<'a>) -> Cow<'a, str> {
        let BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind else {
            unreachable!();
        };

        if self.ignore_case {
            return ident.name.as_str().cow_to_ascii_lowercase();
        }

        Cow::Borrowed(ident.name.as_str()) // avoid string allocs in the default case
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var a=10, b=4, c='abc'", None),
        ("var a, b, c, d", None),
        ("var b; var a; var d;", None),
        ("var _a, a", None),
        ("var A, a", None),
        ("var A, b", None),
        ("var a, A;", Some(serde_json::json!([{ "ignoreCase": true }]))),
        ("var A, a;", Some(serde_json::json!([{ "ignoreCase": true }]))),
        ("var a, B, c;", Some(serde_json::json!([{ "ignoreCase": true }]))),
        ("var A, b, C;", Some(serde_json::json!([{ "ignoreCase": true }]))),
        ("var {a, b, c} = x;", Some(serde_json::json!([{ "ignoreCase": true }]))), // { "ecmaVersion": 6 },
        ("var {A, b, C} = x;", Some(serde_json::json!([{ "ignoreCase": true }]))), // { "ecmaVersion": 6 },
        ("var test = [1,2,3];", None), // { "ecmaVersion": 6 },
        ("var {a,b} = [1,2];", None),  // { "ecmaVersion": 6 },
        ("var [a, B, c] = [1, 2, 3];", Some(serde_json::json!([{ "ignoreCase": true }]))), // { "ecmaVersion": 6 },
        ("var [A, B, c] = [1, 2, 3];", Some(serde_json::json!([{ "ignoreCase": true }]))), // { "ecmaVersion": 6 },
        ("var [A, b, C] = [1, 2, 3];", Some(serde_json::json!([{ "ignoreCase": true }]))), // { "ecmaVersion": 6 },
        ("let {a, b, c} = x;", None),         // { "ecmaVersion": 6 },
        ("let [a, b, c] = [1, 2, 3];", None), // { "ecmaVersion": 6 },
        (
            r#"const {a, b, c} = {a: 1, b: true, c: "Moo"};"#,
            Some(serde_json::json!([{ "ignoreCase": true }])),
        ), // { "ecmaVersion": 6 },
        (
            r#"const [a, b, c] = [1, true, "Moo"];"#,
            Some(serde_json::json!([{ "ignoreCase": true }])),
        ), // { "ecmaVersion": 6 },
        (
            r#"const [c, a, b] = [1, true, "Moo"];"#,
            Some(serde_json::json!([{ "ignoreCase": true }])),
        ), // { "ecmaVersion": 6 },
        ("var {a, x: {b, c}} = {};", None),   // { "ecmaVersion": 6 },
        ("var {c, x: {a, c}} = {};", None),   // { "ecmaVersion": 6 },
        ("var {a, x: [b, c]} = {};", None),   // { "ecmaVersion": 6 },
        ("var [a, {b, c}] = {};", None),      // { "ecmaVersion": 6 },
        ("var [a, {x: {b, c}}] = {};", None), // { "ecmaVersion": 6 },
        ("var a = 42, {b, c } = {};", None),  // { "ecmaVersion": 6 },
        ("var b = 42, {a, c } = {};", None),  // { "ecmaVersion": 6 },
        ("var [b, {x: {a, c}}] = {};", None), // { "ecmaVersion": 6 },
        ("var [b, d, a, c] = {};", None),     // { "ecmaVersion": 6 },
        ("var e, [a, c, d] = {};", None),     // { "ecmaVersion": 6 },
        ("var a, [E, c, D] = [];", Some(serde_json::json!([{ "ignoreCase": true }]))), // { "ecmaVersion": 6 },
        ("var a, f, [e, c, d] = [1,2,3];", None), // { "ecmaVersion": 6 },
        (
            "export default class {
			    render () {
			        let {
			            b
			        } = this,
			            a,
			            c;
			    }
			}",
            None,
        ), // { "ecmaVersion": 6, "sourceType": "module" },
        ("var {} = 1, a", Some(serde_json::json!([{ "ignoreCase": true }]))), // { "ecmaVersion": 6 }
    ];

    let fail = vec![
        ("var b, a", None),
        ("var b , a", None),
        (
            "var b,
			    a;",
            None,
        ),
        ("var b=10, a=20;", None),
        ("var b=10, a=20, c=30;", None),
        ("var all=10, a = 1", None),
        ("var b, c, a, d", None),
        ("var c, d, a, b", None),
        ("var a, A;", None),
        ("var a, B;", None),
        ("var a, B, c;", None),
        ("var B, a;", Some(serde_json::json!([{ "ignoreCase": true }]))),
        ("var B, A, c;", Some(serde_json::json!([{ "ignoreCase": true }]))),
        ("var d, a, [b, c] = {};", Some(serde_json::json!([{ "ignoreCase": true }]))), // { "ecmaVersion": 6 },
        ("var d, a, [b, {x: {c, e}}] = {};", Some(serde_json::json!([{ "ignoreCase": true }]))), // { "ecmaVersion": 6 },
        ("var {} = 1, b, a", Some(serde_json::json!([{ "ignoreCase": true }]))), // { "ecmaVersion": 6 },
        ("var b=10, a=f();", None),
        ("var b=10, a=b;", None),
        ("var b = 0, a = `${b}`;", None),  // { "ecmaVersion": 6 },
        ("var b = 0, a = `${f()}`", None), // { "ecmaVersion": 6 },
        ("var b = 0, c = b, a;", None),
        ("var b = 0, c = 0, a = b + c;", None),
        ("var b = f(), c, d, a;", None),
        ("var b = `${f()}`, c, d, a;", None), // { "ecmaVersion": 6 },
        ("var c, a = b = 0", None),
    ];

    let _fix = vec![
        ("var b, a", "var a, b", None),
        ("var b , a", "var a , b", None),
        ("var b=10, a=20;", "var a=20, b=10;", None),
        ("var b=10, a=20, c=30;", "var a=20, b=10, c=30;", None),
        ("var all=10, a = 1", "var a = 1, all=10", None),
        ("var b, c, a, d", "var a, b, c, d", None),
        ("var c, d, a, b", "var a, b, c, d", None),
        ("var a, A;", "var A, a;", None),
        ("var a, B;", "var B, a;", None),
        ("var a, B, c;", "var B, a, c;", None),
        ("var B, a;", "var a, B;", Some(serde_json::json!([{ "ignoreCase": true }]))),
        ("var B, A, c;", "var A, B, c;", Some(serde_json::json!([{ "ignoreCase": true }]))),
        (
            "var d, a, [b, c] = {};",
            "var a, d, [b, c] = {};",
            Some(serde_json::json!([{ "ignoreCase": true }])),
        ),
        (
            "var d, a, [b, {x: {c, e}}] = {};",
            "var a, d, [b, {x: {c, e}}] = {};",
            Some(serde_json::json!([{ "ignoreCase": true }])),
        ),
        ("var {} = 1, b, a", "var {} = 1, a, b", Some(serde_json::json!([{ "ignoreCase": true }]))),
    ];

    Tester::new(SortVars::NAME, SortVars::PLUGIN, pass, fail).test_and_snapshot();
}
