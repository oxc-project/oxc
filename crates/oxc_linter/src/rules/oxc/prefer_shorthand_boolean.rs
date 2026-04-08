use oxc_ast::AstKind;
use oxc_ast::ast::JSXAttributeItem;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_shorthand_boolean_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use shorthand boolean attribute: `{name}` instead of `{name}={{true}}`"))
        .with_help("Boolean JSX attributes can use the shorthand form. `<Comp disabled />` is equivalent to `<Comp disabled={true} />`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferShorthandBoolean;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the shorthand form for boolean JSX attributes.
    ///
    /// ### Why is this bad?
    ///
    /// `<Component disabled={true} />` is more verbose than the equivalent
    /// `<Component disabled />`. The shorthand form is idiomatic React.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <Component disabled={true} />
    /// <Input readOnly={true} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <Component disabled />
    /// <Input readOnly />
    /// <Component active={false} />
    /// <Component active={isActive} />
    /// ```
    PreferShorthandBoolean,
    oxc,
    style,
    conditional_fix
);

impl Rule for PreferShorthandBoolean {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(element) = node.kind() else {
            return;
        };

        for attr in &element.attributes {
            let JSXAttributeItem::Attribute(jsx_attr) = attr else {
                continue;
            };

            let Some(value) = &jsx_attr.value else {
                continue;
            };

            let oxc_ast::ast::JSXAttributeValue::ExpressionContainer(container) = value else {
                continue;
            };

            let oxc_ast::ast::JSXExpression::BooleanLiteral(lit) = &container.expression else {
                continue;
            };

            if !lit.value {
                continue;
            }

            let Some(ident) = jsx_attr.name.as_identifier() else {
                continue;
            };
            let name = ident.name.to_string();
            let attr_span = jsx_attr.span();
            ctx.diagnostic_with_fix(
                prefer_shorthand_boolean_diagnostic(attr_span, &name),
                |fixer| fixer.replace(attr_span, name),
            );
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "<Component disabled />",
        "<Input readOnly />",
        "<Component active={false} />",
        "<Component active={isActive} />",
        r#"<Component name="test" />"#,
    ];

    let fail = vec!["<Component disabled={true} />", "<Input readOnly={true} />"];

    let fix = vec![
        ("<Component disabled={true} />", "<Component disabled />", None),
        ("<Input readOnly={true} />", "<Input readOnly />", None),
    ];

    Tester::new(PreferShorthandBoolean::NAME, PreferShorthandBoolean::PLUGIN, pass, fail)
        .expect_fix(fix)
        .change_rule_path_extension("tsx")
        .test_and_snapshot();
}
