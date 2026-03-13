use cow_utils::CowUtils;
use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeValue},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    globals::VALID_ARIA_ROLES,
    rule::{DefaultRuleConfig, Rule},
    utils::{get_element_type, get_implicit_role, has_jsx_prop_ignore_case},
};

fn no_redundant_roles_diagnostic(span: Span, element: &str, role: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The `{element}` element has an implicit role of `{role}`. Defining this explicitly is redundant and should be avoided."
    ))
    .with_help(format!("Remove the redundant role `{role}` from the element `{element}`."))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoRedundantRoles(Box<NoRedundantRolesConfig>);

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct NoRedundantRolesConfig(
    /// A map of element names to arrays of roles that are allowed to be redundant.
    /// For example, `{ "nav": ["navigation"] }` allows `<nav role="navigation">`.
    FxHashMap<String, Vec<String>>,
);

impl Default for NoRedundantRolesConfig {
    fn default() -> Self {
        let mut allowed = FxHashMap::default();
        allowed.insert("nav".into(), vec!["navigation".into()]);
        Self(allowed)
    }
}

impl NoRedundantRolesConfig {
    fn extend(&mut self, other: Self) {
        self.0.extend(other.0);
    }

    fn is_allowed(&self, element: &str, role: &str) -> bool {
        self.0.get(element).is_some_and(|allowed_roles| allowed_roles.iter().any(|r| r == role))
    }

    fn normalize(self) -> Self {
        let allowed_redundant_roles = self
            .0
            .into_iter()
            .filter_map(|(element, roles)| {
                let element = element.trim().cow_to_ascii_lowercase().into_owned();
                if element.is_empty() {
                    return None;
                }

                let roles = roles
                    .into_iter()
                    .map(|role| role.trim().cow_to_ascii_lowercase().into_owned())
                    .filter(|role| !role.is_empty())
                    .collect();

                Some((element, roles))
            })
            .collect();

        Self(allowed_redundant_roles)
    }
}

impl std::ops::Deref for NoRedundantRoles {
    type Target = NoRedundantRolesConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that code does not include a redundant `role` property, in the
    /// case that it's identical to the implicit `role` property of the
    /// element type.
    ///
    /// ### Why is this bad?
    ///
    /// Redundant roles can lead to confusion and verbosity in the codebase.
    ///
    /// ### Options
    ///
    /// The options are provided as an object keyed by HTML element name;
    /// the value is an array of implicit ARIA roles that are allowed on
    /// the specified element.
    ///
    /// The default options allow an implicit role of `navigation` on a
    /// `nav` element as is
    /// [advised by W3](https://www.w3.org/WAI/GL/wiki/Using_HTML5_nav_element#Example:The_.3Cnav.3E_element).
    ///
    /// ```json
    /// {
    ///   "jsx-a11y/no-redundant-roles": ["error", { "nav": ["navigation"] }]
    /// }
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <button role="button" />
    /// <body role="document"></body>
    /// <img role="img" src="foo.jpg" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div />
    /// <button></button>
    /// <body></body>
    /// <button role="presentation" />
    /// <MyComponent role="main" />
    /// ```
    NoRedundantRoles,
    jsx_a11y,
    correctness,
    fix,
    config = NoRedundantRolesConfig,
);

impl Rule for NoRedundantRoles {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let user_config =
            serde_json::from_value::<DefaultRuleConfig<NoRedundantRolesConfig>>(value)
                .map(DefaultRuleConfig::into_inner)?
                .normalize();
        let mut config = NoRedundantRolesConfig::default();
        config.extend(user_config);

        Ok(Self(Box::new(config)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let component = get_element_type(ctx, jsx_el);

        let Some(implicit_role) = get_implicit_role(jsx_el, &component) else {
            return;
        };

        let Some(JSXAttributeItem::Attribute(attr)) = has_jsx_prop_ignore_case(jsx_el, "role")
        else {
            return;
        };

        let Some(JSXAttributeValue::StringLiteral(role_values)) = &attr.value else {
            return;
        };

        let role_tokens: Vec<&str> = role_values.value.split_whitespace().collect();
        let Some(explicit_role) = role_tokens
            .iter()
            .map(|role| role.cow_to_ascii_lowercase())
            .find(|role| VALID_ARIA_ROLES.contains(role.as_ref()))
        else {
            return;
        };

        if explicit_role != implicit_role {
            return;
        }

        if self.is_allowed(&component, &explicit_role) {
            return;
        }

        if role_tokens.len() == 1 {
            ctx.diagnostic_with_fix(
                no_redundant_roles_diagnostic(attr.span, &component, &explicit_role),
                |fixer| fixer.delete_range(attr.span),
            );
        } else {
            ctx.diagnostic(no_redundant_roles_diagnostic(attr.span, &component, &explicit_role));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    fn settings() -> serde_json::Value {
        json!({
            "settings": { "jsx-a11y": {
                "components": {
                    "Button": "button",
                }
            } }
        })
    }

    let pass = vec![
        ("<div />", None, None),
        ("<button />", None, None),
        ("<button></button>", None, None),
        ("<button>Foo</button>", None, None),
        ("<button>role</button>", None, None),
        ("<nav />", None, None),
        ("<body />", None, None),
        ("<button role='main' />", None, None),
        ("<MyComponent role='button' />", None, None),
        ("<button role={`${foo}button`} />", None, None),
        ("<Button role={`${foo}button`} />", None, Some(settings())),
        // Default exception: nav + navigation is allowed
        ("<nav role='navigation' />", None, None),
        // Config-based exceptions
        ("<ul role='list' />", Some(json!([{ "ul": ["list"] }])), None),
        ("<ol role='list' />", Some(json!([{ "ol": ["list"] }])), None),
        ("<ul role='LIST' />", Some(json!([{ "ul": [" List "] }])), None),
        // select: role that doesn't match implicit role
        ("<select role='menu'><option>1</option><option>2</option></select>", None, None),
        ("<select role='menu' size={2}><option>1</option><option>2</option></select>", None, None),
        ("<select role='menu' multiple><option>1</option><option>2</option></select>", None, None),
        // img: SVG src has no implicit role
        ("<img src='example.svg' role='img' />", None, None),
        // img: empty alt has no implicit role
        ("<img alt='' role='presentation' />", None, None),
        // The first supported token is not redundant.
        ("<button role='presentation button' />", None, None),
    ];

    let fail = vec![
        ("<button role='button' />", None, None),
        ("<button role='button' data-foo='bar' />", None, None),
        ("<button role='button' data-role='bar' />", None, None),
        ("<button data-role='bar' role='button' />", None, None),
        ("<button role='button'></button>", None, None),
        ("<button role='button'>Foo</button>", None, None),
        ("<button role='button'><p>Test</p></button>", None, None),
        ("<button role='button' title='button'></button>", None, None),
        ("<body role='document' />", None, None),
        // Case insensitive: DOCUMENT matches implicit "document"
        ("<body role='DOCUMENT' />", None, None),
        // The first supported token is redundant, even with fallback tokens.
        ("<button role='button presentation' />", None, None),
        ("<Button role='button' />", None, Some(settings())),
        // Expanded implicit role detection
        ("<article role='article' />", None, None),
        ("<h1 role='heading' />", None, None),
        ("<ul role='list' />", None, None),
        ("<li role='listitem' />", None, None),
        ("<aside role='complementary' />", None, None),
        ("<textarea role='textbox' />", None, None),
        ("<hr role='separator' />", None, None),
        ("<dialog role='dialog' />", None, None),
        ("<form role='form' />", None, None),
        ("<option role='option' />", None, None),
        ("<output role='status' />", None, None),
        ("<section role='region' />", None, None),
        ("<details role='group' />", None, None),
        // select: combobox (default, no multiple, no size > 1)
        ("<select role='combobox'><option>1</option><option>2</option></select>", None, None),
        ("<select role='combobox' size='' />", None, None),
        ("<select role='combobox' size={1} />", None, None),
        ("<select role='combobox' size='1' />", None, None),
        ("<select role='combobox' size={null} />", None, None),
        ("<select role='combobox' size={undefined} />", None, None),
        ("<select role='combobox' multiple={undefined} />", None, None),
        ("<select role='combobox' multiple={false} />", None, None),
        ("<select role='combobox' multiple='' />", None, None),
        // select: listbox (multiple or size > 1)
        ("<select role='listbox' size='3' />", None, None),
        ("<select role='listbox' size={2} />", None, None),
        (
            "<select role='listbox' multiple><option>1</option><option>2</option></select>",
            None,
            None,
        ),
        ("<select role='listbox' multiple={true} />", None, None),
        // img without alt or SVG src
        ("<img role='img' />", None, None),
        ("<img src={someVariable} role='img' />", None, None),
        // Config override: remove default nav exception
        ("<nav role='navigation' />", Some(json!([{ "nav": [] }])), None),
    ];

    let fix = vec![
        ("<button role='button' />", "<button  />"),
        ("<button role='button'>Foo</button>", "<button >Foo</button>"),
        ("<button role='button' data-role='bar' />", "<button  data-role='bar' />"),
        ("<button data-role='bar' role='button' />", "<button data-role='bar'  />"),
        (
            "<button role='button'>
              Foo
             </button>",
            "<button >
              Foo
             </button>",
        ),
        ("<body role='document' />", "<body  />"),
        (
            "<body role='document'><p>Foobarbaz!! document body role</p></body>",
            "<body ><p>Foobarbaz!! document body role</p></body>",
        ),
        ("<article role='article' />", "<article  />"),
        ("<h1 role='heading' />", "<h1  />"),
        ("<ul role='list' />", "<ul  />"),
        ("<textarea role='textbox' />", "<textarea  />"),
        ("<img role='img' />", "<img  />"),
        (
            "<select role='combobox'><option>1</option></select>",
            "<select ><option>1</option></select>",
        ),
    ];

    Tester::new(NoRedundantRoles::NAME, NoRedundantRoles::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
