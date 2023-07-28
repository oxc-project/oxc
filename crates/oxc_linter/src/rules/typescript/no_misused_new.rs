use oxc_ast::{
    ast::{ClassElement, PropertyKey, TSSignature, TSType, TSTypeName},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(no-misused-new): Interfaces cannot be constructed, only classes.")]
#[diagnostic(severity(warning), help("Consider removing this method from your interface."))]
struct NoMisusedNewInterfaceDiagnostic(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(no-misused-new): Class cannot have method named `new`.")]
#[diagnostic(
    severity(warning),
    help("This method name is confusing, consider renaming the method to `constructor`")
)]
struct NoMisusedNewClassDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoMisusedNew;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce valid definition of `new` and `constructor`
    ///
    /// ### Why is this bad?
    ///
    /// JavaScript classes may define a constructor method that runs
    /// when a class instance is newly created.
    /// TypeScript allows interfaces that describe a static class object to define
    /// a new() method (though this is rarely used in real world code).
    /// Developers new to JavaScript classes and/or TypeScript interfaces may
    /// sometimes confuse when to use constructor or new.
    ///
    /// ### Example
    /// ```typescript
    // declare class C {
    //   new(): C;
    // }

    // interface I {
    //   new (): I;
    //   constructor(): void;
    // }
    /// ```
    NoMisusedNew,
    correctness
);

impl Rule for NoMisusedNew {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::TSInterfaceDeclaration(interface_decl) => {
                let decl_name = &interface_decl.id.name;

                for signature in &interface_decl.body.body {
                    if let TSSignature::TSConstructSignatureDeclaration(sig) = signature {
                        if let Some(return_type) = &sig.return_type {
                            if let TSType::TSTypeReference(type_ref) = &return_type.type_annotation
                            {
                                if let TSTypeName::IdentifierName(id) = &type_ref.type_name {
                                    if id.name == decl_name {
                                        ctx.diagnostic(NoMisusedNewInterfaceDiagnostic(Span::new(
                                            sig.span.start,
                                            sig.span.start + 3,
                                        )));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            AstKind::TSMethodSignature(method_sig) => {
                if let PropertyKey::Identifier(id) = &method_sig.key {
                    if id.name == "constructor" {
                        ctx.diagnostic(NoMisusedNewInterfaceDiagnostic(method_sig.key.span()));
                    }
                }
            }
            AstKind::Class(cls) => {
                if let Some(cls_id) = &cls.id {
                    let cls_name = &cls_id.name;

                    for element in &cls.body.body {
                        if let ClassElement::MethodDefinition(method) = element {
                            if method.key.is_specific_id("new") && method.value.body.is_none() {
                                if let Some(return_type) = &method.value.return_type {
                                    if let TSType::TSTypeReference(type_ref) =
                                        &return_type.type_annotation
                                    {
                                        if let TSTypeName::IdentifierName(current_id) =
                                            &type_ref.type_name
                                        {
                                            if current_id.name == cls_name {
                                                ctx.diagnostic(NoMisusedNewClassDiagnostic(
                                                    method.key.span(),
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
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
        "declare abstract class C { foo() {} get new();bar();}",
        "class C { constructor();}",
        "const foo = class { constructor();};",
        "const foo = class { new(): X;};",
        "class C { new() {} }",
        "class C { constructor() {} }",
        "const foo = class { new() {} };",
        "const foo = class { constructor() {} };",
        "interface I { new (): {}; }",
        "type T = { new (): T };",
        "export default class { constructor(); }",
        "interface foo { new <T>(): bar<T>; }",
        "interface foo { new <T>(): 'x'; }",
    ];

    let fail = vec![
        "interface I { new (): I; constructor(): void;}",
        "interface G { new <T>(): G<T>;}",
        "type T = { constructor(): void;};",
        "class C { new(): C;}",
        "declare abstract class C { new(): C;}",
        "interface I { constructor(): '';}",
    ];

    Tester::new_without_config(NoMisusedNew::NAME, pass, fail).test_and_snapshot();
}
