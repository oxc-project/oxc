use serde::Deserialize;

use rustc_hash::FxHashMap;

/// <https://eslint.org/docs/v8.x/use/configure/language-options#using-configuration-files-1>
#[derive(Debug, Default, Deserialize)]
pub struct ESLintGlobals(FxHashMap<String, GlobalValue>);

#[derive(Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GlobalValue {
    Readonly,
    Writeable,
    Off,
}

impl ESLintGlobals {
    pub fn is_enabled(&self, name: &str) -> bool {
        self.0.get(name).is_some_and(|value| *value != GlobalValue::Off)
    }
}
