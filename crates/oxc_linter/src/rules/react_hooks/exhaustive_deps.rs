use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("react-hooks(exhaustive-deps):")]
#[diagnostic(severity(warning), help(""))]
struct ExhaustiveDepsDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct ExhaustiveDeps;

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
    ExhaustiveDeps,
    correctness
);

impl Rule for ExhaustiveDeps {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"function MyComponent() { const local = {}; useEffect(() => { console.log(local) }); }",
    ];

    let fail = vec![
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo?.toString());
          }, []);
        }",
    ];

    Tester::new(ExhaustiveDeps::NAME, pass, fail).test_and_snapshot();
}
