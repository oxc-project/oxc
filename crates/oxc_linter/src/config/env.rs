use std::{self, ops::Deref};

/// Environment
/// https://eslint.org/docs/latest/use/configure/language-options#using-configuration-files
#[derive(Debug, Clone)]
pub struct ESLintEnv(Vec<String>);

impl ESLintEnv {
    pub fn new(env: Vec<String>) -> Self {
        Self(env)
    }
}

/// The `env` field from ESLint config
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
