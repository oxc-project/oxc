use cow_utils::CowUtils;
use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeValue, JSXExpression, JSXOpeningElement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        get_element_type, get_prop_value, get_string_literal_prop_value, has_jsx_prop_ignore_case,
        parse_jsx_value,
    },
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

#[derive(Debug, Clone, JsonSchema)]
#[serde(default)]
pub struct NoRedundantRolesConfig {
    /// A map of element names to arrays of roles that are allowed to be redundant.
    /// For example, `{ "nav": ["navigation"] }` allows `<nav role="navigation">`.
    allowed_redundant_roles: FxHashMap<String, Vec<String>>,
}

impl Default for NoRedundantRolesConfig {
    fn default() -> Self {
        let mut allowed = FxHashMap::default();
        allowed.insert("nav".into(), vec!["navigation".into()]);
        Self { allowed_redundant_roles: allowed }
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
    /// ### Configuration
    ///
    /// This rule accepts a configuration object mapping element names to arrays
    /// of roles that are allowed to be explicitly set even though they match
    /// the element's implicit role.
    ///
    /// By default, `{ "nav": ["navigation"] }` is allowed.
    ///
    /// ```json
    /// {
    ///   "jsx-a11y/no-redundant-roles": ["error", { "nav": ["navigation"], "ul": ["list"] }]
    /// }
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <button role="button"></button>
    /// <body role="document"></body>
    /// <h1 role="heading"></h1>
    /// <article role="article"></article>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <nav></nav>
    /// // `nav` with `navigation` is allowed by default config
    /// <nav role="navigation"></nav>
    /// <button></button>
    /// <body></body>
    /// ```
    NoRedundantRoles,
    jsx_a11y,
    correctness,
    fix,
    config = NoRedundantRolesConfig,
);

/// Check if a JSX attribute has a truthy literal value.
///
/// Valueless attributes (e.g., `<select multiple>`) are treated as `true`.
fn is_truthy_prop(item: &JSXAttributeItem<'_>) -> bool {
    match get_prop_value(item) {
        // Valueless attribute: <select multiple> → true
        None => true,
        Some(JSXAttributeValue::StringLiteral(s)) => !s.value.is_empty(),
        Some(JSXAttributeValue::ExpressionContainer(container)) => match &container.expression {
            JSXExpression::BooleanLiteral(b) => b.value,
            JSXExpression::NumericLiteral(n) => n.value != 0.0 && !n.value.is_nan(),
            JSXExpression::StringLiteral(s) => !s.value.is_empty(),
            _ => false,
        },
        _ => false,
    }
}

fn get_implicit_role<'a>(
    node: &'a JSXOpeningElement<'a>,
    element_type: &str,
) -> Option<&'static str> {
    let implicit_role = match element_type {
        "a" | "area" | "link" => match has_jsx_prop_ignore_case(node, "href") {
            Some(_) => "link",
            None => return None,
        },
        "article" => "article",
        "aside" => "complementary",
        "body" => "document",
        "button" => "button",
        "datalist" => "listbox",
        "details" => "group",
        "dialog" => "dialog",
        "form" => "form",
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => "heading",
        "hr" => "separator",
        "img" => {
            // <img alt=""> → no implicit role (per ESLint)
            if has_jsx_prop_ignore_case(node, "alt")
                .and_then(get_string_literal_prop_value)
                .is_some_and(str::is_empty)
            {
                return None;
            }
            // <img src="foo.svg"> → no implicit role (WebKit SVG workaround)
            if has_jsx_prop_ignore_case(node, "src")
                .and_then(get_string_literal_prop_value)
                .is_some_and(|s| s.contains(".svg"))
            {
                return None;
            }
            "img"
        }
        "input" => has_jsx_prop_ignore_case(node, "type").map_or("textbox", |input_type| {
            match get_string_literal_prop_value(input_type) {
                Some("button" | "image" | "reset" | "submit") => "button",
                Some("checkbox") => "checkbox",
                Some("radio") => "radio",
                Some("range") => "slider",
                _ => "textbox",
            }
        }),
        "li" => "listitem",
        "menu" => {
            // <menu type="toolbar"> → "toolbar", otherwise no implicit role
            return has_jsx_prop_ignore_case(node, "type").and_then(|v| {
                get_string_literal_prop_value(v).and_then(|v| {
                    if v.eq_ignore_ascii_case("toolbar") { Some("toolbar") } else { None }
                })
            });
        }
        "menuitem" => {
            return has_jsx_prop_ignore_case(node, "type").and_then(|v| {
                match get_string_literal_prop_value(v) {
                    Some("checkbox") => Some("menuitemcheckbox"),
                    Some("command") => Some("menuitem"),
                    Some("radio") => Some("menuitemradio"),
                    _ => None,
                }
            });
        }
        "meter" | "progress" => "progressbar",
        "nav" => "navigation",
        "ol" | "ul" => "list",
        "option" => "option",
        "output" => "status",
        "section" => "region",
        "select" => {
            // <select multiple> or <select size={2+}> → "listbox"
            // otherwise → "combobox"
            if has_jsx_prop_ignore_case(node, "multiple").is_some_and(is_truthy_prop) {
                return Some("listbox");
            }
            if has_jsx_prop_ignore_case(node, "size")
                .and_then(get_prop_value)
                .is_some_and(|v| parse_jsx_value(v).is_ok_and(|n| n > 1.0))
            {
                return Some("listbox");
            }
            "combobox"
        }
        "tbody" | "tfoot" | "thead" => "rowgroup",
        "textarea" => "textbox",
        _ => return None,
    };

    Some(implicit_role)
}

impl Rule for NoRedundantRoles {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let mut config = NoRedundantRolesConfig::default();

        if let Some(obj) = value.get(0).and_then(serde_json::Value::as_object) {
            for (element, roles) in obj {
                if let Some(arr) = roles.as_array() {
                    let role_list: Vec<String> =
                        arr.iter().filter_map(|v| v.as_str().map(String::from)).collect();
                    config.allowed_redundant_roles.insert(element.clone(), role_list);
                }
            }
        }

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

        let explicit_role = role_values.value.trim().cow_to_ascii_lowercase();

        if explicit_role != implicit_role {
            return;
        }

        if self
            .allowed_redundant_roles
            .get(&*component)
            .is_some_and(|allowed_roles| allowed_roles.iter().any(|r| r == &explicit_role))
        {
            return;
        }

        ctx.diagnostic_with_fix(
            no_redundant_roles_diagnostic(attr.span, &component, &explicit_role),
            |fixer| fixer.delete_range(attr.span),
        );
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
        // select: role that doesn't match implicit role
        ("<select role='menu'><option>1</option><option>2</option></select>", None, None),
        ("<select role='menu' size={2}><option>1</option><option>2</option></select>", None, None),
        ("<select role='menu' multiple><option>1</option><option>2</option></select>", None, None),
        // img: SVG src has no implicit role
        ("<img src='example.svg' role='img' />", None, None),
        // img: empty alt has no implicit role
        ("<img alt='' role='presentation' />", None, None),
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
