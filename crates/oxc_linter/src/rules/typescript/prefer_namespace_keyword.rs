use oxc_ast::{
    ast::{TSModuleDeclarationKind, TSModuleDeclarationName},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct PreferNamespaceKeyword;

declare_oxc_lint!(
    /// ### What it does
    /// Require using namespace keyword over module keyword to declare custom TypeScript modules.
    ///
    /// ### Why is this bad?
    /// TypeScript historically allowed a form of code organization called "custom modules" (module Example {}), later renamed to "namespaces" (namespace Example).
    ///
    /// Namespaces are an outdated way to organize TypeScript code. ES2015 module syntax is now preferred (import/export).
    ///
    /// For projects still using custom modules / namespaces, it's preferred to refer to them as namespaces. This rule reports when the module keyword is used instead of namespace.
    ///
    /// ### Example
    /// ```javascript
    /// module foo {}
    /// declare module foo {}
    /// ```
    PreferNamespaceKeyword,
    restriction,
);

fn prefer_namespace_keyword_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("typescript-eslint(prefer-namespace-keyword): Use 'namespace' instead of 'module' to declare custom TypeScript modules.")
        .with_help("'Require using `namespace` keyword over `module` keyword to declare custom TypeScript modules")
        .with_labels([span0.into()])
}

impl Rule for PreferNamespaceKeyword {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSModuleDeclaration(decl) = node.kind() else { return };
        if decl.kind != TSModuleDeclarationKind::Module {
            return;
        }
        let TSModuleDeclarationName::Identifier(id) = &decl.id else { return };

        ctx.diagnostic_with_fix(prefer_namespace_keyword_diagnostic(decl.span), |fixer| {
            // replace `module` with `namespace`
            fixer.replace(Span::new(id.span.start - 7, id.span.start - 1), "namespace")
        });
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
