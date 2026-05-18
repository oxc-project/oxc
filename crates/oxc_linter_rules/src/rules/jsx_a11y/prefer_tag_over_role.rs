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
    utils::{get_element_type, get_tags_for_role, has_jsx_prop_ignore_case},
};

fn prefer_tag_over_role_diagnostic(span: Span, tag: &str, role: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `{tag}` over `role` attribute `{role}`."))
        .with_help(format!("Replace HTML elements with `role` attribute `{role}` to corresponding semantic HTML tag `{tag}`."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferTagOverRole;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces using semantic HTML tags over `role` attribute.
    ///
    /// ### Why is this bad?
    ///
    /// Using semantic HTML tags can improve accessibility and readability of the code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div role="button" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <button />
    /// ```
    PreferTagOverRole,
    jsx_a11y,
    correctness,
    version = "0.1.1",
);

impl PreferTagOverRole {
    fn check_roles<'a>(role_prop: &JSXAttributeItem<'a>, jsx_name: &str, ctx: &LintContext<'a>) {
        if let JSXAttributeItem::Attribute(attr) = role_prop
            && let Some(JSXAttributeValue::StringLiteral(role_values)) = &attr.value
        {
            let roles = role_values.value.split_whitespace();
            for role in roles {
                Self::check_role(role, jsx_name, attr.span, ctx);
            }
        }
    }

    fn check_role(role: &str, jsx_name: &str, span: Span, ctx: &LintContext) {
        let tags = get_tags_for_role(role);
        if !tags.is_empty() && !tags.contains(&jsx_name) {
            let tag = tags.join(", ");
            ctx.diagnostic(prefer_tag_over_role_diagnostic(span, &tag, role));
        }
    }
}

impl Rule for PreferTagOverRole {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXOpeningElement(jsx_el) = node.kind() {
            let name = get_element_type(ctx, jsx_el);
            if let Some(role_prop) = has_jsx_prop_ignore_case(jsx_el, "role") {
                Self::check_roles(role_prop, &name, ctx);
            }
        }
    }
}
#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        "<div />",
        "<div role=\"unknown\" />",
        "<div role=\"also unknown\" />",
        "<other />",
        "<img role=\"img\" />",
        "<input role=\"checkbox\" />",
        "<button role=\"button\" />",
        "<header role=\"banner\" />",
        "<a role=\"link\" />",
        "<area role=\"link\" />",
        "<h1 role=\"heading\" />",
        "<h6 role=\"heading\" />",
        "<tbody role=\"rowgroup\" />",
        "<tfoot role=\"rowgroup\" />",
        "<thead role=\"rowgroup\" />",
        "<select role=\"listbox\" />",
        "<section role=\"region\" />",
        "<input role=\"slider\" />",
        "<input role=\"combobox\" />",
        "<input role=\"radio\" />",
        "<input role=\"textbox\" />",
        "<textarea role=\"textbox\" />",
        "<article role=\"article\" />",
        "<aside role=\"complementary\" />",
        "<footer role=\"contentinfo\" />",
        "<form role=\"form\" />",
        "<hr role=\"separator\" />",
        "<li role=\"listitem\" />",
        "<main role=\"main\" />",
        "<nav role=\"navigation\" />",
        "<ol role=\"list\" />",
        "<ul role=\"list\" />",
        "<menu role=\"list\" />",
        "<table role=\"table\" />",
        "<td role=\"cell\" />",
        "<tr role=\"row\" />",
        "<dialog role=\"dialog\" />",
        "<meter role=\"meter\" />",
        "<output role=\"status\" />",
        "<p role=\"paragraph\" />",
        "<progress role=\"progressbar\" />",
        "<select role=\"combobox\" />",
        "<del role=\"deletion\" />",
        "<s role=\"deletion\" />",
        "<em role=\"emphasis\" />",
        "<strong role=\"strong\" />",
        "<dfn role=\"term\" />",
        "<figure role=\"figure\" />",
        "<fieldset role=\"group\" />",
        "<details role=\"group\" />",
        "<sub role=\"subscript\" />",
        "<sup role=\"superscript\" />",
        "<option role=\"option\" />",
        "<th role=\"columnheader\" />",
        "<input role=\"searchbox\" />",
        "<input role=\"spinbutton\" />",
    ];
    let fail: Vec<&str> = vec![
        r#"<div role="checkbox" />"#,
        r#"<div role="button checkbox" />"#,
        r#"<div role="heading" />"#,
        r#"<div role="link" />"#,
        r#"<div role="rowgroup" />"#,
        r#"<span role="checkbox" />"#,
        r#"<other role="checkbox" />"#,
        r#"<other role="checkbox" />"#,
        r#"<div role="banner" />"#,
        r#"<div role="listbox" />"#,
        r#"<div role="region" />"#,
        r#"<div role="slider" />"#,
        r#"<div role="combobox" />"#,
        r#"<div role="radio" />"#,
        r#"<div role="textbox" />"#,
        r#"<div role="article" />"#,
        r#"<div role="complementary" />"#,
        r#"<div role="contentinfo" />"#,
        r#"<div role="dialog" />"#,
        r#"<div role="form" />"#,
        r#"<div role="list" />"#,
        r#"<div role="listitem" />"#,
        r#"<div role="main" />"#,
        r#"<div role="navigation" />"#,
        r#"<div role="separator" />"#,
        r#"<div role="table" />"#,
        r#"<div role="cell" />"#,
        r#"<div role="row" />"#,
        r#"<div role="meter" />"#,
        r#"<div role="status" />"#,
        r#"<div role="paragraph" />"#,
        r#"<div role="progressbar" />"#,
        r#"<div role="figure" />"#,
        r#"<div role="deletion" />"#,
        r#"<div role="emphasis" />"#,
        r#"<div role="strong" />"#,
        r#"<div role="term" />"#,
        r#"<div role="option" />"#,
        r#"<div role="columnheader" />"#,
        r#"<div role="searchbox" />"#,
        r#"<div role="spinbutton" />"#,
    ];
    Tester::new(PreferTagOverRole::NAME, PreferTagOverRole::PLUGIN, pass, fail).test_and_snapshot();
}
