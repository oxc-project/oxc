use cow_utils::CowUtils;
use oxc_ast::{ast::JSXAttributeItem, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext, globals::VALID_ARIA_PROPS, rule::Rule, utils::get_jsx_attribute_name,
    AstNode,
};

fn aria_props_diagnostic(span: Span, prop_name: &str, suggestion: Option<&str>) -> OxcDiagnostic {
    let mut err = OxcDiagnostic::warn(format!("'{prop_name}' is not a valid ARIA attribute."));

    if let Some(suggestion) = suggestion {
        err = err.with_help(format!("Did you mean '{suggestion}'?"));
    } else {
        err = err.with_help("You can find a list of valid ARIA attributes at https://www.w3.org/TR/wai-aria-1.1/#state_prop_def");
    }

    err.with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct AriaProps;

declare_oxc_lint!(
    /// ### What it does
    /// Enforces that elements do not use invalid ARIA attributes.
    ///
    /// ### Why is this bad?
    /// Using invalid ARIA attributes can mislead screen readers and other assistive technologies.
    /// It may cause the accessibility features of the website to fail, making it difficult
    /// for users with disabilities to use the site effectively.
    ///
    /// This rule includes fixes for some common typos.
    ///
    /// ### Example
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <input aria-labeledby="address_label" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <input aria-labelledby="address_label" />
    /// ```
    AriaProps,
    jsx_a11y,
    correctness,
    conditional_fix
);

impl Rule for AriaProps {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXAttributeItem(JSXAttributeItem::Attribute(attr)) = node.kind() {
            let name = get_jsx_attribute_name(&attr.name);
            let name = name.cow_to_ascii_lowercase();
            if name.starts_with("aria-") && !VALID_ARIA_PROPS.contains(&name) {
                let suggestion = COMMON_TYPOS.get(&name).copied();
                let diagnostic = aria_props_diagnostic(attr.span, &name, suggestion);

                if let Some(suggestion) = suggestion {
                    ctx.diagnostic_with_fix(diagnostic, |fixer| {
                        fixer.replace(attr.name.span(), suggestion)
                    });
                } else {
                    ctx.diagnostic(diagnostic);
                }
            }
        }
    }
}

const COMMON_TYPOS: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "aria-labeledby" => "aria-labelledby",
    "aria-role" => "role",
    "aria-sorted" => "aria-sort",
    "aria-lable" => "aria-label",
    "aria-value" => "aria-valuenow",
};

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"<div />",
        r"<div></div>",
        r#"<div aria="wee"></div>"#,
        r#"<div abcARIAdef="true"></div>"#,
        r#"<div fooaria-foobar="true"></div>"#,
        r#"<div fooaria-hidden="true"></div>"#,
        r"<Bar baz />",
        r#"<input type="text" aria-errormessage="foobar" />"#,
    ];

    let fail = vec![
        r#"<div aria-="foobar" />"#,
        r#"<div aria-labeledby="foobar" />"#,
        r#"<div aria-skldjfaria-klajsd="foobar" />"#,
    ];
    let fix =
        vec![(r#"<div aria-labeledby="foobar" />"#, r#"<div aria-labelledby="foobar" />"#, None)];

    Tester::new(AriaProps::NAME, AriaProps::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
