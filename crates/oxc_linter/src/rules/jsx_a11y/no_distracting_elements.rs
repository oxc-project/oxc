use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

use crate::{rule::Rule, utils::get_element_type, LintContext};

fn no_distracting_elements_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use <marquee> or <blink> elements as they can create visual accessibility issues and are deprecated.")
        .with_help("Replace the <marquee> or <blink> element with alternative, more accessible ways to achieve your desired visual effects.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDistractingElements;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that no distracting elements are used.
    ///
    /// ### Why is this necessary?
    ///
    /// Elements that can be visually distracting can cause accessibility issues
    /// with visually impaired users.  Such elements are most likely deprecated,
    /// and should be avoided. By default, `<marquee>` and `<blink>` elements
    /// are visually distracting.
    ///
    /// ### What it checks
    ///
    /// This rule checks for marquee and blink element.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <marquee />
    /// <marquee {...props} />
    /// <marquee lang={undefined} />
    /// <blink />
    /// <blink {...props} />
    /// <blink foo={undefined} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div />
    /// <Marquee />
    /// <Blink />
    /// ```
    NoDistractingElements,
    jsx_a11y,
    correctness
);

impl Rule for NoDistractingElements {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let element_type = get_element_type(ctx, jsx_el);

        if let "marquee" | "blink" = element_type.as_ref() {
            ctx.diagnostic(no_distracting_elements_diagnostic(jsx_el.name.span()));
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
            "settings": { "jsx-a11y": {
                "components": {
                    "Blink": "blink",
                    "Marquee": "marquee"
                }
            } }
        })
    }

    let pass = vec![
        (r"<div />", None, None, None),
        (r"<Marquee />", None, None, None),
        (r"<div marquee />", None, None, None),
        (r"<Blink />", None, None, None),
        (r"<div blink />", None, None, None),
    ];

    let fail = vec![
        (r"<marquee />", None, None, None),
        (r"<marquee {...props} />", None, None, None),
        (r"<marquee lang={undefined} />", None, None, None),
        (r"<blink />", None, None, None),
        (r"<blink {...props} />", None, None, None),
        (r"<blink foo={undefined} />", None, None, None),
        (r"<Blink />", Some(config()), Some(settings()), None),
        (r"<Marquee />", Some(config()), Some(settings()), None),
    ];

    Tester::new(NoDistractingElements::NAME, NoDistractingElements::PLUGIN, pass, fail)
        .test_and_snapshot();
}
