use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{cmp::ContentEq, GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_duplicate_case_diagnostic(first: Span, second: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Duplicate case label")
        .with_help("Remove the duplicated case")
        .with_labels([first.label("This label here"), second.label("is duplicated here")])
}

#[derive(Debug, Default, Clone)]
pub struct NoDuplicateCase;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow duplicate case labels
    ///
    /// ### Why is this bad?
    ///
    /// If a switch statement has duplicate test expressions in case clauses,
    /// it is likely that a programmer copied a case clause but forgot to change the test expression.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var a = 1, one = 1;
    /// switch (a) {
    ///     case 1:
    ///         break;
    ///     case 2:
    ///         break;
    ///     case 1: // duplicate test expression
    ///         break;
    ///     default:
    ///         break;
    /// }
    ///
    /// switch (a) {
    ///     case one:
    ///         break;
    ///     case 2:
    ///         break;
    ///     case one: // duplicate test expression
    ///         break;
    ///     default:
    ///         break;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var a = 1,
    ///     one = 1
    /// switch (a) {
    ///     case 1:
    ///         break
    ///     case 2:
    ///         break
    ///     default:
    ///         break
    /// }
    ///
    /// switch (a) {
    ///     case '1':
    ///         break
    ///     case '2':
    ///         break
    ///     default:
    ///         break
    /// }
    /// ```
    NoDuplicateCase,
    eslint,
    correctness
);

impl Rule for NoDuplicateCase {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let Some(ss) = node.kind().as_switch_statement() else { return };
        let mut previous_tests: Vec<&Expression<'_>> = vec![];
        for test in ss.cases.iter().filter_map(|c| c.test.as_ref()) {
            let test = test.without_parentheses();
            if let Some(prev) = previous_tests.iter().find(|t| t.content_eq(test)) {
                ctx.diagnostic(no_duplicate_case_diagnostic(prev.span(), test.span()));
            } else {
                previous_tests.push(test);
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var a = 1; switch (a) {case 1: break; case 2: break; default: break;}",
        "var a = 1; switch (a) {case 1: break; case '1': break; default: break;}",
        "var a = 1; switch (a) {case 1: break; case true: break; default: break;}",
        "var a = 1; switch (a) {default: break;}",
        "var a = 1, p = {p: {p1: 1, p2: 1}}; switch (a) {case p.p.p1: break; case p.p.p2: break; default: break;}",
        "var a = 1, f = function(b) { return b ? { p1: 1 } : { p1: 2 }; }; switch (a) {case f(true).p1: break; case f(true, false).p1: break; default: break;}",
        "var a = 1, f = function(s) { return { p1: s } }; switch (a) {case f(a + 1).p1: break; case f(a + 2).p1: break; default: break;}",
        "var a = 1, f = function(s) { return { p1: s } }; switch (a) {case f(a == 1 ? 2 : 3).p1: break; case f(a === 1 ? 2 : 3).p1: break; default: break;}",
        "var a = 1, f1 = function() { return { p1: 1 } }, f2 = function() { return { p1: 2 } }; switch (a) {case f1().p1: break; case f2().p1: break; default: break;}",
        "var a = [1,2]; switch(a.toString()){case ([1,2]).toString():break; case ([1]).toString():break; default:break;}",
        "switch(a) { case a: break; } switch(a) { case a: break; }",
        "switch(a) { case toString: break; }",
    ];

    let fail = vec![
        "var a = 1; switch (a) {case 1: break; case 1: break; case 2: break; default: break;}",
        "var a = 1; switch (a) {case 1: break; case (1): break; case 2: break; default: break;}",
        "var a = '1'; switch (a) {case '1': break; case '1': break; case '2': break; default: break;}",
        "var a = 1, one = 1; switch (a) {case one: break; case one: break; case 2: break; default: break;}",
        "var a = 1, p = {p: {p1: 1, p2: 1}}; switch (a) {case p.p.p1: break; case p.p.p1: break; default: break;}",
        "var a = 1, f = function(b) { return b ? { p1: 1 } : { p1: 2 }; }; switch (a) {case f(true).p1: break; case f(true).p1: break; default: break;}",
        "var a = 1, f = function(s) { return { p1: s } }; switch (a) {case f(a + 1).p1: break; case f(a + 1).p1: break; default: break;}",
        "var a = 1, f = function(s) { return { p1: s } }; switch (a) {case f(a === 1 ? 2 : 3).p1: break; case f(a === 1 ? 2 : 3).p1: break; default: break;}",
        "var a = 1, f1 = function() { return { p1: 1 } }; switch (a) {case f1().p1: break; case f1().p1: break; default: break;}",
        "var a = [1, 2]; switch(a.toString()){case ([1, 2]).toString():break; case ([1, 2]).toString():break; default:break;}",
        "switch (a) { case a: case a: }",
        "switch (a) { case a: break; case b: break; case a: break; case c: break; case a: break; }",
        "var a = 1, p = {p: {p1: 1, p2: 1}}; switch (a) {case p.p.p1: break; case p. p // comment\n .p1: break; default: break;}",
        "var a = 1, p = {p: {p1: 1, p2: 1}}; switch (a) {case p .p\n/* comment */\n.p1: break; case p.p.p1: break; default: break;}",
        "var a = 1, p = {p: {p1: 1, p2: 1}}; switch (a) {case p .p\n/* comment */\n.p1: break; case p. p // comment\n .p1: break; default: break;}",
        "var a = 1, p = {p: {p1: 1, p2: 1}}; switch (a) {case p.p.p1: break; case p. p // comment\n .p1: break; case p .p\n/* comment */\n.p1: break; default: break;}",
        "var a = 1, f = function(s) { return { p1: s } }; switch (a) {case f(a + 1).p1: break; case f(a+1).p1: break; default: break;}",
        "var a = 1, f = function(s) { return { p1: s } }; switch (a) {case f(\na + 1 // comment\n).p1: break; case f(a+1)\n.p1: break; default: break;}",
    ];

    Tester::new(NoDuplicateCase::NAME, NoDuplicateCase::PLUGIN, pass, fail).test_and_snapshot();
}
