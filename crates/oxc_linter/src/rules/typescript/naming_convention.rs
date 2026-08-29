use std::{borrow::Cow, ops::Deref};

use lazy_regex::{Regex, RegexBuilder};
use schemars::{
    JsonSchema, SchemaGenerator,
    schema::{ArrayValidation, InstanceType, Schema, SchemaObject},
};
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{
        AccessorPropertyType, BindingIdentifier, BindingPattern, Class, Expression,
        FormalParameter, FormalParameters, Function, FunctionType, MethodDefinitionKind,
        MethodDefinitionType, ModuleExportName, PropertyDefinitionType, PropertyKey, PropertyKind,
        TSAccessibility, TSEnumMemberName, TSType, VariableDeclarator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{NodeId, Reference, SymbolId};
use oxc_span::{GetSpan, Span};
use oxc_syntax::identifier::is_identifier_name;

use crate::{
    AstNode,
    context::LintContext,
    rule::{Rule, TupleRuleConfig},
};

// ---------------------------------------------------------------------------
// Selectors
// ---------------------------------------------------------------------------

/// Bit values mirror `typescript-eslint`'s `Selectors` enum so that the
/// precedence sort behaves identically.
mod selector {
    // variableLike
    pub const VARIABLE: u32 = 1 << 0;
    pub const FUNCTION: u32 = 1 << 1;
    pub const PARAMETER: u32 = 1 << 2;
    // memberLike
    pub const PARAMETER_PROPERTY: u32 = 1 << 3;
    pub const CLASSIC_ACCESSOR: u32 = 1 << 4;
    pub const ENUM_MEMBER: u32 = 1 << 5;
    pub const CLASS_METHOD: u32 = 1 << 6;
    pub const OBJECT_LITERAL_METHOD: u32 = 1 << 7;
    pub const TYPE_METHOD: u32 = 1 << 8;
    pub const CLASS_PROPERTY: u32 = 1 << 9;
    pub const OBJECT_LITERAL_PROPERTY: u32 = 1 << 10;
    pub const TYPE_PROPERTY: u32 = 1 << 11;
    pub const AUTO_ACCESSOR: u32 = 1 << 12;
    // typeLike
    pub const CLASS: u32 = 1 << 13;
    pub const INTERFACE: u32 = 1 << 14;
    pub const TYPE_ALIAS: u32 = 1 << 15;
    pub const ENUM: u32 = 1 << 16;
    pub const TYPE_PARAMETER: u32 = 1 << 17;
    // other
    pub const IMPORT: u32 = 1 << 18;

    pub const COUNT: usize = 19;

    // meta selectors
    pub const VARIABLE_LIKE: u32 = VARIABLE | FUNCTION | PARAMETER;
    pub const MEMBER_LIKE: u32 = PARAMETER_PROPERTY
        | CLASSIC_ACCESSOR
        | ENUM_MEMBER
        | CLASS_METHOD
        | OBJECT_LITERAL_METHOD
        | TYPE_METHOD
        | CLASS_PROPERTY
        | OBJECT_LITERAL_PROPERTY
        | TYPE_PROPERTY
        | AUTO_ACCESSOR;
    pub const TYPE_LIKE: u32 = CLASS | INTERFACE | TYPE_ALIAS | ENUM | TYPE_PARAMETER;
    pub const METHOD: u32 = CLASS_METHOD | OBJECT_LITERAL_METHOD | TYPE_METHOD;
    pub const PROPERTY: u32 = CLASS_PROPERTY | OBJECT_LITERAL_PROPERTY | TYPE_PROPERTY;
    pub const ACCESSOR: u32 = CLASSIC_ACCESSOR | AUTO_ACCESSOR;

    /// Selectors that support the `types` option upstream.
    pub const ALLOWED_TO_HAVE_TYPES: u32 = VARIABLE
        | PARAMETER
        | CLASS_PROPERTY
        | OBJECT_LITERAL_PROPERTY
        | TYPE_PROPERTY
        | PARAMETER_PROPERTY
        | CLASSIC_ACCESSOR;

    /// Human readable selector names, indexed by bit position.
    pub const MESSAGE_NAMES: [&str; COUNT] = [
        "Variable",
        "Function",
        "Parameter",
        "Parameter Property",
        "Classic Accessor",
        "Enum Member",
        "Class Method",
        "Object Literal Method",
        "Type Method",
        "Class Property",
        "Object Literal Property",
        "Type Property",
        "Auto Accessor",
        "Class",
        "Interface",
        "Type Alias",
        "Enum",
        "Type Parameter",
        "Import",
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
enum SelectorName {
    // meta selectors
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "variableLike")]
    VariableLike,
    #[serde(rename = "memberLike")]
    MemberLike,
    #[serde(rename = "typeLike")]
    TypeLike,
    #[serde(rename = "method")]
    Method,
    #[serde(rename = "property")]
    Property,
    #[serde(rename = "accessor")]
    Accessor,
    // individual selectors
    #[serde(rename = "variable")]
    Variable,
    #[serde(rename = "function")]
    Function,
    #[serde(rename = "parameter")]
    Parameter,
    #[serde(rename = "parameterProperty")]
    ParameterProperty,
    #[serde(rename = "classicAccessor")]
    ClassicAccessor,
    #[serde(rename = "enumMember")]
    EnumMember,
    #[serde(rename = "classMethod")]
    ClassMethod,
    #[serde(rename = "objectLiteralMethod")]
    ObjectLiteralMethod,
    #[serde(rename = "typeMethod")]
    TypeMethod,
    #[serde(rename = "classProperty")]
    ClassProperty,
    #[serde(rename = "objectLiteralProperty")]
    ObjectLiteralProperty,
    #[serde(rename = "typeProperty")]
    TypeProperty,
    #[serde(rename = "autoAccessor")]
    AutoAccessor,
    #[serde(rename = "class")]
    Class,
    #[serde(rename = "interface")]
    Interface,
    #[serde(rename = "typeAlias")]
    TypeAlias,
    #[serde(rename = "enum")]
    Enum,
    #[serde(rename = "typeParameter")]
    TypeParameter,
    #[serde(rename = "import")]
    Import,
}

impl SelectorName {
    fn is_meta(self) -> bool {
        matches!(
            self,
            Self::Default
                | Self::VariableLike
                | Self::MemberLike
                | Self::TypeLike
                | Self::Method
                | Self::Property
                | Self::Accessor
        )
    }

    /// Numeric value used for both bit matching and precedence ordering.
    /// `default` is `-1` (all bits set) exactly like upstream.
    fn value(self) -> i64 {
        let bits: u32 = match self {
            Self::Default => return -1,
            Self::VariableLike => selector::VARIABLE_LIKE,
            Self::MemberLike => selector::MEMBER_LIKE,
            Self::TypeLike => selector::TYPE_LIKE,
            Self::Method => selector::METHOD,
            Self::Property => selector::PROPERTY,
            Self::Accessor => selector::ACCESSOR,
            Self::Variable => selector::VARIABLE,
            Self::Function => selector::FUNCTION,
            Self::Parameter => selector::PARAMETER,
            Self::ParameterProperty => selector::PARAMETER_PROPERTY,
            Self::ClassicAccessor => selector::CLASSIC_ACCESSOR,
            Self::EnumMember => selector::ENUM_MEMBER,
            Self::ClassMethod => selector::CLASS_METHOD,
            Self::ObjectLiteralMethod => selector::OBJECT_LITERAL_METHOD,
            Self::TypeMethod => selector::TYPE_METHOD,
            Self::ClassProperty => selector::CLASS_PROPERTY,
            Self::ObjectLiteralProperty => selector::OBJECT_LITERAL_PROPERTY,
            Self::TypeProperty => selector::TYPE_PROPERTY,
            Self::AutoAccessor => selector::AUTO_ACCESSOR,
            Self::Class => selector::CLASS,
            Self::Interface => selector::INTERFACE,
            Self::TypeAlias => selector::TYPE_ALIAS,
            Self::Enum => selector::ENUM,
            Self::TypeParameter => selector::TYPE_PARAMETER,
            Self::Import => selector::IMPORT,
        };
        i64::from(bits)
    }
}

// ---------------------------------------------------------------------------
// Modifiers
// ---------------------------------------------------------------------------

/// Bit values mirror `typescript-eslint`'s `Modifiers` / `TypeModifiers` enums
/// so that the modifier weight used for precedence is identical.
mod modifier {
    pub const CONST: u32 = 1 << 0;
    pub const READONLY: u32 = 1 << 1;
    pub const STATIC: u32 = 1 << 2;
    pub const PUBLIC: u32 = 1 << 3;
    pub const PROTECTED: u32 = 1 << 4;
    pub const PRIVATE: u32 = 1 << 5;
    pub const HASH_PRIVATE: u32 = 1 << 6;
    pub const ABSTRACT: u32 = 1 << 7;
    pub const DESTRUCTURED: u32 = 1 << 8;
    pub const GLOBAL: u32 = 1 << 9;
    pub const EXPORTED: u32 = 1 << 10;
    pub const UNUSED: u32 = 1 << 11;
    pub const REQUIRES_QUOTES: u32 = 1 << 12;
    pub const OVERRIDE: u32 = 1 << 13;
    pub const ASYNC: u32 = 1 << 14;
    pub const DEFAULT: u32 = 1 << 15;
    pub const NAMESPACE: u32 = 1 << 16;
    // type modifiers
    pub const TYPE_BOOLEAN: u32 = 1 << 17;
    pub const TYPE_STRING: u32 = 1 << 18;
    pub const TYPE_NUMBER: u32 = 1 << 19;
    pub const TYPE_FUNCTION: u32 = 1 << 20;
    pub const TYPE_ARRAY: u32 = 1 << 21;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
enum ModifierName {
    #[serde(rename = "const")]
    Const,
    #[serde(rename = "readonly")]
    Readonly,
    #[serde(rename = "static")]
    Static,
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "protected")]
    Protected,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "#private")]
    HashPrivate,
    #[serde(rename = "abstract")]
    Abstract,
    #[serde(rename = "destructured")]
    Destructured,
    #[serde(rename = "global")]
    Global,
    #[serde(rename = "exported")]
    Exported,
    #[serde(rename = "unused")]
    Unused,
    #[serde(rename = "requiresQuotes")]
    RequiresQuotes,
    #[serde(rename = "override")]
    Override,
    #[serde(rename = "async")]
    Async,
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "namespace")]
    Namespace,
}

impl ModifierName {
    fn bit(self) -> u32 {
        match self {
            Self::Const => modifier::CONST,
            Self::Readonly => modifier::READONLY,
            Self::Static => modifier::STATIC,
            Self::Public => modifier::PUBLIC,
            Self::Protected => modifier::PROTECTED,
            Self::Private => modifier::PRIVATE,
            Self::HashPrivate => modifier::HASH_PRIVATE,
            Self::Abstract => modifier::ABSTRACT,
            Self::Destructured => modifier::DESTRUCTURED,
            Self::Global => modifier::GLOBAL,
            Self::Exported => modifier::EXPORTED,
            Self::Unused => modifier::UNUSED,
            Self::RequiresQuotes => modifier::REQUIRES_QUOTES,
            Self::Override => modifier::OVERRIDE,
            Self::Async => modifier::ASYNC,
            Self::Default => modifier::DEFAULT,
            Self::Namespace => modifier::NAMESPACE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum TypeModifierName {
    Boolean,
    String,
    Number,
    Function,
    Array,
}

impl TypeModifierName {
    fn bit(self) -> u32 {
        match self {
            Self::Boolean => modifier::TYPE_BOOLEAN,
            Self::String => modifier::TYPE_STRING,
            Self::Number => modifier::TYPE_NUMBER,
            Self::Function => modifier::TYPE_FUNCTION,
            Self::Array => modifier::TYPE_ARRAY,
        }
    }
}

// ---------------------------------------------------------------------------
// Formats
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
enum PredefinedFormat {
    #[serde(rename = "camelCase")]
    Camel,
    #[serde(rename = "strictCamelCase")]
    StrictCamel,
    #[serde(rename = "PascalCase")]
    Pascal,
    #[serde(rename = "StrictPascalCase")]
    StrictPascal,
    #[serde(rename = "snake_case")]
    Snake,
    #[serde(rename = "UPPER_CASE")]
    Upper,
}

impl PredefinedFormat {
    fn name(self) -> &'static str {
        match self {
            Self::Camel => "camelCase",
            Self::StrictCamel => "strictCamelCase",
            Self::Pascal => "PascalCase",
            Self::StrictPascal => "StrictPascalCase",
            Self::Snake => "snake_case",
            Self::Upper => "UPPER_CASE",
        }
    }

    fn check(self, name: &str) -> bool {
        match self {
            Self::Camel => is_camel_case(name),
            Self::StrictCamel => is_strict_camel_case(name),
            Self::Pascal => is_pascal_case(name),
            Self::StrictPascal => is_strict_pascal_case(name),
            Self::Snake => is_snake_case(name),
            Self::Upper => is_upper_case(name),
        }
    }
}

/*
These format functions mirror `tslint-consistent-codestyle/naming-convention` via
`typescript-eslint`. They intentionally avoid regexes so that non-ASCII
identifiers behave the same way as they do in JavaScript's `toUpperCase` /
`toLowerCase`.
*/

/// JS: `c === c.toUpperCase()`
fn is_upper_form(c: char) -> bool {
    let mut upper = c.to_uppercase();
    upper.next() == Some(c) && upper.next().is_none()
}

/// JS: `c === c.toLowerCase()`
fn is_lower_form(c: char) -> bool {
    let mut lower = c.to_lowercase();
    lower.next() == Some(c) && lower.next().is_none()
}

/// JS: `c === c.toUpperCase() && c !== c.toLowerCase()`
fn is_uppercase_char(c: char) -> bool {
    is_upper_form(c) && !is_lower_form(c)
}

fn is_pascal_case(name: &str) -> bool {
    let Some(first) = name.chars().next() else { return true };
    is_upper_form(first) && !name.contains('_')
}

fn is_strict_pascal_case(name: &str) -> bool {
    let Some(first) = name.chars().next() else { return true };
    is_upper_form(first) && has_strict_camel_humps(name, true)
}

fn is_camel_case(name: &str) -> bool {
    let Some(first) = name.chars().next() else { return true };
    is_lower_form(first) && !name.contains('_')
}

fn is_strict_camel_case(name: &str) -> bool {
    let Some(first) = name.chars().next() else { return true };
    is_lower_form(first) && has_strict_camel_humps(name, false)
}

fn has_strict_camel_humps(name: &str, mut is_upper: bool) -> bool {
    if name.starts_with('_') {
        return false;
    }
    for c in name.chars().skip(1) {
        if c == '_' {
            return false;
        }
        if is_upper == is_uppercase_char(c) {
            if is_upper {
                return false;
            }
        } else {
            is_upper = !is_upper;
        }
    }
    true
}

fn is_snake_case(name: &str) -> bool {
    name.is_empty() || (name.chars().all(is_lower_form) && validate_underscores(name))
}

fn is_upper_case(name: &str) -> bool {
    name.is_empty() || (name.chars().all(is_upper_form) && validate_underscores(name))
}

/// Check for leading, trailing and adjacent underscores.
fn validate_underscores(name: &str) -> bool {
    if name.starts_with('_') {
        return false;
    }
    let mut was_underscore = false;
    for c in name.chars().skip(1) {
        if c == '_' {
            if was_underscore {
                return false;
            }
            was_underscore = true;
        } else {
            was_underscore = false;
        }
    }
    !was_underscore
}

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum UnderscoreOption {
    Forbid,
    Allow,
    Require,
    RequireDouble,
    AllowDouble,
    AllowSingleOrDouble,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
enum SelectorOption {
    /// A single selector.
    One(SelectorName),
    /// Multiple selectors that share the same options.
    Many(Vec<SelectorName>),
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MatchRegexOption {
    /// The regular expression to test the name against.
    regex: String,
    /// Whether the name must match (`true`) or must not match (`false`) the regex.
    r#match: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
enum FilterOption {
    /// A regular expression that the name must match for this option to apply.
    Regex(String),
    /// A regular expression and whether the name must or must not match it.
    Match(MatchRegexOption),
}

/// A single naming convention selector option.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NamingConventionOption {
    /// The selector(s) this option applies to, e.g. `"variable"`,
    /// `"classProperty"`, or a meta selector such as `"default"`,
    /// `"variableLike"`, `"memberLike"`, `"typeLike"`, `"method"`,
    /// `"property"` or `"accessor"`.
    selector: SelectorOption,
    /// One or more predefined formats the name must match:
    /// `camelCase`, `strictCamelCase`, `PascalCase`, `StrictPascalCase`,
    /// `snake_case`, `UPPER_CASE`. Use `null` to skip format checking.
    #[serde(default)]
    format: Option<Vec<PredefinedFormat>>,
    /// A custom regular expression the name must match (or must not match).
    #[serde(default)]
    custom: Option<MatchRegexOption>,
    /// Only apply this option to names matching (or not matching) the regex.
    /// Options with a filter take the highest precedence.
    #[serde(default)]
    filter: Option<FilterOption>,
    /// How leading underscores are treated: `forbid`, `allow`, `require`,
    /// `requireDouble`, `allowDouble`, `allowSingleOrDouble`.
    #[serde(default)]
    leading_underscore: Option<UnderscoreOption>,
    /// How trailing underscores are treated: `forbid`, `allow`, `require`,
    /// `requireDouble`, `allowDouble`, `allowSingleOrDouble`.
    #[serde(default)]
    trailing_underscore: Option<UnderscoreOption>,
    /// The name must start with one of these prefixes. The prefix is
    /// removed before the format is checked.
    #[serde(default)]
    prefix: Option<Vec<String>>,
    /// The name must end with one of these suffixes. The suffix is removed
    /// before the format is checked.
    #[serde(default)]
    suffix: Option<Vec<String>>,
    /// Only apply this option to names that have all of these modifiers,
    /// e.g. `const`, `readonly`, `static`, `public`, `protected`, `private`,
    /// `#private`, `abstract`, `destructured`, `global`, `exported`,
    /// `unused`, `requiresQuotes`, `override`, `async`, `default`,
    /// `namespace`.
    #[serde(default)]
    modifiers: Option<Vec<ModifierName>>,
    /// Only apply this option to names whose type is one of `boolean`,
    /// `string`, `number`, `function`, `array`. This requires type
    /// information which oxlint does not have, so options that use `types`
    /// are skipped for selectors that support them.
    #[serde(default)]
    types: Option<Vec<TypeModifierName>>,
}

/// The full rule configuration: an array of selector options.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(transparent)]
pub struct NamingConventionConfig(Vec<NamingConventionOption>);

impl JsonSchema for NamingConventionConfig {
    fn schema_name() -> String {
        "NamingConventionConfig".to_string()
    }

    /// Selector options are passed variadically, as
    /// `["warn", { ... }, { ... }]`, rather than as a single nested array.
    /// Describing them with `additionalItems` makes the generated schema (and
    /// the TypeScript config types derived from it) spread the options across
    /// the tail of the rule tuple, matching what `from_configuration` accepts.
    fn json_schema(r#gen: &mut SchemaGenerator) -> Schema {
        Schema::Object(SchemaObject {
            instance_type: Some(InstanceType::Array.into()),
            array: Some(Box::new(ArrayValidation {
                additional_items: Some(Box::new(r#gen.subschema_for::<NamingConventionOption>())),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}

#[derive(Debug, Clone)]
struct CompiledRegex {
    regex: Regex,
    r#match: bool,
}

impl CompiledRegex {
    fn new(pattern: &str, r#match: bool) -> Result<Self, lazy_regex::regex::Error> {
        Ok(Self { regex: RegexBuilder::new(pattern).build()?, r#match })
    }
}

#[derive(Debug, Clone)]
struct NormalizedOption {
    selector: SelectorName,
    selector_value: i64,
    modifier_weight: u32,
    custom: Option<CompiledRegex>,
    filter: Option<CompiledRegex>,
    format: Option<Vec<PredefinedFormat>>,
    leading_underscore: Option<UnderscoreOption>,
    trailing_underscore: Option<UnderscoreOption>,
    modifiers: u32,
    prefix: Option<Vec<String>>,
    suffix: Option<Vec<String>>,
    has_types: bool,
}

#[derive(Debug, Clone)]
pub struct NamingConventionInner {
    options: Vec<NormalizedOption>,
    /// For each concrete selector (indexed by bit position), the indices of
    /// the applicable options ordered from highest to lowest precedence.
    validators: Vec<Vec<usize>>,
}

#[derive(Debug, Clone)]
pub struct NamingConvention(Box<NamingConventionInner>);

impl Deref for NamingConvention {
    type Target = NamingConventionInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for NamingConvention {
    fn default() -> Self {
        Self::from_options(default_options()).expect("default options are valid")
    }
}

/// This essentially mirrors ESLint's `camelcase` rule.
fn default_options() -> Vec<NamingConventionOption> {
    let option = |selector: SelectorName,
                  format: &[PredefinedFormat],
                  underscore: Option<UnderscoreOption>| NamingConventionOption {
        selector: SelectorOption::One(selector),
        format: Some(format.to_vec()),
        custom: None,
        filter: None,
        leading_underscore: underscore,
        trailing_underscore: underscore,
        prefix: None,
        suffix: None,
        modifiers: None,
        types: None,
    };
    vec![
        option(SelectorName::Default, &[PredefinedFormat::Camel], Some(UnderscoreOption::Allow)),
        option(SelectorName::Import, &[PredefinedFormat::Camel, PredefinedFormat::Pascal], None),
        option(
            SelectorName::Variable,
            &[PredefinedFormat::Camel, PredefinedFormat::Upper],
            Some(UnderscoreOption::Allow),
        ),
        option(SelectorName::TypeLike, &[PredefinedFormat::Pascal], None),
    ]
}

impl NamingConvention {
    fn from_options(raw: Vec<NamingConventionOption>) -> Result<Self, lazy_regex::regex::Error> {
        let mut options = Vec::with_capacity(raw.len());
        for option in raw {
            let mut weight = 0u32;
            let mut modifiers = 0u32;
            if let Some(mods) = &option.modifiers {
                for m in mods {
                    weight |= m.bit();
                    modifiers |= m.bit();
                }
            }
            if let Some(types) = &option.types {
                for t in types {
                    weight |= t.bit();
                }
            }
            // give selectors with a filter the _highest_ priority
            if option.filter.is_some() {
                weight |= 1 << 30;
            }

            let custom = option
                .custom
                .as_ref()
                .map(|c| CompiledRegex::new(&c.regex, c.r#match))
                .transpose()?;
            let filter = option
                .filter
                .as_ref()
                .map(|f| match f {
                    FilterOption::Regex(regex) => CompiledRegex::new(regex, true),
                    FilterOption::Match(m) => CompiledRegex::new(&m.regex, m.r#match),
                })
                .transpose()?;
            let prefix = option.prefix.filter(|p| !p.is_empty());
            let suffix = option.suffix.filter(|s| !s.is_empty());

            let selectors = match option.selector {
                SelectorOption::One(s) => vec![s],
                SelectorOption::Many(s) => s,
            };
            for selector in selectors {
                options.push(NormalizedOption {
                    selector,
                    selector_value: selector.value(),
                    modifier_weight: weight,
                    custom: custom.clone(),
                    filter: filter.clone(),
                    format: option.format.clone(),
                    leading_underscore: option.leading_underscore,
                    trailing_underscore: option.trailing_underscore,
                    modifiers,
                    prefix: prefix.clone(),
                    suffix: suffix.clone(),
                    has_types: option.types.is_some(),
                });
            }
        }

        let validators = (0..selector::COUNT)
            .map(|index| Self::create_validator(&options, 1u32 << index))
            .collect();

        Ok(Self(Box::new(NamingConventionInner { options, validators })))
    }

    /// Gather the options applicable to `selector_type` and sort them so the
    /// "highest priority" options are checked first.
    fn create_validator(options: &[NormalizedOption], selector_type: u32) -> Vec<usize> {
        let mut configs: Vec<usize> = options
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                (c.selector_value & i64::from(selector_type)) != 0
                    || c.selector == SelectorName::Default
            })
            .map(|(i, _)| i)
            .collect();

        configs.sort_by(|&a, &b| {
            let a = &options[a];
            let b = &options[b];
            if a.selector_value == b.selector_value {
                // in the event of the same selector, order by modifier weight
                // sort descending - the type modifiers are "more important"
                return b.modifier_weight.cmp(&a.modifier_weight);
            }

            let a_is_meta = a.selector.is_meta();
            let b_is_meta = b.selector.is_meta();

            // non-meta selectors should go ahead of meta selectors
            if a_is_meta && !b_is_meta {
                return std::cmp::Ordering::Greater;
            }
            if !a_is_meta && b_is_meta {
                return std::cmp::Ordering::Less;
            }

            let a_is_method_or_property =
                matches!(a.selector, SelectorName::Method | SelectorName::Property);
            let b_is_method_or_property =
                matches!(b.selector, SelectorName::Method | SelectorName::Property);

            // for backward compatibility, method and property have higher
            // precedence than other meta selectors
            if a_is_method_or_property && !b_is_method_or_property {
                return std::cmp::Ordering::Less;
            }
            if !a_is_method_or_property && b_is_method_or_property {
                return std::cmp::Ordering::Greater;
            }

            // both aren't meta selectors
            // sort descending - the meta selectors are "least important"
            b.selector_value.cmp(&a.selector_value)
        });

        configs
    }
}

// ---------------------------------------------------------------------------
// Diagnostics
// ---------------------------------------------------------------------------

fn does_not_match_format_diagnostic(
    span: Span,
    kind: &str,
    name: &str,
    processed: &str,
    formats: &[PredefinedFormat],
) -> OxcDiagnostic {
    let formats = formats.iter().map(|f| f.name()).collect::<Vec<_>>().join(", ");
    let message = if name == processed {
        format!("{kind} name `{name}` must match one of the following formats: {formats}")
    } else {
        format!(
            "{kind} name `{name}` trimmed as `{processed}` must match one of the following formats: {formats}"
        )
    };
    OxcDiagnostic::warn(message)
        .with_help("Rename it to follow the configured naming convention.")
        .with_label(span)
}

fn missing_affix_diagnostic(
    span: Span,
    kind: &str,
    name: &str,
    position: &str,
    affixes: &[String],
) -> OxcDiagnostic {
    let affixes = affixes.join(", ");
    OxcDiagnostic::warn(format!(
        "{kind} name `{name}` must have one of the following {position}es: {affixes}"
    ))
    .with_help("Rename it to follow the configured naming convention.")
    .with_label(span)
}

fn missing_underscore_diagnostic(
    span: Span,
    kind: &str,
    name: &str,
    count: &str,
    position: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("{kind} name `{name}` must have {count} {position} underscore(s)."))
        .with_help("Rename it to follow the configured naming convention.")
        .with_label(span)
}

fn unexpected_underscore_diagnostic(
    span: Span,
    kind: &str,
    name: &str,
    position: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("{kind} name `{name}` must not have a {position} underscore."))
        .with_help("Rename it to follow the configured naming convention.")
        .with_label(span)
}

fn satisfy_custom_diagnostic(
    span: Span,
    kind: &str,
    name: &str,
    custom: &CompiledRegex,
) -> OxcDiagnostic {
    let regex_match = if custom.r#match { "match" } else { "not match" };
    OxcDiagnostic::warn(format!(
        "{kind} name `{name}` must {regex_match} the RegExp: /{}/u",
        custom.regex.as_str()
    ))
    .with_help("Rename it to follow the configured naming convention.")
    .with_label(span)
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces naming conventions for everything across a codebase.
    ///
    /// ### Why is this bad?
    ///
    /// Enforcing naming conventions helps keep the codebase consistent and
    /// reduces overhead when thinking about how to name a variable.
    /// Additionally, a well-designed style guide can help communicate intent,
    /// such as by enforcing all private properties begin with an `_`, and all
    /// global-level constants are written in `UPPER_CASE`.
    ///
    /// This rule allows you to enforce conventions for any identifier, using
    /// granular selectors to create a fine-grained style guide.
    ///
    /// ### Options
    ///
    /// The rule accepts an array of objects. Each object describes a selector
    /// (what to match) and a set of rules for names matched by that selector.
    /// Selector options are:
    ///
    /// - `selector` (required): one of `default`, `variableLike`,
    ///   `memberLike`, `typeLike`, `method`, `property`, `accessor`,
    ///   `variable`, `function`, `parameter`, `parameterProperty`,
    ///   `classicAccessor`, `autoAccessor`, `enumMember`, `classMethod`,
    ///   `objectLiteralMethod`, `typeMethod`, `classProperty`,
    ///   `objectLiteralProperty`, `typeProperty`, `class`, `interface`,
    ///   `typeAlias`, `enum`, `typeParameter`, `import`, or an array of these.
    /// - `modifiers`: only match names that have all of the given modifiers,
    ///   e.g. `const`, `readonly`, `static`, `public`, `protected`,
    ///   `private`, `#private`, `abstract`, `destructured`, `global`,
    ///   `exported`, `unused`, `requiresQuotes`, `override`, `async`,
    ///   `default`, `namespace`.
    /// - `filter`: a regex (or `{ regex, match }` object) that names must
    ///   match for the option to apply.
    /// - `format`: one or more of `camelCase`, `strictCamelCase`,
    ///   `PascalCase`, `StrictPascalCase`, `snake_case`, `UPPER_CASE`, or
    ///   `null` to skip format checking.
    /// - `custom`: `{ regex, match }` the (trimmed) name must satisfy.
    /// - `leadingUnderscore` / `trailingUnderscore`: `forbid`, `allow`,
    ///   `require`, `requireDouble`, `allowDouble`, `allowSingleOrDouble`.
    /// - `prefix` / `suffix`: arrays of affixes the name must start/end with.
    ///
    /// Options are matched from most specific to least specific: individual
    /// selectors beat meta selectors, and within a selector, options with
    /// more modifiers (or with a `filter`) win. The first matching option
    /// decides whether a name is valid.
    ///
    /// The `types` option (which requires TypeScript type information) is not
    /// supported. Options that specify `types` are ignored for the selectors
    /// that could use them (`variable`, `parameter`, `classProperty`,
    /// `objectLiteralProperty`, `typeProperty`, `parameterProperty`,
    /// `classicAccessor`).
    ///
    /// Regular expressions in `filter` and `custom` are compiled with Rust
    /// regex syntax, so JavaScript-only features such as lookaround are not
    /// supported.
    ///
    /// ### Examples
    ///
    /// With the default configuration (camelCase for everything, with
    /// `UPPER_CASE` also allowed for variables and `PascalCase` for types):
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const my_variable = 1;
    /// function My_Function() {}
    /// interface my_interface {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const myVariable = 1;
    /// const MY_CONSTANT = 2;
    /// function myFunction() {}
    /// interface MyInterface {}
    /// ```
    ///
    /// With `[{ "selector": "interface", "format": ["PascalCase"], "custom": { "regex": "^I[A-Z]", "match": false } }]`:
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// interface IFoo {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// interface Foo {}
    /// ```
    ///
    /// With `[{ "selector": "memberLike", "modifiers": ["private"], "format": ["camelCase"], "leadingUnderscore": "require" }]`:
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// class Foo {
    ///   private value = 1;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// class Foo {
    ///   private _value = 1;
    /// }
    /// ```
    NamingConvention,
    typescript,
    style,
    none,
    config = NamingConventionConfig,
    version = "next",
    short_description = "Enforces naming conventions for everything across a codebase.",
);

impl Rule for NamingConvention {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let config = serde_json::from_value::<TupleRuleConfig<NamingConventionConfig>>(value)?
            .into_inner()
            .0;
        // only apply the defaults when the user provides no config
        let options = if config.is_empty() { default_options() } else { config };
        Self::from_options(options).map_err(serde::de::Error::custom)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(func) => {
                self.check_function(func, node, ctx);
                self.check_params(&func.params, ctx);
            }
            AstKind::ArrowFunctionExpression(arrow) => self.check_params(&arrow.params, ctx),
            AstKind::ImportSpecifier(specifier) => {
                // Handle `import { default as Foo }`
                if let ModuleExportName::IdentifierName(imported) = &specifier.imported
                    && imported.name != "default"
                {
                    return;
                }
                self.validate(selector::IMPORT, &specifier.local, modifier::DEFAULT, ctx);
            }
            AstKind::ImportDefaultSpecifier(specifier) => {
                self.validate(selector::IMPORT, &specifier.local, modifier::DEFAULT, ctx);
            }
            AstKind::ImportNamespaceSpecifier(specifier) => {
                self.validate(selector::IMPORT, &specifier.local, modifier::NAMESPACE, ctx);
            }
            AstKind::VariableDeclarator(declarator) => {
                self.check_variable_declarator(declarator, node, ctx);
            }
            AstKind::PropertyDefinition(prop) => {
                if prop.computed {
                    return;
                }
                let mut modifiers = member_modifiers(
                    &prop.key,
                    prop.accessibility,
                    prop.r#static,
                    prop.readonly,
                    prop.r#override,
                    prop.r#type == PropertyDefinitionType::TSAbstractPropertyDefinition,
                );
                match &prop.value {
                    Some(Expression::ArrowFunctionExpression(arrow)) => {
                        if arrow.r#async {
                            modifiers |= modifier::ASYNC;
                        }
                        self.validate_key(selector::CLASS_METHOD, &prop.key, modifiers, ctx);
                    }
                    Some(Expression::FunctionExpression(func)) => {
                        if func.r#async {
                            modifiers |= modifier::ASYNC;
                        }
                        self.validate_key(selector::CLASS_METHOD, &prop.key, modifiers, ctx);
                    }
                    _ => self.validate_key(selector::CLASS_PROPERTY, &prop.key, modifiers, ctx),
                }
            }
            AstKind::MethodDefinition(method) => {
                if method.computed {
                    return;
                }
                let mut modifiers = member_modifiers(
                    &method.key,
                    method.accessibility,
                    method.r#static,
                    false,
                    method.r#override,
                    method.r#type == MethodDefinitionType::TSAbstractMethodDefinition,
                );
                match method.kind {
                    MethodDefinitionKind::Method => {
                        if method.value.r#async {
                            modifiers |= modifier::ASYNC;
                        }
                        self.validate_key(selector::CLASS_METHOD, &method.key, modifiers, ctx);
                    }
                    MethodDefinitionKind::Get | MethodDefinitionKind::Set => {
                        self.validate_key(selector::CLASSIC_ACCESSOR, &method.key, modifiers, ctx);
                    }
                    MethodDefinitionKind::Constructor => {}
                }
            }
            AstKind::AccessorProperty(accessor) => {
                if accessor.computed {
                    return;
                }
                let modifiers = member_modifiers(
                    &accessor.key,
                    accessor.accessibility,
                    accessor.r#static,
                    false,
                    accessor.r#override,
                    accessor.r#type == AccessorPropertyType::TSAbstractAccessorProperty,
                );
                self.validate_key(selector::AUTO_ACCESSOR, &accessor.key, modifiers, ctx);
            }
            AstKind::ObjectProperty(prop) => {
                if prop.computed {
                    return;
                }
                let mut modifiers = modifier::PUBLIC;
                match prop.kind {
                    PropertyKind::Init => match &prop.value {
                        Expression::ArrowFunctionExpression(arrow) => {
                            if arrow.r#async {
                                modifiers |= modifier::ASYNC;
                            }
                            self.validate_key(
                                selector::OBJECT_LITERAL_METHOD,
                                &prop.key,
                                modifiers,
                                ctx,
                            );
                        }
                        Expression::FunctionExpression(func) => {
                            if func.r#async {
                                modifiers |= modifier::ASYNC;
                            }
                            self.validate_key(
                                selector::OBJECT_LITERAL_METHOD,
                                &prop.key,
                                modifiers,
                                ctx,
                            );
                        }
                        _ => self.validate_key(
                            selector::OBJECT_LITERAL_PROPERTY,
                            &prop.key,
                            modifiers,
                            ctx,
                        ),
                    },
                    PropertyKind::Get | PropertyKind::Set => {
                        self.validate_key(selector::CLASSIC_ACCESSOR, &prop.key, modifiers, ctx);
                    }
                }
            }
            AstKind::TSMethodSignature(sig) => {
                if sig.computed {
                    return;
                }
                self.validate_key(selector::TYPE_METHOD, &sig.key, modifier::PUBLIC, ctx);
            }
            AstKind::TSPropertySignature(sig) => {
                if sig.computed {
                    return;
                }
                let is_function_type = sig
                    .type_annotation
                    .as_ref()
                    .is_some_and(|t| matches!(t.type_annotation, TSType::TSFunctionType(_)));
                if is_function_type {
                    self.validate_key(selector::TYPE_METHOD, &sig.key, modifier::PUBLIC, ctx);
                } else {
                    let mut modifiers = modifier::PUBLIC;
                    if sig.readonly {
                        modifiers |= modifier::READONLY;
                    }
                    self.validate_key(selector::TYPE_PROPERTY, &sig.key, modifiers, ctx);
                }
            }
            AstKind::Class(class) => self.check_class(class, node, ctx),
            AstKind::TSEnumDeclaration(decl) => {
                let modifiers = declaration_modifiers(&decl.id, node.id(), ctx);
                self.validate(selector::ENUM, &decl.id, modifiers, ctx);
            }
            AstKind::TSEnumMember(member) => {
                let (name, span): (Cow<'_, str>, Span) = match &member.id {
                    TSEnumMemberName::Identifier(ident) => {
                        (Cow::Borrowed(ident.name.as_str()), ident.span)
                    }
                    TSEnumMemberName::String(lit) | TSEnumMemberName::ComputedString(lit) => {
                        (Cow::Borrowed(lit.value.as_str()), lit.span)
                    }
                    TSEnumMemberName::ComputedTemplateString(_) => return,
                };
                let modifiers = quoting_modifier(&name);
                self.validate_name(selector::ENUM_MEMBER, &name, span, modifiers, ctx);
            }
            AstKind::TSInterfaceDeclaration(decl) => {
                let modifiers = declaration_modifiers(&decl.id, node.id(), ctx);
                self.validate(selector::INTERFACE, &decl.id, modifiers, ctx);
            }
            AstKind::TSTypeAliasDeclaration(decl) => {
                let modifiers = declaration_modifiers(&decl.id, node.id(), ctx);
                self.validate(selector::TYPE_ALIAS, &decl.id, modifiers, ctx);
            }
            AstKind::TSTypeParameter(param) => {
                if !matches!(
                    ctx.nodes().parent_kind(node.id()),
                    AstKind::TSTypeParameterDeclaration(_)
                ) {
                    return;
                }
                let modifiers = unused_modifier(&param.name, ctx);
                self.validate(selector::TYPE_PARAMETER, &param.name, modifiers, ctx);
            }
            _ => {}
        }
    }
}

impl NamingConvention {
    fn check_function<'a>(&self, func: &Function<'a>, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !matches!(
            func.r#type,
            FunctionType::FunctionDeclaration
                | FunctionType::TSDeclareFunction
                | FunctionType::FunctionExpression
        ) {
            return;
        }
        let Some(id) = &func.id else { return };
        let mut modifiers = declaration_modifiers(id, node.id(), ctx);
        if is_global_symbol(id, ctx) {
            modifiers |= modifier::GLOBAL;
        }
        if func.r#async {
            modifiers |= modifier::ASYNC;
        }
        self.validate(selector::FUNCTION, id, modifiers, ctx);
    }

    fn check_class<'a>(&self, class: &Class<'a>, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let Some(id) = &class.id else { return };
        let mut modifiers = declaration_modifiers(id, node.id(), ctx);
        if class.r#abstract {
            modifiers |= modifier::ABSTRACT;
        }
        self.validate(selector::CLASS, id, modifiers, ctx);
    }

    fn check_variable_declarator<'a>(
        &self,
        declarator: &VariableDeclarator<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        let mut base_modifiers = 0;
        let parent_id = ctx.nodes().parent_id(node.id());
        if let AstKind::VariableDeclaration(decl) = ctx.nodes().parent_kind(node.id())
            && decl.kind.is_const()
        {
            base_modifiers |= modifier::CONST;
        }
        if node.scope_id() == ctx.scoping().root_scope_id() {
            base_modifiers |= modifier::GLOBAL;
        }

        let is_async_init = matches!(
            &declarator.init,
            Some(Expression::ArrowFunctionExpression(arrow)) if arrow.r#async
        ) || matches!(
            &declarator.init,
            Some(Expression::FunctionExpression(func)) if func.r#async
        );

        for_each_binding_identifier(&declarator.id, false, &mut |id, destructured| {
            let mut modifiers = base_modifiers;
            if destructured {
                modifiers |= modifier::DESTRUCTURED;
            }
            // `parent_id` is the `VariableDeclaration`, whose parent may be an export.
            modifiers |= declaration_modifiers(id, parent_id, ctx);
            if is_async_init
                && !destructured
                && matches!(declarator.id, BindingPattern::BindingIdentifier(_))
            {
                modifiers |= modifier::ASYNC;
            }
            self.validate(selector::VARIABLE, id, modifiers, ctx);
        });
    }

    fn check_params<'a>(&self, params: &FormalParameters<'a>, ctx: &LintContext<'a>) {
        for param in &params.items {
            self.check_param(param, ctx);
        }
        if let Some(rest) = &params.rest {
            for_each_binding_identifier(&rest.rest.argument, false, &mut |id, destructured| {
                self.validate_param(id, destructured, ctx);
            });
        }
    }

    fn check_param<'a>(&self, param: &FormalParameter<'a>, ctx: &LintContext<'a>) {
        if param.has_modifier() {
            let mut modifiers =
                param.accessibility.map_or(modifier::PUBLIC, accessibility_modifier);
            if param.readonly {
                modifiers |= modifier::READONLY;
            }
            if param.r#override {
                modifiers |= modifier::OVERRIDE;
            }
            for_each_binding_identifier(&param.pattern, false, &mut |id, _| {
                self.validate(selector::PARAMETER_PROPERTY, id, modifiers, ctx);
            });
            return;
        }
        for_each_binding_identifier(&param.pattern, false, &mut |id, destructured| {
            self.validate_param(id, destructured, ctx);
        });
    }

    fn validate_param<'a>(
        &self,
        id: &BindingIdentifier<'a>,
        destructured: bool,
        ctx: &LintContext<'a>,
    ) {
        let mut modifiers = unused_modifier(id, ctx);
        if destructured {
            modifiers |= modifier::DESTRUCTURED;
        }
        self.validate(selector::PARAMETER, id, modifiers, ctx);
    }

    fn validate<'a>(
        &self,
        selector_type: u32,
        id: &BindingIdentifier<'a>,
        modifiers: u32,
        ctx: &LintContext<'a>,
    ) {
        self.validate_name(selector_type, id.name.as_str(), id.span, modifiers, ctx);
    }

    fn validate_key<'a>(
        &self,
        selector_type: u32,
        key: &PropertyKey<'a>,
        modifiers: u32,
        ctx: &LintContext<'a>,
    ) {
        let name: Cow<'_, str> = match key {
            PropertyKey::StaticIdentifier(ident) => Cow::Borrowed(ident.name.as_str()),
            PropertyKey::PrivateIdentifier(ident) => Cow::Borrowed(ident.name.as_str()),
            _ => match key.static_name() {
                Some(name) => name,
                None => return,
            },
        };
        let modifiers = modifiers | quoting_modifier(&name);
        self.validate_name(selector_type, &name, key.span(), modifiers, ctx);
    }

    fn validate_name(
        &self,
        selector_type: u32,
        original_name: &str,
        span: Span,
        modifiers: u32,
        ctx: &LintContext<'_>,
    ) {
        let index = selector_type.trailing_zeros() as usize;
        let kind = selector::MESSAGE_NAMES[index];

        for &config_index in &self.validators[index] {
            let config = &self.options[config_index];

            if let Some(filter) = &config.filter
                && filter.regex.is_match(original_name) != filter.r#match
            {
                // name does not match the filter
                continue;
            }

            if config.modifiers & modifiers != config.modifiers {
                // does not have the required modifiers
                continue;
            }

            if config.has_types && (selector::ALLOWED_TO_HAVE_TYPES & selector_type) != 0 {
                // type information is not available, so we cannot tell whether
                // the type matches; skip this option.
                continue;
            }

            let mut name = original_name;

            let Some(trimmed) = validate_underscore(
                true,
                config.leading_underscore,
                name,
                kind,
                original_name,
                span,
                ctx,
            ) else {
                return;
            };
            name = trimmed;

            let Some(trimmed) = validate_underscore(
                false,
                config.trailing_underscore,
                name,
                kind,
                original_name,
                span,
                ctx,
            ) else {
                return;
            };
            name = trimmed;

            let Some(trimmed) = validate_affix(
                true,
                config.prefix.as_deref(),
                name,
                kind,
                original_name,
                span,
                ctx,
            ) else {
                return;
            };
            name = trimmed;

            let Some(trimmed) = validate_affix(
                false,
                config.suffix.as_deref(),
                name,
                kind,
                original_name,
                span,
                ctx,
            ) else {
                return;
            };
            name = trimmed;

            if let Some(custom) = &config.custom
                && custom.regex.is_match(name) != custom.r#match
            {
                ctx.diagnostic(satisfy_custom_diagnostic(span, kind, original_name, custom));
                return;
            }

            if let Some(formats) = &config.format
                && !formats.is_empty()
            {
                let matches = (modifiers & modifier::REQUIRES_QUOTES) == 0
                    && formats.iter().any(|format| format.check(name));
                if !matches {
                    ctx.diagnostic(does_not_match_format_diagnostic(
                        span,
                        kind,
                        original_name,
                        name,
                        formats,
                    ));
                    return;
                }
            }

            // it's valid for this config, so we don't need to check any more configs
            return;
        }
    }
}

/// Returns the name with the underscore removed if it is valid according to
/// the specified underscore option, `None` otherwise (after reporting).
fn validate_underscore<'n>(
    leading: bool,
    option: Option<UnderscoreOption>,
    name: &'n str,
    kind: &str,
    original_name: &str,
    span: Span,
    ctx: &LintContext<'_>,
) -> Option<&'n str> {
    let Some(option) = option else { return Some(name) };
    let position = if leading { "leading" } else { "trailing" };

    let has_single = if leading { name.starts_with('_') } else { name.ends_with('_') };
    let has_double = if leading { name.starts_with("__") } else { name.ends_with("__") };
    let trim_single = || if leading { &name[1..] } else { &name[..name.len() - 1] };
    let trim_double = || if leading { &name[2..] } else { &name[..name.len() - 2] };

    match option {
        UnderscoreOption::Allow => Some(if has_single { trim_single() } else { name }),
        UnderscoreOption::AllowDouble => Some(if has_double { trim_double() } else { name }),
        UnderscoreOption::AllowSingleOrDouble => Some(if has_double {
            trim_double()
        } else if has_single {
            trim_single()
        } else {
            name
        }),
        UnderscoreOption::Forbid => {
            if has_single {
                ctx.diagnostic(unexpected_underscore_diagnostic(
                    span,
                    kind,
                    original_name,
                    position,
                ));
                return None;
            }
            Some(name)
        }
        UnderscoreOption::Require => {
            if !has_single {
                ctx.diagnostic(missing_underscore_diagnostic(
                    span,
                    kind,
                    original_name,
                    "one",
                    position,
                ));
                return None;
            }
            Some(trim_single())
        }
        UnderscoreOption::RequireDouble => {
            if !has_double {
                ctx.diagnostic(missing_underscore_diagnostic(
                    span,
                    kind,
                    original_name,
                    "two",
                    position,
                ));
                return None;
            }
            Some(trim_double())
        }
    }
}

/// Returns the name with the affix removed if it is valid according to the
/// specified affix option, `None` otherwise (after reporting).
fn validate_affix<'n>(
    prefix: bool,
    affixes: Option<&[String]>,
    name: &'n str,
    kind: &str,
    original_name: &str,
    span: Span,
    ctx: &LintContext<'_>,
) -> Option<&'n str> {
    let Some(affixes) = affixes else { return Some(name) };
    if affixes.is_empty() {
        return Some(name);
    }
    for affix in affixes {
        if prefix {
            if let Some(rest) = name.strip_prefix(affix.as_str()) {
                return Some(rest);
            }
        } else if let Some(rest) = name.strip_suffix(affix.as_str()) {
            return Some(rest);
        }
    }
    let position = if prefix { "prefix" } else { "suffix" };
    ctx.diagnostic(missing_affix_diagnostic(span, kind, original_name, position, affixes));
    None
}

// ---------------------------------------------------------------------------
// Modifier helpers
// ---------------------------------------------------------------------------

fn accessibility_modifier(accessibility: TSAccessibility) -> u32 {
    match accessibility {
        TSAccessibility::Public => modifier::PUBLIC,
        TSAccessibility::Protected => modifier::PROTECTED,
        TSAccessibility::Private => modifier::PRIVATE,
    }
}

fn member_modifiers(
    key: &PropertyKey<'_>,
    accessibility: Option<TSAccessibility>,
    is_static: bool,
    readonly: bool,
    is_override: bool,
    is_abstract: bool,
) -> u32 {
    let mut modifiers = if key.is_private_identifier() {
        modifier::HASH_PRIVATE
    } else {
        accessibility.map_or(modifier::PUBLIC, accessibility_modifier)
    };
    if is_static {
        modifiers |= modifier::STATIC;
    }
    if readonly {
        modifiers |= modifier::READONLY;
    }
    if is_override {
        modifiers |= modifier::OVERRIDE;
    }
    if is_abstract {
        modifiers |= modifier::ABSTRACT;
    }
    modifiers
}

fn quoting_modifier(name: &str) -> u32 {
    if is_identifier_name(name) { 0 } else { modifier::REQUIRES_QUOTES }
}

/// `exported` and `unused` modifiers for a declaration binding.
///
/// `node_id` is the declaration node whose parent is checked for
/// `export` / `export default`.
fn declaration_modifiers(
    id: &BindingIdentifier<'_>,
    node_id: NodeId,
    ctx: &LintContext<'_>,
) -> u32 {
    if is_exported(id, node_id, ctx) {
        // exported bindings are always considered used
        modifier::EXPORTED
    } else {
        unused_modifier(id, ctx)
    }
}

fn unused_modifier(id: &BindingIdentifier<'_>, ctx: &LintContext<'_>) -> u32 {
    let Some(symbol_id) = id.symbol_id.get() else { return 0 };
    if is_unused(symbol_id, ctx) { modifier::UNUSED } else { 0 }
}

fn is_unused(symbol_id: SymbolId, ctx: &LintContext<'_>) -> bool {
    !ctx.scoping()
        .get_resolved_references(symbol_id)
        .any(|reference| reference.is_read() || reference.is_type())
}

fn is_exported(id: &BindingIdentifier<'_>, node_id: NodeId, ctx: &LintContext<'_>) -> bool {
    if matches!(
        ctx.nodes().parent_kind(node_id),
        AstKind::ExportDefaultDeclaration(_) | AstKind::ExportDeclaration(_)
    ) {
        return true;
    }
    let Some(symbol_id) = id.symbol_id.get() else { return false };
    ctx.scoping().get_resolved_references(symbol_id).any(|reference: &Reference| {
        matches!(
            ctx.nodes().parent_kind(reference.node_id()),
            AstKind::ExportDefaultDeclaration(_) | AstKind::ExportSpecifier(_)
        )
    })
}

fn is_global_symbol(id: &BindingIdentifier<'_>, ctx: &LintContext<'_>) -> bool {
    id.symbol_id.get().is_some_and(|symbol_id| {
        ctx.scoping().symbol_scope_id(symbol_id) == ctx.scoping().root_scope_id()
    })
}

/// Visit every binding identifier in a pattern. The callback receives whether
/// the identifier is "destructured" in the sense used by typescript-eslint:
/// a shorthand object pattern property (`const { x }` / `const { x = 2 }`)
/// but not an aliased one (`const { x: y }`).
fn for_each_binding_identifier<'a, 'b>(
    pattern: &'b BindingPattern<'a>,
    destructured: bool,
    f: &mut impl FnMut(&'b BindingIdentifier<'a>, bool),
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => f(id, destructured),
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                for_each_binding_identifier(&prop.value, prop.shorthand, f);
            }
            if let Some(rest) = &obj.rest {
                for_each_binding_identifier(&rest.argument, false, f);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for element in (&arr.elements).into_iter().flatten() {
                for_each_binding_identifier(element, false, f);
            }
            if let Some(rest) = &arr.rest {
                for_each_binding_identifier(&rest.argument, false, f);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            for_each_binding_identifier(&assign.left, destructured, f);
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    // spellchecker:off
    // These cases are transcribed from typescript-eslint's own fixtures, whose
    // shared `filter` is the regex `.gnored` — a deliberate way to match both
    // `ignored` and `Ignored` without case-insensitivity. The spellchecker
    // reads that fragment as a misspelling, so it is silenced for the
    // transcribed data below; upstream silences it the same way, with a
    // `cSpell` directive. Keep this comment inside the silenced block.
    let pass = vec![
        (
            r"
                const child_process = require('child_process');
              ",
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":"child_process"},"format":["camelCase"],"selector":"default"}]),
            ),
        ),
        (
            r"
                let foo = 'a';
                const _foo = 1;
                interface Foo {}
                class Bar {}
                function foo_function_bar() {}
              ",
            Some(
                serde_json::json!([{"custom":{"match":false,"regex":"^unused_\\w"},"format":["camelCase"],"leadingUnderscore":"allow","selector":"default"},{"custom":{"match":false,"regex":"^I[A-Z]"},"format":["PascalCase"],"selector":"typeLike"},{"custom":{"match":true,"regex":"_function_"},"format":["snake_case"],"leadingUnderscore":"allow","selector":"function"}]),
            ),
        ),
        (
            r"
                let foo = 'a';
                const _foo = 1;
                interface foo {}
                class bar {}
                function fooFunctionBar() {}
                function _fooFunctionBar() {}
              ",
            Some(
                serde_json::json!([{"custom":{"match":false,"regex":"^unused_\\w"},"format":["camelCase"],"leadingUnderscore":"allow","selector":["default","typeLike","function"]}]),
            ),
        ),
        (
            r"
                const match = 'test'.match(/test/);
                const [, key, value] = match;
              ",
            Some(serde_json::json!([{"format":["camelCase"],"selector":"default"}])),
        ),
        (
            r"const snake_case = 1;",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"default"},{"format":null,"selector":"variable"}]),
            ),
        ),
        (
            r"const snake_case = 1;",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"default"},{"format":[],"selector":"variable"}]),
            ),
        ),
        (
            r"
                const child_process = require('child_process');
              ",
            Some(
                serde_json::json!([{"format":["camelCase","UPPER_CASE"],"selector":"variable"},{"filter":"child_process","format":["snake_case"],"selector":"variable"}]),
            ),
        ),
        (
            r"
                const foo = {
                  'Property-Name': 'asdf',
                };
              ",
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":"-"},"format":["strictCamelCase"],"selector":"default"}]),
            ),
        ),
        (
            r"
                const foo = {
                  'Property-Name': 'asdf',
                };
              ",
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":"^(Property-Name)$"},"format":["strictCamelCase"],"selector":"default"}]),
            ),
        ),
        (
            r"
                class foo {
                  private fooBoo: number;
                }
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"modifiers":["private"],"selector":["property","accessor"]}]),
            ),
        ),
        (
            r"
                class SomeClass {
                  static OtherConstant = 'hello';
                }
        
                export const { OtherConstant: otherConstant } = SomeClass;
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":"property"},{"format":["camelCase"],"selector":"variable"}]),
            ),
        ),
        (
            r"
                interface SOME_INTERFACE {
                  SomeMethod: () => void;
        
                  some_property: string;
                }
              ",
            Some(
                serde_json::json!([{"format":["UPPER_CASE"],"selector":"default"},{"format":["PascalCase"],"selector":"typeMethod"},{"format":["snake_case"],"selector":"typeProperty"}]),
            ),
        ),
        (
            r"
                type Ignored = {
                  ignored_due_to_modifiers: string;
                  readonly FOO: string;
                };
              ",
            Some(
                serde_json::json!([{"format":["UPPER_CASE"],"modifiers":["readonly"],"selector":"typeProperty"}]),
            ),
        ),
        (
            r"
                const camelCaseVar = 1;
                enum camelCaseEnum {}
                class camelCaseClass {}
                function camelCaseFunction() {}
                interface camelCaseInterface {}
                type camelCaseType = {};
                export const PascalCaseVar = 1;
                export enum PascalCaseEnum {}
                export class PascalCaseClass {}
                export function PascalCaseFunction() {}
                export interface PascalCaseInterface {}
                export type PascalCaseType = {};
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"default"},{"format":["PascalCase"],"modifiers":["exported"],"selector":"variable"},{"format":["PascalCase"],"modifiers":["exported"],"selector":"function"},{"format":["PascalCase"],"modifiers":["exported"],"selector":"class"},{"format":["PascalCase"],"modifiers":["exported"],"selector":"interface"},{"format":["PascalCase"],"modifiers":["exported"],"selector":"typeAlias"},{"format":["PascalCase"],"modifiers":["exported"],"selector":"enum"}]),
            ),
        ),
        (
            r"
                const camelCaseVar = 1;
                enum camelCaseEnum {}
                class camelCaseClass {}
                function camelCaseFunction() {}
                interface camelCaseInterface {}
                type camelCaseType = {};
                const PascalCaseVar = 1;
                enum PascalCaseEnum {}
                class PascalCaseClass {}
                function PascalCaseFunction() {}
                interface PascalCaseInterface {}
                type PascalCaseType = {};
                export {
                  PascalCaseVar,
                  PascalCaseEnum,
                  PascalCaseClass,
                  PascalCaseFunction,
                  PascalCaseInterface,
                  PascalCaseType,
                };
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"default"},{"format":["PascalCase"],"modifiers":["exported"],"selector":"variable"},{"format":["PascalCase"],"modifiers":["exported"],"selector":"function"},{"format":["PascalCase"],"modifiers":["exported"],"selector":"class"},{"format":["PascalCase"],"modifiers":["exported"],"selector":"interface"},{"format":["PascalCase"],"modifiers":["exported"],"selector":"typeAlias"},{"format":["PascalCase"],"modifiers":["exported"],"selector":"enum"}]),
            ),
        ),
        (
            r"
                {
                  const camelCaseVar = 1;
                  function camelCaseFunction() {}
                  declare function camelCaseDeclaredFunction();
                }
                const PascalCaseVar = 1;
                function PascalCaseFunction() {}
                declare function PascalCaseDeclaredFunction();
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"default"},{"format":["PascalCase"],"modifiers":["global"],"selector":"variable"},{"format":["PascalCase"],"modifiers":["global"],"selector":"function"}]),
            ),
        ),
        (
            r"
                const { some_name1 } = {};
                const { ignore: IgnoredDueToModifiers1 } = {};
                const { some_name2 = 2 } = {};
                const IgnoredDueToModifiers2 = 1;
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":"default"},{"format":["snake_case"],"modifiers":["destructured"],"selector":"variable"}]),
            ),
        ),
        (
            r"
                const { some_name1 } = {};
                const { ignore: IgnoredDueToModifiers1 } = {};
                const { some_name2 = 2 } = {};
                const IgnoredDueToModifiers2 = 1;
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":"default"},{"format":null,"modifiers":["destructured"],"selector":"variable"}]),
            ),
        ),
        (
            r"
                export function Foo(
                  { aName },
                  { anotherName = 1 },
                  { ignored: IgnoredDueToModifiers1 },
                  { ignored: IgnoredDueToModifiers3 = 2 },
                  IgnoredDueToModifiers2,
                ) {}
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":"default"},{"format":["camelCase"],"modifiers":["destructured"],"selector":"parameter"}]),
            ),
        ),
        (
            r"
                class Ignored {
                  private static readonly some_name;
                  IgnoredDueToModifiers = 1;
                }
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":"default"},{"format":["snake_case"],"modifiers":["static","readonly"],"selector":"classProperty"}]),
            ),
        ),
        (
            r"
                class Ignored {
                  constructor(
                    private readonly some_name,
                    IgnoredDueToModifiers,
                  ) {}
                }
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":"default"},{"format":["snake_case"],"modifiers":["readonly"],"selector":"parameterProperty"}]),
            ),
        ),
        (
            r"
                class Ignored {
                  private static some_name() {}
                  IgnoredDueToModifiers() {}
                }
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":"default"},{"format":["snake_case"],"modifiers":["static"],"selector":"classMethod"}]),
            ),
        ),
        (
            r"
                class Ignored {
                  private static get some_name() {}
                  get IgnoredDueToModifiers() {}
                }
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":"default"},{"format":["snake_case"],"modifiers":["private","static"],"selector":"accessor"}]),
            ),
        ),
        (
            r"
                abstract class some_name {}
                class IgnoredDueToModifier {}
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":"default"},{"format":["snake_case"],"modifiers":["abstract"],"selector":"class"}]),
            ),
        ),
        (
            r"
                const UnusedVar = 1;
                function UnusedFunc(
                  // this line is intentionally broken out
                  UnusedParam: string,
                ) {}
                class UnusedClass {}
                interface UnusedInterface {}
                type UnusedType<
                  // this line is intentionally broken out
                  UnusedTypeParam,
                > = {};
        
                export const used_var = 1;
                export function used_func(
                  // this line is intentionally broken out
                  used_param: string,
                ) {
                  return used_param;
                }
                export class used_class {}
                export interface used_interface {}
                export type used_type<
                  // this line is intentionally broken out
                  used_typeparam,
                > = used_typeparam;
              ",
            Some(
                serde_json::json!([{"format":["snake_case"],"selector":"default"},{"format":["PascalCase"],"modifiers":["unused"],"selector":"default"}]),
            ),
        ),
        (
            r"
                const ignored1 = {
                  'a a': 1,
                  'b b'() {},
                  get 'c c'() {
                    return 1;
                  },
                  set 'd d'(value: string) {},
                };
                class ignored2 {
                  'a a' = 1;
                  'b b'() {}
                  get 'c c'() {
                    return 1;
                  }
                  set 'd d'(value: string) {}
                }
                interface ignored3 {
                  'a a': 1;
                  'b b'(): void;
                }
                type ignored4 = {
                  'a a': 1;
                  'b b'(): void;
                };
                enum ignored5 {
                  'a a',
                }
              ",
            Some(
                serde_json::json!([{"format":["snake_case"],"selector":"default"},{"format":null,"modifiers":["requiresQuotes"],"selector":"default"}]),
            ),
        ),
        (
            r"
                const ignored1 = {
                  'a a': 1,
                  'b b'() {},
                  get 'c c'() {
                    return 1;
                  },
                  set 'd d'(value: string) {},
                };
                class ignored2 {
                  'a a' = 1;
                  'b b'() {}
                  get 'c c'() {
                    return 1;
                  }
                  set 'd d'(value: string) {}
                }
                interface ignored3 {
                  'a a': 1;
                  'b b'(): void;
                }
                type ignored4 = {
                  'a a': 1;
                  'b b'(): void;
                };
                enum ignored5 {
                  'a a',
                }
              ",
            Some(
                serde_json::json!([{"format":["snake_case"],"selector":"default"},{"format":null,"modifiers":["requiresQuotes"],"selector":["classProperty","objectLiteralProperty","typeProperty","classMethod","objectLiteralMethod","typeMethod","accessor","enumMember"]},{"format":["PascalCase"],"selector":["classProperty","objectLiteralProperty","typeProperty","classMethod","objectLiteralMethod","typeMethod","accessor","enumMember"]}]),
            ),
        ),
        (
            r"
                const obj = {
                  Foo: 42,
                  Bar() {
                    return 42;
                  },
                };
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"memberLike"},{"format":["PascalCase"],"selector":"property"},{"format":["PascalCase"],"selector":"method"}]),
            ),
        ),
        (
            r"
                const obj = {
                  Bar() {
                    return 42;
                  },
                  async async_bar() {
                    return 42;
                  },
                };
                class foo {
                  public Bar() {
                    return 42;
                  }
                  public async async_bar() {
                    return 42;
                  }
                }
                abstract class foo2 {
                  public Bar() {
                    return 42;
                  }
                  public async async_bar() {
                    return 42;
                  }
                }
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"memberLike"},{"format":["snake_case"],"modifiers":["async"],"selector":["method","objectLiteralMethod"]},{"format":["PascalCase"],"selector":"method"}]),
            ),
        ),
        (
            r"
                const async_bar1 = async () => {};
                async function async_bar2() {}
                const async_bar3 = async function async_bar4() {};
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"memberLike"},{"format":["PascalCase"],"selector":"method"},{"format":["snake_case"],"modifiers":["async"],"selector":["variable"]}]),
            ),
        ),
        (
            r"
                class foo extends bar {
                  public someAttribute = 1;
                  public override some_attribute_override = 1;
                  public someMethod() {
                    return 42;
                  }
                  public override some_method_override2() {
                    return 42;
                  }
                }
                abstract class foo2 extends bar {
                  public abstract someAttribute: string;
                  public abstract override some_attribute_override: string;
                  public abstract someMethod(): string;
                  public abstract override some_method_override2(): string;
                }
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"memberLike"},{"format":["snake_case"],"modifiers":["override"],"selector":["memberLike"]}]),
            ),
        ),
        (
            r"
                class foo {
                  private someAttribute = 1;
                  #some_attribute = 1;
        
                  private someMethod() {}
                  #some_method() {}
                }
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"memberLike"},{"format":["snake_case"],"modifiers":["#private"],"selector":["memberLike"]}]),
            ),
        ),
        (
            r"import * as FooBar from 'foo_bar';",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":["import"]},{"format":["camelCase"],"modifiers":["default"],"selector":["import"]}]),
            ),
        ),
        (
            r"import fooBar from 'foo_bar';",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":["import"]},{"format":["camelCase"],"modifiers":["default"],"selector":["import"]}]),
            ),
        ),
        (
            r"import { default as fooBar } from 'foo_bar';",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":["import"]},{"format":["camelCase"],"modifiers":["default"],"selector":["import"]}]),
            ),
        ),
        (
            r"import { foo_bar } from 'foo_bar';",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":["import"]},{"format":["camelCase"],"modifiers":["default"],"selector":["import"]}]),
            ),
        ),
        (
            r#"import { "🍎" as Foo } from 'foo_bar';"#,
            Some(serde_json::json!([{"format":["PascalCase"],"selector":["import"]}])),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { accessor strictCamelCase = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { accessor #strictCamelCase = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { static accessor strictCamelCase = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { static accessor #strictCamelCase = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { private accessor strictCamelCase = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { private static accessor strictCamelCase = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { override accessor strictCamelCase = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { accessor "strictCamelCase" = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { protected accessor strictCamelCase = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { public accessor strictCamelCase = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { abstract accessor strictCamelCase; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        const ignored = { get strictCamelCase() {} };"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        const ignored = { set "strictCamelCase"(ignored) {} };"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { private get strictCamelCase() {} }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { private set "strictCamelCase"(ignored) {} }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { private static get strictCamelCase() {} }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { static get #strictCamelCase() {} }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { accessor strictCamelCase = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { accessor #strictCamelCase = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { static accessor strictCamelCase = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { static accessor #strictCamelCase = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { private accessor strictCamelCase = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { private static accessor strictCamelCase = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { override accessor strictCamelCase = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { accessor "strictCamelCase" = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { protected accessor strictCamelCase = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { public accessor strictCamelCase = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { abstract accessor strictCamelCase; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"class","format":["camelCase"]}
        class strictCamelCase {}"#,
            Some(
                serde_json::json!([{"selector":"class","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"class","format":["camelCase"]}
        abstract class strictCamelCase {}"#,
            Some(
                serde_json::json!([{"selector":"class","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"class","format":["camelCase"]}
        const ignored = class strictCamelCase {}"#,
            Some(
                serde_json::json!([{"selector":"class","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classicAccessor","format":["camelCase"]}
        const ignored = { get strictCamelCase() {} };"#,
            Some(
                serde_json::json!([{"selector":"classicAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classicAccessor","format":["camelCase"]}
        const ignored = { set "strictCamelCase"(ignored) {} };"#,
            Some(
                serde_json::json!([{"selector":"classicAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classicAccessor","format":["camelCase"]}
        class Ignored { private get strictCamelCase() {} }"#,
            Some(
                serde_json::json!([{"selector":"classicAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classicAccessor","format":["camelCase"]}
        class Ignored { private set "strictCamelCase"(ignored) {} }"#,
            Some(
                serde_json::json!([{"selector":"classicAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classicAccessor","format":["camelCase"]}
        class Ignored { private static get strictCamelCase() {} }"#,
            Some(
                serde_json::json!([{"selector":"classicAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classicAccessor","format":["camelCase"]}
        class Ignored { static get #strictCamelCase() {} }"#,
            Some(
                serde_json::json!([{"selector":"classicAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classicAccessor","format":["camelCase"]}
        abstract class Ignored { abstract get strictCamelCase(): number }"#,
            Some(
                serde_json::json!([{"selector":"classicAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classicAccessor","format":["camelCase"]}
        abstract class Ignored { abstract set strictCamelCase(ignored: number) }"#,
            Some(
                serde_json::json!([{"selector":"classicAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        const lower = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        const camelCaseUNSTRICT = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["PascalCase"]}
        const StrictPascalCase = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["PascalCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["PascalCase"]}
        const Pascal = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["PascalCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["PascalCase"]}
        const I18n = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["PascalCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["PascalCase"]}
        const PascalCaseUNSTRICT = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["PascalCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["PascalCase"]}
        const UPPER = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["PascalCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["snake_case"]}
        const snake_case = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["snake_case"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["snake_case"]}
        const lower = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["snake_case"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["strictCamelCase"]}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["strictCamelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["strictCamelCase"]}
        const lower = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["strictCamelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["StrictPascalCase"]}
        const StrictPascalCase = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["StrictPascalCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["StrictPascalCase"]}
        const Pascal = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["StrictPascalCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["StrictPascalCase"]}
        const I18n = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["StrictPascalCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["UPPER_CASE"]}
        const UPPER_CASE = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["UPPER_CASE"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["UPPER_CASE"]}
        const UPPER = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["UPPER_CASE"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        function strictCamelCase () {}"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        (function (strictCamelCase) {});"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        class Ignored { constructor(private strictCamelCase) {} }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        const ignored = { strictCamelCase };"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        interface Ignored { strictCamelCase: string }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        type Ignored = { strictCamelCase: string }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        class Ignored { private strictCamelCase = 1 }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        class Ignored { #strictCamelCase = 1 }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        class Ignored { constructor(private strictCamelCase) {} }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        class Ignored { #strictCamelCase() {} }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        class Ignored { private strictCamelCase() {} }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        const ignored = { strictCamelCase() {} };"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        class Ignored { private get strictCamelCase() {} }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        enum Ignored { strictCamelCase }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        abstract class strictCamelCase {}"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        interface strictCamelCase { }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        type strictCamelCase = { };"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        enum strictCamelCase {}"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        interface Ignored<strictCamelCase> extends Ignored<string> {}"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"selector":"enum","format":["camelCase"]}
        enum strictCamelCase {}"#,
            Some(
                serde_json::json!([{"selector":"enum","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"enumMember","format":["camelCase"]}
        enum Ignored { strictCamelCase }"#,
            Some(
                serde_json::json!([{"selector":"enumMember","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"enumMember","format":["camelCase"]}
        enum Ignored { "strictCamelCase" }"#,
            Some(
                serde_json::json!([{"selector":"enumMember","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"function","format":["camelCase"]}
        function strictCamelCase () {}"#,
            Some(
                serde_json::json!([{"selector":"function","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"function","format":["camelCase"]}
        (function strictCamelCase () {});"#,
            Some(
                serde_json::json!([{"selector":"function","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"function","format":["camelCase"]}
        declare function strictCamelCase ();"#,
            Some(
                serde_json::json!([{"selector":"function","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"interface","format":["camelCase"]}
        interface strictCamelCase {}"#,
            Some(
                serde_json::json!([{"selector":"interface","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classMethod","format":["camelCase"]}
        class Ignored { private strictCamelCase() {} }"#,
            Some(
                serde_json::json!([{"selector":"classMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classMethod","format":["camelCase"]}
        class Ignored { private "strictCamelCase"() {} }"#,
            Some(
                serde_json::json!([{"selector":"classMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classMethod","format":["camelCase"]}
        class Ignored { private async strictCamelCase() {} }"#,
            Some(
                serde_json::json!([{"selector":"classMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classMethod","format":["camelCase"]}
        class Ignored { private static strictCamelCase() {} }"#,
            Some(
                serde_json::json!([{"selector":"classMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classMethod","format":["camelCase"]}
        class Ignored { private static async strictCamelCase() {} }"#,
            Some(
                serde_json::json!([{"selector":"classMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classMethod","format":["camelCase"]}
        class Ignored { private strictCamelCase = () => {} }"#,
            Some(
                serde_json::json!([{"selector":"classMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classMethod","format":["camelCase"]}
        class Ignored { abstract strictCamelCase() }"#,
            Some(
                serde_json::json!([{"selector":"classMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classMethod","format":["camelCase"]}
        class Ignored { #strictCamelCase() }"#,
            Some(
                serde_json::json!([{"selector":"classMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classMethod","format":["camelCase"]}
        class Ignored { static #strictCamelCase() }"#,
            Some(
                serde_json::json!([{"selector":"classMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"objectLiteralMethod","format":["camelCase"]}
        const ignored = { strictCamelCase() {} };"#,
            Some(
                serde_json::json!([{"selector":"objectLiteralMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"objectLiteralMethod","format":["camelCase"]}
        const ignored = { "strictCamelCase"() {} };"#,
            Some(
                serde_json::json!([{"selector":"objectLiteralMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"objectLiteralMethod","format":["camelCase"]}
        const ignored = { strictCamelCase: () => {} };"#,
            Some(
                serde_json::json!([{"selector":"objectLiteralMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeMethod","format":["camelCase"]}
        interface Ignored { strictCamelCase(): string }"#,
            Some(
                serde_json::json!([{"selector":"typeMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeMethod","format":["camelCase"]}
        interface Ignored { "strictCamelCase"(): string }"#,
            Some(
                serde_json::json!([{"selector":"typeMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeMethod","format":["camelCase"]}
        interface Ignored { strictCamelCase: () => string }"#,
            Some(
                serde_json::json!([{"selector":"typeMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeMethod","format":["camelCase"]}
        interface Ignored { "strictCamelCase": () => string }"#,
            Some(
                serde_json::json!([{"selector":"typeMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeMethod","format":["camelCase"]}
        type Ignored = { strictCamelCase(): string }"#,
            Some(
                serde_json::json!([{"selector":"typeMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeMethod","format":["camelCase"]}
        type Ignored = { "strictCamelCase"(): string }"#,
            Some(
                serde_json::json!([{"selector":"typeMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeMethod","format":["camelCase"]}
        type Ignored = { strictCamelCase: () => string }"#,
            Some(
                serde_json::json!([{"selector":"typeMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeMethod","format":["camelCase"]}
        type Ignored = { "strictCamelCase": () => string }"#,
            Some(
                serde_json::json!([{"selector":"typeMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        function ignored(strictCamelCase) {}"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        (function (strictCamelCase) {});"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        declare function ignored(strictCamelCase);"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        function ignored({strictCamelCase}) {}"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        function ignored(...strictCamelCase) {}"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        function ignored({strictCamelCase = 1}) {}"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        function ignored({...strictCamelCase}) {}"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        function ignored([strictCamelCase]) {}"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        function ignored([strictCamelCase = 1]) {}"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        function ignored([...strictCamelCase]) {}"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameterProperty","format":["camelCase"]}
        class Ignored { constructor(private strictCamelCase) {} }"#,
            Some(
                serde_json::json!([{"selector":"parameterProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameterProperty","format":["camelCase"]}
        class Ignored { constructor(readonly strictCamelCase) {} }"#,
            Some(
                serde_json::json!([{"selector":"parameterProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameterProperty","format":["camelCase"]}
        class Ignored { constructor(private readonly strictCamelCase) {} }"#,
            Some(
                serde_json::json!([{"selector":"parameterProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"modifiers":["readonly"],"selector":"parameterProperty","format":["camelCase"]}
        class Ignored { constructor(private readonly strictCamelCase) {} }"#,
            Some(
                serde_json::json!([{"modifiers":["readonly"],"selector":"parameterProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classProperty","format":["camelCase"]}
        class Ignored { private strictCamelCase }"#,
            Some(
                serde_json::json!([{"selector":"classProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classProperty","format":["camelCase"]}
        class Ignored { private "strictCamelCase" = 1 }"#,
            Some(
                serde_json::json!([{"selector":"classProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classProperty","format":["camelCase"]}
        class Ignored { private readonly strictCamelCase = 1 }"#,
            Some(
                serde_json::json!([{"selector":"classProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classProperty","format":["camelCase"]}
        class Ignored { private static strictCamelCase }"#,
            Some(
                serde_json::json!([{"selector":"classProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classProperty","format":["camelCase"]}
        class Ignored { private static readonly strictCamelCase = 1 }"#,
            Some(
                serde_json::json!([{"selector":"classProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classProperty","format":["camelCase"]}
        class Ignored { abstract strictCamelCase }"#,
            Some(
                serde_json::json!([{"selector":"classProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classProperty","format":["camelCase"]}
        class Ignored { declare strictCamelCase }"#,
            Some(
                serde_json::json!([{"selector":"classProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classProperty","format":["camelCase"]}
        class Ignored { #strictCamelCase }"#,
            Some(
                serde_json::json!([{"selector":"classProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classProperty","format":["camelCase"]}
        class Ignored { static #strictCamelCase }"#,
            Some(
                serde_json::json!([{"selector":"classProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"objectLiteralProperty","format":["camelCase"]}
        const ignored = { strictCamelCase };"#,
            Some(
                serde_json::json!([{"selector":"objectLiteralProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"objectLiteralProperty","format":["camelCase"]}
        const ignored = { "strictCamelCase": 1 };"#,
            Some(
                serde_json::json!([{"selector":"objectLiteralProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeProperty","format":["camelCase"]}
        interface Ignored { strictCamelCase }"#,
            Some(
                serde_json::json!([{"selector":"typeProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeProperty","format":["camelCase"]}
        interface Ignored { "strictCamelCase": string }"#,
            Some(
                serde_json::json!([{"selector":"typeProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeProperty","format":["camelCase"]}
        type Ignored = { strictCamelCase }"#,
            Some(
                serde_json::json!([{"selector":"typeProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeProperty","format":["camelCase"]}
        type Ignored = { "strictCamelCase": string }"#,
            Some(
                serde_json::json!([{"selector":"typeProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeAlias","format":["camelCase"]}
        type strictCamelCase = {};"#,
            Some(
                serde_json::json!([{"selector":"typeAlias","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeAlias","format":["camelCase"]}
        type strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"typeAlias","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeParameter","format":["camelCase"]}
        class Ignored<strictCamelCase> {}"#,
            Some(
                serde_json::json!([{"selector":"typeParameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeParameter","format":["camelCase"]}
        function ignored<strictCamelCase>() {}"#,
            Some(
                serde_json::json!([{"selector":"typeParameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeParameter","format":["camelCase"]}
        type Ignored<strictCamelCase> = { ignored: strictCamelCase };"#,
            Some(
                serde_json::json!([{"selector":"typeParameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeParameter","format":["camelCase"]}
        interface Ignored<strictCamelCase> extends Ignored<string> {}"#,
            Some(
                serde_json::json!([{"selector":"typeParameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"]}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"]}
        let strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"]}
        var strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"]}
        const {strictCamelCase} = {ignored: 1};"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"]}
        const {strictCamelCase = 2} = {ignored: 1};"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"]}
        const {...strictCamelCase} = {ignored: 1};"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"]}
        const [strictCamelCase] = [1];"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"]}
        const [strictCamelCase = 1] = [1];"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"]}
        const [...strictCamelCase] = [1];"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"forbid"}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"forbid","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"require"}
        const _strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"require","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"requireDouble"}
        const __strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"requireDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"allow"}
        const _strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"allow","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"allow"}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"allow","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowDouble"}
        const __strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowDouble"}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowSingleOrDouble"}
        const _strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowSingleOrDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowSingleOrDouble"}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowSingleOrDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowSingleOrDouble"}
        const __strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowSingleOrDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"forbid"}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"forbid","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"require"}
        const strictCamelCase_ = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"require","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"requireDouble"}
        const strictCamelCase__ = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"requireDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"allow"}
        const strictCamelCase_ = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"allow","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"allow"}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"allow","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowDouble"}
        const strictCamelCase__ = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowDouble"}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowSingleOrDouble"}
        const strictCamelCase_ = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowSingleOrDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowSingleOrDouble"}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowSingleOrDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowSingleOrDouble"}
        const strictCamelCase__ = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowSingleOrDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"prefix":["MyPrefix"]}
        const MyPrefixstrictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"prefix":["MyPrefix"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"prefix":["MyPrefix1","MyPrefix2"]}
        const MyPrefix2strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"prefix":["MyPrefix1","MyPrefix2"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"suffix":["MySuffix"]}
        const strictCamelCaseMySuffix = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"suffix":["MySuffix"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"suffix":["MySuffix1","MySuffix2"]}
        const strictCamelCaseMySuffix2 = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"suffix":["MySuffix1","MySuffix2"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"forbid"}
        let strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"forbid","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"require"}
        let _strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"require","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"requireDouble"}
        let __strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"requireDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"allow"}
        let _strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"allow","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"allow"}
        let strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"allow","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowDouble"}
        let __strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowDouble"}
        let strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowSingleOrDouble"}
        let _strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowSingleOrDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowSingleOrDouble"}
        let strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowSingleOrDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowSingleOrDouble"}
        let __strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"allowSingleOrDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"forbid"}
        let strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"forbid","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"require"}
        let strictCamelCase_ = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"require","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"requireDouble"}
        let strictCamelCase__ = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"requireDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"allow"}
        let strictCamelCase_ = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"allow","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"allow"}
        let strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"allow","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowDouble"}
        let strictCamelCase__ = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowDouble"}
        let strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowSingleOrDouble"}
        let strictCamelCase_ = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowSingleOrDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowSingleOrDouble"}
        let strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowSingleOrDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowSingleOrDouble"}
        let strictCamelCase__ = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"allowSingleOrDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"prefix":["MyPrefix"]}
        let MyPrefixstrictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"prefix":["MyPrefix"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"prefix":["MyPrefix1","MyPrefix2"]}
        let MyPrefix2strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"prefix":["MyPrefix1","MyPrefix2"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"suffix":["MySuffix"]}
        let strictCamelCaseMySuffix = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"suffix":["MySuffix"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"suffix":["MySuffix1","MySuffix2"]}
        let strictCamelCaseMySuffix2 = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"suffix":["MySuffix1","MySuffix2"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
    ];

    let fail = vec![
        (r"const x_x = 1;", None),
        (r"const x_x = 1;", Some(serde_json::json!([]))),
        (
            r"
                const child_process = require('child_process');
              ",
            Some(
                serde_json::json!([{"filter":{"match":true,"regex":"child_process"},"format":["camelCase"],"selector":"default"}]),
            ),
        ),
        (
            r"
                let unused_foo = 'a';
              ",
            Some(
                serde_json::json!([{"custom":{"match":false,"regex":"^unused_\\w"},"format":["snake_case"],"leadingUnderscore":"allow","selector":"default"}]),
            ),
        ),
        (
            r"
                const _unused_foo = 1;
              ",
            Some(
                serde_json::json!([{"custom":{"match":false,"regex":"^unused_\\w"},"format":["snake_case"],"leadingUnderscore":"allow","selector":"default"}]),
            ),
        ),
        (
            r"
                interface IFoo {}
              ",
            Some(
                serde_json::json!([{"custom":{"match":false,"regex":"^I[A-Z]"},"format":["PascalCase"],"selector":"typeLike"}]),
            ),
        ),
        (
            r"
                class IBar {}
              ",
            Some(
                serde_json::json!([{"custom":{"match":false,"regex":"^I[A-Z]"},"format":["PascalCase"],"selector":"typeLike"}]),
            ),
        ),
        (
            r"
                function fooBar() {}
              ",
            Some(
                serde_json::json!([{"custom":{"match":true,"regex":"function"},"format":["camelCase"],"leadingUnderscore":"allow","selector":"function"}]),
            ),
        ),
        (
            r"
                let unused_foo = 'a';
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"leadingUnderscore":"allow","selector":["variable","function"]}]),
            ),
        ),
        (
            r"
                const _unused_foo = 1;
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"leadingUnderscore":"allow","selector":["variable","function"]}]),
            ),
        ),
        (
            r"
                function foo_bar() {}
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"leadingUnderscore":"allow","selector":["variable","function"]}]),
            ),
        ),
        (
            r"
                interface IFoo {}
              ",
            Some(
                serde_json::json!([{"custom":{"match":false,"regex":"^I[A-Z]"},"format":["PascalCase"],"selector":["class","interface"]}]),
            ),
        ),
        (
            r"
                class IBar {}
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"leadingUnderscore":"allow","selector":["variable","function"]},{"custom":{"match":false,"regex":"^I[A-Z]"},"format":["PascalCase"],"selector":["class","interface"]}]),
            ),
        ),
        (
            r"
                const foo = {
                  'Property Name': 'asdf',
                };
              ",
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":"-"},"format":["strictCamelCase"],"selector":"default"}]),
            ),
        ),
        (
            r"
                class foo {
                  private readonly fooBar: boolean;
                }
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"modifiers":["private","readonly"],"selector":["property","accessor"]}]),
            ),
        ),
        (
            r"
                class SomeClass {
                  static otherConstant = 'hello';
                }
        
                export const { otherConstant } = SomeClass;
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":"property"},{"format":["camelCase"],"selector":"variable"}]),
            ),
        ),
        (
            r"
                declare class Foo {
                  Bar(Baz: string): void;
                }
              ",
            Some(serde_json::json!([{"format":["camelCase"],"selector":"parameter"}])),
        ),
        (
            r"
                export const PascalCaseVar = 1;
                export enum PascalCaseEnum {}
                export class PascalCaseClass {}
                export function PascalCaseFunction() {}
                export interface PascalCaseInterface {}
                export type PascalCaseType = {};
              ",
            Some(
                serde_json::json!([{"format":["snake_case"],"selector":"default"},{"format":["camelCase"],"modifiers":["exported"],"selector":"variable"},{"format":["camelCase"],"modifiers":["exported"],"selector":"function"},{"format":["camelCase"],"modifiers":["exported"],"selector":"class"},{"format":["camelCase"],"modifiers":["exported"],"selector":"interface"},{"format":["camelCase"],"modifiers":["exported"],"selector":"typeAlias"},{"format":["camelCase"],"modifiers":["exported"],"selector":"enum"}]),
            ),
        ),
        (
            r"
                const PascalCaseVar = 1;
                enum PascalCaseEnum {}
                class PascalCaseClass {}
                function PascalCaseFunction() {}
                interface PascalCaseInterface {}
                type PascalCaseType = {};
                export {
                  PascalCaseVar,
                  PascalCaseEnum,
                  PascalCaseClass,
                  PascalCaseFunction,
                  PascalCaseInterface,
                  PascalCaseType,
                };
              ",
            Some(
                serde_json::json!([{"format":["snake_case"],"selector":"default"},{"format":["camelCase"],"modifiers":["exported"],"selector":"variable"},{"format":["camelCase"],"modifiers":["exported"],"selector":"function"},{"format":["camelCase"],"modifiers":["exported"],"selector":"class"},{"format":["camelCase"],"modifiers":["exported"],"selector":"interface"},{"format":["camelCase"],"modifiers":["exported"],"selector":"typeAlias"},{"format":["camelCase"],"modifiers":["exported"],"selector":"enum"}]),
            ),
        ),
        (
            r"
                const PascalCaseVar = 1;
                function PascalCaseFunction() {}
                declare function PascalCaseDeclaredFunction();
              ",
            Some(
                serde_json::json!([{"format":["snake_case"],"selector":"default"},{"format":["camelCase"],"modifiers":["global"],"selector":"variable"},{"format":["camelCase"],"modifiers":["global"],"selector":"function"}]),
            ),
        ),
        (
            r"
                const { some_name1 } = {};
                const { some_name2 = 2 } = {};
                const { ignored: IgnoredDueToModifiers1 } = {};
                const { ignored: IgnoredDueToModifiers2 = 3 } = {};
                const IgnoredDueToModifiers3 = 1;
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":"default"},{"format":["UPPER_CASE"],"modifiers":["destructured"],"selector":"variable"}]),
            ),
        ),
        (
            r"
                export function Foo(
                  { aName },
                  { anotherName = 1 },
                  { ignored: IgnoredDueToModifiers1 },
                  { ignored: IgnoredDueToModifiers3 = 2 },
                  IgnoredDueToModifiers2,
                ) {}
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":"default"},{"format":["UPPER_CASE"],"modifiers":["destructured"],"selector":"parameter"}]),
            ),
        ),
        (
            r"
                class Ignored {
                  private static readonly some_name;
                  IgnoredDueToModifiers = 1;
                }
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":"default"},{"format":["UPPER_CASE"],"modifiers":["static","readonly"],"selector":"classProperty"}]),
            ),
        ),
        (
            r"
                class Ignored {
                  constructor(
                    private readonly some_name,
                    IgnoredDueToModifiers,
                  ) {}
                }
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":"default"},{"format":["UPPER_CASE"],"modifiers":["readonly"],"selector":"parameterProperty"}]),
            ),
        ),
        (
            r"
                class Ignored {
                  private static some_name() {}
                  IgnoredDueToModifiers() {}
                }
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":"default"},{"format":["UPPER_CASE"],"modifiers":["static"],"selector":"classMethod"}]),
            ),
        ),
        (
            r"
                class Ignored {
                  private static get some_name() {}
                  get IgnoredDueToModifiers() {}
                }
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":"default"},{"format":["UPPER_CASE"],"modifiers":["private","static"],"selector":"accessor"}]),
            ),
        ),
        (
            r"
                abstract class some_name {}
                class IgnoredDueToModifier {}
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":"default"},{"format":["UPPER_CASE"],"modifiers":["abstract"],"selector":"class"}]),
            ),
        ),
        (
            r"
                const UnusedVar = 1;
                function UnusedFunc(
                  // this line is intentionally broken out
                  UnusedParam: string,
                ) {}
                class UnusedClass {}
                interface UnusedInterface {}
                type UnusedType<
                  // this line is intentionally broken out
                  UnusedTypeParam,
                > = {};
              ",
            Some(
                serde_json::json!([{"format":["PascalCase"],"selector":"default"},{"format":["snake_case"],"modifiers":["unused"],"selector":"default"}]),
            ),
        ),
        (
            r"
                const ignored1 = {
                  'a a': 1,
                  'b b'() {},
                  get 'c c'() {
                    return 1;
                  },
                  set 'd d'(value: string) {},
                };
                class ignored2 {
                  'a a' = 1;
                  'b b'() {}
                  get 'c c'() {
                    return 1;
                  }
                  set 'd d'(value: string) {}
                }
                interface ignored3 {
                  'a a': 1;
                  'b b'(): void;
                }
                type ignored4 = {
                  'a a': 1;
                  'b b'(): void;
                };
                enum ignored5 {
                  'a a',
                }
              ",
            Some(
                serde_json::json!([{"format":["snake_case"],"selector":"default"},{"format":["PascalCase"],"modifiers":["requiresQuotes"],"selector":"default"}]),
            ),
        ),
        (
            r"
                type Foo = {
                  'foo     Bar': string;
                  '': string;
                  '0': string;
                  'foo': string;
                  'foo-bar': string;
                  '#foo-bar': string;
                };
        
                interface Bar {
                  'boo-----foo': string;
                }
              ",
            None,
        ),
        (
            r"
                class foo {
                  public Bar() {
                    return 42;
                  }
                  public async async_bar() {
                    return 42;
                  }
                  // ❌ error
                  public async asyncBar() {
                    return 42;
                  }
                  // ❌ error
                  public AsyncBar2 = async () => {
                    return 42;
                  };
                  // ❌ error
                  public AsyncBar3 = async function () {
                    return 42;
                  };
                }
                abstract class foo2 {
                  public abstract Bar(): number;
                  public abstract async async_bar(): number;
                  // ❌ error
                  public abstract async ASYNC_BAR(): number;
                }
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"memberLike"},{"format":["PascalCase"],"selector":"method"},{"format":["snake_case"],"modifiers":["async"],"selector":["method","objectLiteralMethod"]}]),
            ),
        ),
        (
            r"
                const obj = {
                  Bar() {
                    return 42;
                  },
                  async async_bar() {
                    return 42;
                  },
                  // ❌ error
                  async AsyncBar() {
                    return 42;
                  },
                  // ❌ error
                  AsyncBar2: async () => {
                    return 42;
                  },
                  // ❌ error
                  AsyncBar3: async function () {
                    return 42;
                  },
                };
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"memberLike"},{"format":["PascalCase"],"selector":"method"},{"format":["snake_case"],"modifiers":["async"],"selector":["method","objectLiteralMethod"]}]),
            ),
        ),
        (
            r"
                const syncbar1 = () => {};
                function syncBar2() {}
                const syncBar3 = function syncBar4() {};
        
                // ❌ error
                const AsyncBar1 = async () => {};
                const async_bar1 = async () => {};
                const async_bar3 = async function async_bar4() {};
                async function async_bar2() {}
                // ❌ error
                const asyncBar5 = async function async_bar6() {};
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"variableLike"},{"format":["snake_case"],"modifiers":["async"],"selector":["variableLike"]}]),
            ),
        ),
        (
            r"
                const syncbar1 = () => {};
                function syncBar2() {}
                const syncBar3 = function syncBar4() {};
        
                const async_bar1 = async () => {};
                // ❌ error
                async function asyncBar2() {}
                const async_bar3 = async function async_bar4() {};
                async function async_bar2() {}
                // ❌ error
                const async_bar3 = async function ASYNC_BAR4() {};
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"variableLike"},{"format":["snake_case"],"modifiers":["async"],"selector":["variableLike"]}]),
            ),
        ),
        (
            r"
                class foo extends bar {
                  public someAttribute = 1;
                  public override some_attribute_override = 1;
                  // ❌ error
                  public override someAttributeOverride = 1;
                }
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"memberLike"},{"format":["snake_case"],"modifiers":["override"],"selector":["memberLike"]}]),
            ),
        ),
        (
            r"
                class foo extends bar {
                  public override some_method_override() {
                    return 42;
                  }
                  // ❌ error
                  public override someMethodOverride() {
                    return 42;
                  }
                }
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"memberLike"},{"format":["snake_case"],"modifiers":["override"],"selector":["memberLike"]}]),
            ),
        ),
        (
            r"
                class foo extends bar {
                  public get someGetter(): string;
                  public override get some_getter_override(): string;
                  // ❌ error
                  public override get someGetterOverride(): string;
                  public set someSetter(val: string);
                  public override set some_setter_override(val: string);
                  // ❌ error
                  public override set someSetterOverride(val: string);
                }
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"memberLike"},{"format":["snake_case"],"modifiers":["override"],"selector":["memberLike"]}]),
            ),
        ),
        (
            r"
                class foo {
                  private firstPrivateField = 1;
                  // ❌ error
                  private first_private_field = 1;
                  // ❌ error
                  #secondPrivateField = 1;
                  #second_private_field = 1;
                }
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"memberLike"},{"format":["snake_case"],"modifiers":["#private"],"selector":["memberLike"]}]),
            ),
        ),
        (
            r"
                class foo {
                  private firstPrivateMethod() {}
                  // ❌ error
                  private first_private_method() {}
                  // ❌ error
                  #secondPrivateMethod() {}
                  #second_private_method() {}
                }
              ",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":"memberLike"},{"format":["snake_case"],"modifiers":["#private"],"selector":["memberLike"]}]),
            ),
        ),
        (
            r"import * as fooBar from 'foo_bar';",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":["import"]},{"format":["PascalCase"],"modifiers":["namespace"],"selector":["import"]}]),
            ),
        ),
        (
            r"import FooBar from 'foo_bar';",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":["import"]},{"format":["PascalCase"],"modifiers":["namespace"],"selector":["import"]}]),
            ),
        ),
        (
            r"import { default as foo_bar } from 'foo_bar';",
            Some(
                serde_json::json!([{"format":["camelCase"],"selector":["import"]},{"format":["PascalCase"],"modifiers":["namespace"],"selector":["import"]}]),
            ),
        ),
        (
            r#"import { "🍎" as foo } from 'foo_bar';"#,
            Some(serde_json::json!([{"format":["PascalCase"],"selector":["import"]}])),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { accessor snake_case = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { accessor #snake_case = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { static accessor snake_case = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { static accessor #snake_case = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { private accessor snake_case = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { private static accessor snake_case = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { override accessor snake_case = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { accessor "snake_case" = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { protected accessor snake_case = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { public accessor snake_case = 10; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { abstract accessor snake_case; }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        const ignored = { get snake_case() {} };"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        const ignored = { set "snake_case"(ignored) {} };"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { private get snake_case() {} }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { private set "snake_case"(ignored) {} }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { private static get snake_case() {} }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"accessor","format":["camelCase"]}
        class Ignored { static get #snake_case() {} }"#,
            Some(
                serde_json::json!([{"selector":"accessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { accessor snake_case = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { accessor #snake_case = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { static accessor snake_case = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { static accessor #snake_case = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { private accessor snake_case = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { private static accessor snake_case = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { override accessor snake_case = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { accessor "snake_case" = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { protected accessor snake_case = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { public accessor snake_case = 10; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"autoAccessor","format":["camelCase"]}
        class Ignored { abstract accessor snake_case; }"#,
            Some(
                serde_json::json!([{"selector":"autoAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"class","format":["camelCase"]}
        class snake_case {}"#,
            Some(
                serde_json::json!([{"selector":"class","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"class","format":["camelCase"]}
        abstract class snake_case {}"#,
            Some(
                serde_json::json!([{"selector":"class","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"class","format":["camelCase"]}
        const ignored = class snake_case {}"#,
            Some(
                serde_json::json!([{"selector":"class","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classicAccessor","format":["camelCase"]}
        const ignored = { get snake_case() {} };"#,
            Some(
                serde_json::json!([{"selector":"classicAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classicAccessor","format":["camelCase"]}
        const ignored = { set "snake_case"(ignored) {} };"#,
            Some(
                serde_json::json!([{"selector":"classicAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classicAccessor","format":["camelCase"]}
        class Ignored { private get snake_case() {} }"#,
            Some(
                serde_json::json!([{"selector":"classicAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classicAccessor","format":["camelCase"]}
        class Ignored { private set "snake_case"(ignored) {} }"#,
            Some(
                serde_json::json!([{"selector":"classicAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classicAccessor","format":["camelCase"]}
        class Ignored { private static get snake_case() {} }"#,
            Some(
                serde_json::json!([{"selector":"classicAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classicAccessor","format":["camelCase"]}
        class Ignored { static get #snake_case() {} }"#,
            Some(
                serde_json::json!([{"selector":"classicAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classicAccessor","format":["camelCase"]}
        abstract class Ignored { abstract get snake_case(): number }"#,
            Some(
                serde_json::json!([{"selector":"classicAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classicAccessor","format":["camelCase"]}
        abstract class Ignored { abstract set snake_case(ignored: number) }"#,
            Some(
                serde_json::json!([{"selector":"classicAccessor","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        const snake_case = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        const snake_case = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        const UPPER_CASE = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        const UPPER = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        const StrictPascalCase = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["PascalCase"]}
        const snake_case = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["PascalCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["PascalCase"]}
        const UPPER_CASE = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["PascalCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["PascalCase"]}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["PascalCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["snake_case"]}
        const UPPER_CASE = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["snake_case"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["snake_case"]}
        const SNAKE_case_UNSTRICT = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["snake_case"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["snake_case"]}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["snake_case"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["snake_case"]}
        const StrictPascalCase = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["snake_case"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["strictCamelCase"]}
        const snake_case = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["strictCamelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["strictCamelCase"]}
        const UPPER_CASE = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["strictCamelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["strictCamelCase"]}
        const UPPER = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["strictCamelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["strictCamelCase"]}
        const StrictPascalCase = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["strictCamelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["strictCamelCase"]}
        const camelCaseUNSTRICT = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["strictCamelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["StrictPascalCase"]}
        const snake_case = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["StrictPascalCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["StrictPascalCase"]}
        const UPPER_CASE = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["StrictPascalCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["StrictPascalCase"]}
        const UPPER = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["StrictPascalCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["StrictPascalCase"]}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["StrictPascalCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["StrictPascalCase"]}
        const PascalCaseUNSTRICT = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["StrictPascalCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["UPPER_CASE"]}
        const lower = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["UPPER_CASE"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["UPPER_CASE"]}
        const snake_case = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["UPPER_CASE"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["UPPER_CASE"]}
        const SNAKE_case_UNSTRICT = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["UPPER_CASE"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["UPPER_CASE"]}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["UPPER_CASE"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["UPPER_CASE"]}
        const StrictPascalCase = 1;"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["UPPER_CASE"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        function snake_case () {}"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        (function (snake_case) {});"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        class Ignored { constructor(private snake_case) {} }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        const ignored = { snake_case };"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        interface Ignored { snake_case: string }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        type Ignored = { snake_case: string }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        class Ignored { private snake_case = 1 }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        class Ignored { #snake_case = 1 }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        class Ignored { constructor(private snake_case) {} }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        class Ignored { #snake_case() {} }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        class Ignored { private snake_case() {} }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        const ignored = { snake_case() {} };"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        class Ignored { private get snake_case() {} }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        enum Ignored { snake_case }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        abstract class snake_case {}"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        interface snake_case { }"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        type snake_case = { };"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        enum snake_case {}"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"filter":"[iI]gnored","selector":"default","format":["camelCase"]}
        interface Ignored<snake_case> extends Ignored<string> {}"#,
            Some(
                serde_json::json!([{"filter":{"match":false,"regex":".gnored"},"selector":"default","format":["camelCase"]}]),
            ),
        ),
        (
            r#"// {"selector":"enum","format":["camelCase"]}
        enum snake_case {}"#,
            Some(
                serde_json::json!([{"selector":"enum","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"enumMember","format":["camelCase"]}
        enum Ignored { snake_case }"#,
            Some(
                serde_json::json!([{"selector":"enumMember","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"enumMember","format":["camelCase"]}
        enum Ignored { "snake_case" }"#,
            Some(
                serde_json::json!([{"selector":"enumMember","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"function","format":["camelCase"]}
        function snake_case () {}"#,
            Some(
                serde_json::json!([{"selector":"function","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"function","format":["camelCase"]}
        (function snake_case () {});"#,
            Some(
                serde_json::json!([{"selector":"function","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"function","format":["camelCase"]}
        declare function snake_case ();"#,
            Some(
                serde_json::json!([{"selector":"function","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"interface","format":["camelCase"]}
        interface snake_case {}"#,
            Some(
                serde_json::json!([{"selector":"interface","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classMethod","format":["camelCase"]}
        class Ignored { private snake_case() {} }"#,
            Some(
                serde_json::json!([{"selector":"classMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classMethod","format":["camelCase"]}
        class Ignored { private "snake_case"() {} }"#,
            Some(
                serde_json::json!([{"selector":"classMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classMethod","format":["camelCase"]}
        class Ignored { private async snake_case() {} }"#,
            Some(
                serde_json::json!([{"selector":"classMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classMethod","format":["camelCase"]}
        class Ignored { private static snake_case() {} }"#,
            Some(
                serde_json::json!([{"selector":"classMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classMethod","format":["camelCase"]}
        class Ignored { private static async snake_case() {} }"#,
            Some(
                serde_json::json!([{"selector":"classMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classMethod","format":["camelCase"]}
        class Ignored { private snake_case = () => {} }"#,
            Some(
                serde_json::json!([{"selector":"classMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classMethod","format":["camelCase"]}
        class Ignored { abstract snake_case() }"#,
            Some(
                serde_json::json!([{"selector":"classMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classMethod","format":["camelCase"]}
        class Ignored { #snake_case() }"#,
            Some(
                serde_json::json!([{"selector":"classMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classMethod","format":["camelCase"]}
        class Ignored { static #snake_case() }"#,
            Some(
                serde_json::json!([{"selector":"classMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"objectLiteralMethod","format":["camelCase"]}
        const ignored = { snake_case() {} };"#,
            Some(
                serde_json::json!([{"selector":"objectLiteralMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"objectLiteralMethod","format":["camelCase"]}
        const ignored = { "snake_case"() {} };"#,
            Some(
                serde_json::json!([{"selector":"objectLiteralMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"objectLiteralMethod","format":["camelCase"]}
        const ignored = { snake_case: () => {} };"#,
            Some(
                serde_json::json!([{"selector":"objectLiteralMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeMethod","format":["camelCase"]}
        interface Ignored { snake_case(): string }"#,
            Some(
                serde_json::json!([{"selector":"typeMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeMethod","format":["camelCase"]}
        interface Ignored { "snake_case"(): string }"#,
            Some(
                serde_json::json!([{"selector":"typeMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeMethod","format":["camelCase"]}
        interface Ignored { snake_case: () => string }"#,
            Some(
                serde_json::json!([{"selector":"typeMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeMethod","format":["camelCase"]}
        interface Ignored { "snake_case": () => string }"#,
            Some(
                serde_json::json!([{"selector":"typeMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeMethod","format":["camelCase"]}
        type Ignored = { snake_case(): string }"#,
            Some(
                serde_json::json!([{"selector":"typeMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeMethod","format":["camelCase"]}
        type Ignored = { "snake_case"(): string }"#,
            Some(
                serde_json::json!([{"selector":"typeMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeMethod","format":["camelCase"]}
        type Ignored = { snake_case: () => string }"#,
            Some(
                serde_json::json!([{"selector":"typeMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeMethod","format":["camelCase"]}
        type Ignored = { "snake_case": () => string }"#,
            Some(
                serde_json::json!([{"selector":"typeMethod","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        function ignored(snake_case) {}"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        (function (snake_case) {});"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        declare function ignored(snake_case);"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        function ignored({snake_case}) {}"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        function ignored(...snake_case) {}"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        function ignored({snake_case = 1}) {}"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        function ignored({...snake_case}) {}"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        function ignored([snake_case]) {}"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        function ignored([snake_case = 1]) {}"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameter","format":["camelCase"]}
        function ignored([...snake_case]) {}"#,
            Some(
                serde_json::json!([{"selector":"parameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameterProperty","format":["camelCase"]}
        class Ignored { constructor(private snake_case) {} }"#,
            Some(
                serde_json::json!([{"selector":"parameterProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameterProperty","format":["camelCase"]}
        class Ignored { constructor(readonly snake_case) {} }"#,
            Some(
                serde_json::json!([{"selector":"parameterProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"parameterProperty","format":["camelCase"]}
        class Ignored { constructor(private readonly snake_case) {} }"#,
            Some(
                serde_json::json!([{"selector":"parameterProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"modifiers":["readonly"],"selector":"parameterProperty","format":["camelCase"]}
        class Ignored { constructor(private readonly snake_case) {} }"#,
            Some(
                serde_json::json!([{"modifiers":["readonly"],"selector":"parameterProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classProperty","format":["camelCase"]}
        class Ignored { private snake_case }"#,
            Some(
                serde_json::json!([{"selector":"classProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classProperty","format":["camelCase"]}
        class Ignored { private "snake_case" = 1 }"#,
            Some(
                serde_json::json!([{"selector":"classProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classProperty","format":["camelCase"]}
        class Ignored { private readonly snake_case = 1 }"#,
            Some(
                serde_json::json!([{"selector":"classProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classProperty","format":["camelCase"]}
        class Ignored { private static snake_case }"#,
            Some(
                serde_json::json!([{"selector":"classProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classProperty","format":["camelCase"]}
        class Ignored { private static readonly snake_case = 1 }"#,
            Some(
                serde_json::json!([{"selector":"classProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classProperty","format":["camelCase"]}
        class Ignored { abstract snake_case }"#,
            Some(
                serde_json::json!([{"selector":"classProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classProperty","format":["camelCase"]}
        class Ignored { declare snake_case }"#,
            Some(
                serde_json::json!([{"selector":"classProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classProperty","format":["camelCase"]}
        class Ignored { #snake_case }"#,
            Some(
                serde_json::json!([{"selector":"classProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"classProperty","format":["camelCase"]}
        class Ignored { static #snake_case }"#,
            Some(
                serde_json::json!([{"selector":"classProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"objectLiteralProperty","format":["camelCase"]}
        const ignored = { snake_case };"#,
            Some(
                serde_json::json!([{"selector":"objectLiteralProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"objectLiteralProperty","format":["camelCase"]}
        const ignored = { "snake_case": 1 };"#,
            Some(
                serde_json::json!([{"selector":"objectLiteralProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeProperty","format":["camelCase"]}
        interface Ignored { snake_case }"#,
            Some(
                serde_json::json!([{"selector":"typeProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeProperty","format":["camelCase"]}
        interface Ignored { "snake_case": string }"#,
            Some(
                serde_json::json!([{"selector":"typeProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeProperty","format":["camelCase"]}
        type Ignored = { snake_case }"#,
            Some(
                serde_json::json!([{"selector":"typeProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeProperty","format":["camelCase"]}
        type Ignored = { "snake_case": string }"#,
            Some(
                serde_json::json!([{"selector":"typeProperty","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeAlias","format":["camelCase"]}
        type snake_case = {};"#,
            Some(
                serde_json::json!([{"selector":"typeAlias","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeAlias","format":["camelCase"]}
        type snake_case = 1;"#,
            Some(
                serde_json::json!([{"selector":"typeAlias","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeParameter","format":["camelCase"]}
        class Ignored<snake_case> {}"#,
            Some(
                serde_json::json!([{"selector":"typeParameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeParameter","format":["camelCase"]}
        function ignored<snake_case>() {}"#,
            Some(
                serde_json::json!([{"selector":"typeParameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeParameter","format":["camelCase"]}
        type Ignored<snake_case> = { ignored: snake_case };"#,
            Some(
                serde_json::json!([{"selector":"typeParameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"typeParameter","format":["camelCase"]}
        interface Ignored<snake_case> extends Ignored<string> {}"#,
            Some(
                serde_json::json!([{"selector":"typeParameter","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"]}
        const snake_case = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"]}
        let snake_case = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"]}
        var snake_case = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"]}
        const {snake_case} = {ignored: 1};"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"]}
        const {snake_case = 2} = {ignored: 1};"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"]}
        const {...snake_case} = {ignored: 1};"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"]}
        const [snake_case] = [1];"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"]}
        const [snake_case = 1] = [1];"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"]}
        const [...snake_case] = [1];"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"forbid"}
        const _strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"forbid","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"require"}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"require","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"requireDouble"}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"requireDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"requireDouble"}
        const _strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"requireDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"forbid"}
        const strictCamelCase_ = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"forbid","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"require"}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"require","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"requireDouble"}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"requireDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"requireDouble"}
        const strictCamelCase_ = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"requireDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"prefix":["MyPrefix"]}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"prefix":["MyPrefix"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"prefix":["MyPrefix1","MyPrefix2"]}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"prefix":["MyPrefix1","MyPrefix2"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"suffix":["MySuffix"]}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"suffix":["MySuffix"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"suffix":["MySuffix1","MySuffix2"]}
        const strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"suffix":["MySuffix1","MySuffix2"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"forbid"}
        let _strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"forbid","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"require"}
        let strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"require","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"requireDouble"}
        let strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"requireDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"leadingUnderscore":"requireDouble"}
        let _strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"leadingUnderscore":"requireDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"forbid"}
        let strictCamelCase_ = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"forbid","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"require"}
        let strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"require","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"requireDouble"}
        let strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"requireDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"trailingUnderscore":"requireDouble"}
        let strictCamelCase_ = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"trailingUnderscore":"requireDouble","filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"prefix":["MyPrefix"]}
        let strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"prefix":["MyPrefix"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"prefix":["MyPrefix1","MyPrefix2"]}
        let strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"prefix":["MyPrefix1","MyPrefix2"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"suffix":["MySuffix"]}
        let strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"suffix":["MySuffix"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
        (
            r#"// {"selector":"variable","format":["camelCase"],"suffix":["MySuffix1","MySuffix2"]}
        let strictCamelCase = 1;"#,
            Some(
                serde_json::json!([{"selector":"variable","format":["camelCase"],"suffix":["MySuffix1","MySuffix2"],"filter":{"match":false,"regex":".gnored"}}]),
            ),
        ),
    ];
    // spellchecker:on

    Tester::new(NamingConvention::NAME, NamingConvention::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .test_and_snapshot();
}
