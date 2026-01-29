use oxc_ast::{
    AstKind,
    ast::{ClassElement, PropertyKey, TSSignature, TSType, TSTypeName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_misused_new_interface_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Interfaces cannot be constructed, only classes.")
        .with_help("Consider removing this method from your interface.")
        .with_label(span)
}

fn no_misused_new_class_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Class cannot have method named `new`.")
        .with_help("This method name is confusing, consider renaming the method to `constructor`")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoMisusedNew;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces valid definition of new and constructor. This rule prevents classes from defining
    /// a method named `new` and interfaces from defining a method named `constructor`.
    ///
    /// ### Why is this bad?
    ///
    /// JavaScript classes may define a constructor method that runs
    /// when a class instance is newly created.
    ///
    /// TypeScript allows interfaces that describe a static class object to
    /// define a `new()` method (though this is rarely used in real world code).
    /// Developers new to JavaScript classes and/or TypeScript interfaces may
    /// sometimes confuse when to use constructor or new.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// declare class C {
    ///   new(): C;
    /// }
    /// ```
    ///
    /// ```typescript
    /// interface I {
    ///   new (): I;
    ///   constructor(): void;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// declare class C {
    ///   constructor();
    /// }
    /// ```
    ///
    /// ```typescript
    /// interface I {
    ///   new (): C;
    /// }
    /// ```
    NoMisusedNew,
    typescript,
    correctness
);

impl Rule for NoMisusedNew {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::TSInterfaceDeclaration(interface_decl) => {
                let decl_name = &interface_decl.id.name;

                for signature in &interface_decl.body.body {
                    let TSSignature::TSConstructSignatureDeclaration(sig) = signature else {
                        continue;
                    };
                    let Some(return_type) = &sig.return_type else {
                        continue;
                    };
                    let TSType::TSTypeReference(type_ref) = &return_type.type_annotation else {
                        continue;
                    };
                    if let TSTypeName::IdentifierReference(id) = &type_ref.type_name
                        && id.name == decl_name
                    {
                        ctx.diagnostic(no_misused_new_interface_diagnostic(Span::sized(
                            sig.span.start,
                            3,
                        )));
                    }
                }
            }
            AstKind::TSMethodSignature(method_sig) => {
                if let PropertyKey::StaticIdentifier(id) = &method_sig.key
                    && id.name == "constructor"
                {
                    ctx.diagnostic(no_misused_new_interface_diagnostic(method_sig.key.span()));
                }
            }
            AstKind::Class(cls) => {
                let Some(cls_id) = &cls.id else {
                    return;
                };
                let cls_name = &cls_id.name;

                for element in &cls.body.body {
                    let ClassElement::MethodDefinition(method) = element else {
                        continue;
                    };
                    if method.key.is_specific_id("new") && method.value.body.is_none() {
                        let Some(return_type) = &method.value.return_type else {
                            continue;
                        };
                        let TSType::TSTypeReference(type_ref) = &return_type.type_annotation else {
                            continue;
                        };
                        if let TSTypeName::IdentifierReference(current_id) = &type_ref.type_name
                            && current_id.name == cls_name
                        {
                            ctx.diagnostic(no_misused_new_class_diagnostic(method.key.span()));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "declare abstract class C { foo(); get new();bar();}",
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

    Tester::new(NoMisusedNew::NAME, NoMisusedNew::PLUGIN, pass, fail).test_and_snapshot();
}
