use std::{
    ffi::OsString,
    io::BufWriter,
    process::{ExitCode, Termination},
};

use napi::{Task, bindgen_prelude::AsyncTask};
use napi_derive::napi;
use oxc_allocator::Allocator;
use oxc_formatter::{FormatOptions, Formatter};
use oxc_napi::OxcError;
use oxc_parser::Parser;
use oxc_span::SourceType;

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
pub async fn format_internal(
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
    format_internal_impl(args, setup_config_cb, format_embedded_cb, format_file_cb).await.report()
        == ExitCode::SUCCESS
}

/// Run the formatter.
async fn format_internal_impl(
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

/// Result of formatting operation.
#[derive(Default)]
#[napi(object)]
pub struct FormatResult {
    /// The formatted code.
    pub code: String,
    /// Parse and format errors.
    pub errors: Vec<OxcError>,
}

fn format_impl(filename: &str, source_text: &str, options: Option<FormatOptions>) -> FormatResult {
    let options = options.unwrap_or_default();
    let allocator = Allocator::default();

    // Infer source type from filename
    // TODO: don't do anything if file type not recognized
    let source_type = SourceType::from_path(filename).unwrap_or_default();

    // Parse the source code
    let ret = Parser::new(&allocator, source_text, source_type).parse();

    // If there are parse errors, return them
    if !ret.errors.is_empty() {
        return FormatResult {
            code: source_text.to_string(),
            errors: OxcError::from_diagnostics(filename, source_text, ret.errors),
        };
    }

    // Format the parsed program
    // TODO: hook up all the napi callbacks
    let formatter = Formatter::new(&allocator, options);
    let formatted = formatter.format(&ret.program);

    // Print the formatted output
    match formatted.print() {
        Ok(printer) => FormatResult { code: printer.into_code(), errors: vec![] },
        Err(_err) => {
            // Return original source if formatting fails
            FormatResult { code: source_text.to_string(), errors: vec![] }
        }
    }
}

/// Format synchronously.
#[napi]
pub fn format_sync(
    filename: String,
    source_text: String,
    options: Option<FormatOptions>,
) -> FormatResult {
    format_impl(&filename, &source_text, options)
}

pub struct FormatTask {
    filename: String,
    source_text: String,
    options: Option<FormatOptions>,
}

#[napi]
impl Task for FormatTask {
    type JsValue = FormatResult;
    type Output = FormatResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        Ok(format_impl(&self.filename, &self.source_text, self.options.take()))
    }

    fn resolve(&mut self, _: napi::Env, result: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(result)
    }
}

/// Format asynchronously.
///
/// Note: This function can be slower than `formatSync` due to the overhead of spawning a thread.
#[napi]
pub fn format(
    filename: String,
    source_text: String,
    options: Option<FormatOptions>,
) -> AsyncTask<FormatTask> {
    AsyncTask::new(FormatTask { filename, source_text, options })
}
