use std::{
    fmt::{self, Debug, Display},
    path::{Path, PathBuf},
};

use itertools::Itertools;
use oxc_resolver::{ResolveOptions, Resolver};
use rustc_hash::{FxHashMap, FxHashSet};

use oxc_span::{CompactStr, format_compact_str};

use crate::{
    AllowWarnDeny, ExternalPluginStore, LintConfig, LintFilter, LintFilterKind, Oxlintrc,
    RuleCategory, RuleEnum,
    config::{
        ESLintRule, OxlintOverrides, OxlintRules, overrides::OxlintOverride, plugins::LintPlugins,
    },
    external_linter::ExternalLinter,
    external_plugin_store::{ExternalRuleId, ExternalRuleLookupError},
    rules::RULES,
};

use super::{
    Config,
    categories::OxlintCategories,
    config_store::{ResolvedOxlintOverride, ResolvedOxlintOverrideRules, ResolvedOxlintOverrides},
};

#[must_use = "You dropped your builder without building a Linter! Did you mean to call .build()?"]
pub struct ConfigStoreBuilder {
    pub(super) rules: FxHashMap<RuleEnum, AllowWarnDeny>,
    pub(super) external_rules: FxHashMap<ExternalRuleId, AllowWarnDeny>,
    config: LintConfig,
    categories: OxlintCategories,
    overrides: OxlintOverrides,

    // Collect all `extends` file paths for the language server.
    // The server will tell the clients to watch for the extends files.
    pub extended_paths: Vec<PathBuf>,
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
        let rules = FxHashMap::default();
        let external_rules = FxHashMap::default();
        let categories: OxlintCategories = OxlintCategories::default();
        let overrides = OxlintOverrides::default();
        let extended_paths = Vec::new();

        Self { rules, external_rules, config, categories, overrides, extended_paths }
    }

    /// Warn on all rules in all plugins and categories, including those in `nursery`.
    /// This is the kitchen sink.
    ///
    /// You can think of this as `oxlint -W all -W nursery`.
    pub fn all() -> Self {
        let config = LintConfig { plugins: LintPlugins::all(), ..LintConfig::default() };
        let overrides = OxlintOverrides::default();
        let categories: OxlintCategories = OxlintCategories::default();
        let rules = RULES.iter().map(|rule| (rule.clone(), AllowWarnDeny::Warn)).collect();
        let external_rules = FxHashMap::default();
        let extended_paths = Vec::new();
        Self { rules, external_rules, config, categories, overrides, extended_paths }
    }

    /// Create a [`ConfigStoreBuilder`] from a loaded or manually built [`Oxlintrc`].
    /// `start_empty` will configure the builder to contain only the
    /// configuration settings from the config. When this is `false`, the config
    /// will be applied on top of a default [`Oxlintrc`].
    ///
    /// # Example
    /// Here's how to create a [`Config`] from a `.oxlintrc.json` file.
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
        external_linter: Option<&ExternalLinter>,
        external_plugin_store: &mut ExternalPluginStore,
    ) -> Result<Self, ConfigBuilderError> {
        // TODO: this can be cached to avoid re-computing the same oxlintrc
        fn resolve_oxlintrc_config(
            config: Oxlintrc,
            resolver: &Resolver,
        ) -> Result<(Oxlintrc, Vec<PathBuf>), ConfigBuilderError> {
            let path = config.path.clone();
            let root_path = path.parent();
            let extends = config.extends.clone();
            let mut extended_paths = Vec::new();

            let mut oxlintrc = config;

            for path in extends.iter().rev() {
                if path.starts_with("eslint:") || path.starts_with("plugin:") {
                    // `eslint:` and `plugin:` named configs are not supported
                    continue;
                }
                // if path does not include a ".", then we will heuristically skip it since it
                // kind of looks like it might be a named config
                if !path.to_string_lossy().contains('.') && !path.to_string_lossy().starts_with('@')
                {
                    continue;
                }
                let path = {
                    let scoped_package_name: std::borrow::Cow<'_, str> = path.to_string_lossy();
                    // I am limiting to scoped packages to test the waters, but this would work the same for unscoped npm packages too.
                    if scoped_package_name.starts_with('@') {
                        // I'm not sure if "/" is a viable fallback, but it's very unlikely that the oxlint file does not have a parent directory.
                        let base_dir = root_path.unwrap_or(Path::new("/"));
                        let resolved = resolver
                            .resolve(base_dir, scoped_package_name.as_ref())
                            .map_err(|err| ConfigBuilderError::InvalidConfigFile {
                                file: scoped_package_name.to_string(),
                                reason: err.to_string(),
                            })?;
                        &resolved.full_path()
                    } else {
                        match root_path {
                            Some(p) => &p.join(path),
                            None => path,
                        }
                    }
                };

                let extends_oxlintrc = Oxlintrc::from_file(path).map_err(|e| {
                    ConfigBuilderError::InvalidConfigFile {
                        file: path.display().to_string(),
                        reason: e.to_string(),
                    }
                })?;

                extended_paths.push(path.clone());

                let (extends, extends_paths) = resolve_oxlintrc_config(extends_oxlintrc, resolver)?;

                oxlintrc = oxlintrc.merge(extends);
                extended_paths.extend(extends_paths);
            }

            Ok((oxlintrc, extended_paths))
        }

        let resolver = Resolver::new(ResolveOptions {
            // Look at the "import"-labeled and then "default"-labeled export in package.json "exports" field if multiple are provided.
            condition_names: vec!["import".into()],
            ..Default::default()
        });

        let (oxlintrc, extended_paths) = resolve_oxlintrc_config(oxlintrc, &resolver)?;

        // Collect external plugins from both base config and overrides
        let mut external_plugins: FxHashSet<(&PathBuf, &str)> = FxHashSet::default();

        if let Some(base_external_plugins) = &oxlintrc.external_plugins {
            external_plugins.extend(base_external_plugins.iter().map(|(k, v)| (k, v.as_str())));
        }

        for r#override in &oxlintrc.overrides {
            if let Some(override_external_plugins) = &r#override.external_plugins {
                external_plugins
                    .extend(override_external_plugins.iter().map(|(k, v)| (k, v.as_str())));
            }
        }

        // If external plugins are not enabled (language server), then skip loading JS plugins.
        // This is so that a project can use JS plugins via `oxlint` CLI, and language server
        // will just silently ignore them - rather than crashing.
        if !external_plugins.is_empty() && external_plugin_store.is_enabled() {
            let Some(external_linter) = external_linter else {
                #[expect(clippy::missing_panics_doc, reason = "infallible")]
                let (_, original_specifier) = external_plugins.iter().next().unwrap();
                return Err(ConfigBuilderError::NoExternalLinterConfigured {
                    plugin_specifier: (*original_specifier).to_string(),
                });
            };

            let resolver = Resolver::new(ResolveOptions {
                main_fields: vec!["module".into(), "main".into()],
                condition_names: vec!["module".into(), "import".into()],
                ..Default::default()
            });

            for (config_path, specifier) in &external_plugins {
                Self::load_external_plugin(
                    config_path,
                    specifier,
                    external_linter,
                    &resolver,
                    external_plugin_store,
                )?;
            }
        }

        let plugins = oxlintrc.plugins.unwrap_or_default();

        let rules =
            if start_empty { FxHashMap::default() } else { Self::warn_correctness(plugins) };

        let mut categories = oxlintrc.categories.clone();

        if !start_empty {
            categories.insert(RuleCategory::Correctness, AllowWarnDeny::Warn);
        }

        let config = LintConfig {
            plugins,
            settings: oxlintrc.settings,
            env: oxlintrc.env,
            globals: oxlintrc.globals,
            path: Some(oxlintrc.path),
        };

        let mut builder = Self {
            rules,
            external_rules: FxHashMap::default(),
            config,
            categories,
            overrides: oxlintrc.overrides,
            extended_paths,
        };

        for filter in oxlintrc.categories.filters() {
            builder = builder.with_filter(&filter);
        }

        {
            let all_rules = builder.get_all_rules();

            oxlintrc
                .rules
                .override_rules(
                    &mut builder.rules,
                    &mut builder.external_rules,
                    &all_rules,
                    external_plugin_store,
                )
                .map_err(ConfigBuilderError::ExternalRuleLookupError)?;
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
    /// config is set. If you are looking to add/remove plugins, use [`and_builtin_plugins`]
    ///
    /// [`with_filters`]: ConfigStoreBuilder::with_filters
    /// [`and_builtin_plugins`]: ConfigStoreBuilder::and_builtin_plugins
    #[inline]
    pub fn with_builtin_plugins(mut self, plugins: LintPlugins) -> Self {
        self.config.plugins = plugins;
        self
    }

    pub fn with_categories(mut self, categories: OxlintCategories) -> Self {
        self.categories = categories;
        self
    }

    /// Enable or disable a set of plugins, leaving unrelated plugins alone.
    ///
    /// See [`ConfigStoreBuilder::with_builtin_plugins`] for details on how plugin configuration affects your
    /// rules.
    #[inline]
    pub fn and_builtin_plugins(mut self, plugins: LintPlugins, enabled: bool) -> Self {
        self.config.plugins.set(plugins, enabled);
        self
    }

    #[inline]
    pub fn plugins(&self) -> LintPlugins {
        self.config.plugins
    }

    #[cfg(test)]
    pub(crate) fn with_rule(mut self, rule: RuleEnum, severity: AllowWarnDeny) -> Self {
        self.rules.insert(rule, severity);
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
                LintFilterKind::Rule(plugin, rule) => {
                    let (plugin, rule) = super::rules::unalias_plugin_name(plugin, rule);
                    self.upsert_where(severity, |r| r.plugin_name() == plugin && r.name() == rule);
                }
                LintFilterKind::Generic(name) => self.upsert_where(severity, |r| r.name() == name),
                LintFilterKind::All => {
                    self.upsert_where(severity, |r| r.category() != RuleCategory::Nursery);
                }
            },
            AllowWarnDeny::Allow => match filter {
                LintFilterKind::Category(category) => {
                    self.rules.retain(|rule, _| rule.category() != *category);
                }
                LintFilterKind::Rule(plugin, rule) => {
                    let (plugin, rule) = super::rules::unalias_plugin_name(plugin, rule);
                    self.rules.retain(|r, _| r.plugin_name() != plugin || r.name() != rule);
                }
                LintFilterKind::Generic(name) => self.rules.retain(|rule, _| rule.name() != name),
                LintFilterKind::All => self.rules.clear(),
            },
        }

        self
    }

    /// Warn/Deny a let of rules based on some predicate. Rules already in `self.rules` get
    /// re-configured, while those that are not are added. Affects rules where `query` returns
    /// `true`.
    fn get_all_rules(&self) -> Vec<RuleEnum> {
        self.get_all_rules_for_plugins(None)
    }

    fn get_all_rules_for_plugins(&self, override_plugins: Option<LintPlugins>) -> Vec<RuleEnum> {
        let mut builtin_plugins = if let Some(override_plugins) = override_plugins {
            self.config.plugins | override_plugins
        } else {
            self.config.plugins
        };

        if builtin_plugins.is_all() {
            RULES.clone()
        } else {
            // we need to include some jest rules when vitest is enabled, see [`VITEST_COMPATIBLE_JEST_RULES`]
            if builtin_plugins.contains(LintPlugins::VITEST) {
                builtin_plugins |= LintPlugins::JEST;
            }

            RULES
                .iter()
                .filter(|rule| {
                    LintPlugins::try_from(rule.plugin_name())
                        .is_ok_and(|plugin_flag| builtin_plugins.contains(plugin_flag))
                })
                .cloned()
                .collect()
        }
    }

    fn upsert_where<F>(&mut self, severity: AllowWarnDeny, query: F)
    where
        F: Fn(&&RuleEnum) -> bool,
    {
        let all_rules = self.get_all_rules();
        // NOTE: we may want to warn users if they're configuring a rule that does not exist.
        let rules_to_configure = all_rules.iter().filter(query);
        for rule in rules_to_configure {
            // If the rule is already in the list, just update its severity.
            // Otherwise, add it to the map.

            if let Some(existing_rule) = self.rules.get_mut(rule) {
                *existing_rule = severity;
            } else {
                self.rules.insert(rule.clone(), severity);
            }
        }
    }

    /// Builds a [`Config`] from the current state of the builder.
    /// # Errors
    /// Returns [`ConfigBuilderError::UnknownRules`] if there are rules that could not be matched.
    pub fn build(
        mut self,
        external_plugin_store: &ExternalPluginStore,
    ) -> Result<Config, ConfigBuilderError> {
        // When a plugin gets disabled before build(), rules for that plugin aren't removed until
        // with_filters() gets called. If the user never calls it, those now-undesired rules need
        // to be taken out.
        let mut plugins = self.plugins();

        // Apply the same Vitest->Jest logic as in get_all_rules()
        if plugins.contains(LintPlugins::VITEST) {
            plugins |= LintPlugins::JEST;
        }

        let overrides = std::mem::take(&mut self.overrides);
        let resolved_overrides = self
            .resolve_overrides(overrides, external_plugin_store)
            .map_err(ConfigBuilderError::ExternalRuleLookupError)?;

        let mut rules: Vec<_> = self
            .rules
            .into_iter()
            .filter(|(r, _)| {
                LintPlugins::try_from(r.plugin_name())
                    .is_ok_and(|plugin_name| plugins.contains(plugin_name))
            })
            .collect();
        rules.sort_unstable_by_key(|(r, _)| r.id());

        let mut external_rules: Vec<_> = self.external_rules.into_iter().collect();
        external_rules.sort_unstable_by_key(|(r, _)| *r);

        Ok(Config::new(rules, external_rules, self.categories, self.config, resolved_overrides))
    }

    fn resolve_overrides(
        &self,
        overrides: OxlintOverrides,
        external_plugin_store: &ExternalPluginStore,
    ) -> Result<ResolvedOxlintOverrides, ExternalRuleLookupError> {
        let resolved = overrides
            .into_iter()
            .map(|override_config| {
                let mut builtin_rules = Vec::new();
                let mut external_rules = Vec::new();
                let mut rules_map = FxHashMap::default();
                let mut external_rules_map = FxHashMap::default();

                let all_rules = self.get_all_rules_for_plugins(override_config.plugins);

                // Resolve rules for this override
                override_config.rules.override_rules(
                    &mut rules_map,
                    &mut external_rules_map,
                    &all_rules,
                    external_plugin_store,
                )?;

                // Convert to vectors
                builtin_rules.extend(rules_map.into_iter());
                external_rules.extend(external_rules_map.into_iter());

                Ok(ResolvedOxlintOverride {
                    files: override_config.files,
                    env: override_config.env,
                    globals: override_config.globals,
                    plugins: override_config.plugins,
                    rules: ResolvedOxlintOverrideRules { builtin_rules, external_rules },
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ResolvedOxlintOverrides::new(resolved))
    }

    /// Warn for all correctness rules in the given set of plugins.
    fn warn_correctness(mut plugins: LintPlugins) -> FxHashMap<RuleEnum, AllowWarnDeny> {
        if plugins.contains(LintPlugins::VITEST) {
            plugins |= LintPlugins::JEST;
        }
        RULES
            .iter()
            .filter(|rule| {
                // NOTE: this logic means there's no way to disable ESLint
                // correctness rules. I think that's fine for now.
                rule.category() == RuleCategory::Correctness
                    && LintPlugins::try_from(rule.plugin_name())
                        .is_ok_and(|plugin_flag| plugins.contains(plugin_flag))
            })
            .map(|rule| (rule.clone(), AllowWarnDeny::Warn))
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
            .sorted_unstable_by_key(|(r, _)| (r.plugin_name(), r.name()))
            .map(|(r, severity)| ESLintRule {
                plugin_name: r.plugin_name().to_string(),
                rule_name: r.name().to_string(),
                severity: *severity,
                config: rule_name_to_rule
                    .get(&get_name(r.plugin_name(), r.name()))
                    .and_then(|r| r.config.clone()),
            })
            .collect();

        oxlintrc.rules = OxlintRules::new(new_rules);
        serde_json::to_string_pretty(&oxlintrc).unwrap()
    }

    fn load_external_plugin(
        resolve_dir: &Path,
        plugin_specifier: &str,
        external_linter: &ExternalLinter,
        resolver: &Resolver,
        external_plugin_store: &mut ExternalPluginStore,
    ) -> Result<(), ConfigBuilderError> {
        use crate::PluginLoadResult;

        // Print warning on 1st attempt to load a plugin
        #[expect(clippy::print_stderr)]
        if external_plugin_store.is_empty() {
            eprintln!(
                "WARNING: JS plugins are experimental and not subject to semver.\nBreaking changes are possible while JS plugins support is under development."
            );
        }

        // Resolve the specifier relative to the config directory
        let resolved = resolver.resolve(resolve_dir, plugin_specifier).map_err(|e| {
            ConfigBuilderError::PluginLoadFailed {
                plugin_specifier: plugin_specifier.to_string(),
                error: e.to_string(),
            }
        })?;
        // TODO: We should support paths which are not valid UTF-8. How?
        let plugin_path = resolved.full_path().to_str().unwrap().to_string();

        if external_plugin_store.is_plugin_registered(&plugin_path) {
            return Ok(());
        }

        // Extract package name from package.json if available
        let package_name = resolved.package_json().and_then(|pkg| pkg.name().map(String::from));

        let result = {
            let plugin_path = plugin_path.clone();
            (external_linter.load_plugin)(plugin_path, package_name).map_err(|e| {
                ConfigBuilderError::PluginLoadFailed {
                    plugin_specifier: plugin_specifier.to_string(),
                    error: e.to_string(),
                }
            })
        }?;

        match result {
            PluginLoadResult::Success { name, offset, rule_names } => {
                // Normalize plugin name (e.g., "eslint-plugin-foo" -> "foo", "@foo/eslint-plugin" -> "@foo")
                use crate::config::plugins::normalize_plugin_name;
                let normalized_name = normalize_plugin_name(&name).into_owned();

                if LintPlugins::try_from(normalized_name.as_str()).is_err() {
                    external_plugin_store.register_plugin(
                        plugin_path,
                        normalized_name,
                        offset,
                        rule_names,
                    );
                    Ok(())
                } else {
                    Err(ConfigBuilderError::ReservedExternalPluginName {
                        plugin_name: normalized_name,
                    })
                }
            }
            PluginLoadResult::Failure(e) => Err(ConfigBuilderError::PluginLoadFailed {
                plugin_specifier: plugin_specifier.to_string(),
                error: e,
            }),
        }
    }
}

fn get_name(plugin_name: &str, rule_name: &str) -> CompactStr {
    if plugin_name == "eslint" {
        CompactStr::from(rule_name)
    } else {
        format_compact_str!("{plugin_name}/{rule_name}")
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

/// An error that can occur while building a [`Config`] from an [`Oxlintrc`].
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ConfigBuilderError {
    /// There were unknown rules that could not be matched to any known plugins/rules.
    UnknownRules {
        rules: Vec<ESLintRule>,
    },
    /// A configuration file was referenced which was not valid for some reason.
    InvalidConfigFile {
        file: String,
        reason: String,
    },
    PluginLoadFailed {
        plugin_specifier: String,
        error: String,
    },
    ExternalRuleLookupError(ExternalRuleLookupError),
    NoExternalLinterConfigured {
        plugin_specifier: String,
    },
    ReservedExternalPluginName {
        plugin_name: String,
    },
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
            ConfigBuilderError::PluginLoadFailed { plugin_specifier, error } => {
                write!(f, "Failed to load JS plugin: {plugin_specifier}\n  {error}")?;
                Ok(())
            }
            ConfigBuilderError::NoExternalLinterConfigured { plugin_specifier } => {
                write!(
                    f,
                    "`jsPlugins` config contains '{plugin_specifier}'. JS plugins are not supported on 32-bit or big-endian platforms at present.",
                )?;
                Ok(())
            }
            ConfigBuilderError::ReservedExternalPluginName { plugin_name } => {
                write!(
                    f,
                    "Plugin name '{plugin_name}' is reserved, and cannot be used for JS plugins",
                )?;
                Ok(())
            }
            ConfigBuilderError::ExternalRuleLookupError(e) => std::fmt::Display::fmt(&e, f),
        }
    }
}

impl std::error::Error for ConfigBuilderError {}

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
        for (rule, severity) in &builder.rules {
            assert_eq!(rule.category(), RuleCategory::Correctness);
            assert_eq!(*severity, AllowWarnDeny::Warn);
            let plugin_name = rule.plugin_name();
            let plugin = LintPlugins::try_from(plugin_name);
            let name = rule.name();
            assert!(
                plugin.is_ok_and(|plugin| builder.plugins().contains(plugin)),
                "{plugin_name}/{name} is in the default rule set but its plugin is not enabled"
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

        for (rule, severity) in &builder.rules {
            assert_eq!(rule.category(), RuleCategory::Correctness);
            assert_eq!(*severity, AllowWarnDeny::Deny);

            let plugin_name = rule.plugin_name();
            let plugin = LintPlugins::try_from(plugin_name);
            let name = rule.name();
            assert!(
                plugin.is_ok_and(|plugin| builder.plugins().contains(plugin)),
                "{plugin_name}/{name} is in the default rule set but its plugin is not enabled"
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

            let (_, severity) = builder
                .rules
                .iter()
                .find(|(r, _)| r.plugin_name() == "eslint" && r.name() == "no-const-assign")
                .expect("Could not find eslint/no-const-assign after configuring it to 'deny'");
            assert_eq!(*severity, AllowWarnDeny::Deny);
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
            assert!(!builder.rules.iter().any(|(r, _)| r.name() == "no-console"));
            let builder = builder.with_filter(&filter);
            let (_, severity) = builder
                .rules
                .iter()
                .find(|(r, _)| r.plugin_name() == "eslint" && r.name() == "no-console")
                .expect("Could not find eslint/no-console after configuring it to 'warn'");

            assert_eq!(*severity, AllowWarnDeny::Warn);
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
        for (rule, severity) in &builder.rules {
            let plugin = rule.plugin_name();
            let name = rule.name();
            assert_eq!(
                *severity,
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

        let builder = builder.and_builtin_plugins(LintPlugins::IMPORT, true);
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

        let external_plugin_store = ExternalPluginStore::default();
        let linter = ConfigStoreBuilder::default()
            .with_builtin_plugins(desired_plugins)
            .build(&external_plugin_store)
            .unwrap();
        for (rule, _) in linter.base.rules.iter() {
            let name = rule.name();
            let plugin = rule.plugin_name();
            assert_ne!(
                LintPlugins::try_from(plugin),
                Ok(LintPlugins::TYPESCRIPT),
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
        let builder = builder.and_builtin_plugins(LintPlugins::ESLINT, true);
        assert_eq!(initial_plugins, builder.plugins());

        // Disable import plugin. Since it's not already enabled, this is also a no-op.
        assert!(!builder.plugins().contains(LintPlugins::IMPORT)); // sanity check that it's not
        // already enabled
        let builder = builder.and_builtin_plugins(LintPlugins::IMPORT, false);
        assert_eq!(initial_plugins, builder.plugins());

        // Enable import plugin. Since it's not already enabled, this turns it on.
        let builder = builder.and_builtin_plugins(LintPlugins::IMPORT, true);
        assert_eq!(LintPlugins::default() | LintPlugins::IMPORT, builder.plugins());
        assert_ne!(initial_plugins, builder.plugins());

        // Turn import back off, resetting plugins to the initial state
        let builder = builder.and_builtin_plugins(LintPlugins::IMPORT, false);
        assert_eq!(initial_plugins, builder.plugins());

        // ==========================================================================================
        // Test ConfigStoreBuilder::with_plugins, which _does_ override plugins
        // ==========================================================================================

        let builder = builder.with_builtin_plugins(LintPlugins::ESLINT);
        assert_eq!(LintPlugins::ESLINT, builder.plugins());

        let expected_plugins = LintPlugins::ESLINT | LintPlugins::TYPESCRIPT | LintPlugins::NEXTJS;
        let builder = builder.with_builtin_plugins(expected_plugins);
        assert_eq!(expected_plugins, builder.plugins());
    }

    #[test]
    fn test_cli_rule_aliases() {
        let builder = ConfigStoreBuilder::default().and_builtin_plugins(LintPlugins::REACT, true);

        // Assert rule doesn't exist by default
        assert_eq!(
            builder
                .rules
                .iter()
                .find(|(r, _)| r.plugin_name() == "react" && r.name() == "exhaustive-deps"),
            None
        );

        let builder = builder.with_filter(
            &LintFilter::new(AllowWarnDeny::Deny, "react-hooks/exhaustive-deps").unwrap(),
        );

        let (rule, sev) = builder
            .rules
            .iter()
            .find(|(r, _)| r.plugin_name() == "react" && r.name() == "exhaustive-deps")
            .expect("react/exhaustive-deps should be configured to Deny");

        assert_eq!(rule.plugin_name(), "react");
        assert_eq!(rule.name(), "exhaustive-deps");
        assert_eq!(sev, &AllowWarnDeny::Deny);

        let builder = builder.with_filter(
            &LintFilter::new(AllowWarnDeny::Allow, "react-hooks/exhaustive-deps").unwrap(),
        );

        // Allowing the rule removes it from rules "overlay"
        assert_eq!(
            builder
                .rules
                .iter()
                .find(|(r, _)| r.plugin_name() == "react" && r.name() == "exhaustive-deps"),
            None
        );
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
        let builder = {
            let mut external_plugin_store = ExternalPluginStore::default();
            ConfigStoreBuilder::from_oxlintrc(false, oxlintrc, None, &mut external_plugin_store)
                .unwrap()
        };
        for (rule, severity) in &builder.rules {
            let name = rule.name();
            let plugin = rule.plugin_name();
            let category = rule.category();
            match category {
                RuleCategory::Correctness => {
                    if name == "no-const-assign" {
                        assert_eq!(
                            *severity,
                            AllowWarnDeny::Deny,
                            "no-const-assign should be denied",
                        );
                    } else {
                        assert_eq!(
                            *severity,
                            AllowWarnDeny::Warn,
                            "{plugin}/{name} should be a warning"
                        );
                    }
                }
                RuleCategory::Suspicious => {
                    assert_eq!(*severity, AllowWarnDeny::Deny, "{plugin}/{name} should be denied");
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
                .any(|(r, severity)| r.name() == "no-debugger" && *severity == AllowWarnDeny::Warn)
        );
        assert!(
            update_rules_config
                .rules()
                .iter()
                .any(|(r, severity)| r.name() == "no-console" && *severity == AllowWarnDeny::Warn)
        );
        assert!(
            !update_rules_config
                .rules()
                .iter()
                .any(|(r, severity)| r.name() == "no-null" && *severity == AllowWarnDeny::Allow)
        );
        assert!(
            update_rules_config
                .rules()
                .iter()
                .any(|(r, severity)| r.name() == "prefer-as-const"
                    && *severity == AllowWarnDeny::Warn)
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
        assert!(warn_all.rules().iter().all(|(_, severity)| *severity == AllowWarnDeny::Warn));

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
        assert!(deny_all.rules().iter().all(|(_, severity)| *severity == AllowWarnDeny::Deny));

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
        assert!(allow_all.rules().iter().all(|(_, severity)| *severity == AllowWarnDeny::Allow));
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
                .any(|(r, severity)| r.name() == "no-var" && *severity == AllowWarnDeny::Warn)
        );
        assert!(
            allow_and_override_config
                .rules()
                .iter()
                .any(|(r, severity)| r.name() == "approx-constant"
                    && *severity == AllowWarnDeny::Deny)
        );
        assert!(
            allow_and_override_config
                .rules()
                .iter()
                .any(|(r, severity)| r.name() == "no-null" && *severity == AllowWarnDeny::Deny)
        );
    }

    #[test]
    fn test_extends_invalid() {
        let invalid_config = {
            let mut external_plugin_store = ExternalPluginStore::default();
            ConfigStoreBuilder::from_oxlintrc(
                true,
                Oxlintrc::from_file(&PathBuf::from(
                    "fixtures/extends_config/extends_invalid_config.json",
                ))
                .unwrap(),
                None,
                &mut external_plugin_store,
            )
        };
        let err = invalid_config.unwrap_err();
        assert!(matches!(err, ConfigBuilderError::InvalidConfigFile { .. }));
        if let ConfigBuilderError::InvalidConfigFile { file, reason } = err {
            assert!(file.ends_with("invalid_config.json"));
            assert!(reason.contains("Failed to parse"));
        }
    }

    #[test]
    fn test_extends_plugins() {
        // Test 1: Default plugins when none are specified
        let default_config = config_store_from_str(
            r#"
            {
                "rules": {}
            }
            "#,
        );
        // Check that default plugins are correctly set
        assert_eq!(default_config.plugins(), LintPlugins::default());

        // Test 2: Parent config with explicitly specified plugins
        let parent_config = config_store_from_str(
            r#"
            {
                "plugins": ["react", "typescript"]
            }
            "#,
        );
        assert_eq!(parent_config.plugins(), LintPlugins::REACT | LintPlugins::TYPESCRIPT);

        // Test 3: Child config that extends parent without specifying plugins
        // Should inherit parent's plugins
        let child_no_plugins_config =
            config_store_from_path("fixtures/extends_config/plugins/child_no_plugins.json");
        assert_eq!(child_no_plugins_config.plugins(), LintPlugins::REACT | LintPlugins::TYPESCRIPT);

        // Test 4: Child config that extends parent and specifies additional plugins
        // Should have parent's plugins plus its own
        let child_with_plugins_config =
            config_store_from_path("fixtures/extends_config/plugins/child_with_plugins.json");
        assert_eq!(
            child_with_plugins_config.plugins(),
            LintPlugins::REACT | LintPlugins::TYPESCRIPT | LintPlugins::JEST
        );

        // Test 5: Empty plugins array should result in empty plugins
        let empty_plugins_config = config_store_from_str(
            r#"
            {
                "plugins": []
            }
            "#,
        );
        assert_eq!(empty_plugins_config.plugins(), LintPlugins::empty());

        // Test 6: Extending multiple config files with plugins
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
        assert!(config.plugins().contains(LintPlugins::JEST));
        assert!(config.plugins().contains(LintPlugins::REACT));

        // Test 7: Adding more plugins to extended configs
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

        // Test 8: Extending a config with a plugin is the same as adding it directly
        let plugin_config = config_store_from_str(r#"{ "plugins": ["jest", "react"] }"#);
        let extends_plugin_config = config_store_from_str(
            r#"
            {
                "extends": [
                    "fixtures/extends_config/plugins/jest.json",
                    "fixtures/extends_config/plugins/react.json"
                ]
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

    fn config_store_from_path(path: &str) -> Config {
        let mut external_plugin_store = ExternalPluginStore::default();
        ConfigStoreBuilder::from_oxlintrc(
            true,
            Oxlintrc::from_file(&PathBuf::from(path)).unwrap(),
            None,
            &mut external_plugin_store,
        )
        .unwrap()
        .build(&external_plugin_store)
        .unwrap()
    }

    fn config_store_from_str(s: &str) -> Config {
        let mut external_plugin_store = ExternalPluginStore::default();
        ConfigStoreBuilder::from_oxlintrc(
            true,
            serde_json::from_str(s).unwrap(),
            None,
            &mut external_plugin_store,
        )
        .unwrap()
        .build(&external_plugin_store)
        .unwrap()
    }

    #[test]
    fn test_extends_scoped_package_subpath() {
        let config =
            config_store_from_path("fixtures/extends_config/packages/extends_scoped_package.json");
        assert!(config.rules().iter().any(|(rule, awd)| rule.plugin_name() == "eslint"
            && rule.name() == "no-console"
            && awd == &AllowWarnDeny::Warn));
    }

    #[test]
    fn test_extends_scoped_package_root() {
        // Create a config on the fly which extends the package root
        let config = config_store_from_str(
            r#"{
                "extends": ["fixtures/extends_config/packages/extends_scoped_package.json"],
                "rules": {}
            }"#,
        );
        // The nested config extends @acme/x via package root (".") exports
        // Verify inherited rule is present
        assert!(config.rules().iter().any(|(rule, awd)| rule.plugin_name() == "eslint"
            && rule.name() == "no-console"
            && awd == &AllowWarnDeny::Warn));
    }
}
