use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("")]
#[diagnostic(severity(warning), help(""))]
struct NoEmptyCharacterClassDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoEmptyCharacterClass;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoEmptyCharacterClass,
    correctness
);

impl Rule for NoEmptyCharacterClass {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var foo = /^abc[a-zA-Z]/;", None),
        ("var regExp = new RegExp(\"^abc[]\");", None),
        ("var foo = /^abc/;", None),
        ("var foo = /[\\[]/;", None),
        ("var foo = /[\\]]/;", None),
        ("var foo = /[a-zA-Z\\[]/;", None),
        ("var foo = /[[]/;", None),
        ("var foo = /[\\[a-z[]]/;", None),
        ("var foo = /[\\-\\[\\]\\/\\{\\}\\(\\)\\*\\+\\?\\.\\\\^\\$\\|]/g;", None),
        ("var foo = /\\s*:\\s*/gim;", None),
        ("var foo = /[\\]]/uy;", None),
        ("var foo = /[\\]]/s;", None),
        ("var foo = /[\\]]/d;", None),
        ("var foo = /\\[]/", None),
    ];

    let fail = vec![
        ("var foo = /^abc[]/;", None),
        ("var foo = /foo[]bar/;", None),
        ("if (foo.match(/^abc[]/)) {}", None),
        ("if (/^abc[]/.test(foo)) {}", None),
        ("var foo = /[]]/;", None),
        ("var foo = /\\[[]/;", None),
        ("var foo = /\\[\\[\\]a-z[]/;", None),
        ("var foo = /[]]/d;", None),
    ];

    Tester::new(NoEmptyCharacterClass::NAME, pass, fail).test_and_snapshot();
}
