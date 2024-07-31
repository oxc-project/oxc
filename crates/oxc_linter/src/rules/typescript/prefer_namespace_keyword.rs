use oxc_ast::{
    ast::{TSModuleDeclarationKind, TSModuleDeclarationName},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::{LintContext, LinterContext},
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
    style
);

impl Rule for PreferNamespaceKeyword {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a, '_>) {
        let AstKind::TSModuleDeclaration(module) = node.kind() else { return };
        if module.id.is_string_literal()
            || !matches!(module.id, TSModuleDeclarationName::Identifier(_))
            || module.kind != TSModuleDeclarationKind::Module
        {
            return;
        }

        ctx.diagnostic_with_fix(prefer_namespace_keyword_diagnostic(module.span), |fixer| {
            let span_size = u32::try_from("module".len()).unwrap_or(6);
            let span_start = if module.declare {
                module.span.start + u32::try_from("declare ".len()).unwrap_or(8)
            } else {
                module.span.start
            };
            fixer.replace(Span::sized(span_start, span_size), "namespace")
        });
    }

    fn should_run(&self, ctx: &LinterContext) -> bool {
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
    ];

    let fail = vec![
        "module foo {}",
        "declare module foo {}",
        "
			declare module foo {
			  declare module bar {}
			}
			      ",
        "declare global {
            module foo {}
        }
        ",
    ];

    let fix = vec![
        ("module foo {}", "namespace foo {}", None),
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
    ];
    Tester::new(PreferNamespaceKeyword::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
