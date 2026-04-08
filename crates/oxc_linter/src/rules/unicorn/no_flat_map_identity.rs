use oxc_ast::AstKind;
use oxc_ast::ast::{BindingPattern, Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_flat_map_identity_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`.flatMap(x => x)` is equivalent to `.flat()`.")
        .with_help("Replace `.flatMap(x => x)` with `.flat()` for clarity.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoFlatMapIdentity;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows using `.flatMap()` with an identity function.
    ///
    /// ### Why is this bad?
    ///
    /// `.flatMap(x => x)` is equivalent to `.flat()` but less readable.
    /// Using `.flat()` directly makes the intent clearer.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// arr.flatMap(x => x);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// arr.flat();
    /// arr.flatMap(x => x * 2);
    /// ```
    NoFlatMapIdentity,
    unicorn,
    style,
    pending
);

impl Rule for NoFlatMapIdentity {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call) = node.kind() else {
            return;
        };

        let Expression::StaticMemberExpression(member) = &call.callee else {
            return;
        };

        if member.property.name.as_str() != "flatMap" {
            return;
        }

        if call.arguments.len() != 1 {
            return;
        }

        let Some(arg_expr) = call.arguments[0].as_expression() else {
            return;
        };

        if is_identity_arrow(arg_expr) {
            ctx.diagnostic(no_flat_map_identity_diagnostic(call.span));
        }
    }
}

fn is_identity_arrow(expr: &Expression) -> bool {
    let Expression::ArrowFunctionExpression(arrow) = expr else {
        return false;
    };

    // Single parameter, no rest
    if arrow.params.items.len() != 1 || arrow.params.rest.is_some() {
        return false;
    }

    let BindingPattern::BindingIdentifier(param_id) = &arrow.params.items[0].pattern else {
        return false;
    };

    // Expression body: (x) => x
    if !arrow.expression {
        return false;
    }

    if let Some(stmt) = arrow.body.statements.first() {
        if let oxc_ast::ast::Statement::ExpressionStatement(expr_stmt) = stmt {
            if let Expression::Identifier(ident) = &expr_stmt.expression {
                return ident.name == param_id.name;
            }
        }
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "arr.flat();",
        "arr.flatMap(x => x * 2);",
        "arr.flatMap(x => [x, x]);",
        "arr.flatMap((x, i) => x);",
        "arr.map(x => x);",
    ];

    let fail = vec!["arr.flatMap(x => x);"];

    Tester::new(NoFlatMapIdentity::NAME, NoFlatMapIdentity::PLUGIN, pass, fail).test_and_snapshot();
}
