use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeId;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_alert_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`alert`, `confirm` and `prompt` functions are not allowed")
        .with_help("Use a custom UI instead")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAlert;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow the use of alert, confirm, and prompt
    ///
    /// ### Why is this bad?
    ///
    /// JavaScript’s alert, confirm, and prompt functions are widely considered to be obtrusive as UI elements and should be replaced by a more appropriate custom UI implementation.
    /// Furthermore, alert is often used while debugging code, which should be removed before deployment to production.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// alert("here!");
    ///
    /// confirm("Are you sure?");
    ///
    /// prompt("What's your name?", "John Doe");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// customAlert("Something happened!");
    ///
    /// customConfirm("Are you sure?");
    ///
    /// customPrompt("Who are you?");
    ///
    /// function foo() {
    ///     var alert = myCustomLib.customAlert;
    ///     alert();
    /// }
    /// ```
    NoAlert,
    eslint,
    restriction,
);

const GLOBAL_THIS: &str = "globalThis";
const GLOBAL_WINDOW: &str = "window";

fn is_prohibited_identifier(value: &str) -> bool {
    matches!(value, "alert" | "confirm" | "prompt")
}

fn is_global_this_ref_or_global_window<'a>(
    scope_id: ScopeId,
    ctx: &LintContext<'a>,
    expr: &Expression<'a>,
) -> bool {
    if let Expression::ThisExpression(_) = expr {
        if ctx.scopes().get_flags(scope_id).is_top() {
            return true;
        }
    }

    let Some(ident) = expr.get_identifier_reference() else {
        return false;
    };

    if ctx.semantic().is_reference_to_global_variable(ident)
        && (expr.is_specific_id(GLOBAL_WINDOW) || (expr.is_specific_id(GLOBAL_THIS)))
    {
        return !is_shadowed(scope_id, ident.name.as_str(), ctx);
    }

    false
}

fn is_shadowed<'a>(scope_id: ScopeId, name: &'a str, ctx: &LintContext<'a>) -> bool {
    ctx.scopes().find_binding(scope_id, name).is_some()
}

impl Rule for NoAlert {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let scope_id = node.scope_id();
        let callee = &call_expr.callee;

        if let Expression::Identifier(ident) = callee {
            let name = ident.name.as_str();
            if !is_shadowed(scope_id, name, ctx) && is_prohibited_identifier(name) {
                return ctx.diagnostic(no_alert_diagnostic(ident.span));
            }

            return;
        }

        let Some(member_expr) = callee.get_member_expr() else { return };
        if !is_global_this_ref_or_global_window(scope_id, ctx, member_expr.object()) {
            return;
        }

        let Some(property_name) = member_expr.static_property_name() else {
            return;
        };
        if is_prohibited_identifier(property_name) {
            ctx.diagnostic(no_alert_diagnostic(member_expr.span()));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "a[o.k](1)",
        "foo.alert(foo)",
        "foo.confirm(foo)",
        "foo.prompt(foo)",
        "function alert() {} alert();",
        "var alert = function() {}; alert();",
        "function foo() { var alert = bar; alert(); }",
        "function foo(alert) { alert(); }",
        "var alert = function() {}; function test() { alert(); }",
        "function foo() { var alert = function() {}; function test() { alert(); } }",
        "function confirm() {} confirm();",
        "function prompt() {} prompt();",
        "window[alert]();",
        "function foo() { this.alert(); }",
        "function foo() { var window = bar; window.alert(); }",
        // "globalThis.alert();",
        // "globalThis['alert']();",                    // { "ecmaVersion": 6 },
        // "globalThis.alert();",                       // { "ecmaVersion": 2017 },
        "var globalThis = foo; globalThis.alert();", // { "ecmaVersion": 2020 },
        "function foo() { var globalThis = foo; globalThis.alert(); }", // { "ecmaVersion": 2020 }
    ];

    let fail = vec![
        "alert(foo)",
        "window.alert(foo)",
        "window['alert'](foo)",
        "confirm(foo)",
        "window.confirm(foo)",
        "window['confirm'](foo)",
        "prompt(foo)",
        "window.prompt(foo)",
        "window['prompt'](foo)",
        "function alert() {} window.alert(foo)",
        "var alert = function() {};
        	window.alert(foo)",
        "function foo(alert) { window.alert(); }",
        "function foo() { alert(); }",
        "function foo() { var alert = function() {}; }
        	alert();",
        "this.alert(foo)",
        "this['alert'](foo)",
        "function foo() { var window = bar; window.alert(); }
        	window.alert();",
        "globalThis['alert'](foo)", // { "ecmaVersion": 2020 },
        "globalThis.alert();",      // { "ecmaVersion": 2020 },
        "function foo() { var globalThis = bar; globalThis.alert(); }
        	globalThis.alert();", // { "ecmaVersion": 2020 },
        "window?.alert(foo)",       // { "ecmaVersion": 2020 },
        "(window?.alert)(foo)",     // { "ecmaVersion": 2020 }
    ];

    Tester::new(NoAlert::NAME, NoAlert::PLUGIN, pass, fail).test_and_snapshot();
}
