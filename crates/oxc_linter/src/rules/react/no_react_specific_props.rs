use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_react_specific_props_diagnostic(span: Span, prop: &str, standard: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{prop}` is a React-specific prop."))
        .with_help(format!(
            "Use the standard HTML attribute `{standard}` instead of `{prop}` when not using React."
        ))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoReactSpecificProps;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects usage of React-specific JSX props like `className` and `htmlFor`
    /// that should be replaced with their standard HTML equivalents.
    ///
    /// ### Why is this bad?
    ///
    /// Outside of React, JSX props should use standard HTML attribute names.
    /// `className` and `htmlFor` are React-specific alternatives to `class`
    /// and `for`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div className="foo" />
    /// <label htmlFor="input" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div class="foo" />
    /// <label for="input" />
    /// ```
    NoReactSpecificProps,
    react,
    suspicious,
    pending
);

impl Rule for NoReactSpecificProps {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXAttribute(attr) = node.kind() else {
            return;
        };

        let name = attr.name.as_identifier().map(|ident| ident.name.as_str());

        match name {
            Some("className") => {
                ctx.diagnostic(no_react_specific_props_diagnostic(attr.span, "className", "class"));
            }
            Some("htmlFor") => {
                ctx.diagnostic(no_react_specific_props_diagnostic(attr.span, "htmlFor", "for"));
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![r#"<div class="foo" />"#, r#"<label for="input" />"#, "<div id=\"bar\" />"];

    let fail = vec![r#"<div className="foo" />"#, r#"<label htmlFor="input" />"#];

    Tester::new(NoReactSpecificProps::NAME, NoReactSpecificProps::PLUGIN, pass, fail)
        .test_and_snapshot();
}
