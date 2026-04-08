use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_inline_type_annotations_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use a named type or interface instead of an inline object type.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoInlineTypeAnnotations;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbids inline object type annotations.
    ///
    /// ### Why is this bad?
    ///
    /// Inline object type annotations duplicate structure at the usage site and
    /// make shared contracts harder to reuse and evolve. A named type or
    /// interface keeps the shape centralized and easier to reference.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// function render(props: { title: string }) {
    ///   return props.title;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// type Props = { title: string };
    ///
    /// function render(props: Props) {
    ///   return props.title;
    /// }
    /// ```
    NoInlineTypeAnnotations,
    oxc,
    restriction,
    none
);

impl Rule for NoInlineTypeAnnotations {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSTypeLiteral(type_literal) = node.kind() else {
            return;
        };

        if matches!(ctx.nodes().parent_kind(node.id()), AstKind::TSTypeAliasDeclaration(_)) {
            return;
        }

        ctx.diagnostic(no_inline_type_annotations_diagnostic(type_literal.span));
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

#[test]
fn test() {
    use serde_json::Value;
    use serde_json::json;

    use crate::tester::Tester;

    let pass: Vec<(&str, Option<Value>)> = vec![
        ("type Props = { title: string; count: number };", None),
        ("interface Props { title: string; count: number }", None),
        ("function render(props: Props) { return props.title; }", None),
        ("const title: string = 'hello';", None),
    ];

    let fail: Vec<(&str, Option<Value>)> = vec![
        ("const props: { title: string } = { title: 'hello' };", None),
        ("function render(props: { title: string; count: number }) { return props.title; }", None),
        ("const render = (): { title: string } => ({ title: 'hello' });", None),
        ("type Loader = () => Promise<{ title: string }>;", Some(json!([]))),
    ];

    Tester::new(NoInlineTypeAnnotations::NAME, NoInlineTypeAnnotations::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .test_and_snapshot();
}
