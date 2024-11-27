use crate::LintPlugins;
use crate::{rules::RULES, RuleWithSeverity};
use rustc_hash::FxHashSet;
use std::{
    hash::{BuildHasher, Hash, Hasher},
    path::Path,
    sync::Arc,
};

use super::{
    overrides::{OverrideId, OxlintOverrides},
    LintConfig,
};
use dashmap::DashMap;
use rustc_hash::FxBuildHasher;

type AppliedOverrideHash = u64;

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

/// Keeps track of a list of config deltas, lazily applying them to a base config as requested by
/// [`ConfigStore::resolve`]. This struct is [`Sync`] + [`Send`] since the linter runs on each file
/// in parallel.
#[derive(Debug)]
pub struct ConfigStore {
    // TODO: flatten base config + overrides into a single "flat" config. Similar idea to ESLint's
    // flat configs, but we would still support v8 configs. Doing this could open the door to
    // supporting flat configs (e.g. eslint.config.js). Still need to figure out how this plays
    // with nested configs.
    /// Resolved override cache. The key is a hash of each override's ID that matched the list of
    /// file globs in order to avoid re-allocating the same set of rules multiple times.
    cache: DashMap<AppliedOverrideHash, ResolvedLinterState, FxBuildHasher>,
    /// "root" level configuration. In the future this may just be the first entry in `overrides`.
    base: ResolvedLinterState,
    /// Config deltas applied to `base`.
    overrides: OxlintOverrides,
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
        // best-best case: no overrides are provided & config is initialized with 0 capacity best
        // case: each file matches only a single override, so we only need `overrides.len()`
        // capacity worst case: files match more than one override. In the most ridiculous case, we
        // could end up needing (overrides.len() ** 2) capacity. I don't really want to
        // pre-allocate that much space unconditionally. Better to re-alloc if we end up needing
        // it.
        let cache = DashMap::with_capacity_and_hasher(overrides.len(), FxBuildHasher);

        Self { cache, base, overrides }
    }

    /// Set the base rules, replacing all existing rules.
    #[cfg(test)]
    #[inline]
    pub fn set_rules(&mut self, new_rules: Vec<RuleWithSeverity>) {
        self.base.rules = Arc::from(new_rules.into_boxed_slice());
    }

    pub fn number_of_rules(&self) -> usize {
        self.base.rules.len()
    }

    pub fn rules(&self) -> &Arc<[RuleWithSeverity]> {
        &self.base.rules
    }

    pub(crate) fn resolve(&self, path: &Path) -> ResolvedLinterState {
        if self.overrides.is_empty() {
            return self.base.clone();
        }

        let mut overrides_to_apply: Vec<OverrideId> = Vec::new();
        let mut hasher = FxBuildHasher.build_hasher();

        // Compute the path of the file relative to the configuration file for glob matching. Globs should match
        // relative to the location of the configuration file.
        // - path: /some/path/like/this/to/file.js
        // - config_path: /some/path/like/.oxlintrc.json
        // => relative_path: this/to/file.js
        // TODO: Handle nested configuration file paths.
        let relative_path = if let Some(config_path) = &self.base.config.path {
            if let Some(parent) = config_path.parent() {
                path.strip_prefix(parent).unwrap_or(path)
            } else {
                path
            }
        } else {
            path
        };

        for (id, override_config) in self.overrides.iter_enumerated() {
            if override_config.files.is_match(relative_path) {
                overrides_to_apply.push(id);
                id.hash(&mut hasher);
            }
        }

        if overrides_to_apply.is_empty() {
            return self.base.clone();
        }

        let key = hasher.finish();
        self.cache
            .entry(key)
            .or_insert_with(|| self.apply_overrides(&overrides_to_apply))
            .value()
            .clone()
    }

    /// NOTE: this function must not borrow any entries from `self.cache` or DashMap will deadlock.
    fn apply_overrides(&self, override_ids: &[OverrideId]) -> ResolvedLinterState {
        let mut plugins = self.base.config.plugins;

        let all_rules = RULES
            .iter()
            .filter(|rule| plugins.contains(LintPlugins::from(rule.plugin_name())))
            .cloned()
            .collect::<Vec<_>>();
        let mut rules = self
            .base
            .rules
            .iter()
            .filter(|rule| plugins.contains(LintPlugins::from(rule.plugin_name())))
            .cloned()
            .collect::<FxHashSet<_>>();

        let overrides = override_ids.iter().map(|id| &self.overrides[*id]);
        for override_config in overrides {
            if !override_config.rules.is_empty() {
                override_config.rules.override_rules(&mut rules, &all_rules);
            }

            // Append the override's plugins to the base list of enabled plugins.
            if let Some(override_plugins) = override_config.plugins {
                plugins |= override_plugins;
            }
        }

        let rules = rules.into_iter().collect::<Vec<_>>();
        let config = if plugins == self.base.config.plugins {
            Arc::clone(&self.base.config)
        } else {
            let mut config = (*self.base.config.as_ref()).clone();

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
        assert_eq!(store.base.config.plugins, LintPlugins::IMPORT);

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
