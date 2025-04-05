use std::{
    cell::{Ref, RefCell},
    fmt::{self, Debug, Display},
};

use itertools::Itertools;
use rustc_hash::FxHashSet;

use oxc_diagnostics::OxcDiagnostic;
use oxc_span::CompactStr;

use crate::{
    AllowWarnDeny, LintConfig, LintFilter, LintFilterKind, Oxlintrc, RuleCategory, RuleEnum,
    RuleWithSeverity,
    config::{
        ConfigStore, ESLintRule, LintPlugins, OxlintOverrides, OxlintRules,
        overrides::OxlintOverride,
    },
    rules::RULES,
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
    /// ```ignore
    /// use oxc_linter::{ConfigBuilder, Oxlintrc};
    /// let oxlintrc = Oxlintrc::from_file("path/to/.oxlintrc.json").unwrap();
    /// let config_store = ConfigStoreBuilder::from_oxlintrc(true, oxlintrc).build();
    /// // you can use `From` as a shorthand for `from_oxlintrc(false, oxlintrc)`
    /// let config_store = ConfigStoreBuilder::from(oxlintrc).build();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`ConfigBuilderError::InvalidConfigFile`] if a referenced config file is not valid.
    pub fn from_oxlintrc(
        start_empty: bool,
        oxlintrc: Oxlintrc,
    ) -> Result<Self, ConfigBuilderError> {
        // TODO(refactor); can we make this function infallible, and move all the error handling to
        // the `build` method?
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
            extends,
        } = oxlintrc;

        let config = LintConfig { plugins, settings, env, globals, path: Some(path) };
        let rules =
            if start_empty { FxHashSet::default() } else { Self::warn_correctness(plugins) };
        let cache = RulesCache::new(config.plugins);
        let mut builder = Self { rules, config, overrides, cache };

        for filter in categories.filters() {
            builder = builder.with_filter(&filter);
        }

        {
            if !extends.is_empty() {
                let config_path = builder.config.path.clone();
                let config_path_parent = config_path.as_ref().and_then(|p| p.parent());

                for path in &extends {
                    if path.starts_with("eslint:") || path.starts_with("plugin:") {
                        // eslint: and plugin: named configs are not supported
                        continue;
                    }
                    // if path does not include a ".", then we will heuristically skip it since it
                    // kind of looks like it might be a named config
                    if !path.to_string_lossy().contains('.') {
                        continue;
                    }

                    // resolve path relative to config path
                    let path = match config_path_parent {
                        Some(config_file_path) => &config_file_path.join(path),
                        None => path,
                    };
                    // TODO: throw an error if this is a self-referential extend
                    // TODO(perf): use a global config cache to avoid re-parsing the same file multiple times
                    match Oxlintrc::from_file(path) {
                        Ok(extended_config) => {
                            // TODO(refactor): can we merge this together? seems redundant to use `override_rules` and then
                            // use `ConfigStoreBuilder`, but we don't have a better way of loading rules from config files other than that.
                            // Use `override_rules` to apply rule configurations and add/remove rules as needed
                            extended_config
                                .rules
                                .override_rules(&mut builder.rules, &builder.cache.borrow());
                            // Use `ConfigStoreBuilder` to load extended config files and then apply rules from those
                            let mut extended_config_store =
                                ConfigStoreBuilder::from_oxlintrc(true, extended_config)?;
                            let rules = std::mem::take(&mut extended_config_store.rules);
                            builder = builder.with_rules(rules);
                            builder = builder.and_plugins(extended_config_store.plugins(), true);
                            if !extended_config_store.overrides.is_empty() {
                                let overrides =
                                    std::mem::take(&mut extended_config_store.overrides);
                                builder = builder.with_overrides(overrides);
                            }
                        }
                        Err(err) => {
                            return Err(ConfigBuilderError::InvalidConfigFile {
                                file: path.display().to_string(),
                                reason: err.to_string(),
                            });
                        }
                    }
                }
            }

            let all_rules = builder.cache.borrow();

            oxlintrc_rules.override_rules(&mut builder.rules, all_rules.as_slice());
        }

        Ok(builder)
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

    pub(crate) fn with_rules<R: IntoIterator<Item = RuleWithSeverity>>(mut self, rules: R) -> Self {
        self.rules.extend(rules);
        self
    }

    /// Appends an override to the end of the current list of overrides.
    pub fn with_overrides<O: IntoIterator<Item = OxlintOverride>>(mut self, overrides: O) -> Self {
        self.overrides.extend(overrides);
        self
    }

    pub fn with_filters<'a, I: IntoIterator<Item = &'a LintFilter>>(mut self, filters: I) -> Self {
        for filter in filters {
            self = self.with_filter(filter);
        }
        self
    }

    pub fn with_filter(mut self, filter: &LintFilter) -> Self {
        let (severity, filter) = filter.into();

        match severity {
            AllowWarnDeny::Deny | AllowWarnDeny::Warn => match filter {
                LintFilterKind::Category(category) => {
                    self.upsert_where(severity, |r| r.category() == *category);
                }
                LintFilterKind::Rule(_, name) => self.upsert_where(severity, |r| r.name() == name),
                LintFilterKind::Generic(name) => self.upsert_where(severity, |r| r.name() == name),
                LintFilterKind::All => {
                    self.upsert_where(severity, |r| r.category() != RuleCategory::Nursery);
                }
            },
            AllowWarnDeny::Allow => match filter {
                LintFilterKind::Category(category) => {
                    self.rules.retain(|rule| rule.category() != *category);
                }
                LintFilterKind::Rule(_, name) => self.rules.retain(|rule| rule.name() != name),
                LintFilterKind::Generic(name) => self.rules.retain(|rule| rule.name() != name),
                LintFilterKind::All => self.rules.clear(),
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
            match self.rules.take(rule) {
                Some(mut existing_rule) => {
                    existing_rule.severity = severity;
                    self.rules.insert(existing_rule);
                }
                _ => {
                    self.rules.insert(RuleWithSeverity::new(rule.clone(), severity));
                }
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
            .sorted_by_key(|x| (x.plugin_name(), x.name()))
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
        Self::from_oxlintrc(false, oxlintrc)
    }
}

impl Debug for ConfigStoreBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigStoreBuilder")
            .field("rules", &self.rules)
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

/// An error that can occur while building a [`ConfigStore`] from an [`Oxlintrc`].
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ConfigBuilderError {
    /// There were unknown rules that could not be matched to any known plugins/rules.
    UnknownRules { rules: Vec<ESLintRule> },
    /// A configuration file was referenced which was not valid for some reason.
    InvalidConfigFile { file: String, reason: String },
}

impl Display for ConfigBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigBuilderError::UnknownRules { rules } => {
                f.write_str("unknown rules: ")?;
                for (i, rule) in rules.iter().enumerate() {
                    if i == 0 {
                        Display::fmt(&rule.full_name(), f)?;
                    } else {
                        write!(f, ", {}", rule.full_name())?;
                    }
                }
                Ok(())
            }
            ConfigBuilderError::InvalidConfigFile { file, reason } => {
                write!(f, "invalid config file {file}: {reason}")
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
    use std::path::PathBuf;

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

        let builder = builder.with_filter(&LintFilter::deny(RuleCategory::Correctness));
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
                builder.with_filter(&LintFilter::new(AllowWarnDeny::Deny, filter_string).unwrap());
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
            let builder = builder.with_filter(&filter);
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
        let builder = ConfigStoreBuilder::default()
            .with_filter(&LintFilter::new(AllowWarnDeny::Allow, "all").unwrap());
        assert!(builder.rules.is_empty(), "Allowing all rules should empty out the rules list");

        let builder = builder.with_filter(&LintFilter::warn(RuleCategory::Correctness));
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
        let builder = ConfigStoreBuilder::from_oxlintrc(false, oxlintrc).unwrap();
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

    #[test]
    fn test_extends_rules_single() {
        let base_config = config_store_from_path("fixtures/extends_config/rules_config.json");
        let derived_config = config_store_from_str(
            r#"
        {
            "extends": [
                "fixtures/extends_config/rules_config.json"
            ]
        }
        "#,
        );

        assert_eq!(base_config.rules(), derived_config.rules());

        let update_rules_config = config_store_from_str(
            r#"
        {
            "extends": [
                "fixtures/extends_config/rules_config.json"
            ],
            "rules": {
                "no-debugger": "warn",
                "no-console": "warn",
                "unicorn/no-null": "off",
                "typescript/prefer-as-const": "warn"
            }
        }
        "#,
        );

        assert!(
            update_rules_config
                .rules()
                .iter()
                .any(|r| r.name() == "no-debugger" && r.severity == AllowWarnDeny::Warn)
        );
        assert!(
            update_rules_config
                .rules()
                .iter()
                .any(|r| r.name() == "no-console" && r.severity == AllowWarnDeny::Warn)
        );
        assert!(
            !update_rules_config
                .rules()
                .iter()
                .any(|r| r.name() == "no-null" && r.severity == AllowWarnDeny::Allow)
        );
        assert!(
            update_rules_config
                .rules()
                .iter()
                .any(|r| r.name() == "prefer-as-const" && r.severity == AllowWarnDeny::Warn)
        );
    }

    #[test]
    fn test_extends_rules_multiple() {
        let warn_all = config_store_from_str(
            r#"
        {
            "extends": [
                "fixtures/extends_config/rules_multiple/allow_all.json",
                "fixtures/extends_config/rules_multiple/deny_all.json",
                "fixtures/extends_config/rules_multiple/warn_all.json"
            ]
        }
        "#,
        );
        assert!(warn_all.rules().iter().all(|r| r.severity == AllowWarnDeny::Warn));

        let deny_all = config_store_from_str(
            r#"
        {
            "extends": [
                "fixtures/extends_config/rules_multiple/allow_all.json",
                "fixtures/extends_config/rules_multiple/warn_all.json",
                "fixtures/extends_config/rules_multiple/deny_all.json"
            ]
        }
        "#,
        );
        assert!(deny_all.rules().iter().all(|r| r.severity == AllowWarnDeny::Deny));

        let allow_all = config_store_from_str(
            r#"
        {
            "extends": [
                "fixtures/extends_config/rules_multiple/warn_all.json",
                "fixtures/extends_config/rules_multiple/deny_all.json",
                "fixtures/extends_config/rules_multiple/allow_all.json"
            ]
        }
        "#,
        );
        assert!(allow_all.rules().iter().all(|r| r.severity == AllowWarnDeny::Allow));
        assert_eq!(allow_all.number_of_rules(), 0);

        let allow_and_override_config = config_store_from_str(
            r#"
        {
            "extends": [
                "fixtures/extends_config/rules_multiple/deny_all.json",
                "fixtures/extends_config/rules_multiple/allow_all.json"
            ],
            "rules": {
                "no-var": "warn",
                "oxc/approx-constant": "error",
                "unicorn/no-null": "error"
            }
        }
        "#,
        );
        assert!(
            allow_and_override_config
                .rules()
                .iter()
                .any(|r| r.name() == "no-var" && r.severity == AllowWarnDeny::Warn)
        );
        assert!(
            allow_and_override_config
                .rules()
                .iter()
                .any(|r| r.name() == "approx-constant" && r.severity == AllowWarnDeny::Deny)
        );
        assert!(
            allow_and_override_config
                .rules()
                .iter()
                .any(|r| r.name() == "no-null" && r.severity == AllowWarnDeny::Deny)
        );
    }

    #[test]
    fn test_extends_invalid() {
        let invalid_config = ConfigStoreBuilder::from_oxlintrc(
            true,
            Oxlintrc::from_file(&PathBuf::from(
                "fixtures/extends_config/extends_invalid_config.json",
            ))
            .unwrap(),
        );
        let err = invalid_config.unwrap_err();
        assert!(matches!(err, ConfigBuilderError::InvalidConfigFile { .. }));
        if let ConfigBuilderError::InvalidConfigFile { file, reason } = err {
            assert!(file.ends_with("invalid_config.json"));
            assert!(reason.contains("Failed to parse"));
        }
    }

    #[test]
    fn test_extends_plugins() {
        let config = config_store_from_str(
            r#"
        {
            "extends": [
                "fixtures/extends_config/plugins/jest.json",
                "fixtures/extends_config/plugins/react.json"
            ]
        }
        "#,
        );
        assert!(config.plugins().contains(LintPlugins::default()));
        assert!(config.plugins().contains(LintPlugins::JEST));
        assert!(config.plugins().contains(LintPlugins::REACT));

        // Test adding more plugins
        let config = config_store_from_str(
            r#"
        {
            "extends": [
                "fixtures/extends_config/plugins/jest.json",
                "fixtures/extends_config/plugins/react.json"
            ],
            "plugins": ["typescript"]
        }
        "#,
        );
        assert_eq!(
            config.plugins(),
            LintPlugins::JEST | LintPlugins::REACT | LintPlugins::TYPESCRIPT
        );

        // Test that extended a config with a plugin is the same as adding it directly
        let plugin_config = config_store_from_str(r#"{ "plugins": ["jest", "react"] }"#);
        let extends_plugin_config = config_store_from_str(
            r#"
        {
            "extends": [
                "fixtures/extends_config/plugins/jest.json",
                "fixtures/extends_config/plugins/react.json"
            ],
            "plugins": []
        }
        "#,
        );
        assert_eq!(
            plugin_config.plugins(),
            extends_plugin_config.plugins(),
            "Extending a config with a plugin is the same as adding it directly"
        );
    }

    #[test]
    fn test_not_extends_named_configs() {
        // For now, test that extending named configs is just ignored
        let config = config_store_from_str(
            r#"
        {
            "extends": [
                "next/core-web-vitals",
                "eslint:recommended",
                "plugin:@typescript-eslint/strict-type-checked",
                "prettier",
                "plugin:unicorn/recommended"
            ]
        }
        "#,
        );
        assert_eq!(config.plugins(), LintPlugins::default());
        assert!(config.rules().is_empty());
    }

    fn config_store_from_path(path: &str) -> ConfigStore {
        ConfigStoreBuilder::from_oxlintrc(true, Oxlintrc::from_file(&PathBuf::from(path)).unwrap())
            .unwrap()
            .build()
            .unwrap()
    }

    fn config_store_from_str(s: &str) -> ConfigStore {
        ConfigStoreBuilder::from_oxlintrc(true, serde_json::from_str(s).unwrap())
            .unwrap()
            .build()
            .unwrap()
    }
}
