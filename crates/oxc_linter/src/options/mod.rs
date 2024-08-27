mod allow_warn_deny;
mod plugins;

use std::{convert::From, path::PathBuf};

use oxc_diagnostics::Error;
use rustc_hash::FxHashSet;

use crate::{
    config::OxlintConfig, fixer::FixKind, rules::RULES, utils::is_jest_rule_adapted_to_vitest,
    FrameworkFlags, RuleCategory, RuleEnum, RuleWithSeverity,
};

pub use allow_warn_deny::AllowWarnDeny;
pub use plugins::LintPluginOptions;

#[derive(Debug)]
pub struct LintOptions {
    /// Allow / Deny rules in order. [("allow" / "deny", rule name)]
    /// Defaults to [("deny", "correctness")]
    pub filter: Vec<(AllowWarnDeny, String)>,
    pub config_path: Option<PathBuf>,
    /// Enable automatic code fixes. Set to [`None`] to disable.
    ///
    /// The kind represents the riskiest fix that the linter can apply.
    pub fix: FixKind,

    pub plugins: LintPluginOptions,

    pub framework_hints: FrameworkFlags,
}

impl Default for LintOptions {
    fn default() -> Self {
        Self {
            filter: vec![(AllowWarnDeny::Warn, String::from("correctness"))],
            config_path: None,
            fix: FixKind::None,
            plugins: LintPluginOptions::default(),
            framework_hints: FrameworkFlags::default(),
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

    /// Set the kind of auto fixes to apply.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_linter::{LintOptions, FixKind};
    ///
    /// // turn off all auto fixes. This is default behavior.
    /// LintOptions::default().with_fix(FixKind::None);
    /// ```
    #[must_use]
    pub fn with_fix(mut self, kind: FixKind) -> Self {
        self.fix = kind;
        self
    }

    #[must_use]
    pub fn with_react_plugin(mut self, yes: bool) -> Self {
        self.plugins.react = yes;
        self
    }

    #[must_use]
    pub fn with_unicorn_plugin(mut self, yes: bool) -> Self {
        self.plugins.unicorn = yes;
        self
    }

    #[must_use]
    pub fn with_typescript_plugin(mut self, yes: bool) -> Self {
        self.plugins.typescript = yes;
        self
    }

    #[must_use]
    pub fn with_oxc_plugin(mut self, yes: bool) -> Self {
        self.plugins.oxc = yes;
        self
    }

    #[must_use]
    pub fn with_import_plugin(mut self, yes: bool) -> Self {
        self.plugins.import = yes;
        self
    }

    #[must_use]
    pub fn with_jsdoc_plugin(mut self, yes: bool) -> Self {
        self.plugins.jsdoc = yes;
        self
    }

    #[must_use]
    pub fn with_jest_plugin(mut self, yes: bool) -> Self {
        self.plugins.jest = yes;
        self
    }

    #[must_use]
    pub fn with_vitest_plugin(mut self, yes: bool) -> Self {
        self.plugins.vitest = yes;
        self
    }

    #[must_use]
    pub fn with_jsx_a11y_plugin(mut self, yes: bool) -> Self {
        self.plugins.jsx_a11y = yes;
        self
    }

    #[must_use]
    pub fn with_nextjs_plugin(mut self, yes: bool) -> Self {
        self.plugins.nextjs = yes;
        self
    }

    #[must_use]
    pub fn with_react_perf_plugin(mut self, yes: bool) -> Self {
        self.plugins.react_perf = yes;
        self
    }

    #[must_use]
    pub fn with_promise_plugin(mut self, yes: bool) -> Self {
        self.plugins.promise = yes;
        self
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
                "react" => self.plugins.react,
                "unicorn" => self.plugins.unicorn,
                "typescript" => self.plugins.typescript,
                "import" => self.plugins.import,
                "jsdoc" => self.plugins.jsdoc,
                "jest" => {
                    if self.plugins.jest {
                        return true;
                    }
                    if self.plugins.vitest && is_jest_rule_adapted_to_vitest(rule.name()) {
                        return true;
                    }
                    false
                }
                "vitest" => self.plugins.vitest,
                "jsx_a11y" => self.plugins.jsx_a11y,
                "nextjs" => self.plugins.nextjs,
                "react_perf" => self.plugins.react_perf,
                "oxc" => self.plugins.oxc,
                "eslint" | "tree_shaking" => true,
                "promise" => self.plugins.promise,
                name => panic!("Unhandled plugin: {name}"),
            })
            .cloned()
            .collect::<Vec<_>>()
    }
}
