use oxc_ast::{
    AstKind,
    ast::{BindingIdentifier, Declaration},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::BoundNames;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_export_then_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not export a binding named `then`.")
        .with_help("Rename this export to prevent unexpected behavior when the module is dynamically imported.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoExportThen;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow explicitly named value exports called `then`.
    ///
    /// This includes declarations, export aliases, and named re-exports. Type-only
    /// exports are allowed. The rule does not resolve names from `export *` statements.
    ///
    /// ### Why is this bad?
    ///
    /// Dynamic imports return a promise that resolves to the module namespace object.
    /// If the module exports a function named `then`, promise resolution calls that
    /// function instead of returning the namespace as expected. The same module can
    /// therefore behave differently when imported statically and dynamically.
    ///
    /// Reserve the name even for non-function values, so a later change to the export
    /// cannot accidentally introduce this behavior.
    ///
    /// See the [MDN documentation on module namespace objects](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/import#module_namespace_object).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// export function then() {}
    /// ```
    ///
    /// ```js
    /// export { callback as then } from './callbacks.js';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// function then() {}
    /// export { then as onResolved };
    /// ```
    NoExportThen,
    oxc,
    nursery,
    version = "next",
    short_description = "Disallow exports named `then`.",
);

impl Rule for NoExportThen {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ExportDeclaration(_)
            | AstKind::ExportNamedDeclaration(_)
            | AstKind::ExportFromDeclaration(_)
            | AstKind::ExportAllDeclaration(_) => {}
            _ => return,
        }
        if !matches!(ctx.nodes().parent_kind(node.id()), AstKind::Program(_)) {
            return;
        }

        match node.kind() {
            AstKind::ExportDeclaration(decl) => {
                if decl.export_kind().is_type() {
                    return;
                }

                let mut check = |ident: &BindingIdentifier<'_>| {
                    if ident.name == "then"
                        && ctx.scoping().symbol_flags(ident.symbol_id()).is_value()
                    {
                        ctx.diagnostic(no_export_then_diagnostic(ident.span));
                    }
                };
                if let Declaration::VariableDeclaration(decl) = &decl.declaration {
                    decl.bound_names(&mut check);
                } else if let Some(ident) = decl.declaration.id() {
                    check(ident);
                }
            }
            AstKind::ExportNamedDeclaration(decl) => {
                if decl.export_kind.is_type() {
                    return;
                }
                for specifier in &decl.specifiers {
                    if specifier.export_kind.is_type() || specifier.exported.name() != "then" {
                        continue;
                    }
                    let scoping = ctx.scoping();
                    if scoping
                        .get_binding(node.scope_id(), specifier.local.name().into())
                        .is_some_and(|symbol_id| !scoping.symbol_flags(symbol_id).is_value())
                    {
                        continue;
                    }
                    ctx.diagnostic(no_export_then_diagnostic(specifier.exported.span()));
                }
            }
            AstKind::ExportFromDeclaration(decl) => {
                if decl.export_kind.is_type() {
                    return;
                }
                for specifier in &decl.specifiers {
                    if !specifier.export_kind.is_type() && specifier.exported.name() == "then" {
                        ctx.diagnostic(no_export_then_diagnostic(specifier.exported.span()));
                    }
                }
            }
            AstKind::ExportAllDeclaration(decl) => {
                if !decl.export_kind.is_type()
                    && let Some(exported) = &decl.exported
                    && exported.name() == "then"
                {
                    ctx.diagnostic(no_export_then_diagnostic(exported.span()));
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
        "let then;",
        "function then() {}",
        "export const other = 1;",
        "export const Then = 1;",
        "const then = 1; export { then as other };",
        "const then = 1; export { then as default };",
        "export default function then() {}",
        "export default class then {}",
        "export default { then() {} };",
        "export { then as other } from './module';",
        "export { then as default } from './module';",
        "export * from './module';",
        "export * as other from './module';",
        "export const { then: other } = object;",
        "export type then = () => void;",
        "export interface then {}",
        "type then = () => void; export { then };",
        "interface Callback {} export { Callback as then };",
        "class Callback {} export type { Callback as then };",
        "class Callback {} export { type Callback as then };",
        "import type { Callback } from './module'; export { Callback as then };",
        "import { Callback } from './module'; export type { Callback as then };",
        "import { Callback } from './module'; export { type Callback as then };",
        "export type { Callback as then } from './module';",
        "export { type Callback as then } from './module';",
        "export type * as then from './module';",
        "namespace Example { export function then() {} }",
        "export namespace then {}",
        "// oxlint-disable-next-line oxc/no-export-then\nexport function then() {}",
    ];

    let fail = vec![
        "export let then;",
        "export var then = 1;",
        "export const then = () => {};",
        "export function then() {}",
        "export async function then() {}",
        "export class then {}",
        "let then; export { then };",
        "const callback = () => {}; export { callback as then };",
        r#"const callback = () => {}; export { callback as "then" };"#,
        r#"const callback = () => {}; export { callback as "th\u0065n" };"#,
        "export { then } from './module';",
        "export { callback as then } from './module';",
        "export { default as then } from './module';",
        "export * as then from './module';",
        "import { callback } from './module'; export { callback as then };",
        "import * as callbacks from './module'; export { callbacks as then };",
        "export const { then } = object;",
        "export const { callback: then } = object;",
        "export const { nested: { then = 1 } } = object;",
        "export const [other, ...then] = array;",
        "export enum then { Value }",
        "export namespace then { export const value = 1; }",
        "interface then {} const then = () => {}; export { then };",
    ];

    Tester::new(NoExportThen::NAME, NoExportThen::PLUGIN, pass, fail).test_and_snapshot();
}
