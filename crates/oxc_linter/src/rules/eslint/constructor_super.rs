use oxc_macros::declare_oxc_lint;

// use oxc_span::Span;
use crate::{context::LintContext, rule::Rule, AstNode};

// #[derive(Debug, Error, Diagnostic)]
// #[error("Expected to call 'super()'.")]
// #[diagnostic(severity(warning), help("Ensure 'super()' is called from constructor"))]
// struct ConstructorSuperDiagnostic(#[label] pub Span);

// #[derive(Debug, Error, Diagnostic)]
// #[error("Unexpected 'super()' because 'super' is not a constructor.")]
// #[diagnostic(severity(warning), help("Do not call 'super()' from constructor."))]
// struct SuperNotConstructorDiagnostic(
//     #[label("unexpected 'super()'")] pub Span,
//     #[label("because this is not a constructor")] pub Span,
// );

#[derive(Debug, Default, Clone)]
pub struct ConstructorSuper;

declare_oxc_lint!(
    /// ### What it does
    /// Require 'super()' calls in constructors.
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// class A extends B {
    ///   constructor() {}
    /// }
    /// ```
    ConstructorSuper,
    eslint,
    nursery // This rule should be implemented with CFG, the current implementation has a lot of
            // false positives.
);

impl Rule for ConstructorSuper {
    fn run<'a>(&self, _node: &AstNode<'a>, _ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("class A { }", None),
        ("class A { constructor() { } }", None),
        ("class A extends null { }", None),
        ("class A extends B { constructor() { super(); } }", None),
        ("class A extends B { }", None),
        ("class A extends B { constructor() { super(); } }", None),
        ("class A extends (class B {}) { constructor() { super(); } }", None),
        ("class A extends (B = C) { constructor() { super(); } }", None),
        ("class A extends (B &&= C) { constructor() { super(); } }", None),
        ("class A extends (B ||= C) { constructor() { super(); } }", None),
        ("class A extends (B ??= C) { constructor() { super(); } }", None),
        ("class A extends (B ||= 5) { constructor() { super(); } }", None),
        ("class A extends (B ??= 5) { constructor() { super(); } }", None),
        ("class A extends (B || C) { constructor() { super(); } }", None),
        ("class A extends (5 && B) { constructor() { super(); } }", None),
    ];

    let fail = vec![
        // ("class A extends B { constructor() {} }", None),
        // ("class A extends null { constructor() { super(); } }", None),
        // ("class A extends null { constructor() { } }", None),
        // ("class A extends 100 { constructor() { super(); } }", None),
        // ("class A extends 'test' { constructor() { super(); } }", None),
    ];

    Tester::new(ConstructorSuper::NAME, ConstructorSuper::PLUGIN, pass, fail).test_and_snapshot();
}
