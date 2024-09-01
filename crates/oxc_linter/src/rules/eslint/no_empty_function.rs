use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_empty_function_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow empty functions")
        .with_help("Unexpected empty function block")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoEmptyFunction;

declare_oxc_lint!(
    /// ### What it does
    /// Disallows the usages of empty functions
    ///
    /// ### Why is this bad?
    /// Empty functions can reduce readability because readers need to guess whether it’s
    /// intentional or not. So writing a clear comment for empty functions is a good practice.
    ///
    /// ### Example
    /// ```javascript
    ///
    /// function foo() {
    /// }
    ///
    /// const bar = () => {};
    ///
    /// ```
    NoEmptyFunction,
    restriction,
);

impl Rule for NoEmptyFunction {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::FunctionBody(fb) = node.kind() {
            if fb.is_empty() && !ctx.semantic().trivias().has_comments_between(fb.span) {
                ctx.diagnostic(no_empty_function_diagnostic(fb.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
        function foo() {
            // empty
        }
        ",
        "
        function* baz() {
            // empty
        }
        ",
        "
        const bar = () => {
            // empty
        };
        ",
        "
        const obj = {
            foo: function() {
                // empty
            },
            bar: function*() {
                // empty
            },
            foobar() {
                // empty
            }
        };
        ",
        "
        class A {
            constructor() {
                // empty
            }
            foo() {
                // empty
            }
            *foo1() {
                // empty
            }
            get bar() {
                // empty
            }
            set bar(value) {
                // empty
            }
            static bar() {
                // empty
            }
            static *barr() {
                // empty
            }
            static get baz() {
                // empty
            }
            static set baz(value) {
                // empty
            }
        }
        ",
    ];

    let fail = vec![
        "function foo() {}",
        "const bar = () => {};",
        "function* baz() {}",
        "
        const obj = {
            foo: function() {
            },
            bar: function*() {
            },
            foobar() {
            }
        };
        ",
        "
        class A {
            constructor() {
            }
            foo() {
            }
            *foo1() {
            }
            get fooz() {
            }
            set fooz(value) {
            }
            static bar() {
            }
            static *barr() {
            }
            static get baz() {
            }
            static set baz(value) {
            }
        }
    ",
    ];

    Tester::new(NoEmptyFunction::NAME, pass, fail).test_and_snapshot();
}
