use oxc_ast::{
    AstKind,
    ast::{
        Declaration, Expression, ModuleExportName, VariableDeclaration, VariableDeclarationKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ReferenceId;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_mutable_exports_diagnostic(span: Span, kind: VariableDeclarationKind) -> OxcDiagnostic {
    let kind_str = if kind == VariableDeclarationKind::Var { "var" } else { "let" };
    OxcDiagnostic::warn(format!("Exporting mutable '{kind_str}' binding, use 'const' instead."))
        .with_label(span)
}

// <https://github.com/import-js/eslint-plugin-import/blob/v2.31.0/docs/rules/no-mutable-exports.md>
#[derive(Debug, Default, Clone)]
pub struct NoMutableExports;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbids the use of mutable exports with var or let.
    ///
    /// ### Why is this bad?
    ///
    /// In general, we should always export constants
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// export let count = 2
    /// export var count = 3
    ///
    /// let count = 4
    /// export { count }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// export const count = 1
    /// export function getCount() {}
    /// export class Counter {}
    /// ```
    ///
    /// ### Functions/Classes
    /// Note that exported function/class declaration identifiers may be reassigned,
    /// but are not flagged by this rule at this time. They may be in the future.
    NoMutableExports,
    import,
    style,
);

impl Rule for NoMutableExports {
    fn run<'a>(&self, node: &oxc_semantic::AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ExportNamedDeclaration(export_name_decl) => {
                // e.g. "export let a = 4;"
                if let Some(declaration) = &export_name_decl.declaration {
                    let Declaration::VariableDeclaration(decl) = declaration else {
                        return;
                    };
                    if matches!(
                        decl.kind,
                        VariableDeclarationKind::Var | VariableDeclarationKind::Let
                    ) {
                        ctx.diagnostic(no_mutable_exports_diagnostic(decl.span, decl.kind));
                    }
                } else if export_name_decl.source.is_none() {
                    // e.g. "let a = 3; export { a }"
                    for specifier in &export_name_decl.specifiers {
                        if let ModuleExportName::IdentifierReference(ident) = &specifier.local {
                            let Some(declaration) =
                                get_reference_declaration(ident.reference_id(), ctx)
                            else {
                                continue;
                            };
                            ctx.diagnostic(no_mutable_exports_diagnostic(
                                declaration.span,
                                declaration.kind,
                            ));
                        }
                    }
                }
            }
            AstKind::ExportDefaultDeclaration(export_default_decl) => {
                // e.g. "let a = 4; export default a"
                let Some(Expression::Identifier(ident)) =
                    export_default_decl.declaration.as_expression()
                else {
                    return;
                };
                let Some(declaration) = get_reference_declaration(ident.reference_id(), ctx) else {
                    return;
                };
                ctx.diagnostic(no_mutable_exports_diagnostic(declaration.span, declaration.kind));
            }
            _ => {}
        }
    }
}

// find "let a = 2;" in "let a = 2; export default a"
// find "let foo = 1" in "let foo = 1; export { foo }"
fn get_reference_declaration<'a>(
    reference_id: ReferenceId,
    ctx: &'a LintContext,
) -> Option<&'a VariableDeclaration<'a>> {
    let reference = ctx.scoping().get_reference(reference_id);
    let symbol_id = reference.symbol_id()?;
    let reference_node = ctx.symbol_declaration(symbol_id);
    if matches!(reference_node.kind(), AstKind::VariableDeclarator(_)) {
        // we need return reference_node's parent node
        if let AstKind::VariableDeclaration(decl) = ctx.nodes().parent_kind(reference_node.id())
            && matches!(decl.kind, VariableDeclarationKind::Let | VariableDeclarationKind::Var)
        {
            return Some(decl);
        }
    }
    None
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "export const count = 1",
        "export function getCount() {}",
        "export class Counter {}",
        "export default count = 1",
        "export default function getCount() {}",
        "export default class Counter {}",
        r"
            const foo = 1;
            export { foo }
        ",
        r"
            const foo = 1;
            export { foo as baz }
        ",
        r"
            const foo = 1;
            export default foo;
        ",
        r"
            const foo = 1;
            export { foo as default }
        ",
        r"
            function foo() {}
            export { foo }
        ",
        r"
            function foo() {}
            export { foo as baz }
        ",
        r"
            function foo() {}
            export { foo as default }
        ",
        r"
            class foo {}
            export { foo }
        ",
        r"
            class foo {}
            export { foo as baz }
        ",
        r"
            class foo {}
            export { foo as default }
        ",
        "export * from './something';",
        r"
            type Foo = {}
            export type { Foo }
        ",
        r"
            const foo = 1
            export { foo as 'baz' }
        ",
    ];

    let fail = vec![
        "export let count = 1",
        "export var count = 1",
        r"
            let foo = 2;
            export { foo }
        ",
        "let foo = 4, baz = 5; export { foo }",
        r"
            var foo = 3;
            export { foo }
        ",
        r"
            let foo = 3;
            export { foo as baz }
        ",
        r"
            var foo = 3;
            export { foo as baz }
        ",
        r"
            let foo = 3;
            export default foo
        ",
        r"
            var foo = 3;
            export default foo
        ",
        r"
            var a = 2;
            let c = 3;
            export {
                c
            }
            export default a;
        ",
        r"
            let a = 3, c = 4;
            export {
                a
            }
        ",
    ];

    Tester::new(NoMutableExports::NAME, NoMutableExports::PLUGIN, pass, fail).test_and_snapshot();
}
