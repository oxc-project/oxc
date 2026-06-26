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
    unicorn,
    pedantic,
    fix,
    docs = DOCUMENTATION,
    version = "0.0.18",
    short_description = "Disallow negated conditions.",
);

impl Rule for NoNegatedCondition {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::IfStatement(if_stmt) => {
                run_on_if_statement(if_stmt, ctx);
            }
            AstKind::ConditionalExpression(conditional_expr) => {
                run_on_conditional_expression(node, conditional_expr, ctx);
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"if (a) {}",
        r"if (a) {} else {}",
        r"if (!a) {}",
        r"if (!a) {} else if (b) {}",
        r"if (!a) {} else if (b) {} else {}",
        r"if (a == b) {}",
        r"if (a == b) {} else {}",
        r"if (a != b) {}",
        r"if (a != b) {} else if (b) {}",
        r"if (a != b) {} else if (b) {} else {}",
        r"if (a !== b) {}",
        r"if (a === b) {} else {}",
        r"a ? b : c",
    ];

    let fail = vec![
        r"if (!a) {;} else {;}",
        r"if (a != b) {;} else {;}",
        r"if (a !== b) {;} else {;}",
        r"!a ? b : c",
        r"a != b ? c : d",
        r"a !== b ? c : d",
        r"(( !a )) ? b : c",
        r"!(( a )) ? b : c",
        r"if(!(( a ))) b(); else c();",
        r"if((( !a ))) b(); else c();",
        r"function a() {return!a ? b : c}",
        r"function a() {return!(( a )) ? b : c}",
        r"!a ? b : c ? d : e",
        r"!a ? b : (( c ? d : e ))",
        r"if(!a) b(); else c()",
        r"if(!a) {b()} else {c()}",
        r"if(!!a) b(); else c();",
        r"(!!a) ? b() : c();",
    ];

    let fix = vec![
        (r"if (!a) {;} else {;}", r"if (a) {;} else {;}"),
        (r"if (a != b) {;} else {;}", r"if (a == b) {;} else {;}"),
        (r"if (a !== b) {;} else {;}", r"if (a === b) {;} else {;}"),
        (r"!a ? b : c", r"a ? c : b"),
        (r"a != b ? c : d", r"a == b ? d : c"),
        (r"a !== b ? c : d", r"a === b ? d : c"),
        (r"(( !a )) ? b : c", r"(( a )) ? c : b"),
        (r"!(( a )) ? b : c", r"(( a )) ? c : b"),
        (r"if(!(( a ))) b(); else c();", r"if(a) {c();} else {b();}"),
        (r"if((( !a ))) b(); else c();", r"if((( a ))) {c();} else {b();}"),
        (r"function a() {return!a ? b : c}", r"function a() {return a ? c : b}"),
        (r"function a() {return!(( a )) ? b : c}", r"function a() {return (( a )) ? c : b}"),
        (r"!a ? b : c ? d : e", r"a ? c ? d : e : b"),
        (r"!a ? b : (( c ? d : e ))", r"a ? (( c ? d : e )) : b"),
        (r"if(!a) b(); else c()", r"if(a) {c()} else {b();}"),
        (r"if(!a) {b()} else {c()}", r"if(a) {c()} else {b()}"),
        (r"if(!!a) b(); else c();", r"if(!a) {c();} else {b();}"),
        (r"(!!a) ? b() : c();", r"(!a) ? c() : b();"),
        (
            "a
![] ? b : c",
            "a
;[] ? c : b",
        ),
        (
            "a
!(b) ? c : d",
            "a
;(b) ? d : c",
        ),
    ];

    Tester::new(NoNegatedCondition::NAME, NoNegatedCondition::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
