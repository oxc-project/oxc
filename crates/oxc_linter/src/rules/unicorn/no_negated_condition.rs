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
        // Multi-line return/throw + comments (unicorn 13–16)
        "function a() {\nreturn ! // comment\na ? b : c;\n}",
        "function a() {\nreturn (! // ReturnStatement argument is parenthesized\na ? b : c);\n}",
        "function a() {\nreturn (\n! // UnaryExpression argument is parenthesized\na) ? b : c;\n}",
        "function a() {\nthrow ! // comment\na ? b : c;\n}",
        r"!a ? b : c ? d : e",
        r"!a ? b : (( c ? d : e ))",
        // ASI after removing `!` (unicorn 19–22)
        "a\n![] ? b : c",
        "a\n!+b ? c : d",
        "a\n!(b) ? c : d",
        "a\n!b ? c : d",
        // Multi-line non-block if (unicorn 23)
        "if (!a)\nb()\nelse\nc()",
        r"if(!a) b(); else c()",
        // Non-block `else return` (unicorn 25)
        "function fn() {\nif(!a) b(); else return\n}",
        r"if(!a) {b()} else {c()}",
        r"if(!!a) b(); else c();",
        r"(!!a) ? b() : c();",
        // Inequality whose left is still `!` (unicorn 29) — two diagnostics
        "function fn() {\nreturn!a !== b ? c : d\nreturn((!((a)) != b)) ? c : d\n}",
        // Report on inner `else if`, not outer (unicorn 30)
        "if (!a) {\nb();\n} else if (!c) {\nd();\n} else {\ne();\n}",
    ];

    let fix = vec![
        (r"if (!a) {;} else {;}", r"if (a) {;} else {;}", None),
        (r"if (a != b) {;} else {;}", r"if (a == b) {;} else {;}", None),
        (r"if (a !== b) {;} else {;}", r"if (a === b) {;} else {;}", None),
        (r"!a ? b : c", r"a ? c : b", None),
        (r"a != b ? c : d", r"a == b ? d : c", None),
        (r"a !== b ? c : d", r"a === b ? d : c", None),
        (r"(( !a )) ? b : c", r"(( a )) ? c : b", None),
        (r"!(( a )) ? b : c", r"(( a )) ? c : b", None),
        (r"if(!(( a ))) b(); else c();", r"if( a ) {c();} else {b();}", None),
        (r"if((( !a ))) b(); else c();", r"if((( a ))) {c();} else {b();}", None),
        (r"function a() {return!a ? b : c}", r"function a() {return a ? c : b}", None),
        (r"function a() {return!(( a )) ? b : c}", r"function a() {return (( a )) ? c : b}", None),
        (
            "function a() {\nreturn ! // comment\na ? b : c;\n}",
            // One space after `(` (unicorn snapshot has two; we preserve a single space after deleting `!`).
            "function a() {\nreturn ( // comment\na ? c : b);\n}",
            None,
        ),
        (
            "function a() {\nreturn (! // ReturnStatement argument is parenthesized\na ? b : c);\n}",
            "function a() {\nreturn ( // ReturnStatement argument is parenthesized\na ? c : b);\n}",
            None,
        ),
        (
            "function a() {\nreturn (\n! // UnaryExpression argument is parenthesized\na) ? b : c;\n}",
            "function a() {\nreturn (\n // UnaryExpression argument is parenthesized\na) ? c : b;\n}",
            None,
        ),
        (
            "function a() {\nthrow ! // comment\na ? b : c;\n}",
            "function a() {\nthrow ( // comment\na ? c : b);\n}",
            None,
        ),
        (r"!a ? b : c ? d : e", r"a ? c ? d : e : b", None),
        (r"!a ? b : (( c ? d : e ))", r"a ? (( c ? d : e )) : b", None),
        ("a\n![] ? b : c", "a\n;[] ? c : b", None),
        ("a\n!+b ? c : d", "a\n;+b ? d : c", None),
        ("a\n!(b) ? c : d", "a\n;(b) ? d : c", None),
        ("a\n!b ? c : d", "a\nb ? d : c", None),
        ("if (!a)\nb()\nelse\nc()", "if (a)\n{c()}\nelse\n{b()}", None),
        (r"if(!a) b(); else c()", r"if(a) {c()} else {b();}", None),
        (
            "function fn() {\nif(!a) b(); else return\n}",
            "function fn() {\nif(a) {return} else {b();}\n}",
            None,
        ),
        (r"if(!a) {b()} else {c()}", r"if(a) {c()} else {b()}", None),
        (r"if(!!a) b(); else c();", r"if(!a) {c();} else {b();}", None),
        (r"(!!a) ? b() : c();", r"(!a) ? c() : b();", None),
        // Two separate fixes applied in one file (fixer applies all non-overlapping diagnostics)
        (
            "function fn() {\nreturn!a !== b ? c : d\nreturn((!((a)) != b)) ? c : d\n}",
            "function fn() {\nreturn!a === b ? d : c\nreturn((!((a)) == b)) ? d : c\n}",
            None,
        ),
        (
            "if (!a) {\nb();\n} else if (!c) {\nd();\n} else {\ne();\n}",
            "if (!a) {\nb();\n} else if (c) {\ne();\n} else {\nd();\n}",
            None,
        ),
    ];

    Tester::new(NoNegatedCondition::NAME, NoNegatedCondition::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
