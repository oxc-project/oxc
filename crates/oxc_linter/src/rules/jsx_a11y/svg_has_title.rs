use oxc_ast::{
    AstKind,
    ast::{Expression, JSXAttributeValue, JSXChild, JSXElement, JSXExpression, JSXOpeningElement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::JSStr;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{get_element_type, get_prop_value, has_jsx_prop, is_hidden_from_screen_reader},
};

fn svg_has_title_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("SVG elements must have an accessible name.")
        .with_help("Add a non-empty `title` as the first child, or use `role=\"img\"` with `aria-label` or `aria-labelledby`. Mark decorative SVGs with `aria-hidden=\"true\"`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct SvgHasTitle;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require SVG elements to have an accessible name, following the approach of
    /// [Biome's noSvgWithoutTitle rule](https://biomejs.dev/linter/rules/no-svg-without-title/).
    ///
    /// Provide a non-empty `title` element as the first child, or set `role="img"`
    /// and provide `aria-label` or `aria-labelledby`. Whitespace and JSX comments
    /// before the title are ignored. The ARIA alternative checks for the presence
    /// of a label attribute; it does not resolve the referenced element or validate
    /// the label's value.
    ///
    /// Decorative SVGs with `aria-hidden="true"`, `role="presentation"`, or
    /// `role="none"` do not require a name. Custom components mapped to `svg`
    /// through the `jsx-a11y` settings are also checked.
    ///
    /// ### Why is this bad?
    ///
    /// Screen readers need a text alternative to describe an SVG's content or
    /// purpose. Without a name, users may miss information conveyed by the image.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <svg><circle /></svg>
    /// <svg><title></title><circle /></svg>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <svg><title>Search</title><circle /></svg>
    /// <svg role="img" aria-label="Search"><circle /></svg>
    /// <svg aria-hidden="true"><circle /></svg>
    /// ```
    SvgHasTitle,
    jsx_a11y,
    nursery,
    version = "next",
    short_description = "Require SVG elements to have an accessible name.",
);

impl Rule for SvgHasTitle {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXElement(element) = node.kind() else {
            return;
        };
        let opening = &element.opening_element;
        if get_element_type(ctx, opening) != "svg" {
            return;
        }

        if is_hidden_from_screen_reader(ctx, opening) {
            return;
        }

        match static_role(opening) {
            Some("presentation" | "none") => return,
            Some("img")
                if has_jsx_prop(opening, "aria-label").is_some()
                    || has_jsx_prop(opening, "aria-labelledby").is_some() =>
            {
                return;
            }
            _ => {}
        }

        if has_title(ctx, element) {
            return;
        }

        ctx.diagnostic(svg_has_title_diagnostic(opening.name.span()));
    }
}

fn static_role<'a>(opening: &'a JSXOpeningElement<'_>) -> Option<&'a str> {
    match get_prop_value(has_jsx_prop(opening, "role")?)? {
        JSXAttributeValue::StringLiteral(value) => value.value.as_str(),
        JSXAttributeValue::ExpressionContainer(container) => {
            let Expression::StringLiteral(value) =
                container.expression.as_expression()?.get_inner_expression()
            else {
                return None;
            };
            value.value.as_str()
        }
        _ => None,
    }
}

fn has_title<'a>(ctx: &LintContext<'a>, element: &JSXElement<'a>) -> bool {
    let first_child = element.children.iter().find(|child| match child {
        JSXChild::Text(text) => !text.value.trim().is_empty(),
        JSXChild::ExpressionContainer(container) => {
            !matches!(container.expression, JSXExpression::EmptyExpression(_))
        }
        _ => true,
    });
    let Some(JSXChild::Element(title)) = first_child else {
        return false;
    };

    get_element_type(ctx, &title.opening_element) == "title"
        && !is_hidden_from_screen_reader(ctx, &title.opening_element)
        && title.children.iter().any(|child| has_title_content(ctx, child))
}

fn has_title_content<'a>(ctx: &LintContext<'a>, child: &JSXChild<'a>) -> bool {
    match child {
        JSXChild::Text(text) => !text.value.trim().is_empty(),
        JSXChild::ExpressionContainer(container) => {
            let Some(expression) = container.expression.as_expression() else {
                return false;
            };
            match expression.get_inner_expression() {
                Expression::NullLiteral(_) | Expression::BooleanLiteral(_) => false,
                Expression::StringLiteral(value) => {
                    value.value.as_str().is_none_or(|value| !value.trim().is_empty())
                }
                Expression::TemplateLiteral(template) => template
                    .single_quasi()
                    .and_then(JSStr::as_str)
                    .is_none_or(|value| !value.trim().is_empty()),
                expression => !expression.is_undefined() && !expression.is_void(),
            }
        }
        JSXChild::Element(element) => !is_hidden_from_screen_reader(ctx, &element.opening_element),
        JSXChild::Fragment(fragment) => {
            fragment.children.iter().any(|child| has_title_content(ctx, child))
        }
        JSXChild::Spread(_) => true,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "<div />",
        "<Svg />",
        "<Icons.Svg />",
        "<svg><title>Pass</title><circle /></svg>",
        "<svg>\n  <title>Pass</title>\n  <circle />\n</svg>",
        "<svg>{/* comment */}<title>Pass</title></svg>",
        r#"<svg role="img" aria-labelledby="title"><span id="title">Pass</span></svg>"#,
        r#"<svg role="img" aria-label="title"><span id="title">Pass</span></svg>"#,
        r#"<svg role="graphics-symbol"><title>Pass</title><rect /></svg>"#,
        r#"<svg aria-hidden="true"><rect /></svg>"#,
        "<svg aria-hidden={true} />",
        "<svg aria-hidden />",
        r#"<svg role="img" aria-label=""><span id="">Pass</span></svg>"#,
        r#"<svg role="presentation">foo</svg>"#,
        r#"<svg role="none" />"#,
        r#"<svg role={"presentation"} />"#,
        r#"<svg role={"img"} aria-label={label} />"#,
        "<svg><title>{title}</title></svg>",
        "<svg><title>{'Search'}</title></svg>",
        "<svg><title>{`Search`}</title></svg>",
        "<svg><title>{`${title}`}</title></svg>",
        "<svg><title>{0}</title></svg>",
        "<svg><title><TranslatedTitle /></title></svg>",
        "<svg><title><>Search</></title></svg>",
    ];

    let fail = vec![
        "<svg />",
        "<svg>foo</svg>",
        "<svg><title></title><circle /></svg>",
        "<svg><rect /><rect /><g><title>foo</title><circle /><circle /></g></svg>",
        "<svg><circle /><title>Search</title></svg>",
        "<svg><title /></svg>",
        "<svg><title> \n </title></svg>",
        "<svg><title>{/* comment */}</title></svg>",
        "<svg><title>{''}</title></svg>",
        "<svg><title>{` `}</title></svg>",
        "<svg><title>{null}</title></svg>",
        "<svg><title>{undefined}</title></svg>",
        "<svg><title>{void 0}</title></svg>",
        "<svg><title>{false}</title></svg>",
        "<svg><title>{true}</title></svg>",
        "<svg><title><></></title></svg>",
        "<svg><title aria-hidden>Search</title></svg>",
        "<svg><title><TranslatedTitle aria-hidden /></title></svg>",
        r#"<svg aria-hidden="false" />"#,
        "<svg aria-hidden={false} />",
        r#"<svg role="img" />"#,
        r#"<svg aria-label="Search" />"#,
        r#"<svg role="graphics-symbol" aria-label="Search" />"#,
        "<svg {...props} />",
    ];

    Tester::new(SvgHasTitle::NAME, SvgHasTitle::PLUGIN, pass, fail).test_and_snapshot();
}

#[test]
fn test_settings() {
    use crate::tester::Tester;

    let settings = serde_json::json!({
        "settings": { "jsx-a11y": {
            "components": { "Icon": "svg", "IconTitle": "title" },
            "polymorphicPropName": "as",
        } }
    });
    let pass = vec![
        ("<Icon><IconTitle>Search</IconTitle></Icon>", None, Some(settings.clone())),
        (r#"<Box as="svg"><title>Search</title></Box>"#, None, Some(settings.clone())),
        ("<Icon aria-hidden />", None, Some(settings.clone())),
    ];
    let fail = vec![
        ("<Icon />", None, Some(settings.clone())),
        (r#"<Box as="svg" />"#, None, Some(settings)),
    ];

    Tester::new(SvgHasTitle::NAME, SvgHasTitle::PLUGIN, pass, fail)
        .with_snapshot_suffix("settings")
        .test_and_snapshot();
}
