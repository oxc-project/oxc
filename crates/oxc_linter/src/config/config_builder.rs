use std::{
    cell::{Ref, RefCell},
    fmt,
};

use oxc_diagnostics::OxcDiagnostic;
use oxc_span::CompactStr;
use rustc_hash::FxHashSet;

use crate::{
    config::{ConfigStore, ESLintRule, LintPlugins, OxlintOverrides, OxlintRules},
    rules::RULES,
    AllowWarnDeny, LintConfig, LintFilter, LintFilterKind, Oxlintrc, RuleCategory, RuleEnum,
    RuleWithSeverity,
};

#[must_use = "You dropped your builder without building a Linter! Did you mean to call .build()?"]
pub struct ConfigStoreBuilder {
    pub(super) rules: FxHashSet<RuleWithSeverity>,
    config: LintConfig,
    overrides: OxlintOverrides,
    cache: RulesCache,
}

impl Default for ConfigStoreBuilder {
    fn default() -> Self {
        Self { rules: Self::warn_correctness(LintPlugins::default()), ..Self::empty() }
    }
}

impl ConfigStoreBuilder {
    /// Create a [`ConfigStoreBuilder`] with default plugins enabled and no
    /// configured rules.
    ///
    /// You can think of this as `oxlint -A all`.
    pub fn empty() -> Self {
        let config = LintConfig::default();
        let rules = FxHashSet::default();
        let overrides = OxlintOverrides::default();
        let cache = RulesCache::new(config.plugins);

        Self { rules, config, overrides, cache }
    }

    /// Warn on all rules in all plugins and categories, including those in `nursery`.
    /// This is the kitchen sink.
    ///
    /// You can think of this as `oxlint -W all -W nursery`.
    pub fn all() -> Self {
        let config = LintConfig { plugins: LintPlugins::all(), ..LintConfig::default() };
        let overrides = OxlintOverrides::default();
        let cache = RulesCache::new(config.plugins);
        Self {
            rules: RULES
                .iter()
                .map(|rule| RuleWithSeverity { rule: rule.clone(), severity: AllowWarnDeny::Warn })
                .collect(),
            config,
            overrides,
            cache,
        }
    }

    /// Create a [`ConfigStoreBuilder`] from a loaded or manually built [`Oxlintrc`].
    /// `start_empty` will configure the builder to contain only the
    /// configuration settings from the config. When this is `false`, the config
    /// will be applied on top of a default [`Oxlintrc`].
    ///
    /// # Example
    /// Here's how to create a [`ConfigStore`] from a `.oxlintrc.json` file.
    /// ```
    /// use oxc_linter::{ConfigBuilder, Oxlintrc};
    /// let oxlintrc = Oxlintrc::from_file("path/to/.oxlintrc.json").unwrap();
    /// let config_store = ConfigStoreBuilder::from_oxlintrc(true, oxlintrc).build();
    /// // you can use `From` as a shorthand for `from_oxlintrc(false, oxlintrc)`
    /// let config_store = ConfigStoreBuilder::from(oxlintrc).build();
    /// ```
    ///
    /// # Errors
    ///
    /// Will return a [`ConfigBuilderError::UnknownRules`] if there are unknown rules in the
    /// config. This can happen if the plugin for a rule is not enabled, or the rule name doesn't
    /// match any recognized rules.
    pub fn from_oxlintrc(start_empty: bool, oxlintrc: Oxlintrc) -> Self {
        // TODO: monorepo config merging, plugin-based extends, etc.
        let Oxlintrc {
            plugins,
            settings,
            env,
            globals,
            categories,
            rules: oxlintrc_rules,
            overrides,
            path,
            ignore_patterns: _,
        } = oxlintrc;

        let config = LintConfig { plugins, settings, env, globals, path: Some(path) };
        let rules =
            if start_empty { FxHashSet::default() } else { Self::warn_correctness(plugins) };
        let cache = RulesCache::new(config.plugins);
        let mut builder = Self { rules, config, overrides, cache };

        if !categories.is_empty() {
            builder = builder.with_filters(categories.filters());
        }

        {
            let all_rules = builder.cache.borrow();
            oxlintrc_rules.override_rules(&mut builder.rules, all_rules.as_slice());
        }

        builder
    }

    /// Configure what linter plugins are enabled.
    ///
    /// Turning on a plugin will not automatically enable any of its rules. You must do this
    /// yourself (using [`with_filters`]) after turning the plugin on. Note that turning off a
    /// plugin that was already on will cause all rules in that plugin to be turned off. Any
    /// configuration you passed to those rules will be lost. You'll need to re-add it if/when you
    /// turn that rule back on.
    ///
    /// This method sets what plugins are enabled and disabled, overwriting whatever existing
    /// config is set. If you are looking to add/remove plugins, use [`and_plugins`]
    ///
    /// [`with_filters`]: ConfigStoreBuilder::with_filters
    /// [`and_plugins`]: ConfigStoreBuilder::and_plugins
    #[inline]
    pub fn with_plugins(mut self, plugins: LintPlugins) -> Self {
        self.config.plugins = plugins;
        self.cache.set_plugins(plugins);
        self
    }

    /// Enable or disable a set of plugins, leaving unrelated plugins alone.
    ///
    /// See [`ConfigStoreBuilder::with_plugins`] for details on how plugin configuration affects your
    /// rules.
    #[inline]
    pub fn and_plugins(mut self, plugins: LintPlugins, enabled: bool) -> Self {
        self.config.plugins.set(plugins, enabled);
        self.cache.set_plugins(self.config.plugins);
        self
    }

    #[inline]
    pub fn plugins(&self) -> LintPlugins {
        self.config.plugins
    }

    #[cfg(test)]
    pub(crate) fn with_rule(mut self, rule: RuleWithSeverity) -> Self {
        self.rules.insert(rule);
        self
    }

    pub fn with_filters<I: IntoIterator<Item = LintFilter>>(mut self, filters: I) -> Self {
        for filter in filters {
            self = self.with_filter(filter);
        }
        self
    }

    pub fn with_filter(mut self, filter: LintFilter) -> Self {
        let (severity, filter) = filter.into();

        match severity {
            AllowWarnDeny::Deny | AllowWarnDeny::Warn => match filter {
                LintFilterKind::Category(category) => {
                    self.upsert_where(severity, |r| r.category() == category);
                }
                LintFilterKind::Rule(_, name) => self.upsert_where(severity, |r| r.name() == name),
                LintFilterKind::Generic(name_or_category) => {
                    if name_or_category == "all" {
                        self.upsert_where(severity, |r| r.category() != RuleCategory::Nursery);
                    } else {
                        self.upsert_where(severity, |r| r.name() == name_or_category);
                    }
                }
            },
            AllowWarnDeny::Allow => match filter {
                LintFilterKind::Category(category) => {
                    self.rules.retain(|rule| rule.category() != category);
                }
                LintFilterKind::Rule(_, name) => {
                    self.rules.retain(|rule| rule.name() != name);
                }
                LintFilterKind::Generic(name_or_category) => {
                    if name_or_category == "all" {
                        self.rules.clear();
                    } else {
                        self.rules.retain(|rule| rule.name() != name_or_category);
                    }
                }
            },
        }

        self
    }

    /// Warn/Deny a let of rules based on some predicate. Rules already in `self.rules` get
    /// re-configured, while those that are not are added. Affects rules where `query` returns
    /// `true`.
    fn upsert_where<F>(&mut self, severity: AllowWarnDeny, query: F)
    where
        F: Fn(&&RuleEnum) -> bool,
    {
        let all_rules = self.cache.borrow();
        // NOTE: we may want to warn users if they're configuring a rule that does not exist.
        let rules_to_configure = all_rules.iter().filter(query);
        for rule in rules_to_configure {
            if let Some(mut existing_rule) = self.rules.take(rule) {
                existing_rule.severity = severity;
                self.rules.insert(existing_rule);
            } else {
                self.rules.insert(RuleWithSeverity::new(rule.clone(), severity));
            }
        }
    }

    /// # Errors
    pub fn build(self) -> Result<ConfigStore, OxcDiagnostic> {
        // When a plugin gets disabled before build(), rules for that plugin aren't removed until
        // with_filters() gets called. If the user never calls it, those now-undesired rules need
        // to be taken out.
        let plugins = self.plugins();
        let mut rules = if self.cache.is_stale() {
            self.rules.into_iter().filter(|r| plugins.contains(r.plugin_name().into())).collect()
        } else {
            self.rules.into_iter().collect::<Vec<_>>()
        };
        rules.sort_unstable_by_key(|r| r.id());
        Ok(ConfigStore::new(rules, self.config, self.overrides))
    }

    /// Warn for all correctness rules in the given set of plugins.
    fn warn_correctness(plugins: LintPlugins) -> FxHashSet<RuleWithSeverity> {
        RULES
            .iter()
            .filter(|rule| {
                // NOTE: this logic means there's no way to disable ESLint
                // correctness rules. I think that's fine for now.
                rule.category() == RuleCategory::Correctness
                    && plugins.contains(LintPlugins::from(rule.plugin_name()))
            })
            .map(|rule| RuleWithSeverity { rule: rule.clone(), severity: AllowWarnDeny::Warn })
            .collect()
    }

    /// # Panics
    /// This function will panic if the `oxlintrc` is not valid JSON.
    pub fn resolve_final_config_file(&self, oxlintrc: Oxlintrc) -> String {
        let mut oxlintrc = oxlintrc;
        let previous_rules = std::mem::take(&mut oxlintrc.rules);

        let rule_name_to_rule = previous_rules
            .rules
            .into_iter()
            .map(|r| (get_name(&r.plugin_name, &r.rule_name), r))
            .collect::<rustc_hash::FxHashMap<_, _>>();

        let new_rules = self
            .rules
            .iter()
            .map(|r: &RuleWithSeverity| ESLintRule {
                plugin_name: r.plugin_name().to_string(),
                rule_name: r.rule.name().to_string(),
                severity: r.severity,
                config: rule_name_to_rule
                    .get(&get_name(r.plugin_name(), r.rule.name()))
                    .and_then(|r| r.config.clone()),
            })
            .collect();

        oxlintrc.rules = OxlintRules::new(new_rules);
        serde_json::to_string_pretty(&oxlintrc).unwrap()
    }
}

fn get_name(plugin_name: &str, rule_name: &str) -> CompactStr {
    if plugin_name == "eslint" {
        CompactStr::from(rule_name)
    } else {
        CompactStr::from(format!("{plugin_name}/{rule_name}"))
    }
}

impl TryFrom<Oxlintrc> for ConfigStoreBuilder {
    type Error = ConfigBuilderError;

    #[inline]
    fn try_from(oxlintrc: Oxlintrc) -> Result<Self, Self::Error> {
        Ok(Self::from_oxlintrc(false, oxlintrc))
    }
}

impl fmt::Debug for ConfigStoreBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigStoreBuilder")
            .field("rules", &self.rules)
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

/// An error that can occur while building a [`ConfigStore`] from an [`Oxlintrc`].
#[derive(Debug, Clone)]
pub enum ConfigBuilderError {
    /// There were unknown rules that could not be matched to any known plugins/rules.
    UnknownRules { rules: Vec<ESLintRule> },
}

impl std::fmt::Display for ConfigBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigBuilderError::UnknownRules { rules } => {
                write!(f, "unknown rules: ")?;
                for rule in rules {
                    write!(f, "{}", rule.full_name())?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ConfigBuilderError {}

struct RulesCache {
    all_rules: RefCell<Option<Vec<RuleEnum>>>,
    plugins: LintPlugins,
    last_fresh_plugins: LintPlugins,
}

impl RulesCache {
    #[inline]
    #[must_use]
    pub fn new(plugins: LintPlugins) -> Self {
        Self { all_rules: RefCell::new(None), plugins, last_fresh_plugins: plugins }
    }

    pub fn set_plugins(&mut self, plugins: LintPlugins) {
        if self.plugins == plugins {
            return;
        }
        self.last_fresh_plugins = self.plugins;
        self.plugins = plugins;
        self.clear();
    }

    pub fn is_stale(&self) -> bool {
        // NOTE: After all_rules cache has been initialized _at least once_ (e.g. its borrowed, or
        // initialize() is called), all_rules will be some if and only if last_fresh_plugins ==
        // plugins. Right before creation, (::new()) and before initialize() is called, these two
        // fields will be equal _but all_rules will be none_. This is OK for this function, but is
        // a possible future foot-gun. ConfigBuilder uses this to re-build its rules list in
        // ::build(). If cache is created but never made stale (by changing plugins),
        // ConfigBuilder's rule list won't need updating anyways, meaning its sound for this to
        // return `false`.
        self.last_fresh_plugins != self.plugins
    }

    #[must_use]
    fn borrow(&self) -> Ref<'_, Vec<RuleEnum>> {
        let cached = self.all_rules.borrow();
        if cached.is_some() {
            Ref::map(cached, |cached| cached.as_ref().unwrap())
        } else {
            drop(cached);
            self.initialize();
            Ref::map(self.all_rules.borrow(), |cached| cached.as_ref().unwrap())
        }
    }

    /// # Panics
    /// If the cache cell is currently borrowed.
    fn clear(&self) {
        *self.all_rules.borrow_mut() = None;
    }

    /// Forcefully initialize this cache with all rules in all plugins currently
    /// enabled.
    ///
    /// This will clobber whatever value is currently stored. It should only be
    /// called when the cache is not populated, either because it has not been
    /// initialized yet or it was cleared with [`Self::clear`].
    ///
    /// # Panics
    /// If the cache cell is currently borrowed.
    fn initialize(&self) {
        debug_assert!(
            self.all_rules.borrow().is_none(),
            "Cannot re-initialize a populated rules cache. It must be cleared first."
        );

        let all_rules: Vec<_> = if self.plugins.is_all() {
            RULES.clone()
        } else {
            let mut plugins = self.plugins;

            // we need to include some jest rules when vitest is enabled, see [`VITEST_COMPATIBLE_JEST_RULES`]
            if plugins.contains(LintPlugins::VITEST) {
                plugins = plugins.union(LintPlugins::JEST);
            }

            RULES
                .iter()
                .filter(|rule| plugins.contains(LintPlugins::from(rule.plugin_name())))
                .cloned()
                .collect()
        };

        *self.all_rules.borrow_mut() = Some(all_rules);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_builder_default() {
        let builder = ConfigStoreBuilder::default();
        assert_eq!(builder.plugins(), LintPlugins::default());

        // populated with all correctness-level ESLint rules at a "warn" severity
        assert!(!builder.rules.is_empty());
        for rule in &builder.rules {
            assert_eq!(rule.category(), RuleCategory::Correctness);
            assert_eq!(rule.severity, AllowWarnDeny::Warn);
            let plugin = rule.rule.plugin_name();
            let name = rule.name();
            assert!(
                builder.plugins().contains(plugin.into()),
                "{plugin}/{name} is in the default rule set but its plugin is not enabled"
            );
        }
    }

    #[test]
    fn test_builder_empty() {
        let builder = ConfigStoreBuilder::empty();
        assert_eq!(builder.plugins(), LintPlugins::default());
        assert!(builder.rules.is_empty());
    }

    #[test]
    fn test_filter_deny_on_default() {
        let builder = ConfigStoreBuilder::default();
        let initial_rule_count = builder.rules.len();

        let builder = builder.with_filters([LintFilter::deny(RuleCategory::Correctness)]);
        let rule_count_after_deny = builder.rules.len();

        // By default, all correctness rules are set to warn. the above filter should only
        // re-configure those rules, and not add/remove any others.
        assert!(!builder.rules.is_empty());
        assert_eq!(initial_rule_count, rule_count_after_deny);

        for rule in &builder.rules {
            assert_eq!(rule.category(), RuleCategory::Correctness);
            assert_eq!(rule.severity, AllowWarnDeny::Deny);

            let plugin = rule.plugin_name();
            let name = rule.name();
            assert!(
                builder.plugins().contains(plugin.into()),
                "{plugin}/{name} is in the default rule set but its plugin is not enabled"
            );
        }
    }

    // change a rule already set to "warn" to "deny"
    #[test]
    fn test_filter_deny_single_enabled_rule_on_default() {
        for filter_string in ["no-const-assign", "eslint/no-const-assign"] {
            let builder = ConfigStoreBuilder::default();
            let initial_rule_count = builder.rules.len();

            let builder =
                builder
                    .with_filters([LintFilter::new(AllowWarnDeny::Deny, filter_string).unwrap()]);
            let rule_count_after_deny = builder.rules.len();
            assert_eq!(
                initial_rule_count, rule_count_after_deny,
                "Changing a single rule from warn to deny should not add a new one, just modify what's already there."
            );

            let no_const_assign = builder
                .rules
                .iter()
                .find(|r| r.plugin_name() == "eslint" && r.name() == "no-const-assign")
                .expect("Could not find eslint/no-const-assign after configuring it to 'deny'");
            assert_eq!(no_const_assign.severity, AllowWarnDeny::Deny);
        }
    }
    // turn on a rule that isn't configured yet and set it to "warn"
    // note that this is an eslint rule, a plugin that's already turned on.
    #[test]
    fn test_filter_warn_single_disabled_rule_on_default() {
        for filter_string in ["no-console", "eslint/no-console"] {
            let filter = LintFilter::new(AllowWarnDeny::Warn, filter_string).unwrap();
            let builder = ConfigStoreBuilder::default();
            // sanity check: not already turned on
            assert!(!builder.rules.iter().any(|r| r.name() == "no-console"));
            let builder = builder.with_filter(filter);
            let no_console = builder
                .rules
                .iter()
                .find(|r| r.plugin_name() == "eslint" && r.name() == "no-console")
                .expect("Could not find eslint/no-console after configuring it to 'warn'");

            assert_eq!(no_console.severity, AllowWarnDeny::Warn);
        }
    }

    #[test]
    fn test_filter_allow_all_then_warn() {
        let builder = ConfigStoreBuilder::default().with_filters([LintFilter::new(
            AllowWarnDeny::Allow,
            "all",
        )
        .unwrap()]);
        assert!(builder.rules.is_empty(), "Allowing all rules should empty out the rules list");

        let builder = builder.with_filters([LintFilter::warn(RuleCategory::Correctness)]);
        assert!(
            !builder.rules.is_empty(),
            "warning on categories after allowing all rules should populate the rules set"
        );
        for rule in &builder.rules {
            let plugin = rule.plugin_name();
            let name = rule.name();
            assert_eq!(
                rule.severity,
                AllowWarnDeny::Warn,
                "{plugin}/{name} should have a warning severity"
            );
            assert_eq!(
                rule.category(),
                RuleCategory::Correctness,
                "{plugin}/{name} should not be enabled b/c we only enabled correctness rules"
            );
        }
    }

    #[test]
    fn test_rules_after_plugin_added() {
        let builder = ConfigStoreBuilder::default();
        let initial_rule_count = builder.rules.len();

        let builder = builder.and_plugins(LintPlugins::IMPORT, true);
        assert_eq!(
            initial_rule_count,
            builder.rules.len(),
            "Enabling a plugin should not add any rules, since we don't know which categories to turn on."
        );
    }

    #[test]
    fn test_rules_after_plugin_removal() {
        // sanity check: the plugin we're removing is, in fact, enabled by default.
        assert!(LintPlugins::default().contains(LintPlugins::TYPESCRIPT));

        let mut desired_plugins = LintPlugins::default();
        desired_plugins.set(LintPlugins::TYPESCRIPT, false);

        let linter = ConfigStoreBuilder::default().with_plugins(desired_plugins).build().unwrap();
        for rule in linter.rules().iter() {
            let name = rule.name();
            let plugin = rule.plugin_name();
            assert_ne!(
                LintPlugins::from(plugin),
                LintPlugins::TYPESCRIPT,
                "{plugin}/{name} is in the rules list after typescript plugin has been disabled"
            );
        }
    }

    #[test]
    fn test_plugin_configuration() {
        let builder = ConfigStoreBuilder::default();
        let initial_plugins = builder.plugins();

        // ==========================================================================================
        // Test ConfigStoreBuilder::and_plugins, which deltas the plugin list instead of overriding it
        // ==========================================================================================

        // Enable eslint plugin. Since it's already enabled, this does nothing.
        assert!(initial_plugins.contains(LintPlugins::ESLINT)); // sanity check that eslint is
                                                                // enabled
        let builder = builder.and_plugins(LintPlugins::ESLINT, true);
        assert_eq!(initial_plugins, builder.plugins());

        // Disable import plugin. Since it's not already enabled, this is also a no-op.
        assert!(!builder.plugins().contains(LintPlugins::IMPORT)); // sanity check that it's not
                                                                   // already enabled
        let builder = builder.and_plugins(LintPlugins::IMPORT, false);
        assert_eq!(initial_plugins, builder.plugins());

        // Enable import plugin. Since it's not already enabled, this turns it on.
        let builder = builder.and_plugins(LintPlugins::IMPORT, true);
        assert_eq!(LintPlugins::default().union(LintPlugins::IMPORT), builder.plugins());
        assert_ne!(initial_plugins, builder.plugins());

        // Turn import back off, resetting plugins to the initial state
        let builder = builder.and_plugins(LintPlugins::IMPORT, false);
        assert_eq!(initial_plugins, builder.plugins());

        // ==========================================================================================
        // Test ConfigStoreBuilder::with_plugins, which _does_ override plugins
        // ==========================================================================================

        let builder = builder.with_plugins(LintPlugins::ESLINT);
        assert_eq!(LintPlugins::ESLINT, builder.plugins());

        let expected_plugins =
            LintPlugins::ESLINT.union(LintPlugins::TYPESCRIPT).union(LintPlugins::NEXTJS);
        let builder = builder.with_plugins(expected_plugins);
        assert_eq!(expected_plugins, builder.plugins());
    }

    #[test]
    fn test_categories() {
        let oxlintrc: Oxlintrc = serde_json::from_str(
            r#"
        {
            "categories": {
                "correctness": "warn",
                "suspicious": "deny"
            },
            "rules": {
                "no-const-assign": "error"
            }
        }
        "#,
        )
        .unwrap();
        let builder = ConfigStoreBuilder::from_oxlintrc(false, oxlintrc);
        for rule in &builder.rules {
            let name = rule.name();
            let plugin = rule.plugin_name();
            let category = rule.category();
            match category {
                RuleCategory::Correctness => {
                    if name == "no-const-assign" {
                        assert_eq!(
                            rule.severity,
                            AllowWarnDeny::Deny,
                            "no-const-assign should be denied",
                        );
                    } else {
                        assert_eq!(
                            rule.severity,
                            AllowWarnDeny::Warn,
                            "{plugin}/{name} should be a warning"
                        );
                    }
                }
                RuleCategory::Suspicious => {
                    assert_eq!(
                        rule.severity,
                        AllowWarnDeny::Deny,
                        "{plugin}/{name} should be denied"
                    );
                }
                invalid => {
                    panic!("Found rule {plugin}/{name} with an unexpected category {invalid:?}");
                }
            }
        }
    }
}
