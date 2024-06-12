use oxc_allocator::Vec;
use oxc_ast::{
    ast::{ExportNamedDeclaration, Statement, TSModuleDeclarationBody},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_useless_empty_export_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "typescript-eslint(no-useless-empty-export): Disallow empty exports that don't change anything in a module file",
    )
    .with_help("Empty export does nothing and can be removed.")
    .with_labels([span0.into()])
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
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Program(program) => {
                if ctx.semantic().module_record().not_esm {
                    return;
                }
                check_node(&program.body, ctx);
            }
            AstKind::TSModuleDeclaration(decl) => {
                if let Some(TSModuleDeclarationBody::TSModuleBlock(block)) = &decl.body {
                    check_node(&block.body, ctx);
                }
            }
            _ => {}
        }
    }
}

fn get_empty_export<'a>(statement: &'a Statement) -> Option<&'a ExportNamedDeclaration<'a>> {
    if let Statement::ExportNamedDeclaration(export_decl) = statement {
        if export_decl.specifiers.is_empty() && export_decl.declaration.is_none() {
            return Some(export_decl);
        }
    }
    None
}

fn is_export_or_import_node_types(statement: &Statement) -> bool {
    matches!(
        statement,
        Statement::ExportAllDeclaration(_)
            | Statement::ExportDefaultDeclaration(_)
            | Statement::ExportNamedDeclaration(_)
            | Statement::ImportDeclaration(_)
            | Statement::TSExportAssignment(_)
            | Statement::TSImportEqualsDeclaration(_)
    )
}

fn check_node<'a>(statements: &Vec<'a, Statement<'a>>, ctx: &LintContext<'a>) {
    if statements.is_empty() {
        return;
    }

    let mut empty_exports = vec![];
    let mut found_other_export = false;

    for statement in statements {
        if let Some(empty_export) = get_empty_export(statement) {
            empty_exports.push(empty_export);
        } else if is_export_or_import_node_types(statement) {
            found_other_export = true;
        }
    }

    if found_other_export {
        for empty_export in &empty_exports {
            ctx.diagnostic_with_fix(
                no_useless_empty_export_diagnostic(empty_export.span),
                |fixer| fixer.delete(&empty_export.span),
            );
        }
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
        "
            import _ = require('_');
            export {};
        ",
    ];

    let fix = vec![
        ("export const _ = {};export {};", "export const _ = {};", None),
        ("export * from '_';export {};", "export * from '_';", None),
        ("export {};export * from '_';", "export * from '_';", None),
        ("const _ = {};export default _;export {};", "const _ = {};export default _;", None),
        ("export {};const _ = {};export default _;", "const _ = {};export default _;", None),
        ("const _ = {};export { _ };export {};", "const _ = {};export { _ };", None),
        ("import _ = require('_');export {};", "import _ = require('_');", None),
    ];

    Tester::new(NoUselessEmptyExport::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
