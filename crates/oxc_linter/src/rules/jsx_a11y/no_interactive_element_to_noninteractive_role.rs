use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{AstKind, ast::JSXAttributeValue};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;

use crate::{
    AstNode,
    context::LintContext,
    globals::HTML_TAG,
    rule::Rule,
    utils::{
        get_element_type, has_jsx_prop_ignore_case, is_interactive_element, is_non_interactive_role,
    },
};

fn no_interactive_element_to_noninteractive_role_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Interactive elements should not be assigned non-interactive roles.")
        .with_help("WAI-ARIA roles should not be used to convert an interactive element to a non-interactive element. Wrap the element or use a different structure.")
        .with_label(span)
}

#[derive(Debug, Clone)]
pub struct NoInteractiveElementToNoninteractiveRole(
    Box<NoInteractiveElementToNoninteractiveRoleConfig>,
);

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NoInteractiveElementToNoninteractiveRoleConfig {
    /// A mapping of HTML element names to arrays of ARIA role strings that are
    /// allowed overrides for that element.
    #[serde(default)]
    pub allowed_roles: FxHashMap<CompactStr, Vec<CompactStr>>,
}

impl Default for NoInteractiveElementToNoninteractiveRole {
    fn default() -> Self {
        let mut allowed_roles = FxHashMap::default();
        allowed_roles.insert(
            CompactStr::new("tr"),
            vec![CompactStr::new("none"), CompactStr::new("presentation")],
        );
        Self(Box::new(NoInteractiveElementToNoninteractiveRoleConfig { allowed_roles }))
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Interactive HTML elements indicate controls in the user interface. Interactive elements include `<a href>`, `<button>`, `<input>`, `<select>`, `<textarea>`.
    ///
    /// WAI-ARIA roles should not be used to convert an interactive element to a non-interactive element.
    /// Non-interactive ARIA roles include `article`, `banner`, `complementary`, `img`, `listitem`, `main`, `region` and `tooltip`.
    ///
    /// ### Why is this bad?
    ///
    /// Using a non-interactive role on an interactive element can confuse assistive technology users.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <button role="img">Save</button>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div role="img"><button>Save</button></div>
    /// ```
    NoInteractiveElementToNoninteractiveRole,
    jsx_a11y,
    correctness,
    config = NoInteractiveElementToNoninteractiveRoleConfig,
    version = "next"
);

impl Rule for NoInteractiveElementToNoninteractiveRole {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let Some(config_val) = value.get(0) else {
            return Ok(Self::default());
        };

        let mut allowed_roles = FxHashMap::default();
        if let Some(obj) = config_val.as_object() {
            for (element, roles_value) in obj {
                if let Some(roles_arr) = roles_value.as_array() {
                    let roles: Vec<CompactStr> =
                        roles_arr.iter().filter_map(|v| v.as_str().map(CompactStr::new)).collect();
                    allowed_roles.insert(CompactStr::new(element), roles);
                }
            }
        }

        Ok(Self(Box::new(NoInteractiveElementToNoninteractiveRoleConfig { allowed_roles })))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let element_type = get_element_type(ctx, jsx_el);

        // Only check known HTML tags
        if !HTML_TAG.contains(element_type.as_ref()) {
            return;
        }

        // --- FIXED LOGIC START ---
        let is_interactive = is_interactive_element(&element_type, jsx_el);
        let is_tr = element_type.as_ref() == "tr";

        // This rule applies to interactive elements OR the special <tr> case
        if !is_interactive && !is_tr {
            return;
        }
        // --- FIXED LOGIC END ---

        // 2. Find the 'role' attribute using the correct Oxc helper
        let Some(role_attr_item) = has_jsx_prop_ignore_case(jsx_el, "role") else {
            return;
        };

        let oxc_ast::ast::JSXAttributeItem::Attribute(role_attr) = role_attr_item else {
            return;
        };

        // 3. Extract the role value
        let Some(JSXAttributeValue::StringLiteral(role_value)) = &role_attr.value else {
            return;
        };

        let role_str = role_value.value.as_str().trim();
        let Some(first_role) = role_str.split_whitespace().next() else {
            return;
        };

        // 4. Check if the role is allowed by configuration (e.g., tr: ["presentation"])
        if let Some(allowed) = self.0.allowed_roles.get(element_type.as_ref())
            && allowed.iter().any(|r| r.as_str() == first_role)
        {
            return;
        }

        // 5. Logic check: Is it a non-interactive role OR a presentation/none role?
        if is_non_interactive_role(first_role) || matches!(first_role, "presentation" | "none") {
            ctx.diagnostic(no_interactive_element_to_noninteractive_role_diagnostic(
                role_attr.span,
            ));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn components_settings() -> serde_json::Value {
        serde_json::json!({
            "settings": { "jsx-a11y": {
                "components": {
                    "Link": "a",
                    "Button": "button",
                }
            } }
        })
    }

    let pass = vec![
        // Interactive elements with interactive roles (Allowed)
        (r#"<a href="http://x.y.z" role="button" />"#, None, None),
        (r#"<a href="http://x.y.z" role="menuitem" />"#, None, None),
        (r#"<button role="button" />"#, None, None),
        (r#"<button role="menuitem" />"#, None, None),
        (r#"<input type="text" role="textbox" />"#, None, None),
        (r#"<input type="checkbox" role="checkbox" />"#, None, None),
        // Non-interactive elements (Ignored by this rule)
        (r#"<div role="img" />"#, None, None),
        (r#"<a role="img" />"#, None, None), // No href = non-interactive
        // Recommended config: tr is allowed to have presentation/none
        (r#"<tr role="presentation" />"#, None, None),
        (r#"<tr role="none" />"#, None, None),
        // Custom component mapped via settings to an interactive element with a correct role
        (r#"<Link href="http://x.y.z" role="button" />"#, None, Some(components_settings())),
    ];

    let fail = vec![
        // Interactive elements with non-interactive roles
        (r#"<button role="img" />"#, None, None),
        (r#"<button role="article" />"#, None, None),
        (r#"<button role="banner" />"#, None, None),
        (r#"<button role="complementary" />"#, None, None),
        (r#"<button role="listitem" />"#, None, None),
        (r#"<button role="main" />"#, None, None),
        (r#"<button role="region" />"#, None, None),
        (r#"<button role="tooltip" />"#, None, None),
        // Anchors with href
        (r#"<a href="h" role="img" />"#, None, None),
        (r#"<a href="h" role="article" />"#, None, None),
        // Inputs
        (r#"<input type="text" role="img" />"#, None, None),
        (r#"<input type="radio" role="img" />"#, None, None),
        // Mapped components via settings
        (r#"<Link href="h" role="img" />"#, None, Some(components_settings())),
        (r#"<Button role="img" />"#, None, Some(components_settings())),
        // Strict mode test: tr should fail if the allowed_roles option is empty
        (r#"<tr role="presentation" />"#, Some(serde_json::json!([{}])), None),
    ];

    Tester::new(
        NoInteractiveElementToNoninteractiveRole::NAME,
        NoInteractiveElementToNoninteractiveRole::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
