use std::{convert::From, path::PathBuf};

use oxc_diagnostics::{Error, OxcDiagnostic, Severity};
use rustc_hash::FxHashSet;
use serde_json::{Number, Value};

use crate::{config::OxlintConfig, rules::RULES, RuleCategory, RuleEnum, RuleWithSeverity};

#[derive(Debug)]
pub struct LintOptions {
    /// Allow / Deny rules in order. [("allow" / "deny", rule name)]
    /// Defaults to [("deny", "correctness")]
    pub filter: Vec<(AllowWarnDeny, String)>,
    pub config_path: Option<PathBuf>,
    pub fix: bool,

    pub react_plugin: bool,
    pub unicorn_plugin: bool,
    pub typescript_plugin: bool,
    pub oxc_plugin: bool,
    pub import_plugin: bool,
    pub jsdoc_plugin: bool,
    pub jest_plugin: bool,
    pub jsx_a11y_plugin: bool,
    pub nextjs_plugin: bool,
    pub react_perf_plugin: bool,
}

impl Default for LintOptions {
    fn default() -> Self {
        Self {
            filter: vec![(AllowWarnDeny::Warn, String::from("correctness"))],
            config_path: None,
            fix: false,
            react_plugin: true,
            unicorn_plugin: true,
            typescript_plugin: true,
            oxc_plugin: true,
            import_plugin: false,
            jsdoc_plugin: false,
            jest_plugin: false,
            jsx_a11y_plugin: false,
            nextjs_plugin: false,
            react_perf_plugin: false,
        }
    }
}

impl LintOptions {
    #[must_use]
    pub fn with_filter(mut self, filter: Vec<(AllowWarnDeny, String)>) -> Self {
        if !filter.is_empty() {
            self.filter = filter;
        }
        self
    }

    #[must_use]
    pub fn with_config_path(mut self, filter: Option<PathBuf>) -> Self {
        self.config_path = filter;
        self
    }

    #[must_use]
    pub fn with_fix(mut self, yes: bool) -> Self {
        self.fix = yes;
        self
    }

    #[must_use]
    pub fn with_react_plugin(mut self, yes: bool) -> Self {
        self.react_plugin = yes;
        self
    }

    #[must_use]
    pub fn with_unicorn_plugin(mut self, yes: bool) -> Self {
        self.unicorn_plugin = yes;
        self
    }

    #[must_use]
    pub fn with_typescript_plugin(mut self, yes: bool) -> Self {
        self.typescript_plugin = yes;
        self
    }

    #[must_use]
    pub fn with_oxc_plugin(mut self, yes: bool) -> Self {
        self.oxc_plugin = yes;
        self
    }

    #[must_use]
    pub fn with_import_plugin(mut self, yes: bool) -> Self {
        self.import_plugin = yes;
        self
    }

    #[must_use]
    pub fn with_jsdoc_plugin(mut self, yes: bool) -> Self {
        self.jsdoc_plugin = yes;
        self
    }

    #[must_use]
    pub fn with_jest_plugin(mut self, yes: bool) -> Self {
        self.jest_plugin = yes;
        self
    }

    #[must_use]
    pub fn with_jsx_a11y_plugin(mut self, yes: bool) -> Self {
        self.jsx_a11y_plugin = yes;
        self
    }

    #[must_use]
    pub fn with_nextjs_plugin(mut self, yes: bool) -> Self {
        self.nextjs_plugin = yes;
        self
    }

    #[must_use]
    pub fn with_react_perf_plugin(mut self, yes: bool) -> Self {
        self.react_perf_plugin = yes;
        self
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AllowWarnDeny {
    Allow, // Off
    Warn,  // Warn
    Deny,  // Error
}

impl AllowWarnDeny {
    pub fn is_warn_deny(self) -> bool {
        self != Self::Allow
    }

    pub fn is_allow(self) -> bool {
        self == Self::Allow
    }
}

impl TryFrom<&str> for AllowWarnDeny {
    type Error = OxcDiagnostic;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "allow" | "off" => Ok(Self::Allow),
            "deny" | "error" => Ok(Self::Deny),
            "warn" => Ok(Self::Warn),
            _ => Err(OxcDiagnostic::error(format!(
                r#"Failed to parse rule severity, expected one of "allow", "off", "deny", "error" or "warn", but got {s:?}"#
            ))),
        }
    }
}

impl TryFrom<&Value> for AllowWarnDeny {
    type Error = OxcDiagnostic;

    fn try_from(value: &Value) -> Result<Self, OxcDiagnostic> {
        match value {
            Value::String(s) => Self::try_from(s.as_str()),
            Value::Number(n) => Self::try_from(n),
            _ => Err(OxcDiagnostic::error(format!(
                "Failed to parse rule severity, expected a string or a number, but got {value:?}"
            ))),
        }
    }
}

impl TryFrom<&Number> for AllowWarnDeny {
    type Error = OxcDiagnostic;

    fn try_from(value: &Number) -> Result<Self, Self::Error> {
        match value.as_i64() {
            Some(0) => Ok(Self::Allow),
            Some(1) => Ok(Self::Warn),
            Some(2) => Ok(Self::Deny),
            _ => Err(OxcDiagnostic::error(format!(
                r#"Failed to parse rule severity, expected one of `0`, `1` or `2`, but got {value:?}"#
            ))),
        }
    }
}

impl From<AllowWarnDeny> for Severity {
    fn from(value: AllowWarnDeny) -> Self {
        match value {
            AllowWarnDeny::Allow => Self::Advice,
            AllowWarnDeny::Warn => Self::Warning,
            AllowWarnDeny::Deny => Self::Error,
        }
    }
}

impl LintOptions {
    /// # Errors
    ///
    /// * Returns `Err` if there are any errors parsing the configuration file.
    pub fn derive_rules_and_config(&self) -> Result<(Vec<RuleWithSeverity>, OxlintConfig), Error> {
        let config =
            self.config_path.as_ref().map(|path| OxlintConfig::from_file(path)).transpose()?;

        let mut rules: FxHashSet<RuleWithSeverity> = FxHashSet::default();
        let all_rules = self.get_filtered_rules();

        for (severity, name_or_category) in &self.filter {
            let maybe_category = RuleCategory::from(name_or_category.as_str());
            match severity {
                AllowWarnDeny::Deny | AllowWarnDeny::Warn => {
                    match maybe_category {
                        Some(category) => rules.extend(
                            all_rules
                                .iter()
                                .filter(|rule| rule.category() == category)
                                .map(|rule| RuleWithSeverity::new(rule.clone(), *severity)),
                        ),
                        None => {
                            if name_or_category == "all" {
                                rules.extend(
                                    all_rules
                                        .iter()
                                        .filter(|rule| rule.category() != RuleCategory::Nursery)
                                        .map(|rule| RuleWithSeverity::new(rule.clone(), *severity)),
                                );
                            } else {
                                rules.extend(
                                    all_rules
                                        .iter()
                                        .filter(|rule| rule.name() == name_or_category)
                                        .map(|rule| RuleWithSeverity::new(rule.clone(), *severity)),
                                );
                            }
                        }
                    };
                }
                AllowWarnDeny::Allow => {
                    match maybe_category {
                        Some(category) => rules.retain(|rule| rule.category() != category),
                        None => {
                            if name_or_category == "all" {
                                rules.clear();
                            } else {
                                rules.retain(|rule| rule.name() != name_or_category);
                            }
                        }
                    };
                }
            }
        }

        if let Some(config) = &config {
            config.override_rules(&mut rules, &all_rules);
        }

        let mut rules = rules.into_iter().collect::<Vec<_>>();

        // for stable diagnostics output ordering
        rules.sort_unstable_by_key(|rule| rule.id());

        Ok((rules, config.unwrap_or_default()))
    }

    /// Get final filtered rules by reading `self.xxx_plugin`
    fn get_filtered_rules(&self) -> Vec<RuleEnum> {
        RULES
            .iter()
            .filter(|rule| match rule.plugin_name() {
                "react" => self.react_plugin,
                "unicorn" => self.unicorn_plugin,
                "typescript" => self.typescript_plugin,
                "import" => self.import_plugin,
                "jsdoc" => self.jsdoc_plugin,
                "jest" => self.jest_plugin,
                "jsx_a11y" => self.jsx_a11y_plugin,
                "nextjs" => self.nextjs_plugin,
                "react_perf" => self.react_perf_plugin,
                "oxc" => self.oxc_plugin,
                "eslint" | "tree_shaking" => true,
                name => panic!("Unhandled plugin: {name}"),
            })
            .cloned()
            .collect::<Vec<_>>()
    }
}
