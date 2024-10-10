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
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq))]
pub struct OxlintEnv(FxHashMap<String, bool>);

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
}

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
}
