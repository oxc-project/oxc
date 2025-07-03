use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use rustc_hash::FxHashMap;

use super::{LintConfig, LintPlugins, categories::OxlintCategories, overrides::OxlintOverrides};
use crate::{
    AllowWarnDeny,
    config::plugins::BuiltinLintPlugins,
    rules::{RULES, RuleEnum},
};

// TODO: support `categories` et. al. in overrides.
#[derive(Debug)]
pub struct ResolvedLinterState {
    // TODO: Arc + Vec -> SyncVec? It would save a pointer dereference.
    pub rules: Arc<[(RuleEnum, AllowWarnDeny)]>,
    pub config: Arc<LintConfig>,
}

impl Clone for ResolvedLinterState {
    fn clone(&self) -> Self {
        Self { rules: Arc::clone(&self.rules), config: Arc::clone(&self.config) }
    }
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
    pub(crate) overrides: OxlintOverrides,
}

impl Config {
    pub fn new(
        rules: Vec<(RuleEnum, AllowWarnDeny)>,
        categories: OxlintCategories,
        config: LintConfig,
        overrides: OxlintOverrides,
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

        // TODO: external rules.

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

            if !override_config.rules.is_empty() {
                override_config.rules.override_rules(&mut rules, &all_rules);
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
        ResolvedLinterState { rules: Arc::from(rules.into_boxed_slice()), config }
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
}

impl ConfigStore {
    pub fn new(base_config: Config, nested_configs: FxHashMap<PathBuf, Config>) -> Self {
        Self { base: base_config, nested_configs }
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
}

#[cfg(test)]
mod test {
    use rustc_hash::FxHashMap;

    use super::{ConfigStore, OxlintOverrides};
    use crate::{
        AllowWarnDeny, LintPlugins, RuleEnum,
        config::{
            LintConfig, OxlintEnv, OxlintGlobals, OxlintSettings, categories::OxlintCategories,
            config_store::Config, plugins::BuiltinLintPlugins,
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
        let overrides: OxlintOverrides = from_json!([{
            "files": ["*.test.{ts,tsx}"],
            "rules": {}
        }]);
        let store = ConfigStore::new(
            Config::new(base_rules, OxlintCategories::default(), LintConfig::default(), overrides),
            FxHashMap::default(),
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
        let overrides: OxlintOverrides = from_json!([{
            "files": ["*.test.{ts,tsx}"],
            "plugins": ["react", "typescript", "unicorn", "oxc", "jsx-a11y"],
            "rules": {}
        }]);
        let store = ConfigStore::new(
            Config::new(base_rules, OxlintCategories::default(), LintConfig::default(), overrides),
            FxHashMap::default(),
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
        let overrides: OxlintOverrides = from_json!([{
            "files": ["*.test.{ts,tsx}"],
            "rules": {
                "@typescript-eslint/no-explicit-any": "off"
            }
        }]);

        let store = ConfigStore::new(
            Config::new(base_rules, OxlintCategories::default(), LintConfig::default(), overrides),
            FxHashMap::default(),
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
        let overrides = from_json!([{
            "files": ["src/**/*.{ts,tsx}"],
            "rules": {
                "no-unused-vars": "warn"
            }
        }]);

        let store = ConfigStore::new(
            Config::new(base_rules, OxlintCategories::default(), LintConfig::default(), overrides),
            FxHashMap::default(),
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
        let overrides = from_json!([{
            "files": ["src/**/*.{ts,tsx}"],
            "rules": {
                "no-explicit-any": "error"
            }
        }]);

        let store = ConfigStore::new(
            Config::new(base_rules, OxlintCategories::default(), LintConfig::default(), overrides),
            FxHashMap::default(),
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
            plugins: BuiltinLintPlugins::IMPORT,
            env: OxlintEnv::default(),
            settings: OxlintSettings::default(),
            globals: OxlintGlobals::default(),
            path: None,
        };
        let overrides = from_json!([{
            "files": ["*.jsx", "*.tsx"],
            "plugins": ["react"],
        }, {
            "files": ["*.ts", "*.tsx"],
            "plugins": ["typescript"],
        }]);

        let store = ConfigStore::new(
            Config::new(vec![], OxlintCategories::default(), base_config, overrides),
            FxHashMap::default(),
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
            plugins: LintPlugins::ESLINT,
            settings: OxlintSettings::default(),
            globals: OxlintGlobals::default(),
            path: None,
        };

        let overrides = from_json!([{
            "files": ["*.tsx"],
            "env": {
                "es2024": true
            },
        }]);

        let store = ConfigStore::new(
            Config::new(vec![], OxlintCategories::default(), base_config, overrides),
            FxHashMap::default(),
        );
        assert!(!store.base.base.config.env.contains("React"));

        let app = store.resolve("App.tsx".as_ref()).config;
        assert!(app.env.contains("es2024"));
    }

    #[test]
    fn test_replace_env() {
        let base_config = LintConfig {
            env: OxlintEnv::from_iter(["es2024".into()]),
            plugins: LintPlugins::ESLINT,
            settings: OxlintSettings::default(),
            globals: OxlintGlobals::default(),
            path: None,
        };

        let overrides = from_json!([{
            "files": ["*.tsx"],
            "env": {
                "es2024": false
            },
        }]);

        let store = ConfigStore::new(
            Config::new(vec![], OxlintCategories::default(), base_config, overrides),
            FxHashMap::default(),
        );
        assert!(store.base.base.config.env.contains("es2024"));

        let app = store.resolve("App.tsx".as_ref()).config;
        assert!(!app.env.contains("es2024"));
    }

    #[test]
    fn test_add_globals() {
        let base_config = LintConfig {
            env: OxlintEnv::default(),
            plugins: LintPlugins::ESLINT,
            settings: OxlintSettings::default(),
            globals: OxlintGlobals::default(),
            path: None,
        };

        let overrides = from_json!([{
            "files": ["*.tsx"],
            "globals": {
                "React": "readonly",
                "Secret": "writeable"
            },
        }]);

        let store = ConfigStore::new(
            Config::new(vec![], OxlintCategories::default(), base_config, overrides),
            FxHashMap::default(),
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
            plugins: LintPlugins::ESLINT,
            settings: OxlintSettings::default(),
            globals: from_json!({
                "React": "readonly",
                "Secret": "writeable"
            }),
            path: None,
        };

        let overrides = from_json!([{
            "files": ["*.tsx"],
            "globals": {
                "React": "off",
                "Secret": "off"
            },
        }]);

        let store = ConfigStore::new(
            Config::new(vec![], OxlintCategories::default(), base_config, overrides),
            FxHashMap::default(),
        );
        assert!(store.base.base.config.globals.is_enabled("React"));
        assert!(store.base.base.config.globals.is_enabled("Secret"));

        let app = store.resolve("App.tsx".as_ref()).config;
        assert!(!app.globals.is_enabled("React"));
        assert!(!app.globals.is_enabled("Secret"));
    }
}
