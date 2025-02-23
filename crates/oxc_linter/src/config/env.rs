use std::{borrow::Borrow, hash::Hash};

use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Predefine global variables.
///
/// Environments specify what global variables are predefined. See [ESLint's
/// list of
/// environments](https://eslint.org/docs/v8.x/use/configure/language-options#specifying-environments)
/// for what environments are available and what each one provides.
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
