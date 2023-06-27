use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-empty-pattern): Disallow empty destructuring patterns")]
#[diagnostic(severity(warning))]
struct NoEmptyPatternDiagnostic(&'static str, #[label("Empty {0} binding pattern")] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoEmptyPattern;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow empty destructuring patterns
    ///
    /// ### Why is this bad?
    /// When using destructuring, it’s possible to create a pattern that has no effect.
    /// This happens when empty curly braces are used to the right of
    /// an embedded object destructuring pattern, such as:
    ///
    /// ```JavaScript
    /// // doesn't create any variables
    /// var {a: {}} = foo;
    /// ```
    /// In this code, no new variables are created because a is just a location helper
    /// while the `{}` is expected to contain the variables to create, such as:
    ///
    /// ```JavaScript
    /// // creates variable b
    /// var {a: { b }} = foo;
    /// ```
    ///
    /// In many cases, the empty object pattern is a mistake
    /// where the author intended to use a default value instead, such as:
    ///
    /// ```JavaScript
    /// // creates variable a
    /// var {a = {}} = foo;
    /// ```
    ///
    /// The difference between these two patterns is subtle,
    /// especially because the problematic empty pattern looks just like an object literal.
    ///
    /// ### Examples of incorrect code for this rule:
    ///
    /// ```JavaScript
    /// var {} = foo;
    /// var [] = foo;
    /// var {a: {}} = foo;
    /// var {a: []} = foo;
    /// function foo({}) {}
    /// function foo([]) {}
    /// function foo({a: {}}) {}
    /// function foo({a: []}) {}
    /// ```
    ///
    /// ### Examples of correct code for this rule:
    ///
    /// ```JavaScript
    /// var {a = {}} = foo;
    /// var {a = []} = foo;
    /// function foo({a = {}}) {}
    /// function foo({a = []}) {}
    /// ```
    ///
    NoEmptyPattern,
    correctness,
);

impl Rule for NoEmptyPattern {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (pattern_type, span) = match node.kind() {
            AstKind::ArrayPattern(array) if array.elements.is_empty() => ("array", array.span),
            AstKind::ObjectPattern(object) if object.properties.is_empty() => {
                ("object", object.span)
            }
            _ => return,
        };

        ctx.diagnostic(NoEmptyPatternDiagnostic(pattern_type, span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var {a = {}} = foo;", None),
        ("var {a, b = {}} = foo;", None),
        ("var {a = []} = foo;", None),
        ("function foo({a = {}}) {}", None),
        ("function foo({a = []}) {}", None),
        ("var [a] = foo", None),
    ];

    let fail = vec![
        ("var {} = foo", None),
        ("var [] = foo", None),
        ("var {a: {}} = foo", None),
        ("var {a, b: {}} = foo", None),
        ("var {a: []} = foo", None),
        ("function foo({}) {}", None),
        ("function foo([]) {}", None),
        ("function foo({a: {}}) {}", None),
        ("function foo({a: []}) {}", None),
    ];

    Tester::new(NoEmptyPattern::NAME, pass, fail).test_and_snapshot();
}
