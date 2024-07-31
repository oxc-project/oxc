use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_useless_empty_export_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow empty exports that don't change anything in a module file")
        .with_help("Empty export does nothing and can be removed.")
        .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessEmptyExport;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow empty exports that don't change anything in a module file.
    ///
    /// ### Example
    ///
    /// ### Bad
    /// ```javascript
    /// export const value = 'Hello, world!';
    /// export {};
    /// ```
    ///
    /// ### Good
    /// ```javascript
    /// export const value = 'Hello, world!';
    /// ```
    ///
    NoUselessEmptyExport,
    correctness
);

impl Rule for NoUselessEmptyExport {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a, '_>) {
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

    Tester::new(NoUselessEmptyExport::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
