use std::{self, ops::Deref};

#[derive(Debug, Clone)]
pub struct Env(Vec<String>);

impl Env {
    pub fn new(env: Vec<String>) -> Self {
        Self(env)
    }
}

/// The `env` field from ESLint config
impl Default for Env {
    fn default() -> Self {
        Self(vec!["builtin".to_string()])
    }
}

impl Deref for Env {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
