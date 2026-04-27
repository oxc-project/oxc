use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_deprecated_vue_config_keycodes_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`Vue.config.keyCodes` are deprecated.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDeprecatedVueConfigKeycodes;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow using deprecated `Vue.config.keyCodes` (in Vue.js 3.0.0+).
    ///
    /// ### Why is this bad?
    ///
    /// `Vue.config.keyCodes` was removed in Vue 3. Code that relies on it will
    /// silently stop working when upgrading.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// Vue.config.keyCodes = { enter: 13 }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// Vue.config.silent = true
    /// ```
    NoDeprecatedVueConfigKeycodes,
    vue,
    correctness,
    version = "next",
);

impl Rule for NoDeprecatedVueConfigKeycodes {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::StaticMemberExpression(outer) = node.kind() else {
            return;
        };
        if outer.property.name != "keyCodes" {
            return;
        }

        let middle_expr = outer.object.get_inner_expression();
        let middle = if let Expression::ChainExpression(chain) = middle_expr {
            chain.expression.as_member_expression()
        } else {
            middle_expr.as_member_expression()
        };
        let Some(middle) = middle else { return };

        if middle.static_property_name() != Some("config") {
            return;
        }

        let Expression::Identifier(ident) = middle.object().get_inner_expression() else {
            return;
        };
        if ident.name != "Vue" {
            return;
        }

        ctx.diagnostic(no_deprecated_vue_config_keycodes_diagnostic(outer.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        ("Vue.config.silent = true", None, None, Some(PathBuf::from("test.js"))),
        ("config.keyCodes = {}", None, None, Some(PathBuf::from("test.js"))),
        ("V.config.keyCodes = {}", None, None, Some(PathBuf::from("test.js"))),
    ];

    let fail = vec![
        ("Vue.config.keyCodes = {}", None, None, Some(PathBuf::from("test.js"))),
        ("Vue?.config?.keyCodes", None, None, Some(PathBuf::from("test.js"))),
        ("(Vue?.config)?.keyCodes", None, None, Some(PathBuf::from("test.js"))),
    ];

    Tester::new(
        NoDeprecatedVueConfigKeycodes::NAME,
        NoDeprecatedVueConfigKeycodes::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
