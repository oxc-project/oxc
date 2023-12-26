use oxc_ast::{ast::JSXElementName, AstKind};
use oxc_diagnostics::thiserror::Error;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use oxc_diagnostics::miette::{self, Diagnostic};

use crate::{rule::Rule, utils::get_element_type, LintContext};

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
        let Some(element_type) = get_element_type(ctx, jsx_el) else {
            return;
        };

        let name = element_type.as_str();

        if let "marquee" | "blink" = name {
            ctx.diagnostic(NoDistractingElementsDiagnostic(iden.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    fn config() -> serde_json::Value {
        serde_json::json!([2,{
            "ignoreNonDOM": true
        }])
    }

    fn settings() -> serde_json::Value {
        serde_json::json!({
            "jsx-a11y": {
                "components": {
                    "Blink": "blink",
                    "Marquee": "marquee"
                }
            }
        })
    }

    let pass = vec![
        (r"<div />", None, None),
        (r"<Marquee />", None, None),
        (r"<div marquee />", None, None),
        (r"<Blink />", None, None),
        (r"<div blink />", None, None),
    ];

    let fail = vec![
        (r"<marquee />", None, None),
        (r"<marquee {...props} />", None, None),
        (r"<marquee lang={undefined} />", None, None),
        (r"<blink />", None, None),
        (r"<blink {...props} />", None, None),
        (r"<blink foo={undefined} />", None, None),
        (r"<Blink />", Some(config()), Some(settings())),
        (r"<Marquee />", Some(config()), Some(settings())),
    ];

    Tester::new_with_settings(NoDistractingElements::NAME, pass, fail).test_and_snapshot();
}
