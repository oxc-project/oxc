// Ported from https://github.com/eslint/eslint/tree/main/lib/rules/no-eval.js

use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-eval): eval can be harmful.")]
#[diagnostic(severity(warning))]
struct NoEvalDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoEval {
    /// Whether to allow references to the `eval` function as long as they are
    /// not called. For example, the following code is valid if this property is
    /// true:
    ///
    /// ```javascript
    /// const foo = eval;
    /// foo();
    ///
    /// (function(exec) {
    ///     exec();
    /// })(eval);
    /// ```
    ///
    /// The default value is `false`.
    pub allow_indirect: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Disallows referencing the 'eval' function.
    ///
    /// ### Why is this bad?
    /// Calling 'eval' is not supported in some secure contexts and can lead to
    /// vulnerabilities.
    ///
    /// ### Example
    /// ```javascript
    /// const someString = "console.log('pwned')"
    /// eval(someString);
    /// ```
    NoEval,
    correctness
);

impl Rule for NoEval {
    fn from_configuration(value: serde_json::Value) -> Self {
        let allow_indirect = value.get(0).map_or(false, |config| {
            config.get("allowIndirect").and_then(serde_json::Value::as_bool).unwrap_or(false)
        });

        Self { allow_indirect }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let kind = node.kind();

        if let AstKind::IdentifierReference(ident) = kind {
            if ident.name == "eval" {
                ctx.diagnostic(NoEvalDiagnostic(ident.span));
            }
            return;
        }

        let AstKind::MemberExpression(data) = kind else {
            return;
        };

        let Some((eval_span, "eval")) = data.static_property_info() else {
            return;
        };

        let mut object = Some(data.object().get_inner_expression());

        loop {
            let (new_object, name) = match object {
                Some(Expression::MemberExpression(member)) => {
                    (Some(member.object().get_inner_expression()), member.static_property_name())
                }
                Some(Expression::Identifier(ident)) => (None, Some(ident.name.as_str())),
                Some(Expression::ThisExpression(_)) => (None, Some("this")),
                None => break,
                _ => return,
            };
            object = new_object;

            match name {
                Some("this") => {
                    // let scope = ctx.scope(node);

                    // if scope.is_get_accessor()
                    // || scope.is_set_accessor()
                    // || scope.is_static_block()
                    // || scope.is_constructor()
                    // || (scope.is_function() && scope.strict_mode())
                    // || (scope.is_top() && ctx.source_type().is_module())
                    // {
                    return;
                    // }
                }
                Some("window" | "global" | "globalThis") => {}
                _ => return,
            };
        }

        ctx.diagnostic(NoEvalDiagnostic(eval_span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("Eval(foo)", None),
        ("setTimeout('foo')", None),
        ("setInterval('foo')", None),
        ("window.setTimeout('foo')", None),
        ("window.setInterval('foo')", None),
        // ("window.eval('foo')", None),
        // ("window.eval('foo')", None),
        ("window.noeval('foo')", None),
        // ("function foo() { var eval = 'foo'; window[eval]('foo') }", None),
        // ("global.eval('foo')", None),
        // ("global.eval('foo')", None),
        ("global.noeval('foo')", None),
        // ("function foo() { var eval = 'foo'; global[eval]('foo') }", None),
        // ("globalThis.eval('foo')", None),
        // ("globalThis.eval('foo')", None),
        // ("globalThis.eval('foo')", None),
        ("globalThis.noneval('foo')", None),
        // ("function foo() { var eval = 'foo'; globalThis[eval]('foo') }", None),
        ("this.noeval('foo');", None),
        ("function foo() { 'use strict'; this.eval('foo'); }", None),
        ("'use strict'; this.eval('foo');", None),
        ("this.eval('foo');", None),
        ("function foo() { this.eval('foo'); }", None),
        ("function foo() { this.eval('foo'); }", None),
        ("var obj = {foo: function() { this.eval('foo'); }}", None),
        ("var obj = {}; obj.foo = function() { this.eval('foo'); }", None),
        ("() => { this.eval('foo') }", None),
        ("function f() { 'use strict'; () => { this.eval('foo') } }", None),
        ("(function f() { 'use strict'; () => { this.eval('foo') } })", None),
        ("class A { foo() { this.eval(); } }", None),
        ("class A { static foo() { this.eval(); } }", None),
        ("class A { field = this.eval(); }", None),
        ("class A { field = () => this.eval(); }", None),
        ("class A { static { this.eval(); } }", None),
        // ("(0, eval)('foo')", Some(serde_json::json!([{ "allowIndirect": true }]))),
        // ("(0, window.eval)('foo')", Some(serde_json::json!([{ "allowIndirect": true }]))),
        // ("(0, window['eval'])('foo')", Some(serde_json::json!([{ "allowIndirect": true }]))),
        // ("var EVAL = eval; EVAL('foo')", Some(serde_json::json!([{ "allowIndirect": true }]))),
        // ("var EVAL = this.eval; EVAL('foo')", Some(serde_json::json!([{ "allowIndirect": true }]))),
        // (
        //     "(function(exe){ exe('foo') })(eval);",
        //     Some(serde_json::json!([{ "allowIndirect": true }])),
        // ),
        // ("window.eval('foo')", Some(serde_json::json!([{ "allowIndirect": true }]))),
        // ("window.window.eval('foo')", Some(serde_json::json!([{ "allowIndirect": true }]))),
        // ("window.window['eval']('foo')", Some(serde_json::json!([{ "allowIndirect": true }]))),
        // ("global.eval('foo')", Some(serde_json::json!([{ "allowIndirect": true }]))),
        // ("global.global.eval('foo')", Some(serde_json::json!([{ "allowIndirect": true }]))),
        // ("this.eval('foo')", Some(serde_json::json!([{ "allowIndirect": true }]))),
        // (
        //     "function foo() { this.eval('foo') }",
        //     Some(serde_json::json!([{ "allowIndirect": true }])),
        // ),
        // ("(0, globalThis.eval)('foo')", Some(serde_json::json!([{ "allowIndirect": true }]))),
        // ("(0, globalThis['eval'])('foo')", Some(serde_json::json!([{ "allowIndirect": true }]))),
        // (
        //     "var EVAL = globalThis.eval; EVAL('foo')",
        //     Some(serde_json::json!([{ "allowIndirect": true }])),
        // ),
        // (
        //     "function foo() { globalThis.eval('foo') }",
        //     Some(serde_json::json!([{ "allowIndirect": true }])),
        // ),
        // (
        //     "globalThis.globalThis.eval('foo');",
        //     Some(serde_json::json!([{ "allowIndirect": true }])),
        // ),
        // ("eval?.('foo')", Some(serde_json::json!([{ "allowIndirect": true }]))),
        // ("window?.eval('foo')", Some(serde_json::json!([{ "allowIndirect": true }]))),
        // ("(window?.eval)('foo')", Some(serde_json::json!([{ "allowIndirect": true }]))),
    ];

    let fail = vec![
        ("eval(foo)", None),
        ("eval('foo')", None),
        ("function foo(eval) { eval('foo') }", None),
        // ("eval(foo)", Some(serde_json::json!([{ "allowIndirect": true }]))),
        // ("eval('foo')", Some(serde_json::json!([{ "allowIndirect": true }]))),
        // (
        //     "function foo(eval) { eval('foo') }",
        //     Some(serde_json::json!([{ "allowIndirect": true }])),
        // ),
        ("(0, eval)('foo')", None),
        ("(0, window.eval)('foo')", None),
        ("(0, window['eval'])('foo')", None),
        // ("var EVAL = eval; EVAL('foo')", None),
        // ("var EVAL = this.eval; EVAL('foo')", None),
        // ("'use strict'; var EVAL = this.eval; EVAL('foo')", None),
        // ("() => { this.eval('foo'); }", None),
        // ("() => { 'use strict'; this.eval('foo'); }", None),
        // ("'use strict'; () => { this.eval('foo'); }", None),
        // ("() => { 'use strict'; () => { this.eval('foo'); } }", None),
        // ("(function(exe){ exe('foo') })(eval);", None),
        ("window.eval('foo')", None),
        ("window.window.eval('foo')", None),
        ("window.window['eval']('foo')", None),
        ("global.eval('foo')", None),
        ("global.global.eval('foo')", None),
        ("global.global[`eval`]('foo')", None),
        // ("this.eval('foo')", None),
        // ("'use strict'; this.eval('foo')", None),
        // ("function foo() { this.eval('foo') }", None),
        ("var EVAL = globalThis.eval; EVAL('foo')", None),
        ("globalThis.eval('foo')", None),
        ("globalThis.globalThis.eval('foo')", None),
        ("globalThis.globalThis['eval']('foo')", None),
        ("(0, globalThis.eval)('foo')", None),
        ("(0, globalThis['eval'])('foo')", None),
        ("window?.eval('foo')", None),
        ("(window?.eval)('foo')", None),
        // ("(window?.window).eval('foo')", None),
        // ("class C { [this.eval('foo')] }", None),
        // ("'use strict'; class C { [this.eval('foo')] }", None),
        // ("class A { static {} [this.eval()]; }", None),
        // ("function foo() { 'use strict'; this.eval(); }", None),
    ];

    Tester::new(NoEval::NAME, pass, fail).test_and_snapshot();
}
