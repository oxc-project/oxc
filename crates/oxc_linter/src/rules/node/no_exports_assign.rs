use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{is_global_exports_assignment_target, is_global_module_exports},
};

fn no_exports_assign(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected assignment to 'exports'.")
        .with_label(span)
        .with_help("Use 'module.exports' instead.")
}

#[derive(Debug, Default, Clone)]
pub struct NoExportsAssign;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows assignment to `exports`.
    ///
    /// ### Why is this bad?
    ///
    /// Directly using `exports = {}` can lead to confusion and potential bugs
    /// because it reassigns the `exports` object, which may break module
    /// exports. It is more predictable and clearer to use `module.exports`
    /// directly or in conjunction with `exports`.
    ///
    /// This rule is aimed at disallowing `exports = {}`, but allows
    /// `module.exports = exports = {}` to avoid conflict with `n/exports-style`
    /// rule's `allowBatchAssign` option.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// exports = {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// module.exports.foo = 1
    /// exports.bar = 2
    /// module.exports = {}
    ///
    /// // allows `exports = {}` if along with `module.exports =`
    /// module.exports = exports = {}
    /// exports = module.exports = {}
    /// ```
    NoExportsAssign,
    node,
    style,
    fix,
    version = "0.9.3",
    short_description = "Disallows assignment to `exports`.",
);

impl Rule for NoExportsAssign {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentExpression(assign_expr) = node.kind() else {
            return;
        };

        if !is_global_exports_assignment_target(&assign_expr.left, ctx) {
            return;
        }

        if let Expression::AssignmentExpression(assign_expr) = &assign_expr.right
            && assign_expr
                .left
                .as_member_expression()
                .is_some_and(|member| is_global_module_exports(member, ctx))
        {
            return;
        }

        if let AstKind::AssignmentExpression(assign_expr) = ctx.nodes().parent_kind(node.id())
            && assign_expr
                .left
                .as_member_expression()
                .is_some_and(|member| is_global_module_exports(member, ctx))
        {
            return;
        }

        ctx.diagnostic_with_fix(no_exports_assign(assign_expr.left.span()), |fixer| {
            fixer.replace(assign_expr.left.span(), "module.exports")
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "module.exports.foo = 1",
        "exports.bar = 1",
        "module.exports = exports = {}",
        "exports = module.exports = {}",
        "function f(exports) { exports = {} }",
        "let exports; exports = {}",
    ];

    let fail = vec!["exports = {}"];

    let fix = vec![("exports = {}", "module.exports = {}")];

    Tester::new(NoExportsAssign::NAME, NoExportsAssign::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
