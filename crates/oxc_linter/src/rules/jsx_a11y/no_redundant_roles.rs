use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeValue},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{get_element_type, has_jsx_prop_ignore_case},
};

fn no_redundant_roles_diagnostic(span: Span, element: &str, role: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The `{element}` element has an implicit role of `{role}`. Defining this explicitly is redundant and should be avoided."
    ))
    .with_help(format!("Remove the redundant role `{role}` from the element `{element}`."))
    .with_label(span)
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
    /// - `<body>`: `document`
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <nav role="navigation"></nav>
    /// <button role="button"></button>
    /// <body role="document"></body>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <nav></nav>
    /// <button></button>
    /// <body></body>
    /// ```
    NoRedundantRoles,
    jsx_a11y,
    correctness,
    fix
);

fn get_default_role_exception(tag: &str) -> Option<&'static str> {
    match tag {
        "nav" => Some("navigation"),
        "button" => Some("button"),
        "body" => Some("document"),
        _ => None,
    }
}

impl Rule for NoRedundantRoles {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let component = get_element_type(ctx, jsx_el);

        if let Some(JSXAttributeItem::Attribute(attr)) = has_jsx_prop_ignore_case(jsx_el, "role")
            && let Some(JSXAttributeValue::StringLiteral(role_values)) = &attr.value
        {
            let roles = role_values.value.split_whitespace().collect::<Vec<_>>();
            for role in &roles {
                let exceptions = get_default_role_exception(&component);
                if exceptions.is_some_and(|set| set.contains(role)) {
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
        ("<body />", None, None),
        ("<button role='main' />", None, None),
        ("<MyComponent role='button' />", None, None),
        ("<button role={`${foo}button`} />", None, None),
        ("<Button role={`${foo}button`} />", None, Some(settings())),
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
        ("<nav role='navigation' />", None, None),
        ("<Button role='button' />", None, Some(settings())),
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
        ("<body role='document' />", "<body  />"),
        (
            "<body role='document'><p>Foobarbaz!! document body role</p></body>",
            "<body ><p>Foobarbaz!! document body role</p></body>",
        ),
    ];

    Tester::new(NoRedundantRoles::NAME, NoRedundantRoles::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
