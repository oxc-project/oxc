use std::{fmt::Debug, pin::Pin, sync::Arc};

use serde::{Deserialize, Serialize};

use oxc_allocator::Allocator;

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

pub type ExternalLinterLintFileCb =
    Arc<dyn Fn(String, Vec<u32>, &Allocator) -> Result<Vec<LintFileResult>, String> + Sync + Send>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PluginLoadResult {
    #[serde(rename_all = "camelCase")]
    Success {
        name: String,
        offset: usize,
        rule_names: Vec<String>,
    },
    Failure(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintFileResult {
    pub rule_index: u32,
    pub message: String,
    pub loc: Loc,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Loc {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone)]
#[cfg_attr(not(all(feature = "oxlint2", not(feature = "disable_oxlint2"))), expect(dead_code))]
pub struct ExternalLinter {
    pub(crate) load_plugin: ExternalLinterLoadPluginCb,
    pub(crate) lint_file: ExternalLinterLintFileCb,
}

impl ExternalLinter {
    pub fn new(
        load_plugin: ExternalLinterLoadPluginCb,
        lint_file: ExternalLinterLintFileCb,
    ) -> Self {
        Self { load_plugin, lint_file }
    }
}

impl Debug for ExternalLinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalLinter").finish()
    }
}
