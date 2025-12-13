use std::borrow::Cow;

use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

fn no_restricted_types_diagnostic(
    type_name: &str,
    message: Option<&str>,
    span: Span,
) -> OxcDiagnostic {
    let msg = message.unwrap_or_default();
    if msg.is_empty() {
        OxcDiagnostic::warn(format!("Do not use `{type_name}` as a type"))
            .with_help("This type is restricted from being used.")
            .with_label(span)
    } else {
        OxcDiagnostic::warn(format!("Do not use `{type_name}` as a type"))
            .with_help(msg.to_string())
            .with_label(span)
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedTypes(Box<NoRestrictedTypesConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
struct NoRestrictedTypesConfig {
    /// A mapping of type names to ban configurations.
    #[serde(default)]
    types: FxHashMap<String, BanConfigValue>,
}

/// Represents the different ways a ban config can be specified in JSON.
/// Can be:
/// - `true` - ban with default message
/// - A string - ban with custom message
/// - An object with `message` and optional `fixWith` and `suggest`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
enum BanConfigValue {
    /// `"TypeName": true` - ban with default message
    /// Note: Only `true` is valid; `false` would fail to deserialize and be ignored.
    Bool(True),
    /// `"TypeName": "message"` - ban with custom message
    Message(String),
    /// `"TypeName": { "message": "...", "fixWith": "...", "suggest": ["..."] }` - full config
    Object {
        /// Custom message explaining why the type is banned.
        #[serde(default)]
        message: Option<String>,
        /// Replacement type for automatic fixing. Applied directly with `--fix`.
        #[serde(default, rename = "fixWith")]
        fix_with: Option<String>,
        /// Suggested replacement types for manual review. Shown as editor suggestions.
        suggest: Option<Vec<String>>,
    },
}

/// A type that only deserializes from `true`.
/// This matches the upstream typescript-eslint schema which only allows `true`, not `false`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
struct True;

impl<'de> Deserialize<'de> for True {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = bool::deserialize(deserializer)?;
        if value { Ok(True) } else { Err(serde::de::Error::custom("expected `true`, got `false`")) }
    }
}

impl BanConfigValue {
    fn message(&self) -> Option<&str> {
        match self {
            BanConfigValue::Bool(_) => None,
            BanConfigValue::Message(msg) => Some(msg.as_str()),
            BanConfigValue::Object { message, .. } => message.as_deref(),
        }
    }

    fn fix_with(&self) -> Option<&str> {
        match self {
            BanConfigValue::Bool(_) | BanConfigValue::Message(_) => None,
            BanConfigValue::Object { fix_with, .. } => fix_with.as_deref(),
        }
    }

    fn suggest(&self) -> Option<&[String]> {
        match self {
            BanConfigValue::Bool(_) | BanConfigValue::Message(_) => None,
            BanConfigValue::Object { suggest, .. } => suggest.as_deref(),
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow certain types from being used.
    ///
    /// ### Why is this bad?
    ///
    /// Some built-in types have aliases, while some types are considered dangerous or harmful.
    /// It's often a good idea to ban certain types to help with consistency and safety.
    ///
    /// ### Examples
    ///
    /// Given `{ "types": { "Foo": { "message": "Use Bar instead", "fixWith": "Bar" } } }`:
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// let value: Foo;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// let value: Bar;
    /// ```
    ///
    /// Other examples of configuration option setups for this rule:
    ///
    /// - Banning the `Foo` type with just a message, no fixes or suggestions:
    ///   `{ "types": { "Foo": "Use `OtherType` instead." } }`
    ///
    /// - Banning `Bar` type with suggestion:
    ///   `{ "types": { "Bar": { "message": "Avoid using `Bar`.", "suggest": "BazQux" } } }`
    ///
    /// - Banning `Object` type with a generic message:
    ///   `{ "types": { "Object": true } }`
    NoRestrictedTypes,
    typescript,
    restriction,
    fix_suggestion,
    config = NoRestrictedTypesConfig,
);

impl Rule for NoRestrictedTypes {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(
            serde_json::from_value::<DefaultRuleConfig<NoRestrictedTypesConfig>>(value)
                .unwrap_or_default()
                .into_inner(),
        ))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            // Handle type references like `let x: Foo` or `let x: NS.Foo`
            AstKind::TSTypeReference(type_ref) => {
                // Check the type name (e.g., `Banned` in `Banned<any>`)
                self.check_banned_types(type_ref.type_name.span(), ctx);

                // If there are type arguments, also check the full type (e.g., `Banned<any>`)
                if type_ref.type_arguments.is_some() {
                    self.check_banned_types(type_ref.span, ctx);
                }
            }
            // Handle primitive keyword types like `let x: string`, `let x: null`, etc.
            AstKind::TSStringKeyword(kw) => {
                if let Some((matched_key, config)) = self.find_matching_type("string") {
                    report(ctx, kw.span, matched_key, config);
                }
            }
            AstKind::TSNumberKeyword(kw) => {
                if let Some((matched_key, config)) = self.find_matching_type("number") {
                    report(ctx, kw.span, matched_key, config);
                }
            }
            AstKind::TSBooleanKeyword(kw) => {
                if let Some((matched_key, config)) = self.find_matching_type("boolean") {
                    report(ctx, kw.span, matched_key, config);
                }
            }
            AstKind::TSNullKeyword(kw) => {
                if let Some((matched_key, config)) = self.find_matching_type("null") {
                    report(ctx, kw.span, matched_key, config);
                }
            }
            AstKind::TSUndefinedKeyword(kw) => {
                if let Some((matched_key, config)) = self.find_matching_type("undefined") {
                    report(ctx, kw.span, matched_key, config);
                }
            }
            AstKind::TSSymbolKeyword(kw) => {
                if let Some((matched_key, config)) = self.find_matching_type("symbol") {
                    report(ctx, kw.span, matched_key, config);
                }
            }
            AstKind::TSBigIntKeyword(kw) => {
                if let Some((matched_key, config)) = self.find_matching_type("bigint") {
                    report(ctx, kw.span, matched_key, config);
                }
            }
            AstKind::TSObjectKeyword(kw) => {
                if let Some((matched_key, config)) = self.find_matching_type("object") {
                    report(ctx, kw.span, matched_key, config);
                }
            }
            AstKind::TSVoidKeyword(kw) => {
                if let Some((matched_key, config)) = self.find_matching_type("void") {
                    report(ctx, kw.span, matched_key, config);
                }
            }
            AstKind::TSNeverKeyword(kw) => {
                if let Some((matched_key, config)) = self.find_matching_type("never") {
                    report(ctx, kw.span, matched_key, config);
                }
            }
            AstKind::TSUnknownKeyword(kw) => {
                if let Some((matched_key, config)) = self.find_matching_type("unknown") {
                    report(ctx, kw.span, matched_key, config);
                }
            }
            AstKind::TSAnyKeyword(kw) => {
                if let Some((matched_key, config)) = self.find_matching_type("any") {
                    report(ctx, kw.span, matched_key, config);
                }
            }
            // Handle empty tuple type `[]`
            AstKind::TSTupleType(tuple) => {
                if tuple.element_types.is_empty() {
                    self.check_banned_types(tuple.span, ctx);
                }
            }
            // Handle empty object type `{}`
            AstKind::TSTypeLiteral(lit) => {
                if lit.members.is_empty() {
                    self.check_banned_types(lit.span, ctx);
                }
            }
            // Handle `class X implements Banned`
            AstKind::TSClassImplements(implements) => {
                self.check_banned_types(implements.span, ctx);
            }
            // Handle `interface X extends Banned`
            AstKind::TSInterfaceHeritage(heritage) => {
                self.check_banned_types(heritage.span, ctx);
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript() && !self.0.types.is_empty()
    }
}

impl NoRestrictedTypes {
    /// Find a matching banned type configuration.
    /// Both the config key and type_name are normalized by removing all whitespace.
    fn find_matching_type(&self, type_name: &str) -> Option<(&str, &BanConfigValue)> {
        // Normalize the type name by removing whitespace
        let normalized_type_name = remove_spaces(type_name);
        for (key, value) in &self.0.types {
            // Normalize the key by removing whitespace
            let normalized_key = remove_spaces(key);
            if normalized_key == normalized_type_name {
                // Return the original (trimmed) key for error messages
                return Some((key.trim(), value));
            }
        }
        None
    }

    fn check_banned_types(&self, span: Span, ctx: &LintContext<'_>) {
        let source_text = ctx.source_range(span);
        if let Some((matched_key, config)) = self.find_matching_type(source_text) {
            report(ctx, span, matched_key, config);
        }
    }
}

fn report(ctx: &LintContext<'_>, span: Span, type_name: &str, config: &BanConfigValue) {
    let diagnostic = no_restricted_types_diagnostic(type_name, config.message(), span);

    if let Some(fix_with) = config.fix_with() {
        // `fixWith` provides an auto-fix
        let fix_with = fix_with.to_string();
        ctx.diagnostic_with_fix(diagnostic, |fixer| fixer.replace(span, fix_with));
    } else if let Some(suggestions) = config.suggest() {
        // TODO: Support multiple suggestions in the future
        if let Some(first_suggestion) = suggestions.first() {
            let suggestion = first_suggestion.clone();
            ctx.diagnostic_with_suggestion(diagnostic, |fixer| fixer.replace(span, suggestion));
        } else {
            ctx.diagnostic(diagnostic);
        }
    } else {
        ctx.diagnostic(diagnostic);
    }
}

/// Remove all whitespace from a string for normalization.
/// This matches the TypeScript reference implementation's `removeSpaces` function.
fn remove_spaces(s: &str) -> Cow<'_, str> {
    if s.contains(char::is_whitespace) {
        Cow::Owned(s.chars().filter(|c| !c.is_whitespace()).collect())
    } else {
        Cow::Borrowed(s)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("let f = Object();", None),
        ("let f: { x: number; y: number } = { x: 1, y: 1 };", None),
        ("let f = Object();", Some(serde_json::json!([{ "types": { "Object": true } }]))),
        ("let f = Object(false);", Some(serde_json::json!([{ "types": { "Object": true } }]))),
        (
            "let g = Object.create(null);",
            Some(serde_json::json!([{ "types": { "Object": true } }])),
        ),
        ("let e: namespace.Object;", Some(serde_json::json!([{ "types": { "Object": true } }]))),
        ("let value: _.NS.Banned;", Some(serde_json::json!([{ "types": { "NS.Banned": true } }]))),
        ("let value: NS.Banned._;", Some(serde_json::json!([{ "types": { "NS.Banned": true } }]))),
        ("let f: any = true", Some(serde_json::json!([{ "types": {} }]))),
    ];

    let fail = vec![
        (
            "let value: bigint;",
            Some(serde_json::json!([{ "types": { "bigint": "Use Ok instead." } }])),
        ),
        (
            "let value: boolean;",
            Some(serde_json::json!([{ "types": { "boolean": "Use Ok instead." } }])),
        ),
        (
            "let value: never;",
            Some(serde_json::json!([{ "types": { "never": "Use Ok instead." } }])),
        ),
        ("let value: null;", Some(serde_json::json!([{ "types": { "null": "Use Ok instead." } }]))),
        (
            "let value: number;",
            Some(serde_json::json!([{ "types": { "number": "Use Ok instead." } }])),
        ),
        (
            "let value: object;",
            Some(serde_json::json!([{ "types": { "object": "Use Ok instead." } }])),
        ),
        (
            "let value: string;",
            Some(serde_json::json!([{ "types": { "string": "Use Ok instead." } }])),
        ),
        (
            "let value: symbol;",
            Some(serde_json::json!([{ "types": { "symbol": "Use Ok instead." } }])),
        ),
        (
            "let value: undefined;",
            Some(serde_json::json!([{ "types": { "undefined": "Use Ok instead." } }])),
        ),
        (
            "let value: unknown;",
            Some(serde_json::json!([{ "types": { "unknown": "Use Ok instead." } }])),
        ),
        (
            "let value: any;",
            Some(serde_json::json!([{ "types": { "any": "Use unknown instead." } }])),
        ),
        ("let value: void;", Some(serde_json::json!([{ "types": { "void": "Use Ok instead." } }]))),
        (
            "let value: [];",
            Some(serde_json::json!([{ "types": { "[]": "Use unknown[] instead." } }])),
        ),
        (
            "let value: [  ];",
            Some(serde_json::json!([{ "types": { "[]": "Use unknown[] instead." } }])),
        ),
        (
            "let value: [[]];",
            Some(serde_json::json!([{ "types": { "[]": "Use unknown[] instead." } }])),
        ),
        ("let value: Banned;", Some(serde_json::json!([{ "types": { "Banned": true } }]))),
        (
            "let value: Banned;",
            Some(serde_json::json!([{ "types": { "Banned": r#"Use "{}" instead."#} }])),
        ),
        (
            "let value: Banned[];",
            Some(serde_json::json!([{ "types": { "Banned": r#"Use "{}" instead."#} }])),
        ),
        (
            "let value: [Banned];",
            Some(serde_json::json!([{ "types": { "Banned": r#"Use "{}" instead."# } }])),
        ),
        ("let value: Banned;", Some(serde_json::json!([{ "types": { "Banned": "" } }]))),
        (
            "let b: { c: Banned };",
            Some(
                serde_json::json!([{ "types": { "Banned": { "fixWith": "Ok", "message": "Use Ok instead." } } }]),
            ),
        ),
        (
            "1 as Banned;",
            Some(
                serde_json::json!([{ "types": { "Banned": { "fixWith": "Ok", "message": "Use Ok instead." } } }]),
            ),
        ),
        (
            "class Derived implements Banned {}",
            Some(
                serde_json::json!([{ "types": { "Banned": { "fixWith": "Ok", "message": "Use Ok instead." } } }]),
            ),
        ),
        (
            "class Derived implements Banned1, Banned2 {}",
            Some(
                serde_json::json!([{ "types": { "Banned1": { "fixWith": "Ok1", "message": "Use Ok1 instead." }, "Banned2": { "fixWith": "Ok2", "message": "Use Ok2 instead." }, }, }]),
            ),
        ),
        (
            "interface Derived extends Banned {}",
            Some(
                serde_json::json!([{ "types": { "Banned": { "fixWith": "Ok", "message": "Use Ok instead." } } }]),
            ),
        ),
        (
            "type Intersection = Banned & {};",
            Some(
                serde_json::json!([{ "types": { "Banned": { "fixWith": "Ok", "message": "Use Ok instead." } } }]),
            ),
        ),
        (
            "type Union = Banned | {};",
            Some(
                serde_json::json!([{ "types": { "Banned": { "fixWith": "Ok", "message": "Use Ok instead." } } }]),
            ),
        ),
        (
            "let value: NS.Banned;",
            Some(
                serde_json::json!([{ "types": { "NS.Banned": { "fixWith": "NS.Ok", "message": "Use NS.Ok instead." } } }]),
            ),
        ),
        (
            "let value: {} = {};",
            Some(
                serde_json::json!([{ "types": { "{}": { "fixWith": "object", "message": "Use object instead." } } }]),
            ),
        ),
        (
            "let value: NS.Banned;",
            Some(
                serde_json::json!([{ "types": { "  NS.Banned  ": { "fixWith": "NS.Ok", "message": "Use NS.Ok instead." } } }]),
            ),
        ),
        (
            "let value: Type<   Banned   >;",
            Some(
                serde_json::json!([{ "types": { "       Banned      ": { "fixWith": "Ok", "message": "Use Ok instead." } } }]),
            ),
        ),
        (
            "type Intersection = Banned<any>;",
            Some(
                serde_json::json!([{ "types": { "Banned<any>": "Don't use `any` as a type parameter to `Banned`" } }]),
            ),
        ),
        (
            "type Intersection = Banned<A,B>;",
            Some(
                serde_json::json!([{ "types": { "Banned<A, B>": "Don't pass `A, B` as parameters to `Banned`" } }]),
            ),
        ),
        (
            "let value: Banned;",
            Some(
                serde_json::json!([{ "types": { "Banned": { "message": "Use a safer type", "suggest": ["SafeType", "AnotherType"] } } }]),
            ),
        ),
    ];

    let fix = vec![
        (
            "let b: { c: Banned };",
            "let b: { c: Ok };",
            Some(
                serde_json::json!([{ "types": { "Banned": { "fixWith": "Ok", "message": "Use Ok instead." } } }]),
            ),
        ),
        (
            "1 as Banned;",
            "1 as Ok;",
            Some(
                serde_json::json!([{ "types": { "Banned": { "fixWith": "Ok", "message": "Use Ok instead." } } }]),
            ),
        ),
        (
            "class Derived implements Banned {}",
            "class Derived implements Ok {}",
            Some(
                serde_json::json!([{ "types": { "Banned": { "fixWith": "Ok", "message": "Use Ok instead." } } }]),
            ),
        ),
        (
            "class Derived implements Banned1, Banned2 {}",
            "class Derived implements Ok1, Ok2 {}",
            Some(
                serde_json::json!([{ "types": { "Banned1": { "fixWith": "Ok1", "message": "Use Ok1 instead." }, "Banned2": { "fixWith": "Ok2", "message": "Use Ok2 instead." } } }]),
            ),
        ),
        (
            "interface Derived extends Banned {}",
            "interface Derived extends Ok {}",
            Some(
                serde_json::json!([{ "types": { "Banned": { "fixWith": "Ok", "message": "Use Ok instead." } } }]),
            ),
        ),
        (
            "type Intersection = Banned & {};",
            "type Intersection = Ok & {};",
            Some(
                serde_json::json!([{ "types": { "Banned": { "fixWith": "Ok", "message": "Use Ok instead." } } }]),
            ),
        ),
        (
            "type Union = Banned | {};",
            "type Union = Ok | {};",
            Some(
                serde_json::json!([{ "types": { "Banned": { "fixWith": "Ok", "message": "Use Ok instead." } } }]),
            ),
        ),
        (
            "let value: NS.Banned;",
            "let value: NS.Ok;",
            Some(
                serde_json::json!([{ "types": { "NS.Banned": { "fixWith": "NS.Ok", "message": "Use NS.Ok instead." } } }]),
            ),
        ),
        (
            "let value: {} = {};",
            "let value: object = {};",
            Some(
                serde_json::json!([{ "types": { "{}": { "fixWith": "object", "message": "Use object instead." } } }]),
            ),
        ),
        (
            "let value: NS.Banned;",
            "let value: NS.Ok;",
            Some(
                serde_json::json!([{ "types": { "  NS.Banned  ": { "fixWith": "NS.Ok", "message": "Use NS.Ok instead." } } }]),
            ),
        ),
        (
            "let value: Type<   Banned   >;",
            "let value: Type<   Ok   >;",
            Some(
                serde_json::json!([{ "types": { "       Banned      ": { "fixWith": "Ok", "message": "Use Ok instead." } } }]),
            ),
        ),
    ];
    Tester::new(NoRestrictedTypes::NAME, NoRestrictedTypes::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
