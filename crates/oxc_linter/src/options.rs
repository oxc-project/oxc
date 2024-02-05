use std::path::PathBuf;

use crate::{
    config::{
        errors::{
            FailedToParseAllowWarnDenyFromJsonValueError,
            FailedToParseAllowWarnDenyFromNumberError, FailedToParseAllowWarnDenyFromStringError,
        },
        ESLintConfig,
    },
    rules::RULES,
    ESLintEnv, ESLintSettings, RuleCategory, RuleEnum,
};
use oxc_diagnostics::Error;
use rustc_hash::FxHashSet;
use serde_json::{Number, Value};

#[derive(Debug)]
pub struct LintOptions {
    /// Allow / Deny rules in order. [("allow" / "deny", rule name)]
    /// Defaults to [("deny", "correctness")]
    pub filter: Vec<(AllowWarnDeny, String)>,
    pub config_path: Option<PathBuf>,
    pub fix: bool,
    pub timing: bool,
    pub import_plugin: bool,
    pub jest_plugin: bool,
    pub jsx_a11y_plugin: bool,
    pub nextjs_plugin: bool,
    pub react_perf_plugin: bool,
    pub env: ESLintEnv,
}

impl Default for LintOptions {
    fn default() -> Self {
        Self {
            filter: vec![(AllowWarnDeny::Deny, String::from("correctness"))],
            config_path: None,
            fix: false,
            timing: false,
            import_plugin: false,
            jest_plugin: false,
            jsx_a11y_plugin: false,
            nextjs_plugin: false,
            react_perf_plugin: false,
            env: ESLintEnv::default(),
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
    pub fn with_timing(mut self, yes: bool) -> Self {
        self.timing = yes;
        self
    }

    #[must_use]
    pub fn with_import_plugin(mut self, yes: bool) -> Self {
        self.import_plugin = yes;
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

    #[must_use]
    pub fn with_env(mut self, env: Vec<String>) -> Self {
        self.env = ESLintEnv::new(env);
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
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, <Self as TryFrom<&str>>::Error> {
        match s {
            "allow" | "off" => Ok(Self::Allow),
            "deny" | "error" => Ok(Self::Deny),
            "warn" => Ok(Self::Warn),
            _ => Err(FailedToParseAllowWarnDenyFromStringError(s.to_string()).into()),
        }
    }
}

impl TryFrom<&Value> for AllowWarnDeny {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, <Self as TryFrom<&Value>>::Error> {
        match value {
            Value::String(s) => Self::try_from(s.as_str()),
            Value::Number(n) => Self::try_from(n),
            _ => Err(FailedToParseAllowWarnDenyFromJsonValueError(value.to_string()).into()),
        }
    }
}

impl TryFrom<&Number> for AllowWarnDeny {
    type Error = Error;

    fn try_from(value: &Number) -> Result<Self, <Self as TryFrom<&Number>>::Error> {
        match value.as_i64() {
            Some(0) => Ok(Self::Allow),
            Some(1) => Ok(Self::Warn),
            Some(2) => Ok(Self::Deny),
            _ => Err(FailedToParseAllowWarnDenyFromNumberError(value.to_string()).into()),
        }
    }
}

const JEST_PLUGIN_NAME: &str = "jest";
const JSX_A11Y_PLUGIN_NAME: &str = "jsx_a11y";
const NEXTJS_PLUGIN_NAME: &str = "nextjs";
const REACT_PERF_PLUGIN_NAME: &str = "react_perf";

impl LintOptions {
    /// # Errors
    ///
    /// * Returns `Err` if there are any errors parsing the configuration file.
    pub fn derive_rules_and_settings_and_env(
        &self,
    ) -> Result<(Vec<RuleEnum>, ESLintSettings, ESLintEnv), Error> {
        let config =
            self.config_path.as_ref().map(|path| ESLintConfig::from_file(path)).transpose()?;

        let mut rules: FxHashSet<RuleEnum> = FxHashSet::default();
        let all_rules = self.get_filtered_rules();

        for (allow_warn_deny, name_or_category) in &self.filter {
            let maybe_category = RuleCategory::from(name_or_category.as_str());
            match allow_warn_deny {
                AllowWarnDeny::Deny | AllowWarnDeny::Warn => {
                    match maybe_category {
                        Some(category) => rules.extend(
                            all_rules.iter().filter(|rule| rule.category() == category).cloned(),
                        ),
                        None => {
                            if name_or_category == "all" {
                                rules.extend(all_rules.iter().cloned());
                            } else {
                                rules.extend(
                                    all_rules
                                        .iter()
                                        .filter(|rule| rule.name() == name_or_category)
                                        .cloned(),
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

        let (settings, env) = config.map(ESLintConfig::properties).unwrap_or_default();

        // for stable diagnostics output ordering
        rules.sort_unstable_by_key(RuleEnum::name);

        Ok((rules, settings, env))
    }

    // get final filtered rules by reading `self.jest_plugin` and `self.jsx_a11y_plugin`
    fn get_filtered_rules(&self) -> Vec<RuleEnum> {
        let mut rules = RULES.clone();

        let mut may_exclude_plugin_rules = |yes: bool, name: &str| {
            if !yes {
                rules.retain(|rule| rule.plugin_name() != name);
            }
        };

        may_exclude_plugin_rules(self.jest_plugin, JEST_PLUGIN_NAME);
        may_exclude_plugin_rules(self.jsx_a11y_plugin, JSX_A11Y_PLUGIN_NAME);
        may_exclude_plugin_rules(self.nextjs_plugin, NEXTJS_PLUGIN_NAME);
        may_exclude_plugin_rules(self.react_perf_plugin, REACT_PERF_PLUGIN_NAME);

        rules
    }
}
