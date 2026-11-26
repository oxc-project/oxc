use std::borrow::Cow;

use rustc_hash::FxHashMap;
use schemars::{JsonSchema, SchemaGenerator, schema::Schema};
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{TSTypeName, TSTypeReference},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{identifier::is_white_space, line_terminator::is_line_terminator};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    fixer::RuleFixer,
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

/// Built from config once: O(1) lookups by normalized name + keyword fast flags.
#[derive(Debug, Clone, Default)]
struct RestrictedTypesIndex {
    /// Key = type name with all whitespace removed (matches TS-ESLint `removeSpaces`).
    /// Value = (display name for diagnostics, ban config).
    by_name: FxHashMap<Box<str>, (Box<str>, BanConfigValue)>,
    /// True if any banned name needs a full source span (e.g. `Banned<any>`, not `[]`/`{}`).
    has_generic_or_complex_keys: bool,
    /// True if any banned name can match a qualified type name (e.g. `NS.Banned`).
    has_qualified_keys: bool,
    ban_string: bool,
    ban_number: bool,
    ban_boolean: bool,
    ban_null: bool,
    ban_undefined: bool,
    ban_symbol: bool,
    ban_bigint: bool,
    ban_object: bool,
    ban_void: bool,
    ban_never: bool,
    ban_unknown: bool,
    ban_any: bool,
    ban_empty_tuple: bool,
    ban_empty_object: bool,
}

impl RestrictedTypesIndex {
    fn from_types_map(types: &FxHashMap<String, BanConfigValue>) -> Self {
        let mut index = Self::default();
        for (key, value) in types {
            let normalized = remove_spaces(key);
            let normalized_key = normalized.as_ref();
            let display = key.trim_matches(is_js_whitespace);

            if normalized_key.contains('.') {
                index.has_qualified_keys = true;
            }

            match normalized_key {
                "string" => index.ban_string = true,
                "number" => index.ban_number = true,
                "boolean" => index.ban_boolean = true,
                "null" => index.ban_null = true,
                "undefined" => index.ban_undefined = true,
                "symbol" => index.ban_symbol = true,
                "bigint" => index.ban_bigint = true,
                "object" => index.ban_object = true,
                "void" => index.ban_void = true,
                "never" => index.ban_never = true,
                "unknown" => index.ban_unknown = true,
                "any" => index.ban_any = true,
                "[]" => index.ban_empty_tuple = true,
                "{}" => index.ban_empty_object = true,
                other if other.contains('<') || other.contains('{') || other.contains('[') => {
                    index.has_generic_or_complex_keys = true;
                }
                _ => {}
            }
            index
                .by_name
                .entry(normalized.into_owned().into_boxed_str())
                .or_insert_with(|| (display.to_string().into_boxed_str(), value.clone()));
        }
        index
    }

    #[inline]
    fn lookup(&self, normalized_name: &str) -> Option<(&str, &BanConfigValue)> {
        self.by_name.get(normalized_name).map(|(display, config)| (&**display, config))
    }

    /// Lookup a source snippet (may contain whitespace); normalizes only when needed.
    #[inline]
    fn lookup_source(&self, source_text: &str) -> Option<(&str, &BanConfigValue)> {
        if source_text.contains(is_js_whitespace) {
            let normalized = remove_spaces(source_text);
            self.lookup(&normalized)
        } else {
            self.lookup(source_text)
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoRestrictedTypes(Box<NoRestrictedTypesConfig>);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct NoRestrictedTypesConfig {
    /// A mapping of type names to ban configurations.
    types: FxHashMap<String, BanConfigValue>,
    /// Built at config load; not serialized.
    #[serde(skip)]
    #[schemars(skip)]
    index: RestrictedTypesIndex,
}

/// Represents the different ways a ban config can be specified in JSON.
/// Can be:
/// - `true` - ban with default message
/// - A string - ban with custom message
/// - An object with `message` and optional `fixWith` and `suggest`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

impl JsonSchema for True {
    fn schema_name() -> String {
        "True".to_string()
    }

    fn schema_id() -> Cow<'static, str> {
        "True".into()
    }

    fn json_schema(r#gen: &mut SchemaGenerator) -> Schema {
        let mut schema = <bool as JsonSchema>::json_schema(r#gen).into_object();
        schema.enum_values = Some(vec![true.into()]);
        schema.into()
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
    version = "1.31.0",
    short_description = "Disallow certain types from being used.",
);

impl Rule for NoRestrictedTypes {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let mut config =
            DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)?;
        config.0.index = RestrictedTypesIndex::from_types_map(&config.0.types);
        Ok(config)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Body must be a single `match node.kind()` so `cargo lintgen` can set `NODE_TYPES`.
        match node.kind() {
            AstKind::TSTypeReference(type_ref) => {
                self.check_type_reference(type_ref, ctx);
            }
            AstKind::TSStringKeyword(kw) => {
                if self.0.index.ban_string {
                    report_keyword(ctx, &self.0.index, "string", kw.span);
                }
            }
            AstKind::TSNumberKeyword(kw) => {
                if self.0.index.ban_number {
                    report_keyword(ctx, &self.0.index, "number", kw.span);
                }
            }
            AstKind::TSBooleanKeyword(kw) => {
                if self.0.index.ban_boolean {
                    report_keyword(ctx, &self.0.index, "boolean", kw.span);
                }
            }
            AstKind::TSNullKeyword(kw) => {
                if self.0.index.ban_null {
                    report_keyword(ctx, &self.0.index, "null", kw.span);
                }
            }
            AstKind::TSUndefinedKeyword(kw) => {
                if self.0.index.ban_undefined {
                    report_keyword(ctx, &self.0.index, "undefined", kw.span);
                }
            }
            AstKind::TSSymbolKeyword(kw) => {
                if self.0.index.ban_symbol {
                    report_keyword(ctx, &self.0.index, "symbol", kw.span);
                }
            }
            AstKind::TSBigIntKeyword(kw) => {
                if self.0.index.ban_bigint {
                    report_keyword(ctx, &self.0.index, "bigint", kw.span);
                }
            }
            AstKind::TSObjectKeyword(kw) => {
                if self.0.index.ban_object {
                    report_keyword(ctx, &self.0.index, "object", kw.span);
                }
            }
            AstKind::TSVoidKeyword(kw) => {
                if self.0.index.ban_void {
                    report_keyword(ctx, &self.0.index, "void", kw.span);
                }
            }
            AstKind::TSNeverKeyword(kw) => {
                if self.0.index.ban_never {
                    report_keyword(ctx, &self.0.index, "never", kw.span);
                }
            }
            AstKind::TSUnknownKeyword(kw) => {
                if self.0.index.ban_unknown {
                    report_keyword(ctx, &self.0.index, "unknown", kw.span);
                }
            }
            AstKind::TSAnyKeyword(kw) => {
                if self.0.index.ban_any {
                    report_keyword(ctx, &self.0.index, "any", kw.span);
                }
            }
            AstKind::TSTupleType(tuple) => {
                if tuple.element_types.is_empty() && self.0.index.ban_empty_tuple {
                    self.check_banned_source(tuple.span, ctx);
                }
            }
            AstKind::TSTypeLiteral(lit) => {
                if lit.members.is_empty() && self.0.index.ban_empty_object {
                    self.check_banned_source(lit.span, ctx);
                }
            }
            AstKind::TSClassImplements(implements) => {
                self.check_type_name(&implements.expression, ctx);
                if implements.type_arguments.is_some() && self.0.index.has_generic_or_complex_keys {
                    self.check_banned_source(implements.span, ctx);
                }
            }
            AstKind::TSInterfaceHeritage(heritage) => {
                self.check_type_name(&heritage.type_name, ctx);
                if heritage.type_arguments.is_some() && self.0.index.has_generic_or_complex_keys {
                    self.check_banned_source(heritage.span, ctx);
                }
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript() && !self.0.types.is_empty()
    }
}

impl NoRestrictedTypes {
    fn check_type_reference(&self, type_ref: &TSTypeReference<'_>, ctx: &LintContext<'_>) {
        self.check_type_name(&type_ref.type_name, ctx);
        if type_ref.type_arguments.is_some() && self.0.index.has_generic_or_complex_keys {
            self.check_banned_source(type_ref.span, ctx);
        }
    }

    fn check_type_name(&self, type_name: &TSTypeName<'_>, ctx: &LintContext<'_>) {
        let index = &self.0.index;
        match type_name_lookup(type_name, index.has_qualified_keys) {
            TypeNameLookup::Name(name) => {
                if let Some((display, config)) = index.lookup(name) {
                    report(ctx, type_name.span(), display, config);
                }
            }
            TypeNameLookup::SourceFallback => {
                self.check_banned_source(type_name.span(), ctx);
            }
            TypeNameLookup::Skip => {}
        }
    }

    fn check_banned_source(&self, span: Span, ctx: &LintContext<'_>) {
        let source_text = ctx.source_range(span);
        if let Some((display, config)) = self.0.index.lookup_source(source_text) {
            report(ctx, span, display, config);
        }
    }
}

enum TypeNameLookup<'a> {
    Name(&'a str),
    SourceFallback,
    Skip,
}

/// Name of a simple type reference. Qualified names are matched by their source text, like
/// typescript-eslint, so a comment inside `NS./* c */Banned` prevents a match.
fn type_name_lookup<'a>(
    type_name: &'a TSTypeName<'a>,
    allow_qualified: bool,
) -> TypeNameLookup<'a> {
    match type_name {
        TSTypeName::IdentifierReference(ident) => TypeNameLookup::Name(ident.name.as_str()),
        TSTypeName::QualifiedName(_) if !allow_qualified => TypeNameLookup::Skip,
        TSTypeName::QualifiedName(_) | TSTypeName::ThisExpression(_) => {
            TypeNameLookup::SourceFallback
        }
    }
}

#[inline]
fn report_keyword(ctx: &LintContext<'_>, index: &RestrictedTypesIndex, keyword: &str, span: Span) {
    if let Some((display, config)) = index.lookup(keyword) {
        report(ctx, span, display, config);
    }
}

fn report(ctx: &LintContext<'_>, span: Span, type_name: &str, config: &BanConfigValue) {
    let diagnostic = no_restricted_types_diagnostic(type_name, config.message(), span);

    let fix_with = config.fix_with().filter(|fix_with| !fix_with.is_empty());
    let suggestions = config.suggest().unwrap_or_default();
    if fix_with.is_none() && suggestions.iter().all(String::is_empty) {
        ctx.diagnostic(diagnostic);
        return;
    }

    let fixer = RuleFixer::new(FixKind::Fix, ctx);
    let suggester = RuleFixer::new(FixKind::Suggestion, ctx);
    ctx.diagnostic_with_suggestions(
        diagnostic,
        fix_with.map(|fix_with| fixer.replace(span, fix_with.to_string())).into_iter().chain(
            suggestions
                .iter()
                .filter(|suggestion| !suggestion.is_empty())
                .map(|suggestion| suggester.replace(span, suggestion.clone())),
        ),
    );
}

fn is_js_whitespace(c: char) -> bool {
    is_white_space(c) || is_line_terminator(c)
}

/// Remove all whitespace from a string for normalization.
/// This matches the TypeScript reference implementation's `removeSpaces` function.
fn remove_spaces(s: &str) -> Cow<'_, str> {
    if s.contains(is_js_whitespace) {
        Cow::Owned(s.chars().filter(|&c| !is_js_whitespace(c)).collect())
    } else {
        Cow::Borrowed(s)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "type T = [/* intentionally empty */];",
            Some(serde_json::json!([{ "types": { "[]": { "fixWith": "unknown[]" } } }])),
        ),
        (
            "type T = {/* intentionally empty */};",
            Some(serde_json::json!([{ "types": { "{}": { "fixWith": "object" } } }])),
        ),
        (
            "type T = NS./* compatibility */Old;",
            Some(serde_json::json!([{ "types": { "NS.Old": { "fixWith": "NS.New" } } }])),
        ),
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
        ("type T = Foo;", Some(serde_json::json!([{ "types": { "Foo": { "fixWith": "" } } }]))),
        ("type T = Foo;", Some(serde_json::json!([{ "types": { "\u{feff}Foo": true } }]))),
        (
            "interface Derived extends Banned<string> {}",
            Some(serde_json::json!([{ "types": { "Banned": { "fixWith": "Ok" } } }])),
        ),
        (
            "class Derived implements Banned<string> {}",
            Some(serde_json::json!([{ "types": { "Banned": { "fixWith": "Ok" } } }])),
        ),
    ];

    let fix = vec![
        (
            "type T = Foo;",
            "type T = Foo;",
            Some(serde_json::json!([{ "types": { "Foo": { "fixWith": "" } } }])),
        ),
        (
            "interface Derived extends Banned<string> {}",
            "interface Derived extends Ok<string> {}",
            Some(serde_json::json!([{ "types": { "Banned": { "fixWith": "Ok" } } }])),
        ),
        (
            "class Derived implements Banned<string> {}",
            "class Derived implements Ok<string> {}",
            Some(serde_json::json!([{ "types": { "Banned": { "fixWith": "Ok" } } }])),
        ),
        (
            "type T = [ ];",
            "type T = unknown[];",
            Some(serde_json::json!([{ "types": { "[]": { "fixWith": "unknown[]" } } }])),
        ),
        (
            "let value: NS . Banned;",
            "let value: NS.Ok;",
            Some(serde_json::json!([{ "types": { "NS.Banned": { "fixWith": "NS.Ok" } } }])),
        ),
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
    let fix_kinds = vec![
        (
            "type T = Foo;",
            "type T = Bar;",
            Some(
                serde_json::json!([{ "types": { "Foo": { "fixWith": "Bar", "suggest": ["Baz"] } } }]),
            ),
            FixKind::Fix,
        ),
        (
            "type T = Foo;",
            "type T = Baz;",
            Some(
                serde_json::json!([{ "types": { "Foo": { "fixWith": "Bar", "suggest": ["Baz"] } } }]),
            ),
            FixKind::Suggestion,
        ),
        (
            "type T = Foo;",
            "type T = Foo;",
            Some(serde_json::json!([{ "types": { "Foo": { "suggest": [""] } } }])),
            FixKind::Suggestion,
        ),
    ];

    let multiple_suggestions = vec![(
        "let value: Banned;",
        ("let value: SafeType;", "let value: AnotherType;"),
        Some(
            serde_json::json!([{ "types": { "Banned": { "suggest": ["SafeType", "AnotherType"] } } }]),
        ),
    )];

    Tester::new(NoRestrictedTypes::NAME, NoRestrictedTypes::PLUGIN, pass, fail)
        .expect_fix(fix)
        .expect_fix(fix_kinds)
        .expect_fix(multiple_suggestions)
        .test_and_snapshot();
}
