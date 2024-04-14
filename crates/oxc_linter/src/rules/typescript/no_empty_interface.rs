use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

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
pub struct NoEmptyInterface {
    allow_single_extends: bool,
}

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
    style
);

impl Rule for NoEmptyInterface {
    fn from_configuration(value: Value) -> Self {
        let allow_single_extends = value.get(0).map_or(true, |config| {
            config.get("allow_single_extends").and_then(Value::as_bool).unwrap_or_default()
        });

        Self { allow_single_extends }
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::TSInterfaceDeclaration(interface) = node.kind() {
            if interface.body.body.is_empty() {
                match &interface.extends {
                    None => {
                        ctx.diagnostic(NoEmptyInterfaceDiagnostic(interface.span));
                    }

                    Some(extends) if extends.len() == 1 => {
                        if !self.allow_single_extends {
                            ctx.diagnostic(NoEmptyInterfaceExtendDiagnostic(interface.span));
                        }
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
        (
            "
			interface Foo {
			  name: string;
			}
			    ",
            None,
        ),
        (
            "
			interface Foo {
			  name: string;
			}

			interface Bar {
			  age: number;
			}

			// valid because extending multiple interfaces can be used instead of a union type
			interface Baz extends Foo, Bar {}
			    ",
            None,
        ),
        (
            "
			interface Foo {
			  name: string;
			}

			interface Bar extends Foo {}
			      ",
            Some(serde_json::json!([{ "allow_single_extends": true }])),
        ),
        (
            "
			interface Foo {
			  props: string;
			}

			interface Bar extends Foo {}

			class Bar {}
			      ",
            Some(serde_json::json!([{ "allow_single_extends": true }])),
        ),
    ];

    let fail = vec![
        ("interface Foo {}", None),
        ("interface Foo extends {}", None),
        (
            "
			interface Foo {
			  props: string;
			}

			interface Bar extends Foo {}

			class Baz {}
			      ",
            Some(serde_json::json!([{ "allow_single_extends": false }])),
        ),
        (
            "
			interface Foo {
			  props: string;
			}

			interface Bar extends Foo {}

			class Bar {}
			      ",
            Some(serde_json::json!([{ "allow_single_extends": false }])),
        ),
        (
            "
			interface Foo {
			  props: string;
			}

			interface Bar extends Foo {}

			const bar = class Bar {};
			      ",
            Some(serde_json::json!([{ "allow_single_extends": false }])),
        ),
        (
            "
			interface Foo {
			  name: string;
			}

			interface Bar extends Foo {}
			      ",
            Some(serde_json::json!([{ "allow_single_extends": false }])),
        ),
        ("interface Foo extends Array<number> {}", None),
        ("interface Foo extends Array<number | {}> {}", None),
        (
            "
			interface Bar {
			  bar: string;
			}
			interface Foo extends Array<Bar> {}
			      ",
            None,
        ),
        (
            "
			type R = Record<string, unknown>;
			interface Foo extends R {}
			      ",
            None,
        ),
        (
            "
			interface Foo<T> extends Bar<T> {}
			      ",
            None,
        ),
        (
            "
			declare module FooBar {
			  type Baz = typeof baz;
			  export interface Bar extends Baz {}
			}
			      ",
            None,
        ),
    ];

    Tester::new(NoEmptyInterface::NAME, pass, fail).test_and_snapshot();
}
