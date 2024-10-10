use oxc_ast::ast::{
    Argument, Expression, JSXAttributeItem, JSXAttributeValue, JSXElementName, ObjectProperty,
    ObjectPropertyKind, StringLiteral,
};
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::{phf_set, Set};

use crate::utils::{get_prop_value, has_jsx_prop_ignore_case, is_create_element_call};
use crate::{context::LintContext, rule::Rule, AstNode};

fn missing_sandbox_prop(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("An iframe element is missing a sandbox attribute")
        .with_help("Add a `sandbox` attribute to the `iframe` element.")
        .with_label(span)
}

fn invalid_sandbox_prop(span: Span, value: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("An iframe element defines a sandbox attribute with invalid value: {value}"))
        .with_help("Check this link for the valid values of `sandbox` attribute: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe#sandbox.")
        .with_label(span)
}

fn invalid_sandbox_combination_prop(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("An `iframe` element defines a sandbox attribute with both allow-scripts and allow-same-origin which is invalid")
        .with_help("Remove `allow-scripts` or `allow-same-origin`.")
        .with_label(span)
}

const ALLOWED_VALUES: Set<&'static str> = phf_set! {
    "",
    "allow-downloads-without-user-activation",
    "allow-downloads",
    "allow-forms",
    "allow-modals",
    "allow-orientation-lock",
    "allow-pointer-lock",
    "allow-popups",
    "allow-popups-to-escape-sandbox",
    "allow-presentation",
    "allow-same-origin",
    "allow-scripts",
    "allow-storage-access-by-user-activation",
    "allow-top-navigation",
    "allow-top-navigation-by-user-activation"
};

#[derive(Debug, Default, Clone)]
pub struct IframeMissingSandbox;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce sandbox attribute on iframe elements
    ///
    /// ### Why is this bad?
    ///
    /// The sandbox attribute enables an extra set of restrictions for the content in the iframe. Using sandbox attribute is considered a good security practice.
    /// To learn more about sandboxing, see [MDN's documentation on the `sandbox` attribute](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe#sandbox).

    ///
    /// This rule checks all React `<iframe>` elements and verifies that there is `sandbox` attribute and that it's value is valid. In addition to that it also reports cases where attribute contains `allow-scripts` and `allow-same-origin` at the same time as this combination allows the embedded document to remove the sandbox attribute and bypass the restrictions.

    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <iframe/>;
    /// <iframe sandbox="invalid-value" />;
    /// <iframe sandbox="allow-same-origin allow-scripts"/>;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <iframe sandbox="" />;
    /// <iframe sandbox="allow-origin" />;
    /// ```
    IframeMissingSandbox,
    correctness,
    pending
);

impl Rule for IframeMissingSandbox {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXOpeningElement(jsx_el) => {
                let JSXElementName::Identifier(identifier) = &jsx_el.name else {
                    return;
                };

                if identifier.name != "iframe" {
                    return;
                }

                has_jsx_prop_ignore_case(jsx_el, "sandbox").map_or_else(
                    || {
                        ctx.diagnostic(missing_sandbox_prop(identifier.span));
                    },
                    |sandbox_prop| {
                        validate_sandbox_attribute(sandbox_prop, ctx);
                    },
                );
            }
            AstKind::CallExpression(call_expr) => {
                if is_create_element_call(call_expr) {
                    let Some(Argument::StringLiteral(str)) = call_expr.arguments.first() else {
                        return;
                    };

                    if str.value != "iframe" {
                        return;
                    }

                    if let Some(Argument::ObjectExpression(obj_expr)) = call_expr.arguments.get(1) {
                        obj_expr
                            .properties
                            .iter()
                            .find_map(|prop| {
                                if let ObjectPropertyKind::ObjectProperty(prop) = prop {
                                    if prop.key.is_specific_static_name("sandbox") {
                                        return Some(prop);
                                    }
                                }

                                None
                            })
                            .map_or_else(
                                || {
                                    ctx.diagnostic(missing_sandbox_prop(obj_expr.span));
                                },
                                |sandbox_prop| {
                                    validate_sandbox_property(sandbox_prop, ctx);
                                },
                            );
                    } else {
                        ctx.diagnostic(missing_sandbox_prop(call_expr.span));
                    }
                }
            }
            _ => {}
        }
    }
}
fn validate_sandbox_value(literal: &StringLiteral, ctx: &LintContext) {
    let attrs = literal.value.split(' ');
    let mut has_allow_same_origin = false;
    let mut has_allow_scripts = false;
    for trimmed_atr in attrs.into_iter().map(str::trim) {
        if !ALLOWED_VALUES.contains(trimmed_atr) {
            ctx.diagnostic(invalid_sandbox_prop(literal.span, trimmed_atr));
        }
        if trimmed_atr == "allow-scripts" {
            has_allow_scripts = true;
        }
        if trimmed_atr == "allow-same-origin" {
            has_allow_same_origin = true;
        }
    }
    if has_allow_scripts && has_allow_same_origin {
        ctx.diagnostic(invalid_sandbox_combination_prop(literal.span));
    }
}

fn validate_sandbox_property(object_property: &ObjectProperty, ctx: &LintContext) {
    if let Expression::StringLiteral(str) = object_property.value.without_parentheses() {
        validate_sandbox_value(str, ctx);
    }
}
fn validate_sandbox_attribute(jsx_el: &JSXAttributeItem, ctx: &LintContext) {
    if let Some(JSXAttributeValue::StringLiteral(str)) = get_prop_value(jsx_el) {
        validate_sandbox_value(str, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"<div sandbox="__unknown__" />;"#,
        r#"<iframe sandbox="" />;"#,
        r#"<iframe sandbox={""} />"#,
        r#"React.createElement("iframe", { sandbox: "" });"#,
        r#"<iframe src="foo.htm" sandbox></iframe>"#,
        r#"React.createElement("iframe", { src: "foo.htm", sandbox: true })"#,
        r#"<iframe src="foo.htm" sandbox sandbox></iframe>"#,
        r#"<iframe sandbox="allow-forms"></iframe>"#,
        r#"<iframe sandbox="allow-modals"></iframe>"#,
        r#"<iframe sandbox="allow-orientation-lock"></iframe>"#,
        r#"<iframe sandbox="allow-pointer-lock"></iframe>"#,
        r#"<iframe sandbox="allow-popups"></iframe>"#,
        r#"<iframe sandbox="allow-popups-to-escape-sandbox"></iframe>"#,
        r#"<iframe sandbox="allow-presentation"></iframe>"#,
        r#"<iframe sandbox="allow-same-origin"></iframe>"#,
        r#"<iframe sandbox="allow-scripts"></iframe>"#,
        r#"<iframe sandbox="allow-top-navigation"></iframe>"#,
        r#"<iframe sandbox="allow-top-navigation-by-user-activation"></iframe>"#,
        r#"<iframe sandbox="allow-forms allow-modals"></iframe>"#,
        r#"<iframe sandbox="allow-popups allow-popups-to-escape-sandbox allow-pointer-lock allow-same-origin allow-top-navigation"></iframe>"#,
        r#"React.createElement("iframe", { sandbox: "allow-forms" })"#,
        r#"React.createElement("iframe", { sandbox: "allow-modals" })"#,
        r#"React.createElement("iframe", { sandbox: "allow-orientation-lock" })"#,
        r#"React.createElement("iframe", { sandbox: "allow-pointer-lock" })"#,
        r#"React.createElement("iframe", { sandbox: "allow-popups" })"#,
        r#"React.createElement("iframe", { sandbox: "allow-popups-to-escape-sandbox" })"#,
        r#"React.createElement("iframe", { sandbox: "allow-presentation" })"#,
        r#"React.createElement("iframe", { sandbox: "allow-same-origin" })"#,
        r#"React.createElement("iframe", { sandbox: "allow-scripts" })"#,
        r#"React.createElement("iframe", { sandbox: "allow-top-navigation" })"#,
        r#"React.createElement("iframe", { sandbox: "allow-top-navigation-by-user-activation" })"#,
        r#"React.createElement("iframe", { sandbox: "allow-forms allow-modals" })"#,
        r#"React.createElement("iframe", { sandbox: "allow-popups allow-popups-to-escape-sandbox allow-pointer-lock allow-same-origin allow-top-navigation" })"#,
    ];

    let fail = vec![
        "<iframe></iframe>;",
        "<iframe/>;",
        r#"React.createElement("iframe");"#,
        r#"React.createElement("iframe", {});"#,
        r#"React.createElement("iframe", null);"#,
        r#"<iframe sandbox="__unknown__"></iframe>"#,
        r#"React.createElement("iframe", { sandbox: "__unknown__" })"#,
        r#"<iframe sandbox="allow-popups __unknown__"/>"#,
        r#"<iframe sandbox="__unknown__ allow-popups"/>"#,
        r#"<iframe sandbox=" allow-forms __unknown__ allow-popups __unknown__  "/>"#,
        r#"<iframe sandbox="allow-scripts allow-same-origin"></iframe>;"#,
        r#"<iframe sandbox="allow-same-origin allow-scripts"/>;"#,
    ];

    Tester::new(IframeMissingSandbox::NAME, pass, fail).test_and_snapshot();
}
