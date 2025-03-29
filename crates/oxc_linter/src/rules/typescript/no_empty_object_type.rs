use std::borrow::Cow;

use oxc_ast::{
    AstKind,
    ast::{TSInterfaceDeclaration, TSTypeLiteral},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_empty_object_type_diagnostic<S: Into<Cow<'static, str>>>(
    span: Span,
    message: S,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(message)
        .with_help("To avoid confusion around the {} type allowing any non-nullish value, this rule bans usage of the {} type.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoEmptyObjectType(Box<NoEmptyObjectTypeConfig>);

#[expect(clippy::struct_field_names)]
#[derive(Debug, Default, Clone)]
pub struct NoEmptyObjectTypeConfig {
    /** Whether to allow empty interfaces. */
    allow_interfaces: AllowInterfaces,
    /** Whether to allow empty object type literals. */
    allow_object_types: AllowObjectTypes,
    /** allow interfaces and object type aliases with the configured name */
    allow_with_name: String,
}

impl std::ops::Deref for NoEmptyObjectType {
    type Target = NoEmptyObjectTypeConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// To avoid confusion around the `{}` type allowing any non-nullish value, this rule bans usage of the `{}` type. That includes interfaces and object type aliases with no fields.
    ///
    /// ### Why is this bad?
    /// The `{}`, or "empty object" type in TypeScript is a common source of confusion for developers unfamiliar with TypeScript's structural typing. `{}` represents any non-nullish value, including literals like 0 and "".
    /// Often, developers writing `{}` actually mean either:
    /// - object: representing any object value
    /// - unknown: representing any value at all, including null and undefined
    /// In other words, the "empty object" type {}` really means "any value that is defined". That includes arrays, class instances, functions, and primitives such as string and symbol.
    ///
    /// Note that this rule does not report on:
    /// - `{}` as a type constituent in an intersection type (e.g. types like TypeScript's built-in `type NonNullable<T> = T & {}`), as this can be useful in type system operations.
    /// - Interfaces that extend from multiple other interfaces.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// let anyObject: {};
    /// let anyValue: {};
    /// interface AnyObjectA {}
    /// interface AnyValueA {}
    /// type AnyObjectB = {};
    /// type AnyValueB = {};
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// let anyObject: object;
    /// let anyValue: unknown;
    /// type AnyObjectA = object;
    /// type AnyValueA = unknown;
    /// type AnyObjectB = object;
    /// type AnyValueB = unknown;
    /// let objectWith: { property: boolean };
    /// interface InterfaceWith {
    ///   property: boolean;
    /// }
    /// type TypeWith = { property: boolean };
    /// ```
    NoEmptyObjectType,
    typescript,
    restriction,
);

impl Rule for NoEmptyObjectType {
    fn from_configuration(value: serde_json::Value) -> Self {
        let (allow_interfaces, allow_object_types, allow_with_name) = value.get(0).map_or(
            (AllowInterfaces::Never, AllowObjectTypes::Never, String::default()),
            |config| {
                (
                    config
                        .get("allowInterfaces")
                        .and_then(serde_json::Value::as_str)
                        .map(AllowInterfaces::from)
                        .unwrap_or_default(),
                    config
                        .get("allowObjectTypes")
                        .and_then(serde_json::Value::as_str)
                        .map(AllowObjectTypes::from)
                        .unwrap_or_default(),
                    config
                        .get("allowWithName")
                        .and_then(serde_json::Value::as_str)
                        .map(String::from)
                        .unwrap_or_default(),
                )
            },
        );
        Self(Box::new(NoEmptyObjectTypeConfig {
            allow_interfaces,
            allow_object_types,
            allow_with_name,
        }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::TSInterfaceDeclaration(interface) if interface.body.body.len() == 0 => {
                check_interface_declaration(
                    ctx,
                    interface,
                    self.allow_interfaces,
                    &self.allow_with_name,
                );
            }
            AstKind::TSTypeLiteral(typeliteral) if typeliteral.members.len() == 0 => {
                check_type_literal(
                    ctx,
                    typeliteral,
                    node.id(),
                    self.allow_object_types,
                    &self.allow_with_name,
                );
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn check_interface_declaration(
    ctx: &LintContext,
    interface: &TSInterfaceDeclaration,
    allow_interfaces: AllowInterfaces,
    allow_with_name: &str,
) {
    if matches!(allow_interfaces, AllowInterfaces::Always) {
        return;
    };
    if interface.id.name.as_str() == allow_with_name {
        return;
    }
    match interface.extends.as_ref() {
        Some(extends) if extends.len() == 1 => {
            match allow_interfaces {
                AllowInterfaces::WithSingleExtends => (),
                _ => ctx.diagnostic(no_empty_object_type_diagnostic(
                    interface.body.span,
                    "Do not use an empty interface declaration.",
                )),
            };
        }
        Some(extends) if extends.len() == 0 => {
            ctx.diagnostic(no_empty_object_type_diagnostic(
                interface.body.span,
                "Do not use an empty interface declaration.",
            ));
        }
        None => ctx.diagnostic(no_empty_object_type_diagnostic(
            interface.body.span,
            "Do not use an empty interface declaration.",
        )),
        _ => (),
    }
}

fn check_type_literal(
    ctx: &LintContext,
    type_literal: &TSTypeLiteral,
    node_id: NodeId,
    allow_object_types: AllowObjectTypes,
    allow_with_name: &str,
) {
    if matches!(allow_object_types, AllowObjectTypes::Always) {
        return;
    };
    let Some(parent_node) = ctx.nodes().parent_node(node_id) else {
        return;
    };
    match parent_node.kind() {
        AstKind::TSIntersectionType(_) => return,
        AstKind::TSTypeAliasDeclaration(alias) => {
            if alias.id.name.as_str() == allow_with_name {
                return;
            }
        }
        _ => (),
    }
    ctx.diagnostic(no_empty_object_type_diagnostic(
        type_literal.span,
        "Do not use the empty object type literal.",
    ));
}

#[derive(Debug, Default, Clone, Copy)]
enum AllowInterfaces {
    #[default]
    Never,
    Always,
    WithSingleExtends,
}

impl From<&str> for AllowInterfaces {
    fn from(raw: &str) -> Self {
        match raw {
            "always" => Self::Always,
            "with-single-extends" => Self::WithSingleExtends,
            _ => Self::Never,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
enum AllowObjectTypes {
    #[default]
    Never,
    Always,
}

impl From<&str> for AllowObjectTypes {
    fn from(raw: &str) -> Self {
        match raw {
            "always" => Self::Always,
            _ => Self::Never,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			interface Base {
			  name: string;
			}
			    ",
            None,
        ),
        (
            "
			interface Base {
			  name: string;
			}

			interface Derived {
			  age: number;
			}

			// valid because extending multiple interfaces can be used instead of a union type
			interface Both extends Base, Derived {}
			    ",
            None,
        ),
        ("interface Base {}", Some(serde_json::json!([{ "allowInterfaces": "always" }]))),
        (
            "
			interface Base {
			  name: string;
			}

			interface Derived extends Base {}
			      ",
            Some(serde_json::json!([{ "allowInterfaces": "with-single-extends" }])),
        ),
        (
            "
			interface Base {
			  props: string;
			}

			interface Derived extends Base {}

			class Derived {}
			      ",
            Some(serde_json::json!([{ "allowInterfaces": "with-single-extends" }])),
        ),
        ("let value: object;", None),
        ("let value: Object;", None),
        ("let value: { inner: true };", None),
        ("type MyNonNullable<T> = T & {};", None),
        ("type Base = {};", Some(serde_json::json!([{ "allowObjectTypes": "always" }]))),
        ("type Base = {};", Some(serde_json::json!([{ "allowWithName": "Base" }]))),
        ("type BaseProps = {};", Some(serde_json::json!([{ "allowWithName": "BaseProps" }]))),
        ("interface Base {}", Some(serde_json::json!([{ "allowWithName": "Base" }]))),
        ("interface BaseProps {}", Some(serde_json::json!([{ "allowWithName": "BaseProps" }]))),
    ];

    let fail = vec![
        ("interface Base {}", None),
        ("interface Base {}", Some(serde_json::json!([{ "allowInterfaces": "never" }]))),
        (
            "
			interface Base {
			  props: string;
			}

			interface Derived extends Base {}

			class Other {}
			      ",
            None,
        ),
        (
            "
			interface Base {
			  props: string;
			}

			interface Derived extends Base {}

			class Derived {}
			      ",
            None,
        ),
        (
            "
			interface Base {
			  props: string;
			}

			interface Derived extends Base {}

			const derived = class Derived {};
			      ",
            None,
        ),
        (
            "
			interface Base {
			  name: string;
			}

			interface Derived extends Base {}
			      ",
            None,
        ),
        ("interface Base extends Array<number> {}", None),
        ("interface Base extends Array<number | {}> {}", None),
        (
            "
			interface Derived {
			  property: string;
			}
			interface Base extends Array<Derived> {}
			      ",
            None,
        ),
        (
            "
			type R = Record<string, unknown>;
			interface Base extends R {}
			      ",
            None,
        ),
        ("interface Base<T> extends Derived<T> {}", None),
        (
            "
			declare namespace BaseAndDerived {
			  type Base = typeof base;
			  export interface Derived extends Base {}
			}
			      ",
            None,
        ),
        ("type Base = {};", None),
        ("type Base = {};", Some(serde_json::json!([{ "allowObjectTypes": "never" }]))),
        ("let value: {};", None),
        ("let value: {};", Some(serde_json::json!([{ "allowObjectTypes": "never" }]))),
        (
            "
			let value: {
			  /* ... */
			};
			      ",
            None,
        ),
        ("type MyUnion<T> = T | {};", None),
        ("type Base = {} | null;", Some(serde_json::json!([{ "allowWithName": "Base" }]))),
        ("type Base = {};", Some(serde_json::json!([{ "allowWithName": "Mismatch" }]))),
        ("interface Base {}", Some(serde_json::json!([{ "allowWithName": "Props" }]))),
    ];

    Tester::new(NoEmptyObjectType::NAME, NoEmptyObjectType::PLUGIN, pass, fail).test_and_snapshot();
}
