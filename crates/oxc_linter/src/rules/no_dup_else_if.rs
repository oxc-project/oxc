use oxc_ast::{
    ast::{IfStatement, ObjectProperty, PropertyKind},
    AstKind, GetSpan, Span,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use rustc_hash::FxHashMap;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-dupe-else-if): Disallow duplicate conditions in `else if` statements")]
#[diagnostic(severity(warning), help("Consider combining these `else if` statements into a single statement"))]
struct NoDupeElseIfDiagnostic(#[label] pub Span, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoDupeElseIf;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow duplicate conditions in `else if` statements.
    ///
    /// ### Why is this bad?
    ///
    /// Repeating the same condition in multiple `else if` statements can be a sign of a code smell or poor code quality, as it could be consolidated into a single `if` statement or more concise code. Additionally, it may lead to bugs or unexpected behavior if the conditions are not properly handled or redundant.
    ///
    /// ### Example
    /// ```javascript
    /// if (a === 1) {
    ///     // some code
    /// } else if (a === 2) {
    ///     // some code
    /// } else if (a === 2) {
    ///     // some code
    /// }
    /// ```
    NoDupeElseIf,
    readability
);

impl Rule for NoDupeElseIf {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::IfStatement(if_stmt) = node.get().kind() {
            if let Some(else_branch) = if_stmt.alternate.as_ref() {
                let mut map = FxHashMap::default();
                for stmt in else_branch.iter() {
                    if let AstKind::IfStatement(else_if_stmt) = stmt.kind() {
                        let hash = else_if_stmt.test.static_hash();
                        if let Some(prev_span) = map.insert(hash, else_if_stmt.test.span()) {
                            ctx.diagnostic(NoDupeElseIfDiagnostic(prev_span, else_if_stmt.test.span()));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
    ("if (a === 1) { } else if (a === 2) { } else if (a === 3) { } else { }", None),
    ("if (a === 1) { } else if (a === 1) { } else { }", None),
    ("if (a === 1) { } else if (a === 2) { } else if (a === 3) { } else if (a === 4) { } else { }", None),
    ("if (a === 1) { } else { }", None),
    ("if (a === 1) { }", None),
    ("if (a === 1) { console.log('hello'); } else if (a === 2) { console.log('world'); } else { console.log('!'); }", None),
    ("if (a === 1) { foo(); } else { bar(); }", None),
    ("if (a === 1) { return true; } else { return false; }", None),
    ("if (a === 1) { doSomething(); } else if (a === 2) { doSomethingElse(); } else { doAnotherThing(); }", None),
    ("if (a === 1) { doSomething(); } else if (a === 2) { doSomethingElse(); } else if (a === 3) { doAnotherThing(); } else { doSomethingCompletelyDifferent(); }", None),
];

let fail = vec![
    ("if (a === 1) { } else if (a === 2) { } else if (a === 2) { }", None),
    ("if (a === 1) { } else if (a === 2) { } else if (a === 1) { }", None),
    ("if (a === 1) { } else if (a === 1) { } else if (a === 2) { } else if (a === 1) { }", None),
    ("if (a === 1) { } else if (a === 2) { } else if (a === 1) { } else if (a === 2) { }", None),
    ("if (a === 1) { } else if (a === 2) { } else if (a === 2) { } else if (a === 1) { }", None),
    ("if (a === 1) { } else if (a === 1) { } else if (a === 1) { }", None),
    ("if (a === 1) { return true; } else if (a === 1) { return false; } else { return true; }", None),
    ("if (a === 1) { } else if (a === 2) { return true; } else { return false; } else { return true; }", None),
    ("if (a === 1) { } else if (a === 2) { doSomething(); } else if (a === 1) { doSomethingElse(); } else { doAnotherThing(); }", None),
    ("if (a === 1) { doSomething(); } else if (a === 2) { doSomethingElse(); } else if (a === 2) { doAnotherThing(); }", None),
];

    Tester::new(NoDupeElseIf::NAME, pass, fail).test_and_snapshot();
}
