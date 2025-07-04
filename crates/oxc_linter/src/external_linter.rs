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
    dyn Fn() -> Pin<
        Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>,
    >,
>;

#[derive(Clone, Debug, Deserialize, Serialize)]
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
        Self { load_plugin, run }
    }
}

impl Debug for ExternalLinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalLinter").finish()
    }
}
