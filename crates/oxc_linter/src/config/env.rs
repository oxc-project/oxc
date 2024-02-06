use rustc_hash::FxHashMap;
use serde::Deserialize;

/// Environment
/// https://eslint.org/docs/latest/use/configure/language-options#using-configuration-files
///
/// TS type is `Record<string, boolean>`
/// https://github.com/eslint/eslint/blob/ce838adc3b673e52a151f36da0eedf5876977514/lib/shared/types.js#L40
#[derive(Debug, Clone, Deserialize)]
pub struct ESLintEnv(FxHashMap<String, bool>);

impl ESLintEnv {
    pub fn from_vec(env: Vec<String>) -> Self {
        let mut map = FxHashMap::default();
        for e in env {
            map.insert(e, true);
        }
        Self(map)
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> + '_ {
       self.0.iter().filter(|(_, v)| **v).map(|(k, _)| k.as_str())
    }
}

impl Default for ESLintEnv {
    fn default() -> Self {
        let mut map = FxHashMap::default();
        map.insert("builtin".to_string(), true);
        Self(map)
    }
}

// TODO: Move tests to here
