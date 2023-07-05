use oxc_ast::{AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(no-explicit-any): Using any disables many type checking rules and is generally best used only as a last resort or when prototyping code")]
#[diagnostic(severity(warning))]
struct NoExplicitAnyDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoExplicitAny;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the any type.
    ///
    /// ### Why is this bad?
    ///
    /// The any type in TypeScript is a dangerous "escape hatch" from the type system.
    /// Using any disables many type checking rules and is generally best used only as a last
    /// resort or when prototyping code. This rule reports on explicit uses of the any keyword
    /// as a type annotation.
    ///
    /// ### Example
    /// ```typescript
    /// const age: any = 'seventeen';
    /// const ages: any[] = ['seventeen'];
    /// const ages: Array<any> = ['seventeen'];
    /// ```
    NoExplicitAny,
    correctness
);

impl Rule for NoExplicitAny {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::TSAnyKeyword(decl) = node.kind() {
            ctx.diagnostic(NoExplicitAnyDiagnostic(decl.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const number: number = 1;",
        "function greet(): string {}",
        "function greet(): Array<string> {}",
        "function greet(): string[] {}",
        "function greet(): Array<Array<string>> {}",
        "function greet(): Array<string[]> {}",
        "function greet(param: Array<string>): Array<string> {}",
    ];

    let fail = vec![
        "const num: any = 1",
    ];

    Tester::new_without_config(NoExplicitAny::NAME, pass, fail).test_and_snapshot();
}
