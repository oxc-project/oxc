use rustc_hash::FxHashMap;
use serde::Deserialize;

use std::{self, ops::Deref};

/// TS type is `Record<string, boolean>`
/// https://github.com/eslint/eslint/blob/ce838adc3b673e52a151f36da0eedf5876977514/lib/shared/types.js#L40
#[derive(Debug, Deserialize)]
struct RawESLintEnv(FxHashMap<String, bool>);

/// Environment
/// https://eslint.org/docs/latest/use/configure/language-options#using-configuration-files
#[derive(Debug, Clone)]
pub struct ESLintEnv(Vec<String>);

impl ESLintEnv {
    pub fn new(env: Vec<String>) -> Self {
        Self(env)
    }

    pub fn parse(env_prop: Option<&serde_json::Value>) -> Result<Self, serde_json::Error> {
        let Some(env_value) = env_prop else {
            return Ok(Self::default());
        };

        let parsed = RawESLintEnv::deserialize(env_value)?;
        // Pick `true` keys
        Ok(Self(parsed.0.iter().filter(|(_, v)| **v).map(|(k, _)| k.to_string()).collect()))
    }
}

impl Default for ESLintEnv {
    fn default() -> Self {
        Self(vec!["builtin".to_string()])
    }
}

impl Deref for ESLintEnv {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// TODO: Move tests to here
