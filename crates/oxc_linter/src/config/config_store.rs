use std::{path::Path, sync::Arc};

use rustc_hash::FxHashSet;

use super::{overrides::OxlintOverrides, LintConfig, LintPlugins};
use crate::{rules::RULES, RuleWithSeverity};

// TODO: support `categories` et. al. in overrides.
#[derive(Debug)]
pub(crate) struct ResolvedLinterState {
    // TODO: Arc + Vec -> SyncVec? It would save a pointer dereference.
    pub rules: Arc<[RuleWithSeverity]>,
    pub config: Arc<LintConfig>,
}

impl Clone for ResolvedLinterState {
    fn clone(&self) -> Self {
        Self { rules: Arc::clone(&self.rules), config: Arc::clone(&self.config) }
    }
}

#[derive(Debug)]
struct Config {
    /// The basic linter state for this configuration.
    base: ResolvedLinterState,

    /// An optional set of overrides to apply to the base state depending on the file being linted.
    overrides: OxlintOverrides,
}

/// Resolves a lint configuration for a given file, by applying overrides based on the file's path.
#[derive(Debug)]
pub struct ConfigStore {
    base: Config,
}

impl ConfigStore {
    pub fn new(
        base_rules: Vec<RuleWithSeverity>,
        base_config: LintConfig,
        overrides: OxlintOverrides,
    ) -> Self {
        let base = ResolvedLinterState {
            rules: Arc::from(base_rules.into_boxed_slice()),
            config: Arc::new(base_config),
        };
        Self { base: Config { base, overrides } }
    }

    pub fn number_of_rules(&self) -> usize {
        self.base.base.rules.len()
    }

    pub fn rules(&self) -> &Arc<[RuleWithSeverity]> {
        &self.base.base.rules
    }

    pub(crate) fn resolve(&self, path: &Path) -> ResolvedLinterState {
        // TODO: based on the `path` provided, resolve the configuration file to use.
        let resolved_config = &self.base;
        Self::apply_overrides(resolved_config, path)
    }

    fn apply_overrides(config: &Config, path: &Path) -> ResolvedLinterState {
        if config.overrides.is_empty() {
            return config.base.clone();
        }

        let relative_path = config
            .base
            .config
            .path
            .as_ref()
            .and_then(|config_path| {
                config_path.parent().map(|parent| path.strip_prefix(parent).unwrap_or(path))
            })
            .unwrap_or(path);

        let overrides_to_apply =
            config.overrides.iter().filter(|config| config.files.is_match(relative_path));

        let mut overrides_to_apply = overrides_to_apply.peekable();

        if overrides_to_apply.peek().is_none() {
            return config.base.clone();
        }

        let mut plugins = config.base.config.plugins;
        let all_rules = RULES
            .iter()
            .filter(|rule| plugins.contains(LintPlugins::from(rule.plugin_name())))
            .cloned()
            .collect::<Vec<_>>();
        let mut rules = config
            .base
            .rules
            .iter()
            .filter(|rule| plugins.contains(LintPlugins::from(rule.plugin_name())))
            .cloned()
            .collect::<FxHashSet<_>>();

        for override_config in overrides_to_apply {
            if !override_config.rules.is_empty() {
                override_config.rules.override_rules(&mut rules, &all_rules);
            }

            if let Some(override_plugins) = override_config.plugins {
                plugins |= override_plugins;
            }
        }

        let rules = rules.into_iter().collect::<Vec<_>>();
        let config = if plugins == config.base.config.plugins {
            Arc::clone(&config.base.config)
        } else {
            let mut config = (*config.base.config).clone();

            config.plugins = plugins;
            Arc::new(config)
        };

        ResolvedLinterState { rules: Arc::from(rules.into_boxed_slice()), config }
    }
}

#[cfg(test)]
mod test {
    use super::{ConfigStore, OxlintOverrides};
    use crate::{
        config::{LintConfig, OxlintEnv, OxlintGlobals, OxlintSettings},
        AllowWarnDeny, LintPlugins, RuleEnum, RuleWithSeverity,
    };

    macro_rules! from_json {
        ($json:tt) => {
            serde_json::from_value(serde_json::json!($json)).unwrap()
        };
    }

    #[allow(clippy::default_trait_access)]
    fn no_explicit_any() -> RuleWithSeverity {
        RuleWithSeverity::new(
            RuleEnum::TypescriptNoExplicitAny(Default::default()),
            AllowWarnDeny::Warn,
        )
    }

    /// an empty ruleset is a no-op
    #[test]
    fn test_no_rules() {
        let base_rules = vec![no_explicit_any()];
        let overrides: OxlintOverrides = from_json!([{
            "files": ["*.test.{ts,tsx}"],
            "rules": {}
        }]);
        let store = ConfigStore::new(base_rules, LintConfig::default(), overrides);

        let rules_for_source_file = store.resolve("App.tsx".as_ref());
        let rules_for_test_file = store.resolve("App.test.tsx".as_ref());

        assert_eq!(rules_for_source_file.rules.len(), 1);
        assert_eq!(rules_for_test_file.rules.len(), 1);
        assert_eq!(
            rules_for_test_file.rules[0].rule.id(),
            rules_for_source_file.rules[0].rule.id()
        );
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
        let store = ConfigStore::new(base_rules, LintConfig::default(), overrides);

        let rules_for_source_file = store.resolve("App.tsx".as_ref());
        let rules_for_test_file = store.resolve("App.test.tsx".as_ref());

        assert_eq!(rules_for_source_file.rules.len(), 1);
        assert_eq!(rules_for_test_file.rules.len(), 1);
        assert_eq!(
            rules_for_test_file.rules[0].rule.id(),
            rules_for_source_file.rules[0].rule.id()
        );
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

        let store = ConfigStore::new(base_rules, LintConfig::default(), overrides);
        assert_eq!(store.number_of_rules(), 1);

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

        let store = ConfigStore::new(base_rules, LintConfig::default(), overrides);
        assert_eq!(store.number_of_rules(), 1);

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

        let store = ConfigStore::new(base_rules, LintConfig::default(), overrides);
        assert_eq!(store.number_of_rules(), 1);

        let app = store.resolve("App.tsx".as_ref()).rules;
        assert_eq!(app.len(), 1);
        assert_eq!(app[0].severity, AllowWarnDeny::Warn);

        let src_app = store.resolve("src/App.tsx".as_ref()).rules;
        assert_eq!(src_app.len(), 1);
        assert_eq!(src_app[0].severity, AllowWarnDeny::Deny);
    }

    #[test]
    fn test_add_plugins() {
        let base_config = LintConfig {
            plugins: LintPlugins::IMPORT,
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

        let store = ConfigStore::new(vec![], base_config, overrides);
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
}
