use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashMap;

use crate::{ast_util::calculate_hash, context::LintContext, rule::Rule, AstNode};

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
    correctness
);

impl Rule for NoDuplicateCase {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::SwitchStatement(ss) = node.kind() else {
            return;
        };

        let mut map = FxHashMap::default();
        map.reserve(ss.cases.len());
        for case in &ss.cases {
            if let Some(test) = case.test.as_ref() {
                let hash = calculate_hash(test.get_inner_expression());

                if let Some(prev_span) = map.insert(hash, test.span()) {
                    ctx.diagnostic(no_duplicate_case_diagnostic(prev_span, test.span()));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var a = 1; switch (a) {case 1: break; case 2: break; default: break;}", None),
        ("var a = 1; switch (a) {case 1: break; case '1': break; default: break;}", None),
        ("var a = 1; switch (a) {case 1: break; case true: break; default: break;}", None),
        ("var a = 1; switch (a) {default: break;}", None),
        (
            "var a = 1, p = {p: {p1: 1, p2: 1}}; switch (a) {case p.p.p1: break; case p.p.p2: break; default: break;}",
            None,
        ),
        (
            "var a = 1, f = function(b) { return b ? { p1: 1 } : { p1: 2 }; }; switch (a) {case f(true).p1: break; case f(true, false).p1: break; default: break;}",
            None,
        ),
        (
            "var a = 1, f = function(s) { return { p1: s } }; switch (a) {case f(a + 1).p1: break; case f(a + 2).p1: break; default: break;}",
            None,
        ),
        (
            "var a = 1, f = function(s) { return { p1: s } }; switch (a) {case f(a == 1 ? 2 : 3).p1: break; case f(a === 1 ? 2 : 3).p1: break; default: break;}",
            None,
        ),
        (
            "var a = 1, f1 = function() { return { p1: 1 } }, f2 = function() { return { p1: 2 } }; switch (a) {case f1().p1: break; case f2().p1: break; default: break;}",
            None,
        ),
        (
            "var a = [1,2]; switch(a.toString()){case ([1,2]).toString():break; case ([1]).toString():break; default:break;}",
            None,
        ),
        ("switch(a) { case a: break; } switch(a) { case a: break; }", None),
        ("switch(a) { case toString: break; }", None),
    ];

    let fail = vec![
        (
            "var a = 1; switch (a) {case 1: break; case 1: break; case 2: break; default: break;}",
            None,
        ),
        (
            "var a = 1; switch (a) {case 1: break; case (1): break; case 2: break; default: break;}",
            None,
        ),
        (
            "var a = '1'; switch (a) {case '1': break; case '1': break; case '2': break; default: break;}",
            None,
        ),
        (
            "var a = 1, one = 1; switch (a) {case one: break; case one: break; case 2: break; default: break;}",
            None,
        ),
        (
            "var a = 1, p = {p: {p1: 1, p2: 1}}; switch (a) {case p.p.p1: break; case p.p.p1: break; default: break;}",
            None,
        ),
        (
            "var a = 1, f = function(b) { return b ? { p1: 1 } : { p1: 2 }; }; switch (a) {case f(true).p1: break; case f(true).p1: break; default: break;}",
            None,
        ),
        (
            "var a = 1, f = function(s) { return { p1: s } }; switch (a) {case f(a + 1).p1: break; case f(a + 1).p1: break; default: break;}",
            None,
        ),
        (
            "var a = 1, f = function(s) { return { p1: s } }; switch (a) {case f(a === 1 ? 2 : 3).p1: break; case f(a === 1 ? 2 : 3).p1: break; default: break;}",
            None,
        ),
        (
            "var a = 1, f1 = function() { return { p1: 1 } }; switch (a) {case f1().p1: break; case f1().p1: break; default: break;}",
            None,
        ),
        (
            "var a = [1, 2]; switch(a.toString()){case ([1, 2]).toString():break; case ([1, 2]).toString():break; default:break;}",
            None,
        ),
        ("switch (a) { case a: case a: }", None),
        (
            "switch (a) { case a: break; case b: break; case a: break; case c: break; case a: break; }",
            None,
        ),
        (
            "var a = 1, p = {p: {p1: 1, p2: 1}}; switch (a) {case p.p.p1: break; case p. p // comment\n .p1: break; default: break;}",
            None,
        ),
        (
            "var a = 1, p = {p: {p1: 1, p2: 1}}; switch (a) {case p .p\n/* comment */\n.p1: break; case p.p.p1: break; default: break;}",
            None,
        ),
        (
            "var a = 1, p = {p: {p1: 1, p2: 1}}; switch (a) {case p .p\n/* comment */\n.p1: break; case p. p // comment\n .p1: break; default: break;}",
            None,
        ),
        (
            "var a = 1, p = {p: {p1: 1, p2: 1}}; switch (a) {case p.p.p1: break; case p. p // comment\n .p1: break; case p .p\n/* comment */\n.p1: break; default: break;}",
            None,
        ),
        (
            "var a = 1, f = function(s) { return { p1: s } }; switch (a) {case f(a + 1).p1: break; case f(a+1).p1: break; default: break;}",
            None,
        ),
        (
            "var a = 1, f = function(s) { return { p1: s } }; switch (a) {case f(\na + 1 // comment\n).p1: break; case f(a+1)\n.p1: break; default: break;}",
            None,
        ),
    ];

    Tester::new(NoDuplicateCase::NAME, pass, fail).test_and_snapshot();
}
