use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, Function, ThisExpression},
};
use oxc_ast_visit::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::Span;

use crate::{AstNode, ast_util::is_method_call, context::LintContext, rule::Rule};

fn no_extra_bind_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The function binding is unnecessary.")
        .with_label(span)
        .with_help("Remove the `.bind` call.")
}

#[derive(Debug, Default, Clone)]
pub struct NoExtraBind;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unnecessary calls to .bind()
    ///
    /// ### Why is this bad?
    ///
    /// This rule is aimed at avoiding the unnecessary use of bind()
    /// and as such will warn whenever an immediately-invoked function expression (IIFE) is using bind()
    /// and doesn’t have an appropriate this value.
    /// This rule won’t flag usage of bind() that includes function argument binding.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const x = function () {
    /// foo();
    /// }.bind(bar);
    ///
    /// const z = (() => {
    ///     this.foo();
    /// }).bind(this);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const x = function () {
    ///     this.foo();
    /// }.bind(bar);
    /// const y = function (a) {
    ///     return a + 1;
    /// }.bind(foo, bar);
    /// ```
    NoExtraBind,
    eslint,
    suspicious,
    pending
);

impl Rule for NoExtraBind {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        if !is_method_call(call_expr, None, Some(&["bind"]), Some(1), Some(1)) {
            return;
        }
        if matches!(call_expr.arguments.first(), Some(Argument::SpreadElement(_))) {
            return;
        }

        let expr = call_expr.callee.get_inner_expression();

        let Some(member_expr) = (match expr {
            Expression::ChainExpression(chain_expr) => chain_expr.expression.as_member_expression(),
            _ => expr.as_member_expression(),
        }) else {
            return;
        };
        let Some((span, _)) = member_expr.static_property_info() else {
            return;
        };
        let obj = member_expr.object().get_inner_expression();
        match obj {
            Expression::FunctionExpression(func_expr) => {
                let Some(body) = &func_expr.body else {
                    return;
                };
                let mut finder = ThisFinder { found: false };
                finder.visit_function_body(body);
                // don't use this expression
                if !finder.found {
                    ctx.diagnostic(no_extra_bind_diagnostic(span));
                }
            }
            Expression::ArrowFunctionExpression(_) => {
                ctx.diagnostic(no_extra_bind_diagnostic(span));
            }
            _ => {}
        }
    }
}

struct ThisFinder {
    found: bool,
}

impl<'a> Visit<'a> for ThisFinder {
    fn visit_this_expression(&mut self, _expr: &ThisExpression) {
        self.found = true;
    }

    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var a = function(b) { return b }.bind(c, d)",
        "var a = function(b) { return b }.bind(...c)", // { "ecmaVersion": 6 },
        "var a = function() { this.b }()",
        "var a = function() { this.b }.foo()",
        "var a = f.bind(a)",
        "var a = function() { return this.b }.bind(c)",
        "var a = (() => { return b }).bind(c, d)", // { "ecmaVersion": 6 },
        "(function() { (function() { this.b }.bind(this)) }.bind(c))",
        "var a = function() { return 1; }[bind](b)",
        "var a = function() { return 1; }[`bi${n}d`](b)", // { "ecmaVersion": 6 },
        "var a = function() { return () => this; }.bind(b)", // { "ecmaVersion": 6 }
        "var a = function() { function v() { this } }.bind()",
    ];

    let fail = vec![
        "var a = function() { return 1; }.bind(b)",
        "var a = function() { return 1; }['bind'](b)",
        "var a = function() { return 1; }[`bind`](b)", // { "ecmaVersion": 6 },
        "var a = (() => { return 1; }).bind(b)",       // { "ecmaVersion": 6 },
        "var a = (() => { return this; }).bind(b)",    // { "ecmaVersion": 6 },
        "var a = function() { (function(){ this.c }) }.bind(b)",
        "var a = function() { function c(){ this.d } }.bind(b)",
        "var a = function() { return 1; }.bind(this)",
        "var a = function() { (function(){ (function(){ this.d }.bind(c)) }) }.bind(b)",
        "var a = (function() { return 1; }).bind(this)",
        "var a = (function() { return 1; }.bind)(this)",
        "var a = function() {}.bind(b++)",
        "var a = function() {}.bind(b())",
        "var a = function() {}.bind(b.c)",
        "var a = function() {}/**/.bind(b)",
        "var a = function() {}/**/['bind'](b)",
        "var a = function() {}//comment
			.bind(b)",
        "var a = function() {}./**/bind(b)",
        "var a = function() {}[/**/'bind'](b)",
        "var a = function() {}.//
			bind(b)",
        "var a = function() {}.bind/**/(b)",
        "var a = function() {}.bind(
			/**/b)",
        "var a = function() {}.bind(b/**/)",
        "var a = function() {}.bind(b//
			)",
        "var a = function() {}.bind(b
			/**/)",
        "var a = function() {}.bind(b)/**/",
        "var a = function() { return 1; }.bind?.(b)", // { "ecmaVersion": 2020 },
        "var a = function() { return 1; }?.bind(b)",  // { "ecmaVersion": 2020 },
        "var a = (function() { return 1; }?.bind)(b)", // { "ecmaVersion": 2020 },
        "var a = function() { return 1; }['bind']?.(b)", // { "ecmaVersion": 2020 },
        "var a = function() { return 1; }?.['bind'](b)", // { "ecmaVersion": 2020 },
        "var a = (function() { return 1; }?.['bind'])(b)", // { "ecmaVersion": 2020 }
        "var a = function() { function v() { this } }.bind(a)",
    ];
    // pending
    // let fix = vec![
    //     ("var a = function() { return 1; }.bind(b)", "var a = function() { return 1; }", None),
    //     ("var a = function() { return 1; }['bind'](b)", "var a = function() { return 1; }", None),
    //     ("var a = function() { return 1; }[`bind`](b)", "var a = function() { return 1; }", None),
    //     ("var a = (() => { return 1; }).bind(b)", "var a = (() => { return 1; })", None),
    //     ("var a = (() => { return this; }).bind(b)", "var a = (() => { return this; })", None),
    //     (
    //         "var a = function() { (function(){ this.c }) }.bind(b)",
    //         "var a = function() { (function(){ this.c }) }",
    //         None,
    //     ),
    //     (
    //         "var a = function() { function c(){ this.d } }.bind(b)",
    //         "var a = function() { function c(){ this.d } }",
    //         None,
    //     ),
    //     ("var a = function() { return 1; }.bind(this)", "var a = function() { return 1; }", None),
    //     (
    //         "var a = function() { (function(){ (function(){ this.d }.bind(c)) }) }.bind(b)",
    //         "var a = function() { (function(){ (function(){ this.d }.bind(c)) }) }",
    //         None,
    //     ),
    //     (
    //         "var a = (function() { return 1; }).bind(this)",
    //         "var a = (function() { return 1; })",
    //         None,
    //     ),
    //     (
    //         "var a = (function() { return 1; }.bind)(this)",
    //         "var a = (function() { return 1; })",
    //         None,
    //     ),
    //     ("var a = function() {}/**/.bind(b)", "var a = function() {}/**/", None),
    //     ("var a = function() {}/**/['bind'](b)", "var a = function() {}/**/", None),
    //     (
    //         "var a = function() {}//comment
    // 		.bind(b)",
    //         "var a = function() {}//comment
    // 		",
    //         None,
    //     ),
    //     ("var a = function() {}.bind(b)/**/", "var a = function() {}/**/", None),
    //     ("var a = function() { return 1; }.bind?.(b)", "var a = function() { return 1; }", None),
    //     ("var a = function() { return 1; }?.bind(b)", "var a = function() { return 1; }", None),
    //     ("var a = (function() { return 1; }?.bind)(b)", "var a = (function() { return 1; })", None),
    //     ("var a = function() { return 1; }['bind']?.(b)", "var a = function() { return 1; }", None),
    //     ("var a = function() { return 1; }?.['bind'](b)", "var a = function() { return 1; }", None),
    //     (
    //         "var a = (function() { return 1; }?.['bind'])(b)",
    //         "var a = (function() { return 1; })",
    //         None,
    //     ),
    // ];
    Tester::new(NoExtraBind::NAME, NoExtraBind::PLUGIN, pass, fail).test_and_snapshot();
}
