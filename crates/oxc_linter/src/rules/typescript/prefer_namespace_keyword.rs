use oxc_ast::{
    ast::{TSModuleDeclaration, TSModuleDeclarationKind, TSModuleDeclarationName},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    AstNode,
};

fn prefer_namespace_keyword_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use 'namespace' instead of 'module' to declare custom TypeScript modules.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferNamespaceKeyword;

declare_oxc_lint!(
    /// ### What it does
    /// This rule reports when the module keyword is used instead of namespace.
    /// This rule does not report on the use of TypeScript module declarations to describe external APIs (declare module 'foo' {}).
    ///
    /// ### Why is this bad?
    /// Namespaces are an outdated way to organize TypeScript code. ES2015 module syntax is now preferred (import/export).
    /// For projects still using custom modules / namespaces, it's preferred to refer to them as namespaces.
    ///
    /// ### Example
    /// ```typescript
    /// module Example {}
    /// ```
    PreferNamespaceKeyword,
    typescript,
    style,
    fix
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

        let token = ctx.source_range(Span::new(module.span.start, module.id.span().start));
        let Some(offset) = token.find("module") else {
            return;
        };

        ctx.diagnostic_with_fix(prefer_namespace_keyword_diagnostic(module.span), |fixer| {
            let span_start = module.span.start + u32::try_from(offset).unwrap();
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

    let pass = vec![
        "declare module 'foo';",
        "declare module 'foo' {}",
        "namespace foo {}",
        "declare namespace foo {}",
        "declare global {}",
        "module''.s",
    ];

    let fail = vec![
        "module foo {}",
        "module A.B {}",
        "declare module foo {}",
        "
        declare module foo {
          declare module bar {}
        }
        ",
        "
        declare global {
            module foo {}
        }
        ",
        "module foo.'a'",
    ];

    let fix = vec![
        ("module foo {}", "namespace foo {}", None),
        ("module A.B {}", "namespace A.B {}", None),
        (
            "
            module A {
              module B {}
            }
            ",
            "
            namespace A {
              namespace B {}
            }
            ",
            None,
        ),
        ("declare module foo {}", "declare namespace foo {}", None),
        (
            "
            declare module foo {
              declare module bar {}
            }
            ",
            "
            declare namespace foo {
              declare namespace bar {}
            }
            ",
            None,
        ),
        ("module foo.'a'", "namespace foo.'a'", None),
    ];

    Tester::new(PreferNamespaceKeyword::NAME, PreferNamespaceKeyword::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
