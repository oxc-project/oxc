use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(no-empty-interface): an empty interface is equivalent to `{{}}`")]
#[diagnostic(severity(warning))]
struct NoEmptyInterfaceDiagnostic(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error(
    "typescript-eslint(no-empty-interface): an interface declaring no members is equivalent to its supertype"
)]
#[diagnostic(severity(warning))]
struct NoEmptyInterfaceExtendDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoEmptyInterface;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the declaration of empty interfaces.
    ///
    /// ### Why is this bad?
    ///
    /// An empty interface in TypeScript does very little: any non-nullable value is assignable to {}.
    /// Using an empty interface is often a sign of programmer error, such as misunderstanding the concept of {} or forgetting to fill in fields.
    /// This rule aims to ensure that only meaningful interfaces are declared in the code.
    ///
    /// ### Example
    /// ```javascript
    /// interface Foo {}
    /// interface Bar extends Foo {}
    /// ```
    NoEmptyInterface,
    correctness
);

impl Rule for NoEmptyInterface {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::TSInterfaceDeclaration(interface) = node.kind() {
            if interface.body.body.is_empty() {
                match &interface.extends {
                    None => {
                        ctx.diagnostic(NoEmptyInterfaceDiagnostic(interface.span));
                    }
                    Some(extends) if extends.len() == 1 => {
                        ctx.diagnostic(NoEmptyInterfaceExtendDiagnostic(interface.span));
                    }
                    _ => {}
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "interface Foo { name: string; }",
        "interface Foo { name: string; }
        interface Bar { age: number; }
        // valid because extending multiple interfaces can be used instead of a union type
        interface Baz extends Foo, Bar {}",
    ];

    let fail = vec![
        "interface Foo {}",
        "interface Foo { props: string; } interface Bar extends Foo {} class Baz {}",
        "interface Foo { props: string; } interface Bar extends Foo {} class Bar {}",
        "interface Foo { props: string; } interface Bar extends Foo {} const bar = class Bar {};",
        "interface Foo { name: string; } interface Bar extends Foo {}",
        "interface Foo extends Array<number> {}",
        "interface Foo extends Array<number | {}> {}",
        "interface Bar { bar: string; } interface Foo extends Array<Bar> {}",
        "type R = Record<string, unknown>; interface Foo extends R {}",
        "interface Foo<T> extends Bar<T> {}",
        "declare module FooBar { type Baz = typeof baz; export interface Bar extends Baz {} }",
    ];

    Tester::new_without_config(NoEmptyInterface::NAME, pass, fail).test_and_snapshot();
}
