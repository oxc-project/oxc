use oxc_ast::{
    AstKind,
    ast::{Declaration, ExportDefaultDeclarationKind, Expression, Function, ModuleExportName},
};
use oxc_ast_visit::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::ThisExpressionFinder};

fn no_this_in_exported_function_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`this` should not be used in exported functions")
        .with_help("Remove `this` or convert to a non-exported function. In bundlers, `this` becomes `undefined` in exported functions.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoThisInExportedFunction;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the use of `this` in exported functions.
    ///
    /// ### Why is this bad?
    ///
    /// In most bundlers, the value of `this` is not preserved for exported functions.
    /// When a function is exported and imported in another module, `this` typically
    /// becomes `undefined` instead of the module namespace object. This can lead to
    /// unexpected runtime errors or incorrect behavior.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// export function foo() {
    ///   console.log(this);
    /// }
    ///
    /// export default function bar() {
    ///   this.something();
    /// }
    ///
    /// function baz() {
    ///   const self = this;
    /// }
    /// export { baz };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// function foo() {
    ///   console.log(this);
    /// }
    ///
    /// export const bar = () => {
    ///   console.log(this);
    /// };
    /// ```
    NoThisInExportedFunction,
    oxc,
    suspicious,
    version = "1.33.0",
);

impl Rule for NoThisInExportedFunction {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ExportNamedDeclaration(export_decl) => {
                if let Some(Declaration::FunctionDeclaration(func)) = &export_decl.declaration {
                    check_function_for_this(func, ctx);
                }
            }
            AstKind::ExportDefaultDeclaration(export_decl) => {
                if let ExportDefaultDeclarationKind::FunctionDeclaration(func) =
                    &export_decl.declaration
                {
                    check_function_for_this(func, ctx);
                }
            }
            AstKind::ExportSpecifier(export_specifier) => {
                if let ModuleExportName::IdentifierReference(ident_ref) = &export_specifier.local
                    && let Some(declaration) = ctx
                        .semantic()
                        .scoping()
                        .get_reference(ident_ref.reference_id())
                        .symbol_id()
                        .map(|symbol_id| ctx.symbol_declaration(symbol_id))
                {
                    let func = match declaration.kind() {
                        AstKind::Function(func) => func,
                        AstKind::VariableDeclarator(var_decl) => {
                            if let Some(init) = var_decl.init.as_ref()
                                && let Expression::FunctionExpression(func) = init
                            {
                                func
                            } else {
                                return;
                            }
                        }
                        _ => return,
                    };
                    check_function_for_this(func, ctx);
                }
            }
            _ => {}
        }
    }
}

fn check_function_for_this(func: &Function, ctx: &LintContext) {
    let Some(body) = &func.body else { return };

    let mut finder =
        ThisExpressionFinder::new().skip_static_blocks().skip_property_definition_values();
    finder.visit_function_body(body);

    for span in finder.into_spans() {
        ctx.diagnostic(no_this_in_exported_function_diagnostic(span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function foo() { console.log(this); }",
        "function foo() { this.something(); }",
        "function foo() { const self = this; }",
        "export const foo = () => { console.log(this); };",
        "export const foo = () => { this.something(); };",
        "export default () => { console.log(this); };",
        "export function foo() { console.log('ok'); }",
        "export default function foo() { console.log('ok'); }",
        "export class Foo { method() { this.bar(); } }",
        "export class Foo { static { console.log(this) } }",
        "export class Foo { static [(console.log(this), 'bar')] = 1 }",
        "export class Foo { static bar = (console.log(this), 1) }",
        "export class Foo { [(console.log(this), 'bar')] = 1 }",
        "export class Foo { bar = (console.log(this), 1) }",
        "class Foo { method() { this.bar(); } } export { Foo };",
        "export const obj = { method() { this.bar(); } };",
        "export function foo() { function bar() { this.baz(); } }",
        "export function foo() { const bar = function() { this.baz(); }; }",
        "export function foo() { class Bar { static { console.log(this) } } }",
        "export function foo() { class Bar { static baz = (console.log(this), 1) } }",
        "export function foo() { class Bar { baz = (console.log(this), 1) } }",
    ];

    let fail = vec![
        "export function foo() { console.log(this); }",
        "export function foo() { this.something(); }",
        "export function foo() { const self = this; }",
        "export default function foo() { console.log(this); }",
        "export default function() { console.log(this); }",
        "export default function foo() { this.bar(); }",
        "export function foo() { console.log(this); this.bar(); const x = this; }",
        "export function foo() { if (true) { console.log(this); } }",
        "export function foo() { { this.bar(); } }",
        "export default function namedFunc() { this.bar(); }",
        "export function foo() { const obj = { get prop() { return this; } }; return this; }",
        "export function foo() { const bar = () => this.baz(); }",
        "function foo() { console.log(this); } export { foo };",
        "var foo = function foo() { console.log(this); }; export { foo };",
        "var foo = function () { console.log(this); }; export { foo };",
        "export function foo() { class Bar { static [(console.log(this), 'baz')] = 1 } }",
        "export function foo() { class Bar { [(console.log(this), 'baz')] = 1 } }",
        "export async function foo() { console.log(this); }",
        "export function* foo() { yield this; }",
        "export async function* foo() { yield this; }",
        "export default async function() { this.bar(); }",
        "export default function*() { yield this; }",
        "async function foo() { this.bar(); } export { foo };",
        "function* foo() { yield this; } export { foo };",
    ];

    Tester::new(NoThisInExportedFunction::NAME, NoThisInExportedFunction::PLUGIN, pass, fail)
        .test_and_snapshot();
}
