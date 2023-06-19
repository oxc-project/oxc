use oxc_ast::{
    ast::{
        BindingPatternKind, Class, ClassElement, Function, MethodDefinitionKind, PropertyDefinition,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};

use crate::{ast_util::IsPrivate, context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
enum IsolatedDeclarationDiagnostic {
    #[error("isolated-declaration: Requires type annotation on exported properties")]
    #[diagnostic(severity(warning))]
    Property(#[label] Span),

    #[error("isolated-declaration: Requires type annotation on export parameters")]
    #[diagnostic(severity(warning))]
    FunctionParam(#[label] Span),

    #[error("isolated-declaration: Requires return type annotation on exported functions")]
    #[diagnostic(severity(warning))]
    FunctionReturnType(#[label] Span),
}

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
    /// The restrictions allow .d.ts files to be generated based on one single .ts file, which improves the
    /// efficiency and possible parallelism of the declaration emitting process. Furthermore, it prevents syntax
    /// errors in one file from propagating to other dependent files.
    ///
    /// ### Example
    /// ```typescript
    /// export class Test {
    ///   x // error under isolated declarations
    ///   private y = 0; // no error, private field types are not serialized in declarations
    ///   #z = 1;// no error, fields is not present in declarations
    ///   constructor(x: number) {
    ///     this.x = 1;
    ///   }
    ///   get a() { // error under isolated declarations
    ///     return 1;
    ///   }
    ///   set a(value) { // error under isolated declarations
    ///     this.x = 1;
    ///   }
    /// }
    /// ```
    IsolatedDeclaration,
    nursery,
);

impl Rule for IsolatedDeclaration {
    fn run_on_symbol(&self, symbol_id: SymbolId, ctx: &LintContext) {
        let symbol_table = ctx.semantic().symbols();
        if symbol_table.get_flag(symbol_id).is_export() {
            let declaration = ctx.nodes()[symbol_table.get_declaration(symbol_id)];
            match declaration.kind() {
                AstKind::Class(class) => Self::check_class(class, ctx),
                AstKind::Function(function) => Self::check_function(function, ctx),
                _ => (),
            }
        }
    }
}

impl IsolatedDeclaration {
    /// Checks that:
    /// 1. all the parameters of the function has type annotation
    /// 2. return type of the function has type annotation
    fn check_function(function: &Function, ctx: &LintContext) {
        Self::check_function_params(function, ctx);
        Self::check_function_return(function, ctx);
    }

    fn check_function_params(function: &Function, ctx: &LintContext) {
        let parameters = &function.params.items;
        for param in parameters {
            let span = match &param.pattern.kind {
                BindingPatternKind::AssignmentPattern(pat) => {
                    pat.left.type_annotation.is_none().then(|| pat.left.span())
                }
                _ => param.pattern.type_annotation.is_none().then_some(param.span),
            };
            if let Some(span) = span {
                ctx.diagnostic(IsolatedDeclarationDiagnostic::FunctionParam(span));
            }
        }
    }

    fn check_function_return(function: &Function, ctx: &LintContext) {
        if function.return_type.is_none() {
            let start = function.params.span.end;
            let span = Span::new(start, start + 1);
            ctx.diagnostic(IsolatedDeclarationDiagnostic::FunctionReturnType(span));
        }
    }

    /// Checks that the property as a type annotation
    fn check_property_definition(property: &PropertyDefinition, ctx: &LintContext) {
        if property.type_annotation.is_none() {
            ctx.diagnostic(IsolatedDeclarationDiagnostic::Property(property.key.span()));
        }
    }

    /// Checks that:
    /// 1. All the non private methods are valid by `Self::check_function`
    /// 2. All the non private class fields have a type annotation
    /// 3. All the non private variables have a type annotation
    fn check_class(class: &Class, ctx: &LintContext) {
        for element in &class.body.body {
            match element {
                ClassElement::MethodDefinition(method) => {
                    if !method.is_private() {
                        match method.kind {
                            MethodDefinitionKind::Constructor | MethodDefinitionKind::Set => {
                                Self::check_function_params(&method.value, ctx);
                            }
                            MethodDefinitionKind::Method | MethodDefinitionKind::Get => {
                                Self::check_function(&method.value, ctx);
                            }
                        }
                    }
                }
                ClassElement::PropertyDefinition(property) => {
                    if !property.is_private() && !property.key.is_private_identifier() {
                        Self::check_property_definition(property, ctx);
                    }
                }
                ClassElement::TSAbstractMethodDefinition(method) => {
                    Self::check_function(&method.method_definition.value, ctx);
                }
                ClassElement::TSAbstractPropertyDefinition(property) => {
                    Self::check_property_definition(&property.property_definition, ctx);
                }
                ClassElement::StaticBlock(_)
                | ClassElement::AccessorProperty(_)
                | ClassElement::TSIndexSignature(_) => (),
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
        ("export class A { public foo(a: number = 1): number { return a; } }", None),
        ("export class A { public constructor() {  } }", None),
    ];

    let fail = vec![
        ("export function foo(a) { return a; }", None),
        ("export class A { public a; }", None),
        ("export class A { foo() { return 0; } }", None),
        ("export abstract class A { abstract foo() { return 0; } }", None),
        ("export abstract class A { abstract a; }", None),
        ("export class A { get foo() { return 0; } }", None),
        ("export class A { public foo(a = 1): number { return a; } }", None),
        ("export class A { set foo(val) { } }", None),
    ];

    Tester::new(IsolatedDeclaration::NAME, pass, fail).test_and_snapshot();
}
