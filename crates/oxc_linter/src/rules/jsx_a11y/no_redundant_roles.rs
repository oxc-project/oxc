use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeValue, JSXOpeningElement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{get_element_implicit_roles, get_element_type, has_jsx_prop_ignore_case},
};

fn no_redundant_roles_diagnostic(span: Span, element: &str, role: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The `{element}` element has an implicit role of `{role}`. Defining this explicitly is redundant and should be avoided."
    ))
    .with_help(format!("Remove the redundant role `{role}` from the element `{element}`."))
    .with_label(span)
}

/// Returns the concrete implicit roles for an `<input>` element based on its
/// actual `type` and `list` attributes, as defined in HTML-AAM.
///
/// The static `ELEMENT_ROLE_MAP` lists *all possible* input roles; this
/// function narrows them to the one that applies to this specific instance.
fn get_input_implicit_roles(jsx_el: &JSXOpeningElement) -> Vec<&'static str> {
    // An input with a `list` attribute pointing at a <datalist> has role combobox.
    // ref: https://www.w3.org/TR/html-aam-1.0/#el-input-textetc-autocomplete
    if has_jsx_prop_ignore_case(jsx_el, "list").is_some() {
        return vec!["combobox"];
    }

    let input_type = has_jsx_prop_ignore_case(jsx_el, "type").and_then(|item| {
        if let JSXAttributeItem::Attribute(attr) = item {
            if let Some(JSXAttributeValue::StringLiteral(s)) = &attr.value {
                return Some(s.value.to_ascii_lowercase());
            }
        }
        None
    });

    // ref: https://www.w3.org/TR/html-aam-1.0/#el-input-text
    match input_type.as_deref() {
        Some("button") | Some("image") | Some("reset") | Some("submit") => vec!["button"],
        Some("checkbox") => vec!["checkbox"],
        Some("radio") => vec!["radio"],
        Some("range") => vec!["slider"],
        Some("number") => vec!["spinbutton"],
        Some("search") => vec!["searchbox"],
        // type=hidden and type=color have no corresponding ARIA role
        Some("hidden") | Some("color") => vec![],
        // type=text, email, tel, url, password, or absent → textbox
        _ => vec!["textbox"],
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoRedundantRoles;

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
    /// - `<nav>`: `navigation`
    /// - `<button>`: `button`
    /// - `<main>`: `main`
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <nav role="navigation"></nav>
    /// <button role="button"></button>
    /// <main role="main"></main>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <nav></nav>
    /// <button></button>
    /// <main></main>
    /// ```
    NoRedundantRoles,
    jsx_a11y,
    correctness,
    fix,
    version = "0.2.1",
);

impl Rule for NoRedundantRoles {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let component = get_element_type(ctx, jsx_el);

        // <header> and <footer> have ancestor-dependent implicit roles:
        // they are "banner"/"contentinfo" only when NOT inside article/aside/main/nav/section.
        // Without ancestor traversal we cannot determine the correct role, so we skip
        // these elements to avoid false positives — matching eslint-plugin-jsx-a11y behaviour.
        // ref: https://www.w3.org/TR/html-aria/
        if matches!(component.as_ref(), "header" | "footer") {
            return;
        }

        let implicit_roles = if component == "input" {
            // <input> implicit role depends on `type` and `list` attributes.
            // The static map lists all possible roles; narrow to the actual one.
            get_input_implicit_roles(jsx_el)
        } else {
            get_element_implicit_roles(&component)
        };

        if let Some(JSXAttributeItem::Attribute(attr)) = has_jsx_prop_ignore_case(jsx_el, "role")
            && let Some(JSXAttributeValue::StringLiteral(role_values)) = &attr.value
        {
            let roles = role_values.value.split_whitespace().collect::<Vec<_>>();
            for role in &roles {
                if implicit_roles.contains(role) {
                    ctx.diagnostic_with_fix(
                        no_redundant_roles_diagnostic(attr.span, &component, role),
                        |fixer| fixer.delete_range(attr.span),
                    );
                }
            }
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
        // header/footer have ancestor-dependent implicit roles; should not flag
        (r#"<header role="banner" />"#, None, None),
        (r#"<footer role="contentinfo" />"#, None, None),
        // <input role="combobox"> without a list attribute is not redundant;
        // the implicit role of a plain text input is textbox, not combobox
        (r#"<input role="combobox" />"#, None, None),
        (r#"<input role="combobox" type="text" />"#, None, None),
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
        ("<nav role='navigation' />", None, None),
        ("<Button role='button' />", None, Some(settings())),
        (r#"<article role="article" />"#, None, None),
        (r#"<aside role="complementary" />"#, None, None),
        (r#"<form role="form" />"#, None, None),
        (r#"<h1 role="heading" />"#, None, None),
        (r#"<h2 role="heading" />"#, None, None),
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
        // input: role matches the role implied by actual attributes
        (r#"<input role="textbox" />"#, None, None),
        (r#"<input list="datalist-id" role="combobox" />"#, None, None),
        (r#"<input type="checkbox" role="checkbox" />"#, None, None),
        (r#"<input type="radio" role="radio" />"#, None, None),
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
        ("<nav role='navigation' />", "<nav  />"),
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
