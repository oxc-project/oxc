use oxc_ast::{AstKind, Span};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-empty-pattern): Disallow empty destructuring patterns")]
#[diagnostic()]
struct NoEmptyPatternDiagnostic(&'static str, #[label("Empty {0} binding pattern")] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoEmptyPattern;

const RULE_NAME: &str = "no-empty-pattern";

impl Rule for NoEmptyPattern {
    const NAME: &'static str = RULE_NAME;

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (pattern_type, span) = match node.get().kind() {
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

    Tester::new(RULE_NAME, pass, fail).test_and_snapshot();
}
