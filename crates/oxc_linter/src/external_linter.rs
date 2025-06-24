use std::{fmt::Debug, sync::Arc};

use napi::{Status, bindgen_prelude::Promise, threadsafe_function::ThreadsafeFunction};
use napi_derive::napi;

#[napi]
pub type ExternalLinterCb =
    Arc<ThreadsafeFunction<(String, u32), ExternalLinterCbResult, (String, u32), Status, false>>;

#[napi(object)]
pub struct ExternalLinterCbResult {
    pub diagnostics: Vec<ExternalLinterCbDiagnostic>,
}

#[napi(object)]
struct ExternalLinterCbDiagnostic {
    pub plugin: String,
    pub rule: String,
    pub message: String,
    pub severity: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Clone)]
pub struct ExternalLinter {
    pub(crate) run: ExternalLinterCb,
}

impl Debug for ExternalLinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalLinter").finish()
    }
}

impl ExternalLinter {
    pub fn new(run: ExternalLinterCb) -> Self {
        Self { run }
    }
}

#[napi]
pub type ExternalLinterLoadPluginCb =
    Arc<ThreadsafeFunction<String, Promise<PluginLoadResult>, String, Status, false>>;

#[napi]
pub enum PluginLoadResult {
    Success,
    Failure(String),
}
