use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_dynamic_namespace_import_access_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Dynamic access to namespace import defeats tree-shaking.")
        .with_help("Use named imports instead of dynamically accessing namespace imports.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDynamicNamespaceImportAccess;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows dynamic member access on namespace imports (e.g., `ns[key]`).
    ///
    /// ### Why is this bad?
    ///
    /// When you use `import * as ns` and then access `ns[variable]`,
    /// bundlers cannot determine which exports are used and must include
    /// the entire module, defeating tree-shaking.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import * as ns from 'mod';
    /// const value = ns[key];
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import * as ns from 'mod';
    /// const value = ns.specificExport;
    /// import { specificExport } from 'mod';
    /// ```
    NoDynamicNamespaceImportAccess,
    import,
    perf,
    pending
);

impl Rule for NoDynamicNamespaceImportAccess {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ComputedMemberExpression(expr) = node.kind() else {
            return;
        };

        // Check if the object is an identifier that is a namespace import
        let Expression::Identifier(ident) = &expr.object else {
            return;
        };

        // Look up the binding to see if it's a namespace import
        let Some(symbol_id) = ctx.scoping().find_binding(node.scope_id(), ident.name) else {
            return;
        };

        let decl_node = ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id));
        if matches!(decl_node.kind(), AstKind::ImportNamespaceSpecifier(_)) {
            ctx.diagnostic(no_dynamic_namespace_import_access_diagnostic(expr.span()));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "import * as ns from 'mod'; const x = ns.foo;",
        "import { foo } from 'mod'; const x = foo;",
        "const obj = {}; const x = obj[key];",
    ];

    let fail = vec!["import * as ns from 'mod'; const x = ns[key];"];

    Tester::new(
        NoDynamicNamespaceImportAccess::NAME,
        NoDynamicNamespaceImportAccess::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
