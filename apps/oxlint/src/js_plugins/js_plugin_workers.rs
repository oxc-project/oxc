//! Registry of JS plugin workers (Node.js worker threads) which run JS plugins in parallel.
//!
//! The main JS thread is always JS thread 0. Its callbacks are passed directly to `lint()`.
//! When `--js-threads N` is used, the JS side spawns `N - 1` JS plugin workers before calling `lint()`.
//! Each worker calls `registerJsPluginWorker` with its own callbacks, which are stored here.
//! `lint()` then takes all registered workers out of the registry.

use std::sync::{Mutex, PoisonError};

use crate::run::{
    JsCreateWorkspaceCb, JsDestroyWorkspaceCb, JsLintFileCb, JsLoadPluginCb, JsSetupRuleConfigsCb,
};

/// Callbacks for a single JS thread (main JS thread or a JS plugin worker).
pub struct JsThreadCallbacks {
    pub load_plugin: JsLoadPluginCb,
    pub setup_rule_configs: JsSetupRuleConfigsCb,
    pub lint_file: JsLintFileCb,
    pub create_workspace: JsCreateWorkspaceCb,
    pub destroy_workspace: JsDestroyWorkspaceCb,
}

/// JS plugin workers which have registered, but have not yet been taken by `lint()`.
static REGISTERED_JS_PLUGIN_WORKERS: Mutex<Vec<JsThreadCallbacks>> = Mutex::new(Vec::new());

/// Register a JS plugin worker.
pub fn register_js_plugin_worker(worker: JsThreadCallbacks) {
    REGISTERED_JS_PLUGIN_WORKERS.lock().unwrap_or_else(PoisonError::into_inner).push(worker);
}

/// Take all registered JS plugin workers out of the registry.
pub fn take_registered_js_plugin_workers() -> Vec<JsThreadCallbacks> {
    std::mem::take(&mut *REGISTERED_JS_PLUGIN_WORKERS.lock().unwrap_or_else(PoisonError::into_inner))
}
