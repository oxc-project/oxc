use std::fmt::Debug;

use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};

use oxc_allocator::Allocator;

use crate::{
    config::{OxlintEnv, OxlintGlobals},
    context::ContextHost,
};

pub type ExternalLinterCreateWorkspaceCb = Box<dyn Fn(String) -> Result<(), String> + Send + Sync>;

pub type ExternalLinterDestroyWorkspaceCb = Box<dyn Fn(String) + Send + Sync>;

pub type ExternalLinterLoadPluginCb = Box<
    dyn Fn(
            // File URL to load plugin from
            String,
            // Plugin name (either alias or package name).
            // If is package name, it is pre-normalized.
            Option<String>,
            // `true` if plugin name is an alias (takes priority over name that plugin defines itself)
            bool,
        ) -> Result<LoadPluginResult, String>
        + Send
        + Sync,
>;

pub type ExternalLinterSetupRuleConfigsCb = Box<dyn Fn(String) -> Result<(), String> + Send + Sync>;

pub type ExternalLinterLintFileCb = Box<
    dyn Fn(
            String,
            Vec<u32>,
            Vec<u32>,
            String,
            String,
            &Allocator,
        ) -> Result<Vec<LintFileResult>, String>
        + Sync
        + Send,
>;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadPluginResult {
    pub name: String,
    pub offset: usize,
    pub rule_names: Vec<String>,
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

pub struct ExternalLinter {
    pub(crate) load_plugin: ExternalLinterLoadPluginCb,
    pub(crate) setup_rule_configs: ExternalLinterSetupRuleConfigsCb,
    pub(crate) lint_file: ExternalLinterLintFileCb,
    pub create_workspace: ExternalLinterCreateWorkspaceCb,
    pub destroy_workspace: ExternalLinterDestroyWorkspaceCb,
}

impl ExternalLinter {
    pub fn new(
        load_plugin: ExternalLinterLoadPluginCb,
        setup_rule_configs: ExternalLinterSetupRuleConfigsCb,
        lint_file: ExternalLinterLintFileCb,
        create_workspace: ExternalLinterCreateWorkspaceCb,
        destroy_workspace: ExternalLinterDestroyWorkspaceCb,
    ) -> Self {
        Self { load_plugin, setup_rule_configs, lint_file, create_workspace, destroy_workspace }
    }
}

impl Debug for ExternalLinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalLinter").finish()
    }
}

/// Struct for serializing globals and envs to send to JS plugins.
///
/// Serializes as `{ "globals": { "React": "readonly" }, "envs": { "browser": true } }`.
/// `envs` only includes the environments that are enabled, so all properties are `true`.
#[derive(Serialize)]
pub struct GlobalsAndEnvs<'c> {
    globals: &'c OxlintGlobals,
    envs: EnabledEnvs<'c>,
}

impl<'c> GlobalsAndEnvs<'c> {
    pub fn new(ctx_host: &'c ContextHost<'_>) -> Self {
        Self { globals: ctx_host.globals(), envs: EnabledEnvs(ctx_host.env()) }
    }
}

struct EnabledEnvs<'c>(&'c OxlintEnv);

impl Serialize for EnabledEnvs<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;

        for env_name in self.0.iter() {
            map.serialize_entry(env_name, &true)?;
        }

        map.end()
    }
}
