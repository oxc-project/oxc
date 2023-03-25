use oxc_ast::{
    ast::{Declaration, Function, ModuleDeclarationKind},
    AstKind, Span,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("")]
#[diagnostic(severity(warning), help(""))]
struct IsolatedDeclarationDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct IsolatedDeclaration;

declare_oxc_lint!(
    /// ### What it does
    /// This rule enforces a set of restrictions on typescript files to enable "isolated declaration",
    /// i.e., .d.ts files can be generated from a single .ts file without resolving its dependencies.
    /// The typescript implementation is at `https://github.com/microsoft/TypeScript/pull/53463`
    /// The thread on isolated declaration is at `https://github.com/microsoft/TypeScript/issues/47947`
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    IsolatedDeclaration,
    correctness
);

impl Rule for IsolatedDeclaration {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ModuleDeclaration(module) = node.get().kind() else { return; };
        match &module.kind {
            ModuleDeclarationKind::ImportDeclaration(_) => todo!(),
            ModuleDeclarationKind::ExportAllDeclaration(_) => todo!(),
            ModuleDeclarationKind::ExportDefaultDeclaration(_) => todo!(),
            ModuleDeclarationKind::ExportNamedDeclaration(decl) => {
                if let Some(decl) = &decl.declaration {
                    match decl {
                        Declaration::FunctionDeclaration(function) => {
                            Self::check_function(function, ctx);
                        }
                        _ => (),
                    }
                }
            }
            ModuleDeclarationKind::TSExportAssignment(_) => todo!(),
            ModuleDeclarationKind::TSNamespaceExportDeclaration(_) => todo!(),
        }
    }
}

impl IsolatedDeclaration {
    /// Checks that:
    /// 1. all the parameters of the function has type annotation
    /// 2. return type of the function has type annotation
    pub fn check_function(function: &Function, ctx: &LintContext<'_>) {
        let parameters = &function.params.items;
        for param in parameters {
            if param.pattern.type_annotation.is_none() {
                ctx.diagnostic(IsolatedDeclarationDiagnostic(param.span));
            }
        }
        if function.return_type.is_none() {
            ctx.diagnostic(IsolatedDeclarationDiagnostic(function.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![("export function foo(a: number): number { return a; }", None)];

    let fail = vec![("export function foo(a) { return a; }", None)];

    Tester::new(IsolatedDeclaration::NAME, pass, fail).test_and_snapshot();
}
