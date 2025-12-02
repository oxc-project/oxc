use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use rustc_hash::FxHashMap;

use crate::{
    AllowWarnDeny,
    external_plugin_store::{ExternalOptionsId, ExternalPluginStore, ExternalRuleId},
    rules::{RULES, RuleEnum},
};

use super::{
    LintConfig, LintPlugins, OxlintEnv, OxlintGlobals, categories::OxlintCategories,
    overrides::GlobSet,
};

// TODO: support `categories` et. al. in overrides.
#[derive(Debug, Clone)]
pub struct ResolvedLinterState {
    // TODO: Arc + Vec -> SyncVec? It would save a pointer dereference.
    pub rules: Arc<[(RuleEnum, AllowWarnDeny)]>,
    pub config: Arc<LintConfig>,

    pub external_rules: Arc<[(ExternalRuleId, ExternalOptionsId, AllowWarnDeny)]>,
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
    pub(crate) external_rules: Vec<(ExternalRuleId, ExternalOptionsId, AllowWarnDeny)>,
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
        mut external_rules: Vec<(ExternalRuleId, ExternalOptionsId, AllowWarnDeny)>,
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
                    external_rules.retain(|(_, _, sev)| sev.is_warn_deny());
                    external_rules.into_boxed_slice()
                }),
            },
            base_rules: rules,
            categories,
            overrides,
        }
    }

    pub fn plugins(&self) -> LintPlugins {
        self.base.config.plugins
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

        let path = relative_path.to_string_lossy();
        let overrides_to_apply =
            self.overrides.iter().filter(|config| config.files.is_match(path.as_ref()));

        let mut overrides_to_apply = overrides_to_apply.peekable();

        if overrides_to_apply.peek().is_none() {
            return self.base.clone();
        }

        let mut env = self.base.config.env.clone();
        let mut globals = self.base.config.globals.clone();
        let mut plugins = self.base.config.plugins;
        let settings = self.base.config.settings.clone();

        for override_config in overrides_to_apply.clone() {
            if let Some(override_plugins) = override_config.plugins {
                plugins |= override_plugins;
            }
        }

        let mut rules = self
            .base_rules
            .iter()
            .filter(|(rule, _)| {
                LintPlugins::try_from(rule.plugin_name())
                    .is_ok_and(|plugin| plugins.contains(plugin))
            })
            .cloned()
            .collect::<FxHashMap<_, _>>();

        let all_rules = RULES
            .iter()
            .filter(|rule| {
                LintPlugins::try_from(rule.plugin_name())
                    .is_ok_and(|plugin| plugins.contains(plugin))
            })
            .cloned()
            .collect::<Vec<_>>();

        // Build a hashmap of existing external rules keyed by rule id with value (options_id, severity)
        let mut external_rules = self
            .base
            .external_rules
            .iter()
            .map(|&(rule_id, options_id, severity)| (rule_id, (options_id, severity)))
            .collect::<FxHashMap<_, _>>();

        // Track which plugins have already had their category rules applied.
        // Start with the root plugins since they already have categories applied in base_rules.
        let mut configured_plugins = self.base.config.plugins;

        for override_config in overrides_to_apply {
            if let Some(override_plugins) = override_config.plugins
                && override_plugins != plugins
            {
                // Only apply categories to plugins that:
                // 1. Are in the current accumulated plugin set
                // 2. Have NOT been configured yet (not in root or previous overrides)
                let unconfigured_plugins = plugins & !configured_plugins;

                if !unconfigured_plugins.is_empty() {
                    for (rule, severity) in all_rules.iter().filter_map(|rule| {
                        let rule_plugin = LintPlugins::try_from(rule.plugin_name())
                            .unwrap_or(LintPlugins::empty());
                        // Only apply categories to rules from unconfigured plugins
                        if unconfigured_plugins.contains(rule_plugin) {
                            self.categories
                                .get(&rule.category())
                                .map(|severity| (rule.clone(), severity))
                        } else {
                            None
                        }
                    }) {
                        rules.entry(rule).or_insert(*severity);
                    }
                    // Mark these plugins as configured
                    configured_plugins |= unconfigured_plugins;
                }
            }

            for (rule, severity) in &override_config.rules.builtin_rules {
                if *severity == AllowWarnDeny::Allow {
                    rules.remove(rule);
                } else {
                    let _ = rules.remove(rule);
                    rules.insert(rule.clone(), *severity);
                }
            }

            for (external_rule_id, options_id, severity) in &override_config.rules.external_rules {
                external_rules.insert(*external_rule_id, (*options_id, *severity));
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
            && settings == self.base.config.settings
        {
            Arc::clone(&self.base.config)
        } else {
            let mut config = (*self.base.config).clone();

            config.plugins = plugins;
            config.env = env;
            config.globals = globals;
            config.settings = settings;
            Arc::new(config)
        };

        let rules =
            rules.into_iter().filter(|(_, severity)| severity.is_warn_deny()).collect::<Vec<_>>();

        let external_rules = external_rules
            .into_iter()
            .filter(|(_, (_, severity))| severity.is_warn_deny())
            .map(|(rule_id, (options_id, severity))| (rule_id, options_id, severity))
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

    /// Returns the number of rules, optionally filtering out tsgolint rules if type_aware_enabled is false.
    pub fn number_of_rules(&self, type_aware_enabled: bool) -> Option<usize> {
        if !self.nested_configs.is_empty() {
            return None;
        }
        let count = if type_aware_enabled {
            self.base.base.rules.len()
        } else {
            self.base.base.rules.iter().filter(|(rule, _)| !rule.is_tsgolint_rule()).count()
        };
        Some(count)
    }

    pub fn rules(&self) -> &Arc<[(RuleEnum, AllowWarnDeny)]> {
        &self.base.base.rules
    }

    pub fn plugins(&self) -> LintPlugins {
        self.base.base.config.plugins
    }

    pub(crate) fn get_related_config(&self, path: &Path) -> &Config {
        if self.nested_configs.is_empty() {
            &self.base
        } else if let Some(config) = self.get_nearest_config(path) {
            config
        } else {
            &self.base
        }
    }

    // NOTE: This function is not crate visible because it is used in `oxlint` as well to resolve configs
    // for the `tsgolint` linter.
    pub fn resolve(&self, path: &Path) -> ResolvedLinterState {
        Config::apply_overrides(self.get_related_config(path), path)
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

    pub(crate) fn resolve_plugin_rule_names(
        &self,
        external_rule_id: ExternalRuleId,
    ) -> (/* plugin name */ &str, /* rule name */ &str) {
        self.external_plugin_store.resolve_plugin_rule_names(external_rule_id)
    }

    pub fn external_plugin_store(&self) -> &ExternalPluginStore {
        &self.external_plugin_store
    }
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, str::FromStr};

    use rustc_hash::FxHashMap;
    use serde_json::Value;

    use super::{ConfigStore, ExternalRuleId, ResolvedOxlintOverrides};
    use crate::{
        AllowWarnDeny, ExternalOptionsId, ExternalPluginStore, LintPlugins, RuleCategory, RuleEnum,
        config::{
            LintConfig, OxlintEnv, OxlintGlobals, OxlintSettings,
            categories::OxlintCategories,
            config_store::{Config, ResolvedOxlintOverride, ResolvedOxlintOverrideRules},
            overrides::GlobSet,
        },
        rule::Rule,
        rules::{
            EslintCurly, EslintNoUnusedVars, ReactJsxFilenameExtension, TypescriptNoExplicitAny,
            TypescriptNoMisusedPromises,
        },
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
            files: GlobSet::new(vec!["*.test.{ts,tsx}"]),
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
            files: GlobSet::new(vec!["*.test.{ts,tsx}"]),
            plugins: Some(
                LintPlugins::REACT
                    | LintPlugins::TYPESCRIPT
                    | LintPlugins::UNICORN
                    | LintPlugins::OXC
                    | LintPlugins::JSX_A11Y,
            ),
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
            files: GlobSet::new(vec!["*.test.{ts,tsx}"]),
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
        assert_eq!(store.number_of_rules(false), Some(1));

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
            files: GlobSet::new(vec!["src/**/*.{ts,tsx}"]),
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
        assert_eq!(store.number_of_rules(false), Some(1));

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
            files: GlobSet::new(vec!["src/**/*.{ts,tsx}"]),
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
        assert_eq!(store.number_of_rules(false), Some(1));

        let app = store.resolve("App.tsx".as_ref()).rules;
        assert_eq!(app.len(), 1);
        assert_eq!(app[0].1, AllowWarnDeny::Warn);

        let src_app = store.resolve("src/App.tsx".as_ref()).rules;
        assert_eq!(src_app.len(), 1);
        assert_eq!(src_app[0].1, AllowWarnDeny::Deny);
    }

    #[test]
    fn test_add_plugins() {
        let base_config = LintConfig { plugins: LintPlugins::IMPORT, ..Default::default() };
        let overrides = ResolvedOxlintOverrides::new(vec![
            ResolvedOxlintOverride {
                env: None,
                files: GlobSet::new(vec!["*.jsx", "*.tsx"]),
                plugins: Some(LintPlugins::REACT),
                globals: None,
                rules: ResolvedOxlintOverrideRules {
                    builtin_rules: vec![],
                    external_rules: vec![],
                },
            },
            ResolvedOxlintOverride {
                env: None,
                files: GlobSet::new(vec!["*.ts", "*.tsx"]),
                plugins: Some(LintPlugins::TYPESCRIPT),
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

        assert_eq!(store.base.base.config.plugins, LintPlugins::IMPORT);

        let app = store.resolve("other.mjs".as_ref()).config;
        assert_eq!(app.plugins, LintPlugins::IMPORT);

        let app = store.resolve("App.jsx".as_ref()).config;
        assert_eq!(app.plugins, LintPlugins::IMPORT | LintPlugins::REACT);

        let app = store.resolve("App.ts".as_ref()).config;
        assert_eq!(app.plugins, LintPlugins::IMPORT | LintPlugins::TYPESCRIPT);

        let app = store.resolve("App.tsx".as_ref()).config;
        assert_eq!(app.plugins, LintPlugins::IMPORT | LintPlugins::REACT | LintPlugins::TYPESCRIPT);
    }

    #[test]
    fn test_add_env() {
        let base_config = LintConfig { plugins: LintPlugins::ESLINT, ..Default::default() };
        let overrides = ResolvedOxlintOverrides::new(vec![ResolvedOxlintOverride {
            env: Some(OxlintEnv::from_iter(["es2024".to_string()])),
            files: GlobSet::new(vec!["*.tsx"]),
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
        let base_config =
            LintConfig { env: OxlintEnv::from_iter(["es2024".into()]), ..Default::default() };
        let overrides = ResolvedOxlintOverrides::new(vec![ResolvedOxlintOverride {
            files: GlobSet::new(vec!["*.tsx"]),
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
        let base_config = LintConfig { plugins: LintPlugins::ESLINT, ..Default::default() };

        let overrides = ResolvedOxlintOverrides::new(vec![ResolvedOxlintOverride {
            files: GlobSet::new(vec!["*.tsx"]),
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
    fn test_external_rules_preserved_with_overrides() {
        // reproduction for https://github.com/oxc-project/oxc/issues/14504
        // the bug occurred due to a simple omission and the fix was simple.
        // this test is just to communicate what was going wrong and to avoid a regression.
        // i noticed js plugins aren't considered stable yet, so feel free to edit or remove this test

        let mut external_plugin_store = ExternalPluginStore::default();
        external_plugin_store.register_plugin(
            "./plugin.js".into(),
            "custom".into(),
            0,
            vec!["no-debugger".into()],
        );

        let rule_id = external_plugin_store.lookup_rule_id("custom", "no-debugger").unwrap();

        let overrides = ResolvedOxlintOverrides::new(vec![ResolvedOxlintOverride {
            files: GlobSet::new(vec!["*.ts"]),
            env: None,
            plugins: None,
            globals: None,
            rules: ResolvedOxlintOverrideRules { builtin_rules: vec![], external_rules: vec![] },
        }]);

        let store = ConfigStore::new(
            Config::new(
                vec![],
                vec![(rule_id, ExternalOptionsId::NONE, AllowWarnDeny::Deny)],
                OxlintCategories::default(),
                LintConfig::default(),
                overrides,
            ),
            FxHashMap::default(),
            external_plugin_store,
        );

        // Bug: external rules were lost when overrides matched
        let resolved = store.resolve("foo.ts".as_ref());
        assert_eq!(resolved.external_rules.len(), 1);
        assert_eq!(resolved.external_rules[0].0, rule_id);
    }

    #[test]
    fn test_replace_globals() {
        let base_config = LintConfig {
            plugins: LintPlugins::ESLINT,
            globals: from_json!({
                "React": "readonly",
                "Secret": "writeable"
            }),
            ..Default::default()
        };

        let overrides = ResolvedOxlintOverrides::new(vec![ResolvedOxlintOverride {
            files: GlobSet::new(vec!["*.tsx"]),
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

    #[test]
    fn test_override_rule_not_reset_by_later_override_with_different_plugins() {
        // This test reproduces the issue from https://github.com/oxc-project/oxc/issues/12859
        // When multiple overrides apply to a file and they have different plugins,
        // the later override should not reset rules that were explicitly set in earlier overrides.

        // Root config with react, typescript, unicorn plugins and restriction category
        let base_config = LintConfig {
            plugins: LintPlugins::REACT | LintPlugins::TYPESCRIPT | LintPlugins::UNICORN,
            env: OxlintEnv::default(),
            settings: OxlintSettings::default(),
            globals: OxlintGlobals::default(),
            path: None,
        };

        // Set up categories to enable restriction rules
        let mut categories = OxlintCategories::default();
        categories.insert(RuleCategory::Restriction, AllowWarnDeny::Warn);

        // Create overrides similar to the user's config
        let overrides = ResolvedOxlintOverrides::new(vec![
            // First override: typescript plugin for *.{ts,tsx,mts}
            ResolvedOxlintOverride {
                env: None,
                files: GlobSet::new(vec!["*.{ts,tsx,mts}"]),
                plugins: Some(LintPlugins::TYPESCRIPT),
                globals: None,
                rules: ResolvedOxlintOverrideRules {
                    builtin_rules: vec![],
                    external_rules: vec![],
                },
            },
            // Second override: react plugin for *.{ts,tsx} with jsx-filename-extension turned off
            ResolvedOxlintOverride {
                env: None,
                files: GlobSet::new(vec!["*.{ts,tsx}"]),
                plugins: Some(LintPlugins::REACT),
                globals: None,
                rules: ResolvedOxlintOverrideRules {
                    builtin_rules: vec![(
                        RuleEnum::ReactJsxFilenameExtension(ReactJsxFilenameExtension::default()),
                        AllowWarnDeny::Allow,
                    )],
                    external_rules: vec![],
                },
            },
            // Third override: unicorn plugin for *.{ts,tsx,mts}
            ResolvedOxlintOverride {
                env: None,
                files: GlobSet::new(vec!["*.{ts,tsx,mts}"]),
                plugins: Some(LintPlugins::UNICORN),
                globals: None,
                rules: ResolvedOxlintOverrideRules {
                    builtin_rules: vec![],
                    external_rules: vec![],
                },
            },
        ]);

        // Create base rules - jsx-filename-extension should be enabled by restriction category
        let base_rules = vec![(
            RuleEnum::ReactJsxFilenameExtension(ReactJsxFilenameExtension::default()),
            AllowWarnDeny::Warn,
        )];

        let store = ConfigStore::new(
            Config::new(base_rules, vec![], categories, base_config, overrides),
            FxHashMap::default(),
            ExternalPluginStore::default(),
        );

        // Resolve rules for a .tsx file
        let rules_for_tsx = store.resolve("App.tsx".as_ref());

        // The jsx-filename-extension rule should be disabled (Allow) because the second override
        // explicitly set it to Allow, and the third override should not reset it
        let jsx_filename_rule = rules_for_tsx
            .rules
            .iter()
            .find(|(rule, _)| matches!(rule, RuleEnum::ReactJsxFilenameExtension(_)));

        // This test should fail with the current implementation
        // The bug causes the rule to be re-enabled (Warn) instead of staying disabled (Allow)
        assert!(
            jsx_filename_rule.is_none(),
            "jsx-filename-extension should be disabled (not present in active rules)"
        );
    }

    #[test]
    fn test_categories_only_applied_to_new_plugins_not_in_root() {
        // Test that categories are only applied to plugins that weren't in the root config

        // Root config with only typescript plugin
        let base_config = LintConfig {
            plugins: (LintPlugins::TYPESCRIPT),
            env: OxlintEnv::default(),
            settings: OxlintSettings::default(),
            globals: OxlintGlobals::default(),
            path: None,
        };

        // Set up categories
        let mut categories = OxlintCategories::default();
        categories.insert(RuleCategory::Restriction, AllowWarnDeny::Warn);

        // Override adds react plugin (new plugin not in root)
        let overrides = ResolvedOxlintOverrides::new(vec![ResolvedOxlintOverride {
            env: None,
            files: GlobSet::new(vec!["*.tsx"]),
            plugins: Some(LintPlugins::REACT),
            globals: None,
            rules: ResolvedOxlintOverrideRules { builtin_rules: vec![], external_rules: vec![] },
        }]);

        let store = ConfigStore::new(
            Config::new(vec![], vec![], categories, base_config, overrides),
            FxHashMap::default(),
            ExternalPluginStore::default(),
        );

        // For .tsx files, react rules should be enabled by categories since react wasn't in root
        let rules_for_tsx = store.resolve("App.tsx".as_ref());

        // Check that react rules are present (categories were applied to the new plugin)
        let has_react_rules =
            rules_for_tsx.rules.iter().any(|(rule, _)| rule.plugin_name() == "react");

        assert!(has_react_rules, "React rules should be enabled by categories for new plugin");
    }

    #[test]
    fn test_rule_config_override_replaces_properly() {
        let base_rules = vec![(
            RuleEnum::EslintNoUnusedVars(EslintNoUnusedVars::default()),
            AllowWarnDeny::Deny,
        )];
        let override_rule =
            EslintNoUnusedVars::from_configuration(Value::from_str(r#"["local"]"#).unwrap());
        let overrides = ResolvedOxlintOverrides::new(vec![ResolvedOxlintOverride {
            env: None,
            files: GlobSet::new(vec!["*.tsx"]),
            plugins: None,
            globals: None,
            rules: ResolvedOxlintOverrideRules {
                builtin_rules: vec![(
                    RuleEnum::EslintNoUnusedVars(override_rule),
                    AllowWarnDeny::Deny,
                )],
                external_rules: vec![],
            },
        }]);

        let store = ConfigStore::new(
            Config::new(
                base_rules.clone(),
                vec![],
                OxlintCategories::default(),
                LintConfig::default(),
                overrides,
            ),
            FxHashMap::default(),
            ExternalPluginStore::default(),
        );

        assert!(
            format!("{:?}", base_rules[0].0)
                == format!(
                    "{:?}",
                    store
                        .resolve("app.ts".as_ref())
                        .rules
                        .iter()
                        .find(|(rule, _)| matches!(rule, RuleEnum::EslintNoUnusedVars(_)))
                        .unwrap()
                        .0
                )
        );
        assert!(
            format!("{:?}", base_rules[0].0)
                != format!(
                    "{:?}",
                    store
                        .resolve("app.tsx".as_ref())
                        .rules
                        .iter()
                        .find(|(rule, _)| matches!(rule, RuleEnum::EslintNoUnusedVars(_)))
                        .unwrap()
                        .0
                )
        );
    }

    #[test]
    fn test_categories_not_reapplied_to_root_plugins() {
        // Test that categories are NOT re-applied to plugins that were already in root

        // Root config with react plugin
        let base_config = LintConfig {
            plugins: (LintPlugins::REACT),
            env: OxlintEnv::default(),
            settings: OxlintSettings::default(),
            globals: OxlintGlobals::default(),
            path: None,
        };

        // Set up categories
        let mut categories = OxlintCategories::default();
        categories.insert(RuleCategory::Restriction, AllowWarnDeny::Warn);

        // Base rules with jsx-filename-extension disabled
        let base_rules = vec![(
            RuleEnum::ReactJsxFilenameExtension(ReactJsxFilenameExtension::default()),
            AllowWarnDeny::Allow, // Disabled at root
        )];

        // Override adds typescript plugin
        let overrides = ResolvedOxlintOverrides::new(vec![ResolvedOxlintOverride {
            env: None,
            files: GlobSet::new(vec!["*.tsx"]),
            plugins: Some(LintPlugins::TYPESCRIPT),
            globals: None,
            rules: ResolvedOxlintOverrideRules { builtin_rules: vec![], external_rules: vec![] },
        }]);

        let store = ConfigStore::new(
            Config::new(base_rules, vec![], categories, base_config, overrides),
            FxHashMap::default(),
            ExternalPluginStore::default(),
        );

        // For .tsx files, jsx-filename-extension should remain disabled
        let rules_for_tsx = store.resolve("App.tsx".as_ref());

        let jsx_filename_rule = rules_for_tsx
            .rules
            .iter()
            .find(|(rule, _)| matches!(rule, RuleEnum::ReactJsxFilenameExtension(_)));

        assert!(
            jsx_filename_rule.is_none(),
            "jsx-filename-extension should remain disabled (not re-enabled by categories)"
        );
    }

    #[test]
    fn test_number_of_rules() {
        let base_config = LintConfig::default();

        let base_rules = vec![
            (RuleEnum::EslintCurly(EslintCurly::default()), AllowWarnDeny::Deny),
            (
                RuleEnum::TypescriptNoMisusedPromises(TypescriptNoMisusedPromises::default()),
                AllowWarnDeny::Deny,
            ),
        ];

        let store = ConfigStore::new(
            Config::new(
                base_rules.clone(),
                vec![],
                OxlintCategories::default(),
                base_config.clone(),
                ResolvedOxlintOverrides::new(vec![]),
            ),
            FxHashMap::default(),
            ExternalPluginStore::default(),
        );

        let mut nested_configs = FxHashMap::default();
        nested_configs.insert(
            PathBuf::new(),
            Config::new(
                vec![],
                vec![],
                OxlintCategories::default(),
                base_config.clone(),
                ResolvedOxlintOverrides::new(vec![]),
            ),
        );

        let store_with_nested_configs = ConfigStore::new(
            Config::new(
                base_rules,
                vec![],
                OxlintCategories::default(),
                base_config,
                ResolvedOxlintOverrides::new(vec![]),
            ),
            nested_configs,
            ExternalPluginStore::default(),
        );

        assert_eq!(store.number_of_rules(false), Some(1));
        assert_eq!(store.number_of_rules(true), Some(2));
        assert_eq!(store_with_nested_configs.number_of_rules(false), None);
        assert_eq!(store_with_nested_configs.number_of_rules(true), None);
    }

    #[test]
    fn test_external_rule_options_override_precedence() {
        // Prepare external plugin store with a custom plugin and rule
        let mut store = ExternalPluginStore::new(true);
        store.register_plugin(
            PathBuf::from("path/to/custom"),
            "custom".to_string(),
            0,
            vec!["my-rule".to_string()],
        );

        // Base config has external rule with options A, severity warn
        let base_external_rule_id = store.lookup_rule_id("custom", "my-rule").unwrap();
        let base_options_id =
            store.add_options(ExternalRuleId::DUMMY, vec![serde_json::json!({ "opt": "A" })]);

        let base = Config::new(
            vec![],
            vec![(base_external_rule_id, base_options_id, AllowWarnDeny::Warn)],
            OxlintCategories::default(),
            LintConfig::default(),
            ResolvedOxlintOverrides::new(vec![ResolvedOxlintOverride {
                files: GlobSet::new(vec!["*.js"]),
                env: None,
                globals: None,
                plugins: None,
                // Override redefines the same rule with options B and severity error
                rules: ResolvedOxlintOverrideRules {
                    builtin_rules: vec![],
                    external_rules: vec![(
                        base_external_rule_id,
                        store.add_options(
                            ExternalRuleId::DUMMY,
                            vec![serde_json::json!({ "opt": "B" })],
                        ),
                        AllowWarnDeny::Deny,
                    )],
                },
            }]),
        );

        let config_store = ConfigStore::new(base, FxHashMap::default(), store);
        let resolved = config_store.resolve(&PathBuf::from_str("/root/a.js").unwrap());

        // Should prefer override (options B, severity error)
        assert_eq!(resolved.external_rules.len(), 1);
        let (rule_id, options_id, severity) = resolved.external_rules[0];
        assert_eq!(rule_id, base_external_rule_id);
        assert_eq!(severity, AllowWarnDeny::Deny);
        // `options_id` should not equal the base options ID
        assert_ne!(options_id, base_options_id);
    }
}
