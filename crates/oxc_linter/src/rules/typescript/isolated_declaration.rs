use oxc_ast::{
    ast::{Class, ClassElement, Declaration, Function, ModuleDeclarationKind, PropertyDefinition},
    AstKind, Span,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;

use crate::{ast_util::IsPrivate, context::LintContext, rule::Rule, AstNode};

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
                            Self::check_function(function, ctx)
                        }
                        Declaration::ClassDeclaration(class) => Self::check_class(class, ctx),
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

    pub fn check_property_definition(property: &PropertyDefinition, ctx: &LintContext<'_>) {
        if property.type_annotation.is_none() {
            ctx.diagnostic(IsolatedDeclarationDiagnostic(property.span));
        }
    }

    /// Checks that:
    /// 1. All the non private methods are valid by `Self::check_function`
    /// 2. All the non private class fields have a type annotation
    /// 3. All the non private variables have a type annotation
    pub fn check_class(class: &Class, ctx: &LintContext<'_>) {
        for element in &class.body.body {
            match element {
                ClassElement::StaticBlock(_) => (),
                ClassElement::MethodDefinition(method) => {
                    if !method.is_private() {
                        Self::check_function(&method.value, ctx);
                    }
                }
                ClassElement::PropertyDefinition(property) => {
                    if !property.is_private() && !property.key.is_private_identifier() {
                        Self::check_property_definition(property, ctx)
                    }
                }
                ClassElement::AccessorProperty(_) => todo!(),
                ClassElement::TSAbstractMethodDefinition(method) => {
                    Self::check_function(&method.method_definition.value, ctx);
                }
                ClassElement::TSAbstractPropertyDefinition(property) => {
                    Self::check_property_definition(&property.property_definition, ctx);
                }
                ClassElement::TSIndexSignature(_) => todo!(),
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("export function foo(a: number): number { return a; }", None),
        ("export class A { public a: number; }", None),
        ("export class A { private a; }", None),
        ("export class A { #a; }", None),
        ("export class A { foo(): number { return 0; } }", None),
        ("export class A { private foo() { return 0; } }", None),
        ("export class A { public foo(a: number): number { return a; } }", None),
    ];

    let fail = vec![
        ("export function foo(a) { return a; }", None),
        ("export class A { public a; }", None),
        ("export class A { foo() { return 0; } }", None),
        ("export abstract class A { abstract foo() { return 0; } }", None),
        ("export abstract class A { abstract a; }", None),
        ("export class A { get foo() { return 0; } }", None),
        ("export class A { set foo(val) { } }", None),
    ];

    Tester::new(IsolatedDeclaration::NAME, pass, fail).test_and_snapshot();
}
