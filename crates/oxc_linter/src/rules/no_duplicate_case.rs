use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashMap;

use crate::{ast_util::calculate_hash, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-duplicate-case): Disallow duplicate case labels")]
#[diagnostic(severity(warning), help("Remove the duplicated case"))]
struct NoDuplicateCaseDiagnostic(#[label] pub Span, #[label] pub Span);

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
    /// ```javascript
    /// var a = 1;
    /// switch (a) {
    ///     case 1:
    ///         break;
    ///     case 1:
    ///         break;
    ///     case 2:
    ///         break;
    ///     default:
    ///         break;
    /// }
    /// ```
    NoDuplicateCase,
    correctness
);

impl Rule for NoDuplicateCase {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::SwitchStatement(ss) = node.get().kind() {
            let mut map = FxHashMap::default();
            map.reserve(ss.cases.len());
            for case in ss.cases.iter() {
                if let Some(test) = case.test.as_ref() {
                    let hash = calculate_hash(test);

                    if let Some(prev_span) = map.insert(hash, test.span()) {
                        ctx.diagnostic(NoDuplicateCaseDiagnostic(prev_span, test.span()));
                    }
                }
            }
        }
    }
}

#[test]
#[allow(clippy::too_many_lines)]
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
