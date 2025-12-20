use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_with_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected use of `with` statement.")
        .with_help("Do not use the `with` statement.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoWith;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow [`with`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/with) statements.
    ///
    /// ### Why is this bad?
    ///
    /// The with statement is potentially problematic because it adds members
    /// of an object to the current scope, making it impossible to tell what a
    /// variable inside the block actually refers to.
    ///
    /// It is generally considered a bad practice and is forbidden in strict mode.
    ///
    /// This rule is not necessary in TypeScript code if `alwaysStrict` is enabled.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// with (point) {
    ///     r = Math.sqrt(x * x + y * y); // is r a member of point?
    /// }
    /// ```
    NoWith,
    eslint,
    correctness
);

impl Rule for NoWith {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::WithStatement(with_statement) = node.kind() {
            ctx.diagnostic(no_with_diagnostic(Span::sized(with_statement.span.start, 4)));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo.bar();",
        "/* with keyword in block comment */ foo();",
        "// with in line comment\nfoo();",
        "var obj = { with: 1 }; obj.with;",
        "obj.with = 1;",
        "class C { with() {} } new C().with();",
        "console.log('with in string');",
        "console.log(`with in template`);",
        "const o = {}; o['with'] = 2;",
        "const p = { ['with']: 3 }; p.with;",
        "const { with: w } = { with: 4 }; w;",
    ];

    let fail = vec!["with(foo) { bar() }"];

    Tester::new(NoWith::NAME, NoWith::PLUGIN, pass, fail).test_and_snapshot();
}
