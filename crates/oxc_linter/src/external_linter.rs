use std::{fmt::Debug, sync::Arc};

use napi::{Status, bindgen_prelude::Promise, threadsafe_function::ThreadsafeFunction};
use napi_derive::napi;

#[napi]
pub type ExternalLinterCb =
    Arc<ThreadsafeFunction<(), /* TODO: correct return type */ (), (), Status, false>>;

#[napi]
pub type ExternalLinterLoadPluginCb =
    Arc<ThreadsafeFunction<String, Promise<PluginLoadResult>, String, Status, false>>;

#[napi]
pub enum PluginLoadResult {
    Success,
    Failure(String),
}

#[derive(Clone)]
#[expect(dead_code)]
pub struct ExternalLinter {
    pub(crate) load_plugin: ExternalLinterLoadPluginCb,
    pub(crate) run: ExternalLinterCb,
}

impl ExternalLinter {
    pub fn new(run: ExternalLinterCb, load_plugin: ExternalLinterLoadPluginCb) -> Self {
        ExternalLinter { load_plugin, run }
    }
}

impl Debug for ExternalLinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalLinter").finish()
    }
}
