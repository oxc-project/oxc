mod allow_warn_deny;
mod filter;
mod plugins;

use std::{convert::From, path::PathBuf};

use oxc_diagnostics::Error;
use rustc_hash::FxHashSet;

pub use allow_warn_deny::AllowWarnDeny;
pub use filter::{InvalidFilterKind, LintFilter, LintFilterKind};
pub use plugins::{LintPluginOptions, LintPlugins};

use crate::{
    config::{LintConfig, Oxlintrc},
    fixer::FixKind,
    rules::RULES,
    utils::is_jest_rule_adapted_to_vitest,
    FrameworkFlags, RuleCategory, RuleEnum, RuleWithSeverity,
};

/// Subset of options used directly by the [`Linter`]. Derived from
/// [`OxlintOptions`], which is the public-facing API. Do not expose this
/// outside of this crate.
///
/// [`Linter`]: crate::Linter
#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) struct LintOptions {
    pub fix: FixKind,
    pub framework_hints: FrameworkFlags,
    pub plugins: LintPlugins,
}

impl From<OxlintOptions> for LintOptions {
    fn from(options: OxlintOptions) -> Self {
        Self {
            fix: options.fix,
            framework_hints: options.framework_hints,
            plugins: options.plugins.into(),
        }
    }
}

#[derive(Debug)]
pub struct OxlintOptions {
    /// Allow / Deny rules in order. [("allow" / "deny", rule name)]
    /// Defaults to [("deny", "correctness")]
    pub filter: Vec<LintFilter>,
    pub config_path: Option<PathBuf>,
    /// Enable automatic code fixes. Set to [`None`] to disable.
    ///
    /// The kind represents the riskiest fix that the linter can apply.
    pub fix: FixKind,

    pub plugins: LintPluginOptions,

    pub framework_hints: FrameworkFlags,
}

impl Default for OxlintOptions {
    fn default() -> Self {
        Self {
            filter: vec![LintFilter::warn(RuleCategory::Correctness)],
            config_path: None,
            fix: FixKind::None,
            plugins: LintPluginOptions::default(),
            framework_hints: FrameworkFlags::default(),
        }
    }
}

impl OxlintOptions {
    #[must_use]
    pub fn with_filter(mut self, filter: Vec<LintFilter>) -> Self {
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

    #[must_use]
    pub fn with_node_plugin(mut self, yes: bool) -> Self {
        self.plugins.node = yes;
        self
    }

    #[must_use]
    pub fn with_security_plugin(mut self, yes: bool) -> Self {
        self.plugins.security = yes;
        self
    }
}

impl OxlintOptions {
    /// # Errors
    ///
    /// * Returns `Err` if there are any errors parsing the configuration file.
    pub(crate) fn derive_rules_and_config(
        &self,
    ) -> Result<(Vec<RuleWithSeverity>, LintConfig), Error> {
        let config = self.config_path.as_ref().map(|path| Oxlintrc::from_file(path)).transpose()?;

        let mut rules: FxHashSet<RuleWithSeverity> = FxHashSet::default();
        let all_rules = self.get_filtered_rules();

        for (severity, filter) in self.filter.iter().map(Into::into) {
            match severity {
                AllowWarnDeny::Deny | AllowWarnDeny::Warn => match filter {
                    LintFilterKind::Category(category) => {
                        rules.extend(
                            all_rules
                                .iter()
                                .filter(|rule| rule.category() == *category)
                                .map(|rule| RuleWithSeverity::new(rule.clone(), severity)),
                        );
                    }
                    LintFilterKind::Rule(_, name) => {
                        rules.extend(
                            all_rules
                                .iter()
                                .filter(|rule| rule.name() == name)
                                .map(|rule| RuleWithSeverity::new(rule.clone(), severity)),
                        );
                    }
                    LintFilterKind::Generic(name_or_category) => {
                        if name_or_category == "all" {
                            rules.extend(
                                all_rules
                                    .iter()
                                    .filter(|rule| rule.category() != RuleCategory::Nursery)
                                    .map(|rule| RuleWithSeverity::new(rule.clone(), severity)),
                            );
                        } else {
                            rules.extend(
                                all_rules
                                    .iter()
                                    .filter(|rule| rule.name() == name_or_category)
                                    .map(|rule| RuleWithSeverity::new(rule.clone(), severity)),
                            );
                        }
                    }
                },
                AllowWarnDeny::Allow => match filter {
                    LintFilterKind::Category(category) => {
                        rules.retain(|rule| rule.category() != *category);
                    }
                    LintFilterKind::Rule(_, name) => {
                        rules.retain(|rule| rule.name() != name);
                    }
                    LintFilterKind::Generic(name_or_category) => {
                        if name_or_category == "all" {
                            rules.clear();
                        } else {
                            rules.retain(|rule| rule.name() != name_or_category);
                        }
                    }
                },
            }
        }

        if let Some(config) = &config {
            config.rules.override_rules(&mut rules, &all_rules);
        }

        let mut rules = rules.into_iter().collect::<Vec<_>>();

        // for stable diagnostics output ordering
        rules.sort_unstable_by_key(|rule| rule.id());

        Ok((rules, config.map(Into::into).unwrap_or_default()))
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
                "node" => self.plugins.node,
                "security" => self.plugins.security,
                name => panic!("Unhandled plugin: {name}"),
            })
            .cloned()
            .collect::<Vec<_>>()
    }
}
