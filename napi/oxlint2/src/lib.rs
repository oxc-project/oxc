use napi::Status;
use napi::bindgen_prelude::{FnArgs, Uint8Array};
use napi::threadsafe_function::ThreadsafeFunction;
use oxlint::{ExternalLinterCb, ExternalLinterLoadPluginCb, lint as oxlint_lint};

use napi_derive::napi;
use std::{
    process::{ExitCode, Termination},
    sync::Arc,
};

// #[napi]
// pub type LintFileFn = ThreadsafeFunction<(String, u32), (String), (String, u32), Status, true>;

#[napi]
pub fn lint(
    buffers: Vec<Uint8Array>,
    lint_file: ExternalLinterCb,
    load_plugin: ExternalLinterLoadPluginCb,
) -> bool {
    // let buffers = buffers.iter_mut().map(|buffer| unsafe { buffer.as_mut() }).collect::<Vec<_>>();

    // options.lint_file.call(value, mode);

    oxlint_lint(
        //    Some(buffers)
        None, lint_file,
    )
    .report()
        == ExitCode::SUCCESS
}
