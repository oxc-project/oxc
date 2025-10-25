use std::{fmt::Debug, sync::Arc};

use serde::Deserialize;

use oxc_allocator::Allocator;

pub type ExternalLinterLoadPluginCb = Arc<
    dyn Fn(
            String,
            Option<String>,
        ) -> Result<PluginLoadResult, Box<dyn std::error::Error + Send + Sync>>
        + Send
        + Sync,
>;

pub type ExternalLinterLintFileCb =
    Arc<dyn Fn(String, Vec<u32>, &Allocator) -> Result<Vec<LintFileResult>, String> + Sync + Send>;

#[derive(Clone, Debug, Deserialize)]
pub enum PluginLoadResult {
    #[serde(rename_all = "camelCase")]
    Success {
        name: String,
        offset: usize,
        rule_names: Vec<String>,
    },
    Failure(String),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LintFileResult {
    pub rule_index: u32,
    pub message: String,
    pub start: u32,
    pub end: u32,
    pub fixes: Option<Vec<JsFix>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsFix {
    pub range: [u32; 2],
    pub text: String,
}

#[derive(Clone)]
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
