use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn avoid_re_export_all_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Avoid re-exporting `*` from a module, it leads to unused imports and prevents treeshaking.",
    )
    .with_help("Prefer named re-exports so consumers and bundlers only pull the symbols they need.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct AvoidReExportAll;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbids non-type `export *` re-exports.
    ///
    /// ### Why is this bad?
    ///
    /// Star re-exports make it easier to create barrel files that pull in more
    /// modules than necessary, which hurts runtime startup and bundling.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// export * from "./foo";
    /// export * as foo from "./foo";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// export { foo } from "./foo";
    /// export type * from "./types";
    /// ```
    AvoidReExportAll,
    oxc,
    restriction,
    none
);

impl Rule for AvoidReExportAll {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ExportAllDeclaration(export_all) = node.kind() else {
            return;
        };

        if export_all.is_typescript_syntax() {
            return;
        }

        ctx.diagnostic(avoid_re_export_all_diagnostic(export_all.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"export { foo } from "./foo";"#,
        r#"export { foo as bar } from "./foo";"#,
        r#"export type * from "./types";"#,
        r#"export type * as types from "./types";"#,
    ];

    let fail = vec![r#"export * from "./foo";"#, r#"export * as foo from "./foo";"#];

    Tester::new(AvoidReExportAll::NAME, AvoidReExportAll::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .test_and_snapshot();
}
