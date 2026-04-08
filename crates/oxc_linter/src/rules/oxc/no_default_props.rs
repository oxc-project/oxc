use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_default_props_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("defaultProps is deprecated")
        .with_help("Use JavaScript default parameters instead of defaultProps. defaultProps is deprecated since React 18.3.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDefaultProps;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects usage of `Component.defaultProps`.
    ///
    /// ### Why is this bad?
    ///
    /// `defaultProps` is deprecated since React 18.3 and will be removed in
    /// a future major version. Use JavaScript default parameters instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// Component.defaultProps = { color: "red" };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function Component({ color = "red" }) { return <div />; }
    /// ```
    NoDefaultProps,
    oxc,
    restriction,
    none
);

impl Rule for NoDefaultProps {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentExpression(assign) = node.kind() else {
            return;
        };

        let oxc_ast::ast::AssignmentTarget::StaticMemberExpression(member) = &assign.left else {
            return;
        };

        if member.property.name == "defaultProps" {
            ctx.diagnostic(no_default_props_diagnostic(member.property.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function Component({ color = 'red' }) { return <div />; }",
        "Component.displayName = 'Component';",
        "Component.propTypes = {};",
    ];

    let fail = vec!["Component.defaultProps = { color: 'red' };", "MyButton.defaultProps = {};"];

    Tester::new(NoDefaultProps::NAME, NoDefaultProps::PLUGIN, pass, fail)
        .change_rule_path_extension("tsx")
        .test_and_snapshot();
}
