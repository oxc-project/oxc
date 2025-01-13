use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_useless_empty_export_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Empty exports do nothing in module files")
        .with_help("Remove this empty export.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessEmptyExport;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow empty exports that don't change anything in a module file.
    ///
    /// ## Why is this bad?
    /// An empty `export {}` statement is sometimes useful in TypeScript code to
    /// turn a file that would otherwise be a script file into a module file.
    /// Per the [TypeScript Handbook Modules page](https://www.typescriptlang.org/docs/handbook/modules/introduction.html):
    ///
    /// In TypeScript, just as in ECMAScript 2015, any file containing a
    /// top-level import or export is considered a module. Conversely, a file
    /// without any top-level import or export declarations is treated as a
    /// script whose contents are available in the global scope (and therefore
    /// to modules as well).
    ///
    /// However, an `export {}` statement does nothing if there are any other
    /// top-level import or export statements in a file.
    ///
    /// This rule reports an `export {}` that doesn't do anything in a file
    /// already using ES modules.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// export const value = 'Hello, world!';
    /// export {};
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// export const value = 'Hello, world!';
    /// ```
    NoUselessEmptyExport,
    typescript,
    correctness,
    fix
);

impl Rule for NoUselessEmptyExport {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ExportNamedDeclaration(decl) = node.kind() else { return };
        if decl.declaration.is_some() || !decl.specifiers.is_empty() {
            return;
        }
        let module_record = ctx.module_record();
        if module_record.exported_bindings.is_empty()
            && module_record.local_export_entries.is_empty()
            && module_record.indirect_export_entries.is_empty()
            && module_record.star_export_entries.is_empty()
            && module_record.export_default.is_none()
        {
            return;
        }
        ctx.diagnostic_with_fix(no_useless_empty_export_diagnostic(decl.span), |fixer| {
            fixer.delete(&decl.span)
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "declare module '_'",
        "import {} from '_';",
        "import * as _ from '_';",
        "export = {};",
        "export = 3;",
        "export const _ = {};",
        "
            const _ = {};
            export default _;
        ",
        "
            export * from '_';
            export = {};
        ",
        "export {};",
    ];

    let fail = vec![
        "
            export const _ = {};
            export {};
        ",
        "
            export * from '_';
            export {};
        ",
        "
            export {};
            export * from '_';
        ",
        "
            const _ = {};
            export default _;
            export {};
        ",
        "
            export {};
            const _ = {};
            export default _;
        ",
        "
            const _ = {};
            export { _ };
            export {};
        ",
        // "
        // import _ = require('_');
        // export {};
        // ",
    ];

    let fix = vec![
        ("export const _ = {};export {};", "export const _ = {};"),
        ("export * from '_';export {};", "export * from '_';"),
        ("export {};export * from '_';", "export * from '_';"),
        ("const _ = {};export default _;export {};", "const _ = {};export default _;"),
        ("export {};const _ = {};export default _;", "const _ = {};export default _;"),
        ("const _ = {};export { _ };export {};", "const _ = {};export { _ };"),
        // ("import _ = require('_');export {};", "import _ = require('_');"),
    ];

    Tester::new(NoUselessEmptyExport::NAME, NoUselessEmptyExport::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
