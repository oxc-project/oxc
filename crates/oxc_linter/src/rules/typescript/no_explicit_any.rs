use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{rule::Rule, LintContext};

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(no-explicit-any): Unexpected `any`. Specify a different type.")]
#[diagnostic(severity(warning), help("Consider using `unknown`."))]
struct NoExplicitAnyDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoExplicitAny;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow usage of the `any` type.
    ///
    /// ### Why is this bad?
    ///
    /// The `any` type is similar to ignoring type checking for the variable.
    ///
    /// In cases where you don’t know what type you want to accept,
    /// or when you want to accept anything because you will be blindly
    /// passing it through without interacting with it, you can use `unknown`.
    ///
    /// ### Example
    /// ```typescript
    /// const age: any = "seventeen";
    /// ```
    NoExplicitAny,
    correctness,
);

impl Rule for NoExplicitAny {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::TSAnyKeyword(any_key) = node.kind() {
            ctx.diagnostic(NoExplicitAnyDiagnostic(Span::new(
                any_key.span.start,
                any_key.span.start + 3,
            )));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const age: number = 17;",
        "const ages: number[] = [17];",
        "const ages: Array<number> = [17];",
        "function greet(): string {}",
        "function greet(): string[] {}",
        "function greet(): Array<string> {}",
        "function greet(): Array<Array<string>> {}",
        "function greet(param: Array<string>): string {}",
        "function greet(param: Array<string>): Array<string> {}",
    ];

    let fail = vec![
        "const age: any = 'seventeen';",
        "const ages: any[] = ['seventeen'];",
        "const ages: Array<any> = ['seventeen'];",
        "function greet(): any {}",
        "function greet(): any[] {}",
        "function greet(): Array<any> {}",
        "function greet(): Array<Array<any>> {}",
        "function greet(param: Array<any>): string {}",
        "function greet(param: Array<any>): Array<any> {}",
    ];

    Tester::new_without_config(NoExplicitAny::NAME, pass, fail).test_and_snapshot();
}
