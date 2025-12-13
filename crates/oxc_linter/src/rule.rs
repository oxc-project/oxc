#[cfg(feature = "ruledocs")]
use std::borrow::Cow;
use std::{fmt, hash::Hash};

use schemars::{JsonSchema, SchemaGenerator, schema::Schema};
use serde::{Deserialize, Serialize};

use oxc_semantic::AstTypesBitset;

use crate::{
    AstNode, FixKind,
    context::{ContextHost, LintContext},
    utils::PossibleJestNode,
};

pub trait Rule: Sized + Default + fmt::Debug {
    /// Initialize from eslint json configuration
    fn from_configuration(_value: serde_json::Value) -> Self {
        Self::default()
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

    /// Run on each Jest node (e.g. `it`, `describe`, `test`, `expect`, etc.).
    /// This is only called if the Jest plugin is enabled and the file is a test file.
    /// It should be used to run rules that are specific to Jest or Vitest.
    #[expect(unused_variables)]
    #[inline]
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
    }

    /// Check if a rule should be run at all.
    ///
    /// You usually do not need to implement this function. If you do, use it to
    /// enable rules on a file-by-file basis. Do not check if plugins are
    /// enabled/disabled; this is handled by the [`linter`].
    ///
    /// [`linter`]: crate::Linter
    #[expect(unused_variables)]
    #[inline]
    fn should_run(&self, ctx: &ContextHost) -> bool {
        true
    }
}

/// A wrapper type for deserializing ESLint-style rule configurations.
///
/// ESLint configurations are typically arrays where the first element contains
/// the actual rule configuration. This type automatically extracts and deserializes
/// that first element. If the array is empty, it uses the default value.
///
/// # Examples
///
/// ```ignore
/// impl Rule for MyRule {
///     fn from_configuration(value: serde_json::Value) -> Self {
///         let config = serde_json::from_value::<DefaultRuleConfig<MyRuleConfig>>(value)
///             .unwrap_or_default();
///         Self(config.into_inner())
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct DefaultRuleConfig<T>(T);

impl<T> DefaultRuleConfig<T> {
    /// Unwraps the inner configuration value.
    pub fn into_inner(self) -> T {
        self.0
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
        use serde::de::Error;

        let value = serde_json::Value::deserialize(deserializer)?;

        if let serde_json::Value::Array(arr) = value {
            let config = arr
                .into_iter()
                .next()
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_else(T::default);
            Ok(DefaultRuleConfig(config))
        } else {
            Err(D::Error::custom("Expected array for rule configuration"))
        }
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
    /// Only `run_on_jest_node` is implemented
    RunOnJestNode,
}

impl RuleRunFunctionsImplemented {
    pub fn is_run_implemented(self) -> bool {
        matches!(self, Self::Run | Self::Unknown)
    }

    pub fn is_run_once_implemented(self) -> bool {
        matches!(self, Self::RunOnce | Self::Unknown)
    }

    pub fn is_run_on_jest_node_implemented(self) -> bool {
        matches!(self, Self::RunOnJestNode | Self::Unknown)
    }
}

pub trait RuleMeta {
    const NAME: &'static str;

    const PLUGIN: &'static str;

    const CATEGORY: RuleCategory;

    const IS_TSGOLINT_RULE: bool = false;

    /// What kind of auto-fixing can this rule do?
    const FIX: RuleFixMeta = RuleFixMeta::None;

    fn documentation() -> Option<&'static str> {
        None
    }

    #[expect(unused_variables)]
    fn config_schema(generator: &mut SchemaGenerator) -> Option<Schema> {
        None
    }
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

    #[inline]
    pub fn is_pending(self) -> bool {
        matches!(self, Self::FixPending)
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

impl From<RuleFixMeta> for FixKind {
    fn from(value: RuleFixMeta) -> Self {
        value.fix_kind()
    }
}

#[cfg(test)]
mod test {
    use crate::{RuleMeta, RuleRunner};

    use super::RuleCategory;

    #[test]
    #[cfg(feature = "ruledocs")]
    fn ensure_documentation() {
        use crate::rules::RULES;
        use markdown::{Options, to_html_with_options};

        assert!(!RULES.is_empty());
        let options = Options::gfm();

        for rule in RULES.iter() {
            let name = rule.name();
            assert!(
                rule.documentation().is_some_and(|s| !s.is_empty()),
                "Rule '{name}' is missing documentation."
            );
            // will panic if provided invalid markdown
            let html = to_html_with_options(rule.documentation().unwrap(), &options).unwrap();
            assert!(!html.is_empty());
        }
    }

    #[test]
    fn test_deserialize_rule_category() {
        let tests = [
            ("correctness", RuleCategory::Correctness),
            ("suspicious", RuleCategory::Suspicious),
            ("restriction", RuleCategory::Restriction),
            ("perf", RuleCategory::Perf),
            ("pedantic", RuleCategory::Pedantic),
            ("style", RuleCategory::Style),
            ("nursery", RuleCategory::Nursery),
        ];

        for (input, expected) in tests {
            let de: RuleCategory = serde_json::from_str(&format!("{input:?}")).unwrap();
            // deserializes to expected value
            assert_eq!(de, expected, "{input}");
            // try_from on a str produces the same value as deserializing
            assert_eq!(de, RuleCategory::try_from(input).unwrap(), "{input}");
        }
    }

    #[test]
    fn test_rule_runner_impls() {
        use crate::rules::*;
        use oxc_ast::AstType::*;

        // The RuleRunner code is automatically generated by the `oxc_linter_codegen` crate.
        // This is set of manually verified test cases to ensure that the generated code
        // is working as expected and is not skipping rules for nodes that actually should be linted.
        assert_rule_runs_on_node_types(&eslint::no_debugger::NoDebugger, &[DebuggerStatement]);
        assert_rule_runs_on_node_types(&eslint::no_with::NoWith, &[WithStatement]);
        assert_rule_runs_on_node_types(
            &eslint::arrow_body_style::ArrowBodyStyle::default(),
            &[ArrowFunctionExpression],
        );
        assert_rule_runs_on_node_types(
            &eslint::no_else_return::NoElseReturn::default(),
            &[IfStatement],
        );
        assert_rule_runs_on_node_types(
            &eslint::max_params::MaxParams::default(),
            &[Function, ArrowFunctionExpression],
        );
        assert_rule_runs_on_node_types(
            &import::no_dynamic_require::NoDynamicRequire::default(),
            &[ImportExpression, CallExpression],
        );
        assert_rule_runs_on_node_types(
            &jest::prefer_jest_mocked::PreferJestMocked,
            &[TSAsExpression, TSTypeAssertion],
        );
        assert_rule_runs_on_node_types(&jest::prefer_spy_on::PreferSpyOn, &[AssignmentExpression]);
        assert_rule_runs_on_node_types(
            &jsx_a11y::anchor_is_valid::AnchorIsValid::default(),
            &[JSXElement],
        );
        assert_rule_runs_on_node_types(
            &jsx_a11y::aria_activedescendant_has_tabindex::AriaActivedescendantHasTabindex,
            &[JSXOpeningElement],
        );
        assert_rule_runs_on_node_types(
            &nextjs::no_head_element::NoHeadElement,
            &[JSXOpeningElement],
        );
        assert_rule_runs_on_node_types(
            &nextjs::google_font_display::GoogleFontDisplay,
            &[JSXOpeningElement],
        );
        assert_rule_runs_on_node_types(
            &unicorn::consistent_assert::ConsistentAssert,
            &[ImportDeclaration],
        );
    }

    fn assert_rule_runs_on_node_types<R: RuleMeta + RuleRunner>(
        rule: &R,
        node_types: &[oxc_ast::AstType],
    ) {
        let types = rule.types_info();
        assert!(types.is_some(), "{}: NODE_TYPES is None", R::NAME);
        let types = types.unwrap();
        for node_type in node_types {
            assert!(
                types.has(*node_type),
                "{}: missing {:?} in its NODE_TYPES (this means it will incorrectly skip nodes it needs to lint)",
                R::NAME,
                node_type
            );
        }
        for node_type in types {
            assert!(
                node_types.contains(&node_type),
                "{}: has {:?} in its NODE_TYPES but it should not (this means it will lint nodes it does not need to)",
                R::NAME,
                node_type
            );
        }
    }
}
