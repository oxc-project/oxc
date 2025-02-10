use oxc_ast::{
    ast::{AssignmentTarget, Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_exports_assign(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected assignment to 'exports'.")
        .with_label(span)
        .with_help("Use 'module.exports' instead.")
}

fn is_exports(node: &AssignmentTarget, ctx: &LintContext) -> bool {
    let AssignmentTarget::AssignmentTargetIdentifier(id) = node else {
        return false;
    };
    id.is_global_reference_name("exports", ctx.symbols())
}

fn is_module_exports(expr: Option<&MemberExpression>, ctx: &LintContext) -> bool {
    let Some(mem_expr) = expr else {
        return false;
    };

    let Some(obj_id) = mem_expr.object().get_identifier_reference() else {
        return false;
    };

    mem_expr.static_property_name() == Some("exports")
        && obj_id.is_global_reference_name("module", ctx.symbols())
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
    fix
);

impl Rule for NoExportsAssign {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentExpression(assign_expr) = node.kind() else {
            return;
        };

        if !is_exports(&assign_expr.left, ctx) {
            return;
        }

        if let Expression::AssignmentExpression(assign_expr) = &assign_expr.right {
            if is_module_exports(assign_expr.left.as_member_expression(), ctx) {
                return;
            };
        }

        if let Some(AstKind::AssignmentExpression(assign_expr)) = ctx.nodes().parent_kind(node.id())
        {
            if is_module_exports(assign_expr.left.as_member_expression(), ctx) {
                return;
            }
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
        .with_node_plugin(true)
        .test_and_snapshot();
}
