use std::{
    ffi::OsString,
    io::BufWriter,
    process::{ExitCode, Termination},
};

use napi_derive::napi;

use crate::{
    command::format_command,
    format::FormatRunner,
    init::{init_miette, init_tracing},
    lsp::run_lsp,
    prettier_plugins::{JsFormatEmbeddedCb, create_external_formatter},
    result::CliRunResult,
};

// NAPI based JS CLI entry point.
// For pure Rust CLI entry point, see `main.rs`.

/// NAPI entry point.
///
/// JS side passes in:
/// 1. `args`: Command line arguments (process.argv.slice(2))
/// 2. `format_embedded_cb`: Callback to format embedded code in templates
///
/// Returns `true` if formatting succeeded without errors, `false` otherwise.
#[expect(clippy::allow_attributes)]
#[allow(clippy::trailing_empty_array, clippy::unused_async)] // https://github.com/napi-rs/napi-rs/issues/2758
#[napi]
pub async fn format(
    args: Vec<String>,
    #[napi(ts_arg_type = "(tagName: string, code: string) => Promise<string>")]
    format_embedded_cb: JsFormatEmbeddedCb,
) -> bool {
    format_impl(args, format_embedded_cb).await.report() == ExitCode::SUCCESS
}

/// Run the formatter.
async fn format_impl(args: Vec<String>, format_embedded_cb: JsFormatEmbeddedCb) -> CliRunResult {
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
    let external_formatter = create_external_formatter(format_embedded_cb);

    // stdio is blocked by LineWriter, use a BufWriter to reduce syscalls.
    // See `https://github.com/rust-lang/rust/issues/60673`.
    let mut stdout = BufWriter::new(std::io::stdout());
    let mut stderr = BufWriter::new(std::io::stderr());
    FormatRunner::new(command)
        .with_external_formatter(Some(external_formatter))
        .run(&mut stdout, &mut stderr)
}
