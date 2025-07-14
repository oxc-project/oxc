use std::{fmt::Debug, pin::Pin, sync::Arc};

use serde::{Deserialize, Serialize};

pub type ExternalLinterLoadPluginCb = Arc<
    dyn Fn(
            String,
        ) -> Pin<
            Box<
                dyn Future<
                        Output = Result<PluginLoadResult, Box<dyn std::error::Error + Send + Sync>>,
                    > + Send,
            >,
        > + Send
        + Sync
        + 'static,
>;

pub type ExternalLinterCb = Arc<
    dyn Fn(String, Vec<u32>) -> Result<Vec<LintResult>, Box<dyn std::error::Error + Send + Sync>>
        + Sync
        + Send,
>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PluginLoadResult {
    Success { name: String, offset: usize, rules: Vec<String> },
    Failure(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintResult {
    pub external_rule_id: u32,
    pub message: String,
    pub loc: Loc,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Loc {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone)]
#[cfg_attr(any(not(feature = "oxlint2"), feature = "disable_oxlint2"), expect(dead_code))]
pub struct ExternalLinter {
    pub(crate) load_plugin: ExternalLinterLoadPluginCb,
    pub(crate) run: ExternalLinterCb,
}

impl ExternalLinter {
    pub fn new(run: ExternalLinterCb, load_plugin: ExternalLinterLoadPluginCb) -> Self {
        Self { load_plugin, run }
    }
}

impl Debug for ExternalLinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalLinter").finish()
    }
}
