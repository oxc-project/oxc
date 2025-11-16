use oxc_ast::{
    AstKind,
    ast::{
        ArrowFunctionExpression, Declaration, ExportDefaultDeclarationKind, Function,
        ThisExpression,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

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
    suspicious
);

// Visitor to find `this` expressions within a function body
struct ThisFinder {
    found_this_expressions: Vec<Span>,
}

impl ThisFinder {
    fn new() -> Self {
        Self { found_this_expressions: Vec::new() }
    }
}

impl<'a> Visit<'a> for ThisFinder {
    fn visit_this_expression(&mut self, expr: &ThisExpression) {
        self.found_this_expressions.push(expr.span);
    }
    // Don't traverse into nested function declarations - they have their own `this` context
    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {
        // noop
    }
    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        walk::walk_arrow_function_expression(self, arrow);
    }
}

fn check_function_for_this(func: &Function, ctx: &LintContext) {
    let Some(body) = &func.body else { return };

    let mut finder = ThisFinder::new();
    finder.visit_function_body(body);

    for span in finder.found_this_expressions {
        ctx.diagnostic(no_this_in_exported_function_diagnostic(span));
    }
}

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
            _ => {}
        }
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
        "class Foo { method() { this.bar(); } } export { Foo };",
        "export const obj = { method() { this.bar(); } };",
        "export function foo() { function bar() { this.baz(); } }",
        "export function foo() { const bar = function() { this.baz(); }; }",
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
    ];

    Tester::new(NoThisInExportedFunction::NAME, NoThisInExportedFunction::PLUGIN, pass, fail)
        .test_and_snapshot();
}
