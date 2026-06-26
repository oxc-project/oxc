use std::{borrow::Borrow, hash::Hash};

use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Predefine global variables.
///
/// Environments specify what global variables are predefined.
/// Available environments:
/// - amd - require() and define() globals.
/// - applescript - AppleScript globals.
/// - astro - Astro globals.
/// - atomtest - Atom test globals.
/// - audioworklet - AudioWorklet globals.
/// - browser - browser globals.
/// - builtin - Latest ECMAScript globals, equivalent to es2026.
/// - commonjs - CommonJS globals and scoping.
/// - embertest - Ember test globals.
/// - es2015 - ECMAScript 2015 globals.
/// - es2016 - ECMAScript 2016 globals.
/// - es2017 - ECMAScript 2017 globals.
/// - es2018 - ECMAScript 2018 globals.
/// - es2019 - ECMAScript 2019 globals.
/// - es2020 - ECMAScript 2020 globals.
/// - es2021 - ECMAScript 2021 globals.
/// - es2022 - ECMAScript 2022 globals.
/// - es2023 - ECMAScript 2023 globals.
/// - es2024 - ECMAScript 2024 globals.
/// - es2025 - ECMAScript 2025 globals.
/// - es2026 - ECMAScript 2026 globals.
/// - es6 - ECMAScript 6 globals except modules.
/// - greasemonkey - GreaseMonkey globals.
/// - jasmine - Jasmine globals.
/// - jest - Jest globals.
/// - jquery - jQuery globals.
/// - meteor - Meteor globals.
/// - mocha - Mocha globals.
/// - mongo - MongoDB globals.
/// - nashorn - Java 8 Nashorn globals.
/// - node - Node.js globals and scoping.
/// - phantomjs - PhantomJS globals.
/// - prototypejs - Prototype.js globals.
/// - protractor - Protractor globals.
/// - qunit - QUnit globals.
/// - serviceworker - Service Worker globals.
/// - shared-node-browser - Node.js and Browser common globals.
/// - shelljs - ShellJS globals.
/// - svelte - Svelte globals.
/// - vitest - Vitest globals.
/// - vue - Vue globals.
/// - webextensions - WebExtensions globals.
/// - worker - Web Workers globals.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
pub struct OxlintEnv(FxHashMap<String, bool>);

impl FromIterator<String> for OxlintEnv {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        let map = iter.into_iter().map(|key| (key, true)).collect();

        Self(map)
    }
}

impl Default for OxlintEnv {
    fn default() -> Self {
        let mut map = FxHashMap::default();
        map.insert("builtin".to_string(), true);

        Self(map)
    }
}

impl OxlintEnv {
    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        String: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.0.get(key).is_some_and(|v| *v)
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> + '_ {
        // Filter out false values
        self.0.iter().filter_map(|(k, v)| (*v).then_some(k.as_str()))
    }

    pub(crate) fn override_envs(&self, envs_to_override: &mut OxlintEnv) {
        for (env, supported) in self.0.clone() {
            envs_to_override.0.insert(env, supported);
        }
    }
}

#[cfg(test)]
mod test {
    use serde::Deserialize;

    use super::OxlintEnv;

    #[test]
    fn test_parse_env() {
        let env = OxlintEnv::deserialize(&serde_json::json!({
            "browser": true, "node": true, "es6": false
        }))
        .unwrap();
        assert_eq!(env.iter().count(), 2);
        assert!(env.contains("browser"));
        assert!(env.contains("node"));
        assert!(!env.contains("es6"));
        assert!(!env.contains("builtin"));
    }
    #[test]
    fn test_parse_env_default() {
        let env = OxlintEnv::default();
        assert_eq!(env.iter().count(), 1);
        assert!(env.contains("builtin"));
    }

    #[test]
    fn test_override_envs() {
        let mut env = OxlintEnv::default();
        let override_env = OxlintEnv::deserialize(&serde_json::json!({
            "browser": true,
        }))
        .unwrap();

        override_env.override_envs(&mut env);

        assert!(env.contains("browser"));
    }
}
