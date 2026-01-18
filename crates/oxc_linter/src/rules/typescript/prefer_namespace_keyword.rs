use oxc_ast::{
    AstKind,
    ast::{TSModuleDeclaration, TSModuleDeclarationKind, TSModuleDeclarationName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn prefer_namespace_keyword_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `namespace` instead of `module` to declare custom TypeScript modules.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferNamespaceKeyword;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule reports when the module keyword is used instead of namespace.
    /// This rule does not report on the use of TypeScript module declarations to describe external APIs (declare module 'foo' {}).
    ///
    /// ### Why is this bad?
    ///
    /// Namespaces are an outdated way to organize TypeScript code. ES2015 module syntax is now preferred (import/export).
    /// For projects still using custom modules / namespaces, it's preferred to refer to them as namespaces.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// module Example {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// namespace Example {}
    /// ```
    ///
    /// Note: This rule is largely superseded by the parser's TS1540 error which
    /// now reports when `module` keyword is used with identifier names.
    PreferNamespaceKeyword,
    typescript,
    style,
    pending // Fix capability removed - parser now reports TS1540 before linter runs
);

fn is_valid_module(module: &TSModuleDeclaration) -> bool {
    matches!(module.id, TSModuleDeclarationName::Identifier(_))
        && module.kind == TSModuleDeclarationKind::Module
}

impl Rule for PreferNamespaceKeyword {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSModuleDeclaration(module) = node.kind() else { return };

        if !is_valid_module(module) {
            return;
        }

        // Ignore nested `TSModuleDeclaration`s
        // e.g. the 2 inner `TSModuleDeclaration`s in `module A.B.C {}`
        if let AstKind::TSModuleDeclaration(_) = ctx.nodes().parent_kind(node.id()) {
            return;
        }

        ctx.diagnostic_with_fix(prefer_namespace_keyword_diagnostic(module.span), |fixer| {
            let mut span_start = module.span.start;
            span_start += ctx.find_next_token_from(span_start, "module").unwrap();
            fixer.replace(Span::sized(span_start, 6), "namespace")
        });
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    // Note: Most test cases were removed because the parser now reports TS1540
    // for using `module` keyword with identifier names. The rule is now largely
    // superseded by the parser's error handling.
    let pass = vec![
        "declare module 'foo';",
        "declare module 'foo' {}",
        "namespace foo {}",
        "declare namespace foo {}",
        "declare global {}",
    ];

    let fail: Vec<&str> = vec![];

    Tester::new(PreferNamespaceKeyword::NAME, PreferNamespaceKeyword::PLUGIN, pass, fail)
        .test_and_snapshot();
}
