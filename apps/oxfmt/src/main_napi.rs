use std::{
    ffi::OsString,
    io::BufWriter,
    process::{ExitCode, Termination},
};

use napi_derive::napi;

use crate::{
    cli::{CliRunResult, FormatRunner, format_command, init_miette, init_tracing},
    core::{ExternalFormatter, JsFormatEmbeddedCb, JsFormatFileCb, JsSetupConfigCb},
    lsp::run_lsp,
};

// NAPI based JS CLI entry point.
// For pure Rust CLI entry point, see `main.rs`.

/// NAPI entry point.
///
/// JS side passes in:
/// 1. `args`: Command line arguments (process.argv.slice(2))
/// 2. `setup_config_cb`: Callback to setup Prettier config
/// 3. `format_embedded_cb`: Callback to format embedded code in templates
/// 4. `format_file_cb`: Callback to format files
///
/// Returns `true` if formatting succeeded without errors, `false` otherwise.
#[expect(clippy::allow_attributes)]
#[allow(clippy::trailing_empty_array, clippy::unused_async)] // https://github.com/napi-rs/napi-rs/issues/2758
#[napi]
pub async fn format(
    args: Vec<String>,
    #[napi(ts_arg_type = "(configJSON: string, numThreads: number) => Promise<string[]>")]
    setup_config_cb: JsSetupConfigCb,
    #[napi(ts_arg_type = "(tagName: string, code: string) => Promise<string>")]
    format_embedded_cb: JsFormatEmbeddedCb,
    #[napi(
        ts_arg_type = "(parserName: string, fileName: string, code: string) => Promise<string>"
    )]
    format_file_cb: JsFormatFileCb,
) -> bool {
    format_impl(args, setup_config_cb, format_embedded_cb, format_file_cb).await.report()
        == ExitCode::SUCCESS
}

/// Run the formatter.
async fn format_impl(
    args: Vec<String>,
    setup_config_cb: JsSetupConfigCb,
    format_embedded_cb: JsFormatEmbeddedCb,
    format_file_cb: JsFormatFileCb,
) -> CliRunResult {
    // Convert String args to OsString for compatibility with bpaf
    let args: Vec<OsString> = args.into_iter().map(OsString::from).collect();

    // Use `run_inner()` to report errors instead of panicking.
    let command = match format_command().run_inner(&*args) {
        Ok(cmd) => cmd,
        Err(e) => {
            e.print_message(100);
            return if e.exit_code() == 0 {
                CliRunResult::None
            } else {
                CliRunResult::InvalidOptionConfig
            };
        }
    };

    // Handle LSP mode
    if command.misc_options.lsp {
        run_lsp().await;
        return CliRunResult::None;
    }

    // Otherwise, CLI mode
    init_tracing();
    init_miette();

    command.handle_threads();

    // Create external formatter from JS callback
    let external_formatter =
        ExternalFormatter::new(setup_config_cb, format_embedded_cb, format_file_cb);

    // stdio is blocked by LineWriter, use a BufWriter to reduce syscalls.
    // See `https://github.com/rust-lang/rust/issues/60673`.
    let mut stdout = BufWriter::new(std::io::stdout());
    let mut stderr = BufWriter::new(std::io::stderr());
    FormatRunner::new(command)
        .with_external_formatter(Some(external_formatter))
        .run(&mut stdout, &mut stderr)
}
