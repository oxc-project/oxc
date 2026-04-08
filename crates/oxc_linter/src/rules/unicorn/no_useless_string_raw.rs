use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_useless_string_raw_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`String.raw` is unnecessary when the template has no escape sequences.")
        .with_help(
            "Remove `String.raw` since the template literal doesn't contain any backslash escapes.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessStringRaw;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows using `String.raw` when the template literal has no escape sequences.
    ///
    /// ### Why is this bad?
    ///
    /// `String.raw` is only useful when the template literal contains escape
    /// sequences that you want to keep as-is. Using it without escape sequences
    /// adds unnecessary noise.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// String.raw`hello world`;
    /// String.raw`foo bar`;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// String.raw`hello\nworld`;
    /// String.raw`C:\Users\foo`;
    /// `hello world`;
    /// ```
    NoUselessStringRaw,
    unicorn,
    suspicious,
    pending
);

impl Rule for NoUselessStringRaw {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TaggedTemplateExpression(tagged) = node.kind() else {
            return;
        };

        // Check if the tag is String.raw
        let Expression::StaticMemberExpression(member) = &tagged.tag else {
            return;
        };

        let Expression::Identifier(obj) = &member.object else {
            return;
        };

        if obj.name.as_str() != "String" || member.property.name.as_str() != "raw" {
            return;
        }

        // Check if any quasi (template part) contains a backslash
        let has_escape = tagged.quasi.quasis.iter().any(|quasi| {
            let raw = ctx.source_range(quasi.span);
            raw.contains('\\')
        });

        if !has_escape {
            ctx.diagnostic(no_useless_string_raw_diagnostic(tagged.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"String.raw`hello\nworld`;",
        r"String.raw`C:\Users\foo`;",
        "`hello world`;",
        r"String.raw`foo\tbar`;",
    ];

    let fail =
        vec!["String.raw`hello world`;", "String.raw`foo bar`;", "String.raw`no escapes here`;"];

    Tester::new(NoUselessStringRaw::NAME, NoUselessStringRaw::PLUGIN, pass, fail)
        .test_and_snapshot();
}
