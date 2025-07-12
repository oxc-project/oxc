use std::{
    process::{ExitCode, Termination},
    sync::{Arc, mpsc::channel},
};

use napi::{
    Status,
    bindgen_prelude::Promise,
    threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};
use napi_derive::napi;

use oxlint::{
    ExternalLinter, ExternalLinterCb, ExternalLinterLoadPluginCb, PluginLoadResult,
    lint as oxlint_lint,
};

#[napi]
pub type JsRunCb = ThreadsafeFunction<(String, Vec<u32>), (), (String, Vec<u32>), Status, false>;

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
    Arc::new(move |file_path: String, rule_ids: Vec<u32>| {
        let cb = Arc::clone(&cb);

        let (tx, rx) = channel();

        let status = cb.call_with_return_value(
            (file_path, rule_ids),
            ThreadsafeFunctionCallMode::NonBlocking,
            move |result, _env| {
                let _ = match &result {
                    Ok(()) => tx.send(Ok(())),
                    Err(e) => tx.send(Err(e.to_string())),
                };

                result
            },
        );

        if status != Status::Ok {
            return Err(format!("Failed to schedule callback: {status:?}").into());
        }

        match rx.recv() {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(format!("Callback reported error: {e}").into()),
            Err(e) => Err(format!("Callback did not respond: {e}").into()),
        }
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
