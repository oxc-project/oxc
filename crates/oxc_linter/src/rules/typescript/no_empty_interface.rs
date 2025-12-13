use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

fn no_empty_interface_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("an empty interface is equivalent to `{}`").with_label(span)
}

fn no_empty_interface_extend_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("an interface declaring no members is equivalent to its supertype")
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoEmptyInterface {
    /// When set to `true`, allows empty interfaces that extend a single interface.
    #[serde(alias = "allow_single_extends")]
    // for backwards-compatibility, we made a mistake in the naming previously
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
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// interface Foo {}
    /// interface Bar extends Foo {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// interface Foo {
    ///     member: string;
    /// }
    /// interface Bar extends Foo {
    ///     member: string;
    /// }
    /// ```
    NoEmptyInterface,
    typescript,
    style,
    config = NoEmptyInterface,
);

impl Rule for NoEmptyInterface {
    fn from_configuration(value: Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoEmptyInterface>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::TSInterfaceDeclaration(interface) = node.kind()
            && interface.body.body.is_empty()
        {
            if interface.extends.is_empty() {
                ctx.diagnostic(no_empty_interface_diagnostic(interface.span));
            } else if interface.extends.len() == 1 && !self.allow_single_extends {
                ctx.diagnostic(no_empty_interface_extend_diagnostic(interface.span));
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
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
            Some(serde_json::json!([{ "allowSingleExtends": true }])),
        ),
        (
            "
			interface Foo {
			  props: string;
			}

			interface Bar extends Foo {}

			class Bar {}
			      ",
            Some(serde_json::json!([{ "allowSingleExtends": true }])),
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
            Some(serde_json::json!([{ "allowSingleExtends": false }])),
        ),
        (
            "
			interface Foo {
			  props: string;
			}

			interface Bar extends Foo {}

			class Bar {}
			      ",
            Some(serde_json::json!([{ "allowSingleExtends": false }])),
        ),
        (
            "
			interface Foo {
			  props: string;
			}

			interface Bar extends Foo {}

			const bar = class Bar {};
			      ",
            Some(serde_json::json!([{ "allowSingleExtends": false }])),
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
            Some(serde_json::json!([{ "allowSingleExtends": false }])),
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

    Tester::new(NoEmptyInterface::NAME, NoEmptyInterface::PLUGIN, pass, fail).test_and_snapshot();
}
