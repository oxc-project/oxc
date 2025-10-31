use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_named_export_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Named exports are not allowed.")
        .with_help("Replace named exports with a single export default to ensure a consistent module entry point.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNamedExport;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prohibit named exports.
    ///
    /// ### Why is this bad?
    ///
    /// Named exports require strict identifier matching and can lead to fragile imports,
    /// while default exports enforce a single, consistent module entry point.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// export const foo = 'foo';
    ///
    /// const bar = 'bar';
    /// export { bar }
    ///
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// export default 'bar';
    ///
    /// const foo = 'foo';
    /// export { foo as default }
    /// ```
    NoNamedExport,
    import,
    style
);

impl Rule for NoNamedExport {
    fn run<'a>(&self, node: &oxc_semantic::AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ExportAllDeclaration(all_decl) => {
                ctx.diagnostic(no_named_export_diagnostic(all_decl.span));
            }
            AstKind::ExportNamedDeclaration(named_decl) => {
                let specifiers = &named_decl.specifiers;
                if specifiers.is_empty() {
                    ctx.diagnostic(no_named_export_diagnostic(named_decl.span));
                }
                if specifiers.iter().any(|specifier| specifier.exported.name() != "default") {
                    ctx.diagnostic(no_named_export_diagnostic(named_decl.span));
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "module.export.foo = function () {}",
        "module.export.foo = function () {}",
        "export default function bar() {};",
        "let foo; export { foo as default }",
        "import * as foo from './foo';",
        "import foo from './foo';",
        "import {default as foo} from './foo';",
        "let foo; export { foo as \"default\" }",
    ];

    let fail = vec![
        "export const foo = 'foo';",
        "
            export const foo = 'foo';
            export default bar;
        ",
        "
            export const foo = 'foo';
            export function bar() {};
        ",
        "export const foo = 'foo';",
        "
            const foo = 'foo';
            export { foo };
        ",
        "let foo, bar; export { foo, bar }",
        "export const { foo, bar } = item;",
        "export const { foo, bar: baz } = item;",
        "export const { foo: { bar, baz } } = item;",
        "
            let item;
            export const foo = item;
            export { item };
        ",
        "export * from './foo';",
        "export const { foo } = { foo: 'bar' };",
        "export const { foo: { bar } } = { foo: { bar: 'baz' } };",
        "export { a, b } from 'foo.js'",
        "export type UserId = number;",
        "export foo from 'foo.js'",
        "export Memory, { MemoryValue } from './Memory'",
    ];

    Tester::new(NoNamedExport::NAME, NoNamedExport::PLUGIN, pass, fail).test_and_snapshot();
}
