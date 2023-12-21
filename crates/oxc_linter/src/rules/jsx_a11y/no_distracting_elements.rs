use oxc_ast::{ast::JSXElementName, AstKind};
use oxc_diagnostics::thiserror::Error;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use oxc_diagnostics::miette::{self, Diagnostic};

use crate::{rule::Rule, LintContext};

#[derive(Debug, Error, Diagnostic)]
#[error(
    "eslint-plugin-jsx-a11y(no-distracting-elements): Do not use <marquee> or <blink> elements as they can create visual accessibility issues and are deprecated."
)]
#[diagnostic(severity(warning), help("Replace the <marquee> or <blink> element with alternative, more accessible ways to achieve your desired visual effects."))]
struct NoDistractingElementsDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoDistractingElements;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that no distracting elements are used.
    ///
    /// ### Why is this necessary?
    ///
    /// Elements that can be visually distracting can cause accessibility issues with visually impaired users.
    /// Such elements are most likely deprecated, and should be avoided. By default, <marquee> and <blink> elements are visually distracting.
    ///
    /// ### What it checks
    ///
    /// This rule checks for marquee and blink element.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <marquee />
    /// <marquee {...props} />
    /// <marquee lang={undefined} />
    /// <blink />
    /// <blink {...props} />
    /// <blink foo={undefined} />
    ///
    /// // Good
    /// <div />
    /// <Marquee />
    /// <Blink />
    /// ```
    NoDistractingElements,
    correctness
);

impl Rule for NoDistractingElements {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else { return };
        let JSXElementName::Identifier(iden) = &jsx_el.name else { return };

        let name = iden.name.as_str();

        if let "marquee" | "blink" = name {
            ctx.diagnostic(NoDistractingElementsDiagnostic(iden.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"<div />", None),
        (r"<Marquee />", None),
        (r"<div marquee />", None),
        (r"<Blink />", None),
        (r"<div blink />", None),
    ];

    let fail = vec![
        (r"<marquee />", None),
        (r"<marquee {...props} />", None),
        (r"<marquee lang={undefined} />", None),
        (r"<blink />", None),
        (r"<blink {...props} />", None),
        (r"<blink foo={undefined} />", None),
    ];

    Tester::new(NoDistractingElements::NAME, pass, fail)
        .with_jsx_a11y_plugin(true)
        .test_and_snapshot();
}
