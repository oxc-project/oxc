use oxc_ast::{
    ast::{ClassElement, Expression, MethodDefinitionKind, Statement},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(constructor-super): Expected to call 'super()'.")]
#[diagnostic(severity(warning), help("Ensure 'super()' is called from constructor"))]
struct ConstructorSuperDiagnostic(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(constructor-super): Unexpected 'super()' because 'super' is not a constructor.")]
#[diagnostic(severity(warning), help("Do not call 'super()' from constructor."))]
struct SuperNotConstructorDiagnostic(
    #[label("unexpected 'super()'")] pub Span,
    #[label("because this is not a constructor")] pub Span,
);

#[derive(Debug, Default, Clone)]
pub struct ConstructorSuper;

declare_oxc_lint!(
    /// ### What it does
    /// Require 'super()' calls in constructors.
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// class A extends B {
    ///   constructor() {}
    /// }
    /// ```
    ConstructorSuper,
    nursery
);

impl Rule for ConstructorSuper {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Class(class) = node.get().kind() else { return };
        let Some(ctor) = class.body.body.iter().find_map(|el| match el {
            ClassElement::MethodDefinition(method_definition)
                if method_definition.kind == MethodDefinitionKind::Constructor =>
            {
                Some(method_definition)
            }
            _ => None,
        }) else { return };

        // In cases where there's no super-class, calling 'super()' inside the constructor
        // is handled by the parser.
        if let Some(super_class) = &class.super_class {
            ctor.value.body.as_ref().map_or_else(|| {
                ctx.diagnostic(ConstructorSuperDiagnostic(ctor.span));
            }, |function_body| {
                function_body.statements.iter().find_map(|stmt| {
                    let Statement::ExpressionStatement(expr) = stmt else { return None };
                    let Expression::CallExpression(call_expr) = &expr.expression else { return None };
                    if matches!(call_expr.callee, Expression::Super(_)) {
                        Some(call_expr.span)
                    } else {
                        None
                    }
                }).map_or_else(|| {
                    ctx.diagnostic(ConstructorSuperDiagnostic(ctor.span));
                }, |span| {
                    if let Some(super_class_span) = super_class.span() {
                        ctx.diagnostic(SuperNotConstructorDiagnostic(span, super_class_span));
                    }
                });
            });
        }
    }
}

trait NonConstructor {
    fn span(&self) -> Option<Span>;
}

impl<'a> NonConstructor for Expression<'a> {
    fn span(&self) -> Option<Span> {
        match self {
            Self::NullLiteral(lit) => Some(lit.span),
            Self::NumberLiteral(lit) => Some(lit.span),
            Self::StringLiteral(lit) => Some(lit.span),
            _ => None,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("class A { }", None),
        ("class A { constructor() { } }", None),
        ("class A extends null { }", None),
        ("class A extends B { constructor() { super(); } }", None),
        ("class A extends B { }", None),
        ("class A extends B { constructor() { super(); } }", None),
        ("class A extends (class B {}) { constructor() { super(); } }", None),
        ("class A extends (B = C) { constructor() { super(); } }", None),
        ("class A extends (B &&= C) { constructor() { super(); } }", None),
        ("class A extends (B ||= C) { constructor() { super(); } }", None),
        ("class A extends (B ??= C) { constructor() { super(); } }", None),
        ("class A extends (B ||= 5) { constructor() { super(); } }", None),
        ("class A extends (B ??= 5) { constructor() { super(); } }", None),
        ("class A extends (B || C) { constructor() { super(); } }", None),
        ("class A extends (5 && B) { constructor() { super(); } }", None),
    ];

    let fail = vec![
        ("class A extends B { constructor() {} }", None),
        ("class A extends null { constructor() { super(); } }", None),
        ("class A extends null { constructor() { } }", None),
        ("class A extends 100 { constructor() { super(); } }", None),
        ("class A extends 'test' { constructor() { super(); } }", None),
    ];

    Tester::new(ConstructorSuper::NAME, pass, fail).test_and_snapshot();
}
