use std::{
    borrow::{Borrow, Cow},
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
};

use oxc_semantic::SymbolId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::{ContextHost, LintContext},
    utils::PossibleJestNode,
    AllowWarnDeny, AstNode, FixKind, RuleEnum,
};

pub trait Rule: Sized + Default + fmt::Debug {
    /// Initialize from eslint json configuration
    fn from_configuration(_value: serde_json::Value) -> Self {
        Self::default()
    }

    /// Visit each AST Node
    #[expect(unused_variables)]
    #[inline]
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}

    /// Visit each symbol
    #[expect(unused_variables)]
    #[inline]
    fn run_on_symbol(&self, symbol_id: SymbolId, ctx: &LintContext<'_>) {}

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

pub trait RuleMeta {
    const NAME: &'static str;

    const CATEGORY: RuleCategory;

    /// What kind of auto-fixing can this rule do?
    const FIX: RuleFixMeta = RuleFixMeta::None;

    fn documentation() -> Option<&'static str> {
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
        match self {
            Self::Correctness => write!(f, "Correctness"),
            Self::Suspicious => write!(f, "Suspicious"),
            Self::Pedantic => write!(f, "Pedantic"),
            Self::Perf => write!(f, "Perf"),
            Self::Style => write!(f, "Style"),
            Self::Restriction => write!(f, "Restriction"),
            Self::Nursery => write!(f, "Nursery"),
        }
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

    pub fn description(self) -> Cow<'static, str> {
        match self {
            Self::None => Cow::Borrowed("No auto-fix is available for this rule."),
            Self::FixPending => Cow::Borrowed("An auto-fix is still under development."),
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

#[derive(Debug, Clone)]
pub struct RuleWithSeverity {
    pub rule: RuleEnum,
    pub severity: AllowWarnDeny,
}

impl Hash for RuleWithSeverity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rule.hash(state);
    }
}

impl PartialEq for RuleWithSeverity {
    fn eq(&self, other: &Self) -> bool {
        self.rule == other.rule
    }
}

impl Eq for RuleWithSeverity {}

impl Deref for RuleWithSeverity {
    type Target = RuleEnum;

    fn deref(&self) -> &Self::Target {
        &self.rule
    }
}

impl Borrow<RuleEnum> for RuleWithSeverity {
    fn borrow(&self) -> &RuleEnum {
        &self.rule
    }
}

impl RuleWithSeverity {
    pub fn new(rule: RuleEnum, severity: AllowWarnDeny) -> Self {
        Self { rule, severity }
    }
}

#[cfg(test)]
mod test {
    use markdown::{to_html_with_options, Options};

    use super::RuleCategory;
    use crate::rules::RULES;

    #[test]
    fn ensure_documentation() {
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
}
