use std::{
    io::BufWriter,
    process::{ExitCode, Termination},
};

use napi_derive::napi;

use crate::{
    command::format_command,
    format::FormatRunner,
    init::{init_miette, init_tracing},
    prettier_plugins::{JsFormatEmbeddedCb, create_external_formatter},
    result::CliRunResult,
};

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
pub async fn format(args: Vec<String>, format_embedded_cb: JsFormatEmbeddedCb) -> bool {
    format_impl(&args, format_embedded_cb).report() == ExitCode::SUCCESS
}

/// Run the formatter.
fn format_impl(args: &[String], format_embedded_cb: JsFormatEmbeddedCb) -> CliRunResult {
    init_tracing();
    init_miette();

    // Parse command line arguments
    let command = match format_command()
        .run_inner(args.iter().map(|s| s.as_ref() as &str).collect::<Vec<_>>().as_slice())
    {
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

    command.handle_threads();

    // Create external formatter from JS callback
    let external_formatter = create_external_formatter(format_embedded_cb);

    // stdio is blocked by LineWriter, use a BufWriter to reduce syscalls.
    // See `https://github.com/rust-lang/rust/issues/60673`.
    let mut stdout = BufWriter::new(std::io::stdout());
    FormatRunner::new(command).with_external_formatter(Some(external_formatter)).run(&mut stdout)
}
