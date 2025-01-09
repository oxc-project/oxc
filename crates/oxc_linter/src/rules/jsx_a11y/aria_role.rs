use oxc_ast::{
    ast::{JSXAttributeValue, JSXExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    globals::{HTML_TAG, VALID_ARIA_ROLES},
    rule::Rule,
    utils::{get_element_type, get_prop_value, has_jsx_prop},
    AstNode,
};

fn aria_role_diagnostic(span: Span, help_suffix: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Elements with ARIA roles must use a valid, non-abstract ARIA role.")
        .with_help(format!(
            "Set a valid, non-abstract ARIA role for element with ARIA{help_suffix}"
        ))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct AriaRole(Box<AriaRoleConfig>);

#[derive(Debug, Default, Clone)]
pub struct AriaRoleConfig {
    ignore_non_dom: bool,
    allowed_invalid_roles: Vec<String>,
}

impl std::ops::Deref for AriaRole {
    type Target = AriaRoleConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Elements with ARIA roles must use a valid, non-abstract ARIA role. A
    /// reference to role definitions can be found at
    /// [WAI-ARIA](https://www.w3.org/TR/wai-aria/#role_definitions) site.
    ///
    ///
    /// ### Why is this bad?
    ///
    /// The intent of this Success Criterion is to ensure that Assistive
    /// Technologies (AT) can gather information about, activate (or set) and
    /// keep up to date on the status of user interface controls in the
    /// content(such as screen readers, screen magnifiers, and speech
    /// recognition software, used by people with disabilities).
    ///
    /// When standard controls from accessible technologies are used, this
    /// process is straightforward. If the user interface elements are used
    /// according to specification the conditions of this provision will be met.
    ///
    /// If custom controls are created, however, or interface elements are
    /// programmed (in code or script) to have a different role and/or function
    /// than usual, then additional measures need to be taken to ensure that the
    /// controls provide important information to assistive technologies and
    /// allow themselves to be controlled by assistive technologies.  A
    /// particularly important state of a user interface control is whether or
    /// not it has focus. The focus state of a control can be programmatically
    /// determined, and notifications about change of focus are sent to user
    /// agents and assistive technology.  Other examples of user interface
    /// control state are whether or not a checkbox or radio button has been
    /// selected, or whether or not a collapsible tree or list node is expanded
    /// or collapsed.
    ///
    /// ### Rule options
    /// This rule takes one optional object argument of type object:
    /// ```json
    /// {
    ///     "rules": {
    ///         "jsx-a11y/aria-role": [ 2, {
    ///             "allowedInvalidRoles": ["text"],
    ///             "ignoreNonDOM": true
    ///         }],
    ///     }
    ///  }
    /// ```
    /// `allowedInvalidRules` is an optional string array of custom roles that
    /// should be allowed in addition to the ARIA spec, such as for cases when
    /// you need to use a non-standard role.
    ///
    /// For the `ignoreNonDOM` option, this determines if developer created
    /// components are checked.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```jsx
    /// <div role="datepicker"></div> <!-- Bad: "datepicker" is not an ARIA role -->
    /// <div role="range"></div>      <!-- Bad: "range" is an _abstract_ ARIA role -->
    /// <div role=""></div>           <!-- Bad: An empty ARIA role is not allowed -->
    /// <Foo role={role}></Foo>       <!-- Bad: ignoreNonDOM is set to false or not set -->
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div role="button"></div>     <!-- Good: "button" is a valid ARIA role -->
    /// <div role={role}></div>       <!-- Good: role is a variable & cannot be determined until runtime. -->
    /// <div></div>                   <!-- Good: No ARIA role -->
    /// <Foo role={role}></Foo>       <!-- Good: ignoreNonDOM is set to true -->
    /// ```
    AriaRole,
    jsx_a11y,
    correctness
);

impl Rule for AriaRole {
    fn from_configuration(value: serde_json::Value) -> Self {
        let Some(value) = value.as_array() else {
            return Self::default();
        };
        let mut ignore_non_dom = false;
        let mut allowed_invalid_roles: Vec<String> = vec![];

        let _ = value.iter().find(|v| {
            if let serde_json::Value::Object(obj) = v {
                if let Some(serde_json::Value::Bool(val)) = obj.get("ignoreNonDOM") {
                    ignore_non_dom = *val;
                }

                if let Some(serde_json::Value::Array(val)) = obj.get("allowedInvalidRoles") {
                    allowed_invalid_roles =
                        val.iter().map(|v| v.as_str().unwrap().to_string()).collect();
                }

                return true;
            }
            false
        });

        Self(Box::new(AriaRoleConfig { ignore_non_dom, allowed_invalid_roles }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXElement(jsx_el) = node.kind() {
            if let Some(aria_role) = has_jsx_prop(&jsx_el.opening_element, "role") {
                let element_type = get_element_type(ctx, &jsx_el.opening_element);

                if self.ignore_non_dom && !HTML_TAG.contains(&element_type) {
                    return;
                }

                let oxc_ast::ast::JSXAttributeItem::Attribute(attr) = aria_role else {
                    return;
                };

                match get_prop_value(aria_role) {
                    Some(JSXAttributeValue::ExpressionContainer(container)) => {
                        let jsexp = &container.expression;
                        if matches!(jsexp, JSXExpression::NullLiteral(_)) || jsexp.is_undefined() {
                            ctx.diagnostic(aria_role_diagnostic(attr.span, ""));
                        }
                    }
                    Some(JSXAttributeValue::StringLiteral(str)) => {
                        let words_str = String::from(str.value.as_str());
                        let words = words_str.split_whitespace();
                        if let Some(error_prop) = words.into_iter().find(|word| {
                            !VALID_ARIA_ROLES.contains(word)
                                && !self.allowed_invalid_roles.contains(&(*word).to_string())
                        }) {
                            ctx.diagnostic(aria_role_diagnostic(
                                str.span,
                                &format!(", `{error_prop}` is an invalid aria role"),
                            ));
                        }
                    }
                    _ => {
                        ctx.diagnostic(aria_role_diagnostic(attr.span, ""));
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn ignore_non_dom_schema() -> serde_json::Value {
        serde_json::json!([2,{
            "ignoreNonDOM": true
        }])
    }

    fn allowed_invalid_roles() -> serde_json::Value {
        serde_json::json!([2,{
            "allowedInvalidRoles": ["invalid-role", "other-invalid-role"],
        }])
    }

    fn settings() -> serde_json::Value {
        serde_json::json!({
            "settings": { "jsx-a11y": {
                "polymorphicPropName": "asChild",
                "components": {
                    "Div": "div",
                }
            } }
        })
    }

    let pass = vec![
        ("<div />", None, None, None),
        ("<div></div>", None, None, None),
        ("<div role={role} />", None, None, None),
        ("<div role={role || 'button'} />", None, None, None),
        ("<div role={role || 'foobar'} />", None, None, None),
        ("<div role='tabpanel row' />", None, None, None),
        ("<div role='switch' />", None, None, None),
        ("<div role='doc-abstract' />", None, None, None),
        ("<div role='doc-appendix doc-bibliography' />", None, None, None),
        ("<Bar baz />", None, None, None),
        ("<img role='invalid-role' />", Some(allowed_invalid_roles()), None, None),
        ("<img role='invalid-role tabpanel' />", Some(allowed_invalid_roles()), None, None),
        (
            "<img role='invalid-role other-invalid-role' />",
            Some(allowed_invalid_roles()),
            None,
            None,
        ),
        ("<Foo role='bar' />", Some(ignore_non_dom_schema()), None, None),
        ("<fakeDOM role='bar' />", Some(ignore_non_dom_schema()), None, None),
        ("<img role='presentation' />", Some(ignore_non_dom_schema()), None, None),
        ("<Div role='button' />", None, Some(settings()), None),
        ("<Box asChild='div' role='button' />", None, Some(settings()), None),
        ("<svg role='graphics-document document' />", None, None, None),
    ];

    let fail = vec![
        ("<div role='foobar' />", None, None, None),
        ("<div role='datepicker'></div>", None, None, None),
        ("<div role='range'></div>", None, None, None),
        ("<div role='Button'></div>", None, None, None),
        ("<div role='></div>", None, None, None),
        ("<div role='tabpanel row foobar'></div>", None, None, None),
        ("<div role='tabpanel row range'></div>", None, None, None),
        ("<div role='doc-endnotes range'></div>", None, None, None),
        ("<div role />", None, None, None),
        ("<div role='unknown-invalid-role' />", Some(allowed_invalid_roles()), None, None),
        ("<div role={null}></div>", None, None, None),
        ("<Foo role='datepicker' />", None, None, None),
        ("<Foo role='Button' />", None, None, None),
        ("<Div role='Button' />", None, Some(settings()), None),
        ("<Div role='Button' />", Some(ignore_non_dom_schema()), Some(settings()), None),
        ("<Box asChild='div' role='Button' />", None, None, None),
    ];

    Tester::new(AriaRole::NAME, AriaRole::PLUGIN, pass, fail).test_and_snapshot();
}
