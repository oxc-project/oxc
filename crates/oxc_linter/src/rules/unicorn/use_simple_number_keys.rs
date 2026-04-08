use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn use_simple_number_keys_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use a simpler number as a property key.")
        .with_help("Simplify the numeric property key (e.g., use `1` instead of `1.0`).")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct UseSimpleNumberKeys;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of simple number literals as object property keys.
    ///
    /// ### Why is this bad?
    ///
    /// Numeric property keys with unnecessary decimal points or leading zeros
    /// are confusing. `{ 1.0: 'foo' }` should be `{ 1: 'foo' }`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const obj = { 1.0: 'foo' };
    /// const obj = { 0x1: 'foo' };
    /// const obj = { 0o1: 'foo' };
    /// const obj = { 0b1: 'foo' };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const obj = { 1: 'foo' };
    /// const obj = { 'key': 'foo' };
    /// ```
    UseSimpleNumberKeys,
    unicorn,
    style,
    pending
);

impl Rule for UseSimpleNumberKeys {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NumericLiteral(num) = node.kind() else {
            return;
        };

        // Check if this numeric literal is used as a property key
        let parent = ctx.nodes().parent_node(node.id());
        let is_property_key = matches!(parent.kind(), AstKind::ObjectProperty(prop) if {
            matches!(&prop.key, oxc_ast::ast::PropertyKey::NumericLiteral(k) if k.span == num.span)
        });

        if !is_property_key {
            return;
        }

        let raw = ctx.source_range(num.span);

        // Check if the raw representation is not the simplest form
        if raw.contains('.')
            || raw.contains('x')
            || raw.contains('X')
            || raw.contains('o')
            || raw.contains('O')
            || raw.contains('b')
            || raw.contains('B')
            || raw.contains('e')
            || raw.contains('E')
        {
            ctx.diagnostic(use_simple_number_keys_diagnostic(num.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const obj = { 1: 'foo' };",
        "const obj = { 42: 'foo' };",
        "const obj = { 'key': 'foo' };",
        "const obj = { 0: 'foo' };",
    ];

    let fail = vec![
        "const obj = { 1.0: 'foo' };",
        "const obj = { 0x1: 'foo' };",
        "const obj = { 0o1: 'foo' };",
        "const obj = { 0b1: 'foo' };",
        "const obj = { 1e2: 'foo' };",
    ];

    Tester::new(UseSimpleNumberKeys::NAME, UseSimpleNumberKeys::PLUGIN, pass, fail)
        .test_and_snapshot();
}
