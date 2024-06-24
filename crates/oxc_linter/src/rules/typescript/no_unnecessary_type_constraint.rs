use oxc_ast::{ast::TSType, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_unnecessary_type_constraint_diagnostic(
    x0: &str,
    x1: &str,
    span2: Span,
    span3: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("typescript-eslint(no-unnecessary-type-constraint): constraining the generic type {x0:?} to {x1:?} does nothing and is unnecessary"))
        .with_help(format!("Remove the unnecessary {x1:?} constraint"))
        .with_labels([span2.into(), span3.into()])
}

#[derive(Debug, Default, Clone)]
pub struct NoUnnecessaryTypeConstraint;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unnecessary constraints on generic types.
    ///
    /// ### Why is this bad?
    ///
    /// Generic type parameters (<T>) in TypeScript may be "constrained" with an extends keyword.
    /// When no extends is provided, type parameters default a constraint to unknown. It is therefore redundant to extend from any or unknown.
    ///
    /// ### Example
    /// ```javascript
    /// interface FooAny<T extends any> {}
    /// interface FooUnknown<T extends unknown> {}
    /// type BarAny<T extends any> = {};
    /// type BarUnknown<T extends unknown> = {};
    /// class BazAny<T extends any> {
    ///   quxAny<U extends any>() {}
    /// }
    /// const QuuxAny = <T extends any>() => {};
    /// function QuuzAny<T extends any>() {}
    /// ```
    NoUnnecessaryTypeConstraint,
    suspicious
);

impl Rule for NoUnnecessaryTypeConstraint {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::TSTypeParameterDeclaration(decl) = node.kind() {
            for param in &decl.params {
                if let Some(ty) = &param.constraint {
                    let (value, ty_span) = match ty {
                        TSType::TSAnyKeyword(t) => ("any", t.span),
                        TSType::TSUnknownKeyword(t) => ("unknown", t.span),
                        _ => continue,
                    };
                    ctx.diagnostic(no_unnecessary_type_constraint_diagnostic(
                        param.name.name.as_str(),
                        value,
                        param.name.span,
                        ty_span,
                    ));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function data() {}",
        "function data<T>() {}",
        "function data<T, U>() {}",
        "function data<T extends number>() {}",
        "function data<T extends number | string>() {}",
        "function data<T extends any | number>() {}",
        "type X = any; function data<T extends X>() {}",
        "const data = () => {};",
        "const data = <T, >() => {};",
        "const data = <T, U>() => {};",
        "const data = <T extends number>() => {};",
        "const data = <T extends number | string>() => {};",
    ];

    let fail = vec![
        "function data<T extends any>() {}",
        "function data<T extends any, U>() {}",
        "function data<T, U extends any>() {}",
        "function data<T extends any, U extends T>() {}",
        "const data = <T extends any>() => {};",
        "const data = <T extends any,>() => {};",
        "const data = <T extends any, >() => {};",
        "const data = <T extends any ,>() => {};",
        "const data = <T extends any , >() => {};",
        "const data = <T extends any = unknown>() => {};",
        "const data = <T extends any, U extends any>() => {};",
        "function data<T extends unknown>() {}",
        "const data = <T extends any>() => {};",
        "const data = <T extends unknown>() => {};",
        "class Data<T extends unknown> {}",
        "const Data = class<T extends unknown> {};",
        "class Data { member<T extends unknown>() {} }",
        "const Data = class { member<T extends unknown>() {} };",
        "interface Data<T extends unknown> {}",
        "type Data<T extends unknown> = {};",
    ];

    Tester::new(NoUnnecessaryTypeConstraint::NAME, pass, fail).test_and_snapshot();
}
