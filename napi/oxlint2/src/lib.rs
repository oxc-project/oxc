use std::{
    process::{ExitCode, Termination},
    sync::Arc,
};

use napi::{Status, bindgen_prelude::Promise, threadsafe_function::ThreadsafeFunction};
use napi_derive::napi;

use oxlint::{
    ExternalLinter, ExternalLinterCb, ExternalLinterLoadPluginCb, PluginLoadResult,
    lint as oxlint_lint,
};

#[napi]
pub type JsRunCb = ThreadsafeFunction<(), /* no input */ (), (), Status, false>;

#[napi]
pub type JsLoadPluginCb = ThreadsafeFunction<
    String, /* PluginName */
    Promise<String /* PluginLoadResult */>,
    String, /* PluginName */
    Status,
    false,
>;

fn wrap_run(cb: JsRunCb) -> ExternalLinterCb {
    let cb = Arc::new(cb);
    Arc::new(move || {
        Box::pin({
            let cb = Arc::clone(&cb);
            async move { cb.call_async(()).await.map_err(Into::into) }
        })
    })
}

fn wrap_load_plugin(cb: JsLoadPluginCb) -> ExternalLinterLoadPluginCb {
    let cb = Arc::new(cb);
    Arc::new(move |plugin_name| {
        Box::pin({
            let cb = Arc::clone(&cb);
            async move {
                let result = cb.call_async(plugin_name).await?.into_future().await?;

                let plugin_load_result: PluginLoadResult = serde_json::from_str(&result)?;

                Ok(plugin_load_result)
            }
        })
    })
}

#[expect(clippy::allow_attributes)]
#[allow(clippy::trailing_empty_array, clippy::unused_async)] // https://github.com/napi-rs/napi-rs/issues/2758
#[napi]
pub async fn lint(load_plugin: JsLoadPluginCb, run: JsRunCb) -> bool {
    let rust_load_plugin = wrap_load_plugin(load_plugin);
    let rust_run = wrap_run(run);

    oxlint_lint(Some(ExternalLinter::new(rust_run, rust_load_plugin))).report() == ExitCode::SUCCESS
}
