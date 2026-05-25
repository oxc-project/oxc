use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    rules::shared::no_negated_condition::{
        DOCUMENTATION, run_on_conditional_expression, run_on_if_statement,
    },
};

#[derive(Debug, Default, Clone)]
pub struct NoNegatedCondition;

declare_oxc_lint!(
    NoNegatedCondition,
    eslint,
    pedantic,
    pending,
    docs = DOCUMENTATION,
    version = "0.0.18",
);

impl Rule for NoNegatedCondition {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::IfStatement(if_stmt) => {
                run_on_if_statement(if_stmt, ctx);
            }
            AstKind::ConditionalExpression(conditional_expr) => {
                run_on_conditional_expression(conditional_expr, ctx);
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "if (a) {}",
        "if (a) {} else {}",
        "if (!a) {}",
        "if (!a) {} else if (b) {}",
        "if (!a) {} else if (b) {} else {}",
        "if (a == b) {}",
        "if (a == b) {} else {}",
        "if (a != b) {}",
        "if (a != b) {} else if (b) {}",
        "if (a != b) {} else if (b) {} else {}",
        "if (a !== b) {}",
        "if (a === b) {} else {}",
        "a ? b : c",
    ];

    let fail = vec![
        "if (!a) {;} else {;}",
        "if (a != b) {;} else {;}",
        "if (a !== b) {;} else {;}",
        "!a ? b : c",
        "a != b ? c : d",
        "a !== b ? c : d",
    ];

    Tester::new(NoNegatedCondition::NAME, NoNegatedCondition::PLUGIN, pass, fail)
        .test_and_snapshot();
}
