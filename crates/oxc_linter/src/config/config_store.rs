use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use rustc_hash::FxHashMap;

use crate::{
    AllowWarnDeny, LintPlugins,
    external_plugin_store::{ExternalPluginStore, ExternalRuleId},
    rules::{RULES, RuleEnum},
};

use super::{
    BuiltinLintPlugins, LintConfig, OxlintEnv, OxlintGlobals, categories::OxlintCategories,
    overrides::GlobSet,
};

// TODO: support `categories` et. al. in overrides.
#[derive(Debug)]
pub struct ResolvedLinterState {
    // TODO: Arc + Vec -> SyncVec? It would save a pointer dereference.
    pub rules: Arc<[(RuleEnum, AllowWarnDeny)]>,
    pub config: Arc<LintConfig>,

    pub external_rules: Arc<[(ExternalRuleId, AllowWarnDeny)]>,
}

impl Clone for ResolvedLinterState {
    fn clone(&self) -> Self {
        Self {
            rules: Arc::clone(&self.rules),
            config: Arc::clone(&self.config),
            external_rules: Arc::clone(&self.external_rules),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ResolvedOxlintOverrides(Vec<ResolvedOxlintOverride>);

impl ResolvedOxlintOverrides {
    pub(crate) fn new(overrides: Vec<ResolvedOxlintOverride>) -> Self {
        Self(overrides)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, ResolvedOxlintOverride> {
        self.0.iter()
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedOxlintOverride {
    pub files: GlobSet,
    pub env: Option<OxlintEnv>,
    pub globals: Option<OxlintGlobals>,
    pub plugins: Option<LintPlugins>,
    pub rules: ResolvedOxlintOverrideRules,
}

#[derive(Debug, Clone)]
pub struct ResolvedOxlintOverrideRules {
    pub(crate) builtin_rules: Vec<(RuleEnum, AllowWarnDeny)>,
    pub(crate) external_rules: Vec<(ExternalRuleId, AllowWarnDeny)>,
}

#[derive(Debug, Clone)]
pub struct Config {
    /// The basic linter state for this configuration.
    /// For files that match no overrides, this lint config will be used.
    pub(crate) base: ResolvedLinterState,

    /// Used as the base config for applying overrides
    /// NOTE: `AllowWarnDeny::Allow` should exist here to allow us to correctly
    ///       keep a rule disabled when an override is applied with a different plugin set
    pub(crate) base_rules: Vec<(RuleEnum, AllowWarnDeny)>,

    /// Categories specified at the root. This is used to resolve which rules
    /// should be enabled when a different plugin is enabled as part of an override.
    pub(crate) categories: OxlintCategories,

    /// An optional set of overrides to apply to the base state depending on the file being linted.
    pub(crate) overrides: ResolvedOxlintOverrides,
}

impl Config {
    pub fn new(
        rules: Vec<(RuleEnum, AllowWarnDeny)>,
        mut external_rules: Vec<(ExternalRuleId, AllowWarnDeny)>,
        categories: OxlintCategories,
        config: LintConfig,
        overrides: ResolvedOxlintOverrides,
    ) -> Self {
        Config {
            base: ResolvedLinterState {
                rules: Arc::from(
                    rules
                        .iter()
                        .filter(|(_, severity)| severity.is_warn_deny())
                        .cloned()
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                ),
                config: Arc::new(config),
                external_rules: Arc::from({
                    external_rules.retain(|(_, sev)| sev.is_warn_deny());
                    external_rules.into_boxed_slice()
                }),
            },
            base_rules: rules,
            categories,
            overrides,
        }
    }

    pub fn plugins(&self) -> &LintPlugins {
        &self.base.config.plugins
    }

    pub fn rules(&self) -> &Arc<[(RuleEnum, AllowWarnDeny)]> {
        &self.base.rules
    }

    pub fn number_of_rules(&self) -> usize {
        self.base.rules.len()
    }

    pub fn apply_overrides(&self, path: &Path) -> ResolvedLinterState {
        if self.overrides.is_empty() {
            return self.base.clone();
        }

        let relative_path = self
            .base
            .config
            .path
            .as_ref()
            .and_then(|config_path| {
                config_path.parent().map(|parent| path.strip_prefix(parent).unwrap_or(path))
            })
            .unwrap_or(path);

        let overrides_to_apply =
            self.overrides.iter().filter(|config| config.files.is_match(relative_path));

        let mut overrides_to_apply = overrides_to_apply.peekable();

        if overrides_to_apply.peek().is_none() {
            return self.base.clone();
        }

        let mut env = self.base.config.env.clone();
        let mut globals = self.base.config.globals.clone();
        let mut plugins = self.base.config.plugins.clone();

        for override_config in overrides_to_apply.clone() {
            if let Some(override_plugins) = &override_config.plugins {
                plugins.builtin |= override_plugins.builtin;
                for p in &override_plugins.external {
                    plugins.external.insert(p.clone());
                }
            }
        }

        let mut rules = self
            .base_rules
            .iter()
            .filter(|(rule, _)| {
                plugins.builtin.contains(BuiltinLintPlugins::from(rule.plugin_name()))
            })
            .cloned()
            .collect::<FxHashMap<_, _>>();

        let all_rules = RULES
            .iter()
            .filter(|rule| plugins.builtin.contains(BuiltinLintPlugins::from(rule.plugin_name())))
            .cloned()
            .collect::<Vec<_>>();

        let mut external_rules = FxHashMap::default();

        for override_config in overrides_to_apply {
            if let Some(override_plugins) = &override_config.plugins {
                if *override_plugins != plugins {
                    for (rule, severity) in all_rules.iter().filter_map(|rule| {
                        self.categories
                            .get(&rule.category())
                            .map(|severity| (rule.clone(), severity))
                    }) {
                        rules.entry(rule).or_insert(*severity);
                    }
                }
            }

            for (rule, severity) in &override_config.rules.builtin_rules {
                if *severity == AllowWarnDeny::Allow {
                    rules.remove(rule);
                } else {
                    rules.insert(rule.clone(), *severity);
                }
            }

            for (external_rule_id, severity) in &override_config.rules.external_rules {
                external_rules.insert(*external_rule_id, *severity);
            }

            if let Some(override_env) = &override_config.env {
                override_env.override_envs(&mut env);
            }

            if let Some(override_globals) = &override_config.globals {
                override_globals.override_globals(&mut globals);
            }
        }

        let config: Arc<LintConfig> = if plugins == self.base.config.plugins
            && env == self.base.config.env
            && globals == self.base.config.globals
        {
            Arc::clone(&self.base.config)
        } else {
            let mut config = (*self.base.config).clone();

            config.plugins = plugins;
            config.env = env;
            config.globals = globals;
            Arc::new(config)
        };

        let rules =
            rules.into_iter().filter(|(_, severity)| severity.is_warn_deny()).collect::<Vec<_>>();

        let external_rules = external_rules
            .into_iter()
            .filter(|(_, severity)| severity.is_warn_deny())
            .collect::<Vec<_>>();

        ResolvedLinterState {
            rules: Arc::from(rules.into_boxed_slice()),
            config,
            external_rules: Arc::from(external_rules.into_boxed_slice()),
        }
    }
}

/// Stores the configuration state for the linter including:
/// 1. the root configuration (base)
/// 2. any nested configurations (`nested_configs`)
///
/// If an explicit config has been provided `-c config.json`, then `nested_configs` will be empty
#[derive(Debug, Clone)]
pub struct ConfigStore {
    base: Config,
    nested_configs: FxHashMap<PathBuf, Config>,
    external_plugin_store: Arc<ExternalPluginStore>,
}

impl ConfigStore {
    pub fn new(
        base_config: Config,
        nested_configs: FxHashMap<PathBuf, Config>,
        external_plugin_store: ExternalPluginStore,
    ) -> Self {
        Self {
            base: base_config,
            nested_configs,
            external_plugin_store: Arc::new(external_plugin_store),
        }
    }

    pub fn number_of_rules(&self) -> Option<usize> {
        self.nested_configs.is_empty().then_some(self.base.base.rules.len())
    }

    pub fn rules(&self) -> &Arc<[(RuleEnum, AllowWarnDeny)]> {
        &self.base.base.rules
    }

    pub fn plugins(&self) -> &LintPlugins {
        &self.base.base.config.plugins
    }

    pub(crate) fn resolve(&self, path: &Path) -> ResolvedLinterState {
        let resolved_config = if self.nested_configs.is_empty() {
            &self.base
        } else if let Some(config) = self.get_nearest_config(path) {
            config
        } else {
            &self.base
        };

        Config::apply_overrides(resolved_config, path)
    }

    fn get_nearest_config(&self, path: &Path) -> Option<&Config> {
        // TODO(perf): should we cache the computed nearest config for every directory,
        // so we don't have to recompute it for every file?
        let mut current = path.parent();
        while let Some(dir) = current {
            if let Some(config) = self.nested_configs.get(dir) {
                return Some(config);
            }
            current = dir.parent();
        }
        None
    }

    #[cfg_attr(not(all(feature = "oxlint2", not(feature = "disable_oxlint2"))), expect(dead_code))]
    pub(crate) fn resolve_plugin_rule_names(
        &self,
        external_rule_id: ExternalRuleId,
    ) -> (/* plugin name */ &str, /* rule name */ &str) {
        self.external_plugin_store.resolve_plugin_rule_names(external_rule_id)
    }
}

#[cfg(test)]
mod test {
    use rustc_hash::{FxHashMap, FxHashSet};

    use super::{ConfigStore, ResolvedOxlintOverrides};
    use crate::{
        AllowWarnDeny, BuiltinLintPlugins, ExternalPluginStore, LintPlugins, RuleEnum,
        config::{
            LintConfig, OxlintEnv, OxlintGlobals, OxlintSettings,
            categories::OxlintCategories,
            config_store::{Config, ResolvedOxlintOverride, ResolvedOxlintOverrideRules},
            overrides::GlobSet,
        },
        rules::{EslintNoUnusedVars, TypescriptNoExplicitAny},
    };

    macro_rules! from_json {
        ($json:tt) => {
            serde_json::from_value(serde_json::json!($json)).unwrap()
        };
    }

    #[expect(clippy::default_trait_access)]
    fn no_explicit_any() -> (RuleEnum, AllowWarnDeny) {
        (RuleEnum::TypescriptNoExplicitAny(Default::default()), AllowWarnDeny::Warn)
    }

    /// an empty ruleset is a no-op
    #[test]
    fn test_no_rules() {
        let base_rules = vec![no_explicit_any()];
        let overrides = ResolvedOxlintOverrides::new(vec![ResolvedOxlintOverride {
            env: None,
            files: GlobSet::new(vec!["*.test.{ts,tsx}"]).unwrap(),
            plugins: None,
            globals: None,
            rules: ResolvedOxlintOverrideRules { builtin_rules: vec![], external_rules: vec![] },
        }]);
        let store = ConfigStore::new(
            Config::new(
                base_rules,
                vec![],
                OxlintCategories::default(),
                LintConfig::default(),
                overrides,
            ),
            FxHashMap::default(),
            ExternalPluginStore::default(),
        );

        let rules_for_source_file = store.resolve("App.tsx".as_ref());
        let rules_for_test_file = store.resolve("App.test.tsx".as_ref());

        assert_eq!(rules_for_source_file.rules.len(), 1);
        assert_eq!(rules_for_test_file.rules.len(), 1);
        assert_eq!(rules_for_test_file.rules[0].0.id(), rules_for_source_file.rules[0].0.id());
    }

    /// adding plugins but no rules is a no-op
    #[test]
    fn test_no_rules_and_new_plugins() {
        let base_rules = vec![no_explicit_any()];
        let overrides = ResolvedOxlintOverrides::new(vec![ResolvedOxlintOverride {
            env: None,
            files: GlobSet::new(vec!["*.test.{ts,tsx}"]).unwrap(),
            plugins: Some(LintPlugins::new(
                BuiltinLintPlugins::REACT
                    .union(BuiltinLintPlugins::TYPESCRIPT)
                    .union(BuiltinLintPlugins::UNICORN)
                    .union(BuiltinLintPlugins::OXC)
                    .union(BuiltinLintPlugins::JSX_A11Y),
                FxHashSet::default(),
            )),
            globals: None,
            rules: ResolvedOxlintOverrideRules { builtin_rules: vec![], external_rules: vec![] },
        }]);
        let store = ConfigStore::new(
            Config::new(
                base_rules,
                vec![],
                OxlintCategories::default(),
                LintConfig::default(),
                overrides,
            ),
            FxHashMap::default(),
            ExternalPluginStore::default(),
        );

        let rules_for_source_file = store.resolve("App.tsx".as_ref());
        let rules_for_test_file = store.resolve("App.test.tsx".as_ref());

        assert_eq!(rules_for_source_file.rules.len(), 1);
        assert_eq!(rules_for_test_file.rules.len(), 1);
        assert_eq!(rules_for_test_file.rules[0].0.id(), rules_for_source_file.rules[0].0.id());
    }

    #[test]
    fn test_remove_rule() {
        let base_rules = vec![no_explicit_any()];
        let overrides = ResolvedOxlintOverrides::new(vec![ResolvedOxlintOverride {
            env: None,
            files: GlobSet::new(vec!["*.test.{ts,tsx}"]).unwrap(),
            plugins: None,
            globals: None,
            rules: ResolvedOxlintOverrideRules {
                builtin_rules: vec![(
                    RuleEnum::TypescriptNoExplicitAny(TypescriptNoExplicitAny::default()),
                    AllowWarnDeny::Allow,
                )],
                external_rules: vec![],
            },
        }]);

        let store = ConfigStore::new(
            Config::new(
                base_rules,
                vec![],
                OxlintCategories::default(),
                LintConfig::default(),
                overrides,
            ),
            FxHashMap::default(),
            ExternalPluginStore::default(),
        );
        assert_eq!(store.number_of_rules(), Some(1));

        let rules_for_source_file = store.resolve("App.tsx".as_ref());
        assert_eq!(rules_for_source_file.rules.len(), 1);

        assert!(store.resolve("App.test.tsx".as_ref()).rules.is_empty());
        assert!(store.resolve("App.test.ts".as_ref()).rules.is_empty());
    }

    #[test]
    fn test_add_rule() {
        let base_rules = vec![no_explicit_any()];
        let overrides = ResolvedOxlintOverrides::new(vec![ResolvedOxlintOverride {
            env: None,
            files: GlobSet::new(vec!["src/**/*.{ts,tsx}"]).unwrap(),
            plugins: None,
            globals: None,
            rules: ResolvedOxlintOverrideRules {
                builtin_rules: vec![(
                    RuleEnum::EslintNoUnusedVars(EslintNoUnusedVars::default()),
                    AllowWarnDeny::Warn,
                )],
                external_rules: vec![],
            },
        }]);

        let store = ConfigStore::new(
            Config::new(
                base_rules,
                vec![],
                OxlintCategories::default(),
                LintConfig::default(),
                overrides,
            ),
            FxHashMap::default(),
            ExternalPluginStore::default(),
        );
        assert_eq!(store.number_of_rules(), Some(1));

        assert_eq!(store.resolve("App.tsx".as_ref()).rules.len(), 1);
        assert_eq!(store.resolve("src/App.tsx".as_ref()).rules.len(), 2);
        assert_eq!(store.resolve("src/App.ts".as_ref()).rules.len(), 2);
        assert_eq!(store.resolve("src/foo/bar/baz/App.tsx".as_ref()).rules.len(), 2);
        assert_eq!(store.resolve("src/foo/bar/baz/App.spec.tsx".as_ref()).rules.len(), 2);
    }

    #[test]
    fn test_change_rule_severity() {
        let base_rules = vec![no_explicit_any()];
        let overrides = ResolvedOxlintOverrides::new(vec![ResolvedOxlintOverride {
            env: None,
            files: GlobSet::new(vec!["src/**/*.{ts,tsx}"]).unwrap(),
            plugins: None,
            globals: None,
            rules: ResolvedOxlintOverrideRules {
                builtin_rules: vec![(
                    RuleEnum::TypescriptNoExplicitAny(TypescriptNoExplicitAny::default()),
                    AllowWarnDeny::Deny,
                )],
                external_rules: vec![],
            },
        }]);

        let store = ConfigStore::new(
            Config::new(
                base_rules,
                vec![],
                OxlintCategories::default(),
                LintConfig::default(),
                overrides,
            ),
            FxHashMap::default(),
            ExternalPluginStore::default(),
        );
        assert_eq!(store.number_of_rules(), Some(1));

        let app = store.resolve("App.tsx".as_ref()).rules;
        assert_eq!(app.len(), 1);
        assert_eq!(app[0].1, AllowWarnDeny::Warn);

        let src_app = store.resolve("src/App.tsx".as_ref()).rules;
        assert_eq!(src_app.len(), 1);
        assert_eq!(src_app[0].1, AllowWarnDeny::Deny);
    }

    #[test]
    fn test_add_plugins() {
        let base_config = LintConfig {
            plugins: LintPlugins::new(BuiltinLintPlugins::IMPORT, FxHashSet::default()),
            env: OxlintEnv::default(),
            settings: OxlintSettings::default(),
            globals: OxlintGlobals::default(),
            path: None,
        };
        let overrides = ResolvedOxlintOverrides::new(vec![
            ResolvedOxlintOverride {
                env: None,
                files: GlobSet::new(vec!["*.jsx", "*.tsx"]).unwrap(),
                plugins: Some(LintPlugins::new(BuiltinLintPlugins::REACT, FxHashSet::default())),
                globals: None,
                rules: ResolvedOxlintOverrideRules {
                    builtin_rules: vec![],
                    external_rules: vec![],
                },
            },
            ResolvedOxlintOverride {
                env: None,
                files: GlobSet::new(vec!["*.ts", "*.tsx"]).unwrap(),
                plugins: Some(LintPlugins::new(
                    BuiltinLintPlugins::TYPESCRIPT,
                    FxHashSet::default(),
                )),
                globals: None,
                rules: ResolvedOxlintOverrideRules {
                    builtin_rules: vec![],
                    external_rules: vec![],
                },
            },
        ]);

        let store = ConfigStore::new(
            Config::new(vec![], vec![], OxlintCategories::default(), base_config, overrides),
            FxHashMap::default(),
            ExternalPluginStore::default(),
        );

        assert_eq!(store.base.base.config.plugins.builtin, BuiltinLintPlugins::IMPORT);

        let app = store.resolve("other.mjs".as_ref()).config;
        assert_eq!(app.plugins.builtin, BuiltinLintPlugins::IMPORT);

        let app = store.resolve("App.jsx".as_ref()).config;
        assert_eq!(app.plugins.builtin, BuiltinLintPlugins::IMPORT | BuiltinLintPlugins::REACT);

        let app = store.resolve("App.ts".as_ref()).config;
        assert_eq!(
            app.plugins.builtin,
            BuiltinLintPlugins::IMPORT | BuiltinLintPlugins::TYPESCRIPT
        );

        let app = store.resolve("App.tsx".as_ref()).config;
        assert_eq!(
            app.plugins.builtin,
            BuiltinLintPlugins::IMPORT | BuiltinLintPlugins::REACT | BuiltinLintPlugins::TYPESCRIPT
        );
    }

    #[test]
    fn test_add_env() {
        let base_config = LintConfig {
            env: OxlintEnv::default(),
            plugins: BuiltinLintPlugins::ESLINT.into(),
            settings: OxlintSettings::default(),
            globals: OxlintGlobals::default(),
            path: None,
        };
        let overrides = ResolvedOxlintOverrides::new(vec![ResolvedOxlintOverride {
            env: Some(OxlintEnv::from_iter(["es2024".to_string()])),
            files: GlobSet::new(vec!["*.tsx"]).unwrap(),
            plugins: None,
            globals: None,
            rules: ResolvedOxlintOverrideRules { builtin_rules: vec![], external_rules: vec![] },
        }]);

        let store = ConfigStore::new(
            Config::new(vec![], vec![], OxlintCategories::default(), base_config, overrides),
            FxHashMap::default(),
            ExternalPluginStore::default(),
        );
        assert!(!store.base.base.config.env.contains("React"));

        let app = store.resolve("App.tsx".as_ref()).config;
        assert!(app.env.contains("es2024"));
    }

    #[test]
    fn test_replace_env() {
        let base_config = LintConfig {
            env: OxlintEnv::from_iter(["es2024".into()]),
            plugins: BuiltinLintPlugins::ESLINT.into(),
            settings: OxlintSettings::default(),
            globals: OxlintGlobals::default(),
            path: None,
        };
        let overrides = ResolvedOxlintOverrides::new(vec![ResolvedOxlintOverride {
            files: GlobSet::new(vec!["*.tsx"]).unwrap(),
            env: Some(from_json!({ "es2024": false })),
            plugins: None,
            globals: None,
            rules: ResolvedOxlintOverrideRules { builtin_rules: vec![], external_rules: vec![] },
        }]);

        let store = ConfigStore::new(
            Config::new(vec![], vec![], OxlintCategories::default(), base_config, overrides),
            FxHashMap::default(),
            ExternalPluginStore::default(),
        );
        assert!(store.base.base.config.env.contains("es2024"));

        let app = store.resolve("App.tsx".as_ref()).config;
        assert!(!app.env.contains("es2024"));
    }

    #[test]
    fn test_add_globals() {
        let base_config = LintConfig {
            env: OxlintEnv::default(),
            plugins: BuiltinLintPlugins::ESLINT.into(),
            settings: OxlintSettings::default(),
            globals: OxlintGlobals::default(),
            path: None,
        };

        let overrides = ResolvedOxlintOverrides::new(vec![ResolvedOxlintOverride {
            files: GlobSet::new(vec!["*.tsx"]).unwrap(),
            env: None,
            plugins: None,
            globals: Some(from_json!({ "React": "readonly", "Secret": "writeable" })),
            rules: ResolvedOxlintOverrideRules { builtin_rules: vec![], external_rules: vec![] },
        }]);

        let store = ConfigStore::new(
            Config::new(vec![], vec![], OxlintCategories::default(), base_config, overrides),
            FxHashMap::default(),
            ExternalPluginStore::default(),
        );
        assert!(!store.base.base.config.globals.is_enabled("React"));
        assert!(!store.base.base.config.globals.is_enabled("Secret"));

        let app = store.resolve("App.tsx".as_ref()).config;
        assert!(app.globals.is_enabled("React"));
        assert!(app.globals.is_enabled("Secret"));
    }

    #[test]
    fn test_replace_globals() {
        let base_config = LintConfig {
            env: OxlintEnv::default(),
            plugins: BuiltinLintPlugins::ESLINT.into(),
            settings: OxlintSettings::default(),
            globals: from_json!({
                "React": "readonly",
                "Secret": "writeable"
            }),
            path: None,
        };

        let overrides = ResolvedOxlintOverrides::new(vec![ResolvedOxlintOverride {
            files: GlobSet::new(vec!["*.tsx"]).unwrap(),
            env: None,
            plugins: None,
            globals: Some(from_json!({ "React": "off", "Secret": "off" })),
            rules: ResolvedOxlintOverrideRules { builtin_rules: vec![], external_rules: vec![] },
        }]);

        let store = ConfigStore::new(
            Config::new(vec![], vec![], OxlintCategories::default(), base_config, overrides),
            FxHashMap::default(),
            ExternalPluginStore::default(),
        );
        assert!(store.base.base.config.globals.is_enabled("React"));
        assert!(store.base.base.config.globals.is_enabled("Secret"));

        let app = store.resolve("App.tsx".as_ref()).config;
        assert!(!app.globals.is_enabled("React"));
        assert!(!app.globals.is_enabled("Secret"));
    }
}
