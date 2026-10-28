#[cfg(feature = "ruledocs")]
use std::borrow::Cow;
use std::{fmt, hash::Hash};

use bitflags::bitflags;
use schemars::{
    JsonSchema, SchemaGenerator,
    schema::{InstanceType, Schema, SchemaObject},
};
use serde::{Deserialize, Serialize};

use oxc_semantic::AstTypesBitset;

use crate::{
    AstNode, FixKind,
    context::{ContextHost, LintContext},
};

pub trait Rule: Sized + Default + fmt::Debug {
    /// Initialize from eslint json configuration
    fn from_configuration(_value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(Self::default())
    }

    /// Serialize rule configuration to JSON. Only used for sending rule configurations
    /// to another linter. This allows oxlint to handle the parsing and error handling.
    /// Type-aware rules implemented in tsgolint will need to override this method.
    ///
    /// - Returns `None` if no configuration should be serialized (default)
    /// - Returns `Some(Err(_))` if serialization fails
    /// - Returns `Some(Ok(_))` if serialization succeeds
    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        None
    }

    #[expect(unused_variables)]
    #[cfg(feature = "ruledocs")]
    fn schema(generator: &mut SchemaGenerator) -> Option<Schema> {
        None
    }

    /// Visit each AST Node
    #[expect(unused_variables)]
    #[inline]
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}

    /// Run only once. Useful for inspecting scopes and trivias etc.
    #[expect(unused_variables)]
    #[inline]
    fn run_once(&self, ctx: &LintContext) {}

    /// Check if a rule should be run at all.
    ///
    /// You usually do not need to implement this function. If you do, use it to
    /// enable rules on a file-by-file basis. Do not check if plugins are
    /// enabled/disabled; this is handled by the linter.
    #[expect(unused_variables)]
    #[inline]
    fn should_run(&self, ctx: &ContextHost) -> bool {
        true
    }
}

/// Pretty-print a JSON value and collapse all whitespace runs to single spaces,
/// for embedding a "received config" snippet in a rule-config deserialization error.
///
/// Kept non-generic (and out of the generic `Deserialize` impls below) so the heavy
/// `serde_json::to_string_pretty` machinery is compiled once instead of being
/// monomorphized into every rule-config `deserialize` instantiation. Returns `None`
/// when serialization fails, so callers fall back to the bare error.
fn compact_json_for_error(value: &serde_json::Value) -> Option<String> {
    serde_json::to_string_pretty(value)
        .ok()
        .map(|value_str| value_str.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// Extract the first item from an ESLint-style rule configuration.
///
/// `None` represents a missing configuration (`null` or an empty array). Keeping this helper
/// non-generic avoids duplicating the array and error handling for every rule configuration type.
#[inline(never)]
fn normalize_default_rule_config(
    value: serde_json::Value,
) -> Result<Option<serde_json::Value>, serde_json::Error> {
    match value {
        serde_json::Value::Array(arr) => Ok(arr.into_iter().next()),
        serde_json::Value::Null => Ok(None),
        _ => Err(<serde_json::Error as serde::de::Error>::custom(
            "Expected array for rule configuration",
        )),
    }
}

/// Add the received configuration to a deserialization error.
#[cold]
#[inline(never)]
fn default_rule_config_error(
    error: serde_json::Error,
    value: &serde_json::Value,
) -> serde_json::Error {
    match compact_json_for_error(value) {
        Some(compact) => <serde_json::Error as serde::de::Error>::custom(format!(
            "{error}\n  received config: `{compact}`"
        )),
        None => error,
    }
}

/// A wrapper type for deserializing ESLint-style rule configurations.
///
/// ESLint configurations are typically arrays where the first element contains
/// the actual rule configuration. This type automatically extracts and deserializes
/// that first element. If the array is empty, it uses the default value.
///
/// If the configuration is invalid (e.g. does not match the expected type, has a
/// field that does not exist), it will return a deserialization error that is
/// handled by the linter and reported to the user.
///
/// # Examples
///
/// ```ignore
/// impl Rule for MyRule {
///     fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
///         DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
///     }
/// }
/// ```
///
/// For rules that take a tuple configuration object, e.g. `["foobar", { "param": true, "other_param": false }]`,
/// use [`TupleRuleConfig`] instead.
#[derive(Debug, Clone)]
pub struct DefaultRuleConfig<T>(T);

impl<T> DefaultRuleConfig<T> {
    /// Unwraps the inner configuration value.
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Deserialize an ESLint-style rule configuration from a JSON value.
    pub fn from_value(value: serde_json::Value) -> Result<Self, serde_json::Error>
    where
        T: serde::de::DeserializeOwned + Default,
    {
        let config = match normalize_default_rule_config(value)? {
            Some(value) => {
                T::deserialize(&value).map_err(|error| default_rule_config_error(error, &value))?
            }
            None => T::default(),
        };

        Ok(Self(config))
    }
}

impl<T: Default> Default for DefaultRuleConfig<T> {
    fn default() -> Self {
        Self(T::default())
    }
}

impl<'de, T> serde::Deserialize<'de> for DefaultRuleConfig<T>
where
    T: serde::de::DeserializeOwned + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        Self::from_value(value).map_err(serde::de::Error::custom)
    }
}

/// A wrapper type for deserializing ESLint-style tuple rule configurations.
///
/// Some ESLint rules take configurations in tuple form, e.g. `["foo", { "bar": "baz" }]`
/// where different array elements correspond to different config fields. This type
/// deserializes the entire array as `T`, which should be a tuple struct with
/// `#[serde(default)]` to handle partial configurations.
///
/// If the configuration is invalid, it will return a deserialization error that is
/// handled by the linter and reported to the user.
///
/// # Examples
///
/// ```ignore
/// #[derive(Debug, Default, Clone, Deserialize)]
/// #[serde(default)]
/// pub struct MyTupleRule(EnumOption, ObjectConfig);
///
/// impl Rule for MyTupleRule {
///     fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
///         serde_json::from_value::<TupleRuleConfig<Self>>(value).map(TupleRuleConfig::into_inner)
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TupleRuleConfig<T>(T);

impl<T> TupleRuleConfig<T> {
    /// Unwraps the inner configuration value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Default> Default for TupleRuleConfig<T> {
    fn default() -> Self {
        Self(T::default())
    }
}

impl<'de, T> serde::Deserialize<'de> for TupleRuleConfig<T>
where
    T: serde::de::DeserializeOwned + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let value = serde_json::Value::deserialize(deserializer)?;

        if let serde_json::Value::Array(arr) = value {
            let config = if arr.is_empty() {
                T::default()
            } else {
                // Parse the entire array as the tuple configuration.
                let arr_value = serde_json::Value::Array(arr);
                T::deserialize(&arr_value).map_err(|e| {
                    match compact_json_for_error(&arr_value) {
                        Some(compact) => D::Error::custom(format!("{e}, received `{compact}`")),
                        None => D::Error::custom(e),
                    }
                })?
            };

            Ok(TupleRuleConfig(config))
        } else if value == serde_json::Value::Null {
            // Missing configuration (null) is treated as default (no rule options provided)
            Ok(TupleRuleConfig(T::default()))
        } else {
            Err(D::Error::custom("Expected array for rule configuration"))
        }
    }
}

/// A wrapper type for deserializing ESLint-style mixed tuple rule configurations, where the first configuration object can be skipped.
///
/// Some ESLint rules take configurations in tuple form, e.g. `["foo", "qux", { "bar": "baz" }]`
/// where the first argument is optional and the second argument is the actual configuration object.
/// This type deserializes the array as a tuple `(Option<T1>, Option<T2>) | (Option<T2>)`.
#[derive(Debug, Clone)]
pub struct MixedTupleRuleConfig<T1, T2>(pub T1, pub T2);

impl<T1: Default, T2: Default> Default for MixedTupleRuleConfig<T1, T2> {
    fn default() -> Self {
        Self(T1::default(), T2::default())
    }
}

impl<'de, T1, T2> serde::Deserialize<'de> for MixedTupleRuleConfig<T1, T2>
where
    T1: serde::de::DeserializeOwned + Default,
    T2: serde::de::DeserializeOwned + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let value = serde_json::Value::deserialize(deserializer)?;

        if let serde_json::Value::Array(arr) = value {
            let config = match arr.len() {
                0 => Self(T1::default(), T2::default()),
                1 => {
                    // If there's only one element, we need to determine whether it's T1 or T2.
                    // We can try deserializing it as T2 first, and if that fails, try T1.
                    let first_value = &arr[0];
                    if let Ok(t2) = T2::deserialize(first_value) {
                        Self(T1::default(), t2)
                    } else if let Ok(t1) = T1::deserialize(first_value) {
                        Self(t1, T2::default())
                    } else {
                        return Err(D::Error::custom(
                            "Expected configuration to match either the first or second tuple type",
                        ));
                    }
                }
                2 => {
                    let t1 = T1::deserialize(&arr[0]).map_err(|e| {
                        D::Error::custom(format!("Error deserializing first tuple element: {e}"))
                    })?;
                    let t2 = T2::deserialize(&arr[1]).map_err(|e| {
                        D::Error::custom(format!("Error deserializing second tuple element: {e}"))
                    })?;
                    Self(t1, t2)
                }
                _ => {
                    return Err(D::Error::custom(
                        "Expected at most 2 elements in the configuration array",
                    ));
                }
            };

            Ok(config)
        } else if value == serde_json::Value::Null {
            // Missing configuration (null) is treated as default (no rule options provided)
            Ok(Self(T1::default(), T2::default()))
        } else {
            Err(D::Error::custom("Expected array for rule configuration"))
        }
    }
}

impl<T1, T2> JsonSchema for MixedTupleRuleConfig<T1, T2>
where
    T1: JsonSchema,
    T2: JsonSchema,
{
    fn schema_name() -> String {
        "MixedTupleRuleConfig".to_string()
    }

    fn is_referenceable() -> bool {
        false
    }

    fn json_schema(r#gen: &mut SchemaGenerator) -> Schema {
        Schema::Object(SchemaObject {
            subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
                any_of: Some(vec![
                    SchemaObject {
                        instance_type: Some(InstanceType::Array.into()),
                        array: Some(Box::new(schemars::schema::ArrayValidation {
                            items: Some(schemars::schema::SingleOrVec::Vec(vec![
                                r#gen.subschema_for::<T1>(),
                                r#gen.subschema_for::<T2>(),
                            ])),
                            min_items: Some(2),
                            max_items: Some(2),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }
                    .into(),
                    r#gen.subschema_for::<T1>(),
                    r#gen.subschema_for::<T2>(),
                ]),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}

pub trait RuleRunner: Rule {
    /// `AstType`s that this rule acts on, or `None` if the codegen
    /// can't figure it out and the linter should call `run` on every node.
    const NODE_TYPES: Option<&AstTypesBitset>;

    /// What `Rule` functions are implemented by this `Rule`. For example, if a rule only
    /// implements `run_once`, then the linter can skip calling `run`, so
    /// this value would be tagged as [`RuleRunFunctionsImplemented::RunOnce`].
    const RUN_FUNCTIONS: RuleRunFunctionsImplemented = RuleRunFunctionsImplemented::Unknown;

    fn types_info(&self) -> Option<&'static AstTypesBitset> {
        Self::NODE_TYPES
    }

    fn run_info(&self) -> RuleRunFunctionsImplemented {
        Self::RUN_FUNCTIONS
    }
}

/// Enum approximating a bitset of which `Rule` functions are implemented.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RuleRunFunctionsImplemented {
    /// Unknown which functions are implemented.
    Unknown,
    /// Only `run` is implemented
    Run,
    /// Only `run_once` is implemented
    RunOnce,
}

impl RuleRunFunctionsImplemented {
    pub fn is_run_implemented(self) -> bool {
        matches!(self, Self::Run | Self::Unknown)
    }

    pub fn is_run_once_implemented(self) -> bool {
        matches!(self, Self::RunOnce | Self::Unknown)
    }
}

pub trait RuleMeta {
    const NAME: &'static str;

    const PLUGIN: &'static str;

    const CATEGORY: RuleCategory;

    /// Flags describing this rule's capabilities.
    ///
    /// Set by `declare_oxc_lint!` based on the rule's declaration.
    const FLAGS: RuleFlags = RuleFlags::empty();

    /// What kind of auto-fixing can this rule do?
    const FIX: RuleFixMeta = RuleFixMeta::None;

    fn documentation() -> Option<&'static str> {
        None
    }

    #[expect(unused_variables)]
    fn config_schema(generator: &mut SchemaGenerator) -> Option<Schema> {
        None
    }

    /// The version of oxlint in which this rule was first available.
    ///
    /// Set via `version = "x.y.z"` or `version = "next"` in `declare_oxc_lint!`.
    const VERSION: &'static str;

    /// Additional information about the rule.
    ///
    /// Set via `short_description = "..."` in `declare_oxc_lint!`.
    const INFO: RuleInfo = RuleInfo { short_description: "" };
}

bitflags! {
    /// Flags describing a lint rule's capabilities.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RuleFlags: u8 {
        /// Whether this rule is exposed to the tsgolint integration.
        const IS_TSGOLINT_RULE = 1 << 0;
        /// Whether this rule declares a configuration type in `declare_oxc_lint!`.
        const HAS_CONFIG = 1 << 1;
        /// Whether this rule is recommended.
        const IS_RECOMMENDED = 1 << 2;
    }
}

/// Additional information describing a lint rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuleInfo {
    /// A short, one-line summary of what the rule does.
    pub short_description: &'static str,
}

/// Rule categories defined by rust-clippy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RuleCategory {
    /// Code that is outright wrong or useless
    Correctness,
    /// Code that is most likely wrong or useless
    Suspicious,
    /// Lints which are rather strict or have occasional false positives
    Pedantic,
    /// Code that can be written to run faster
    Perf,
    /// Code that should be written in a more idiomatic way
    Style,
    /// Lints which prevent the use of language and library features
    /// The restriction category should, emphatically, not be enabled as a whole.
    /// The contained lints may lint against perfectly reasonable code, may not have an alternative suggestion,
    /// and may contradict any other lints (including other categories).
    /// Lints should be considered on a case-by-case basis before enabling.
    Restriction,
    /// New lints that are still under development
    Nursery,
}

impl RuleCategory {
    pub fn description(self) -> &'static str {
        match self {
            Self::Correctness => "Code that is outright wrong or useless.",
            Self::Suspicious => "code that is most likely wrong or useless.",
            Self::Pedantic => "Lints which are rather strict or have occasional false positives.",
            Self::Perf => "Code that can be written to run faster.",
            Self::Style => "Code that should be written in a more idiomatic way.",
            Self::Restriction => {
                "Lints which prevent the use of language and library features. Must not be enabled as a whole, should be considered on a case-by-case basis before enabling."
            }
            Self::Nursery => "New lints that are still under development.",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Correctness => "correctness",
            Self::Suspicious => "suspicious",
            Self::Pedantic => "pedantic",
            Self::Perf => "perf",
            Self::Style => "style",
            Self::Restriction => "restriction",
            Self::Nursery => "nursery",
        }
    }
}

impl TryFrom<&str> for RuleCategory {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "correctness" => Ok(Self::Correctness),
            "suspicious" => Ok(Self::Suspicious),
            "pedantic" => Ok(Self::Pedantic),
            "perf" => Ok(Self::Perf),
            "style" => Ok(Self::Style),
            "restriction" => Ok(Self::Restriction),
            "nursery" => Ok(Self::Nursery),
            _ => Err(()),
        }
    }
}

impl fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let category_name = match self {
            Self::Correctness => "Correctness",
            Self::Suspicious => "Suspicious",
            Self::Pedantic => "Pedantic",
            Self::Perf => "Perf",
            Self::Style => "Style",
            Self::Restriction => "Restriction",
            Self::Nursery => "Nursery",
        };
        f.write_str(category_name)
    }
}

// NOTE: this could be packed into a single byte if we wanted. I don't think
// this is needed, but we could do it if it would have a performance impact.
/// Describes the auto-fixing capabilities of a `Rule`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleFixMeta {
    /// An auto-fix is not available.
    #[default]
    None,
    /// An auto-fix could be implemented, but it has not been yet.
    FixPending,
    /// An auto-fix is available for some violations, but not all.
    Conditional(FixKind),
    /// An auto-fix is available.
    Fixable(FixKind),
}

impl RuleFixMeta {
    #[inline]
    pub fn is_none(self) -> bool {
        matches!(self, Self::None)
    }

    #[inline]
    pub const fn fix_kind(self) -> FixKind {
        match self {
            Self::Conditional(kind) | Self::Fixable(kind) => {
                debug_assert!(
                    !kind.is_none(),
                    "This lint rule indicates that it provides an auto-fix but its FixKind is None. This is a bug. If this rule does not provide a fix, please use RuleFixMeta::None. Otherwise, please provide a valid FixKind"
                );
                kind
            }
            RuleFixMeta::None | RuleFixMeta::FixPending => FixKind::None,
        }
    }

    /// Does this `Rule` have some kind of auto-fix available?
    ///
    /// Also returns `true` for suggestions.
    #[inline]
    pub fn has_fix(self) -> bool {
        matches!(self, Self::Fixable(_) | Self::Conditional(_))
    }

    pub fn supports_fix(self, kind: FixKind) -> bool {
        matches!(self, Self::Fixable(fix_kind) | Self::Conditional(fix_kind) if fix_kind.can_apply(kind))
    }

    #[cfg(feature = "ruledocs")]
    pub fn description(self) -> Cow<'static, str> {
        match self {
            Self::None => Cow::Borrowed("No auto-fix is available for this rule."),
            Self::FixPending => Cow::Borrowed(
                "An auto-fix is planned for this rule, but not implemented at this time.",
            ),
            Self::Fixable(kind) | Self::Conditional(kind) => {
                // e.g. an auto-fix is available for this rule
                // e.g. a suggestion is available for this rule
                // e.g. a dangerous auto-fix is available for this rule
                // e.g. an auto-fix is available for this rule for some violations
                // e.g. an auto-fix and a suggestion are available for this rule
                let noun = match (kind.contains(FixKind::Fix), kind.contains(FixKind::Suggestion)) {
                    (true, true) => "auto-fix and a suggestion are available for this rule",
                    (true, false) => "auto-fix is available for this rule",
                    (false, true) => "suggestion is available for this rule",
                    _ => unreachable!(
                        "Fix kinds must contain Fix and/or Suggestion, but {self:?} has neither."
                    ),
                };
                let mut message =
                    if kind.is_dangerous() { format!("dangerous {noun}") } else { noun.into() };

                let article = match message.chars().next() {
                    Some('a' | 'e' | 'i' | 'o' | 'u') => "An",
                    Some(_) => "A",
                    None => unreachable!(),
                };

                if matches!(self, Self::Conditional(_)) {
                    message += " for some violations";
                }

                Cow::Owned(format!("{article} {message}."))
            }
        }
    }

    pub fn emoji(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Conditional(kind) | Self::Fixable(kind) => Some(kind.emoji()),
            Self::FixPending => Some("🚧"),
        }
    }
}

impl fmt::Display for RuleFixMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::FixPending => write!(f, "pending"),
            Self::Conditional(kind) => write!(f, "conditional_{}", kind.to_string()),
            Self::Fixable(kind) => write!(f, "fixable_{}", kind.to_string()),
        }
    }
}

impl From<RuleFixMeta> for FixKind {
    fn from(value: RuleFixMeta) -> Self {
        value.fix_kind()
    }
}
