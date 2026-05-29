use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeValue},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{get_element_implicit_roles, get_element_type, has_jsx_prop_ignore_case},
};

fn no_redundant_roles_diagnostic(span: Span, element: &str, role: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The `{element}` element has an implicit role of `{role}`. Defining this explicitly is redundant and should be avoided."
    ))
    .with_help(format!("Remove the redundant role `{role}` from the element `{element}`."))
    .with_label(span)
}

/// Redundant roles that are allowed by default unless overridden by the
/// `allowedRedundantRoles` option, mirroring eslint-plugin-jsx-a11y's
/// `DEFAULT_ROLE_EXCEPTIONS`.
const DEFAULT_ROLE_EXCEPTIONS: &[(&str, &str)] = &[("nav", "navigation")];

#[derive(Debug, Clone, Default, Deserialize)]
pub struct NoRedundantRoles(Box<NoRedundantRolesConfig>);

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub struct NoRedundantRolesConfig {
    /// A mapping of element names to the redundant roles that are explicitly
    /// allowed on them. Providing an entry overrides the default exceptions for
    /// that element.
    #[serde(default, flatten)]
    pub allowed_redundant_roles: FxHashMap<CompactStr, Vec<CompactStr>>,
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
    /// ### Examples
    ///
    /// This rule applies for the following elements and their implicit roles:
    ///
    /// - `<button>`: `button`
    /// - `<main>`: `main`
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <button role="button"></button>
    /// <main role="main"></main>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <button></button>
    /// <main></main>
    /// ```
    ///
    /// ### Options
    ///
    /// This rule takes an object whose keys are element names and whose values
    /// are arrays of redundant roles to allow on that element. Providing an
    /// entry overrides the default exceptions for that element.
    ///
    /// By default `role="navigation"` is allowed on `<nav>`. Additional roles
    /// can be allowed, for example to keep explicit list semantics that some
    /// browsers drop when `list-style: none` is applied:
    ///
    /// ```json
    /// {
    ///   "jsx-a11y/no-redundant-roles": ["error", { "ul": ["list"], "ol": ["list"], "li": ["listitem"] }]
    /// }
    /// ```
    NoRedundantRoles,
    jsx_a11y,
    correctness,
    fix,
    config = NoRedundantRolesConfig,
    version = "0.2.1",
);

impl Rule for NoRedundantRoles {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let component = get_element_type(ctx, jsx_el);
        let implicit_roles = get_element_implicit_roles(&component);

        if let Some(JSXAttributeItem::Attribute(attr)) = has_jsx_prop_ignore_case(jsx_el, "role")
            && let Some(JSXAttributeValue::StringLiteral(role_values)) = &attr.value
        {
            let roles = role_values.value.split_whitespace().collect::<Vec<_>>();
            for role in &roles {
                if implicit_roles.contains(role)
                    && !self.is_allowed_redundant_role(&component, role)
                {
                    ctx.diagnostic_with_fix(
                        no_redundant_roles_diagnostic(attr.span, &component, role),
                        |fixer| fixer.delete_range(attr.span),
                    );
                }
            }
        }
    }
}

impl NoRedundantRoles {
    /// A redundant `role` is allowed when the element has an explicit entry in
    /// the `allowedRedundantRoles` option, or otherwise falls back to the
    /// default exceptions.
    fn is_allowed_redundant_role(&self, element: &str, role: &str) -> bool {
        match self.0.allowed_redundant_roles.get(element) {
            Some(roles) => roles.iter().any(|r| r == role),
            None => DEFAULT_ROLE_EXCEPTIONS.iter().any(|&(el, r)| el == element && r == role),
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn settings() -> serde_json::Value {
        serde_json::json!({
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
        ("<main />", None, None),
        ("<button role='main' />", None, None),
        ("<MyComponent role='button' />", None, None),
        ("<button role={`${foo}button`} />", None, None),
        ("<Button role={`${foo}button`} />", None, Some(settings())),
        (r#"<select role="menu"><option>1</option><option>2</option></select>"#, None, None),
        // `nav` / `navigation` is allowed by default.
        (r#"<nav role="navigation" />"#, None, None),
        // Explicitly allowed redundant roles via the option.
        (r#"<ul role="list" />"#, Some(serde_json::json!([{ "ul": ["list"] }])), None),
        (r#"<ol role="list" />"#, Some(serde_json::json!([{ "ol": ["list"] }])), None),
        (r#"<li role="listitem" />"#, Some(serde_json::json!([{ "li": ["listitem"] }])), None),
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
        ("<Button role='button' />", None, Some(settings())),
        (r#"<article role="article" />"#, None, None),
        (r#"<aside role="complementary" />"#, None, None),
        (r#"<footer role="contentinfo" />"#, None, None),
        (r#"<form role="form" />"#, None, None),
        (r#"<h1 role="heading" />"#, None, None),
        (r#"<h2 role="heading" />"#, None, None),
        (r#"<header role="banner" />"#, None, None),
        (r#"<hr role="separator" />"#, None, None),
        (r#"<img role="img" />"#, None, None),
        (r#"<li role="listitem" />"#, None, None),
        (r#"<main role="main" />"#, None, None),
        (r#"<ol role="list" />"#, None, None),
        (r#"<ul role="list" />"#, None, None),
        (r#"<select role="combobox" />"#, None, None),
        (r#"<select role="listbox" />"#, None, None),
        (r#"<table role="table" />"#, None, None),
        (r#"<tbody role="rowgroup" />"#, None, None),
        (r#"<td role="cell" />"#, None, None),
        (r#"<textarea role="textbox" />"#, None, None),
        (r#"<section role="region" />"#, None, None),
        (r#"<dialog role="dialog" />"#, None, None),
        (r#"<fieldset role="group" />"#, None, None),
        (r#"<figure role="figure" />"#, None, None),
        (r#"<meter role="meter" />"#, None, None),
        (r#"<output role="status" />"#, None, None),
        (r#"<p role="paragraph" />"#, None, None),
        (r#"<progress role="progressbar" />"#, None, None),
        (r#"<tr role="row" />"#, None, None),
        // An option for a different element does not suppress this one.
        (r#"<ul role="list" />"#, Some(serde_json::json!([{ "li": ["listitem"] }])), None),
        // A user-provided option overrides the default exception for that element.
        (r#"<nav role="navigation" />"#, Some(serde_json::json!([{ "nav": [] }])), None),
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
        ("<main role='main' />", "<main  />"),
        (
            "<main role='main'><p>Foobarbaz!!  main role</p></main>",
            "<main ><p>Foobarbaz!!  main role</p></main>",
        ),
    ];

    Tester::new(NoRedundantRoles::NAME, NoRedundantRoles::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
