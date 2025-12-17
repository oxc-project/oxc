use std::borrow::Cow;

use lazy_regex::Regex;
use oxc_ast::{
    AstKind,
    ast::{TSInterfaceDeclaration, TSTypeLiteral},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

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
#[derive(Debug, Default, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NoEmptyObjectTypeConfig {
    /// Whether to allow empty interfaces.
    ///
    /// Allowed values are:
    /// - `'always'`: to always allow interfaces with no fields
    /// - `'never'` _(default)_: to never allow interfaces with no fields
    /// - `'with-single-extends'`: to allow empty interfaces that `extend` from a single base interface
    ///
    /// Examples of **correct** code for this rule with `{ allowInterfaces: 'with-single-extends' }`:
    /// ```ts
    /// interface Base {
    ///   value: boolean;
    /// }
    /// interface Derived extends Base {}
    /// ```
    allow_interfaces: AllowInterfaces,
    /// Whether to allow empty object type literals.
    ///
    /// Allowed values are:
    /// - `'always'`: to always allow object type literals with no fields
    /// - `'never'` _(default)_: to never allow object type literals with no fields
    allow_object_types: AllowObjectTypes,
    /// A stringified regular expression to allow interfaces and object type aliases with the configured name.
    ///
    /// This can be useful if your existing code style includes a pattern of declaring empty types with `{}` instead of `object`.
    ///
    /// Example of **incorrect** code for this rule with `{ allowWithName: 'Props$' }`:
    /// ```ts
    /// interface InterfaceValue {}
    /// type TypeValue = {};
    /// ```
    ///
    /// Example of **correct** code for this rule with `{ allowWithName: 'Props$' }`:
    /// ```ts
    /// interface InterfaceProps {}
    /// type TypeProps = {};
    /// ```
    allow_with_name: Option<Regex>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
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

#[derive(Debug, Default, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
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

impl std::ops::Deref for NoEmptyObjectType {
    type Target = NoEmptyObjectTypeConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// To avoid confusion around the `{}` type allowing any non-nullish value, this rule bans usage of the `{}` type. That includes interfaces and object type aliases with no fields.
    ///
    /// ### Why is this bad?
    ///
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
    config = NoEmptyObjectTypeConfig,
);

impl Rule for NoEmptyObjectType {
    fn from_configuration(value: serde_json::Value) -> Self {
        let (allow_interfaces, allow_object_types, allow_with_name) = value.get(0).map_or(
            (AllowInterfaces::Never, AllowObjectTypes::Never, None),
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
                        .and_then(|pattern| Regex::new(pattern).ok()),
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
            AstKind::TSInterfaceDeclaration(interface) if interface.body.body.is_empty() => {
                check_interface_declaration(
                    ctx,
                    interface,
                    self.allow_interfaces,
                    self.allow_with_name.as_ref(),
                );
            }
            AstKind::TSTypeLiteral(typeliteral) if typeliteral.members.is_empty() => {
                check_type_literal(
                    ctx,
                    typeliteral,
                    node.id(),
                    self.allow_object_types,
                    self.allow_with_name.as_ref(),
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
    allow_with_name: Option<&Regex>,
) {
    if allow_interfaces == AllowInterfaces::Always {
        return;
    }
    if let Some(pattern) = allow_with_name
        && pattern.is_match(interface.id.name.as_str())
    {
        return;
    }
    if interface.extends.is_empty()
        || (allow_interfaces == AllowInterfaces::Never && interface.extends.len() == 1)
    {
        ctx.diagnostic(no_empty_object_type_diagnostic(
            interface.body.span,
            "Do not use an empty interface declaration.",
        ));
    }
}

fn check_type_literal(
    ctx: &LintContext,
    type_literal: &TSTypeLiteral,
    node_id: NodeId,
    allow_object_types: AllowObjectTypes,
    allow_with_name: Option<&Regex>,
) {
    if matches!(allow_object_types, AllowObjectTypes::Always) {
        return;
    }
    match ctx.nodes().parent_kind(node_id) {
        AstKind::TSIntersectionType(_) => return,
        AstKind::TSTypeAliasDeclaration(alias) => {
            if let Some(pattern) = allow_with_name
                && pattern.is_match(alias.id.name.as_str())
            {
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
        ("type BaseProps = {};", Some(serde_json::json!([{ "allowWithName": "Props$" }]))),
        ("interface Base {}", Some(serde_json::json!([{ "allowWithName": "Base" }]))),
        ("interface BaseProps {}", Some(serde_json::json!([{ "allowWithName": "BaseProps" }]))),
        ("interface BaseProps {}", Some(serde_json::json!([{ "allowWithName": "Props$" }]))),
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
