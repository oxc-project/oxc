use std::ffi::OsString;
use std::path::PathBuf;

use napi_derive::napi;

use oxc_formatter::Oxfmtrc;
use oxc_napi::OxcError;
use serde_json::Value;

use crate::{
    cli::{FormatRunner, Mode, format_command, init_miette, init_rayon, init_tracing},
    core::{
        ExternalFormatter, FormatFileStrategy, FormatResult as CoreFormatResult,
        JsFormatEmbeddedCb, JsFormatFileCb, JsSetupConfigCb, SourceFormatter,
    },
    lsp::run_lsp,
    stdin::StdinRunner,
};

/// NAPI based JS CLI entry point.
/// For pure Rust CLI entry point, see `main.rs`.
///
/// JS side passes in:
/// 1. `args`: Command line arguments (process.argv.slice(2))
/// 2. `setup_config_cb`: Callback to setup Prettier config
/// 3. `format_embedded_cb`: Callback to format embedded code in templates
/// 4. `format_file_cb`: Callback to format files
///
/// Returns a tuple of `[mode, exitCode]`:
/// - `mode`: If main logic will run in JS side, use this to indicate which mode
/// - `exitCode`: If main logic already ran in Rust side, return the exit code
#[expect(clippy::allow_attributes)]
#[allow(clippy::trailing_empty_array, clippy::unused_async)] // https://github.com/napi-rs/napi-rs/issues/2758
#[napi]
pub async fn run_cli(
    args: Vec<String>,
    #[napi(ts_arg_type = "(configJSON: string, numThreads: number) => Promise<string[]>")]
    setup_config_cb: JsSetupConfigCb,
    #[napi(ts_arg_type = "(tagName: string, code: string) => Promise<string>")]
    format_embedded_cb: JsFormatEmbeddedCb,
    #[napi(
        ts_arg_type = "(parserName: string, fileName: string, code: string) => Promise<string>"
    )]
    format_file_cb: JsFormatFileCb,
) -> (String, Option<u8>) {
    // Convert String args to OsString for compatibility with bpaf
    let args: Vec<OsString> = args.into_iter().map(OsString::from).collect();

    // Use `run_inner()` to report errors instead of panicking.
    let command = match format_command().run_inner(&*args) {
        Ok(cmd) => cmd,
        Err(e) => {
            e.print_message(100);
            // `bpaf` returns exit_code 0 for --help/--version, non-0 for parse errors
            let exit_code = u8::from(e.exit_code() != 0);
            return ("cli".to_string(), Some(exit_code));
        }
    };

    match command.mode {
        Mode::Init => ("init".to_string(), None),
        Mode::Migrate(_) => ("migrate:prettier".to_string(), None),
        Mode::Lsp => {
            run_lsp().await;
            ("lsp".to_string(), Some(0))
        }
        Mode::Stdin(_) => {
            init_tracing();
            init_miette();

            let result = StdinRunner::new(command)
                // Create external formatter from JS callback
                .with_external_formatter(Some(ExternalFormatter::new(
                    setup_config_cb,
                    format_embedded_cb,
                    format_file_cb,
                )))
                .run();

            ("stdin".to_string(), Some(result.exit_code()))
        }
        Mode::Cli(_) => {
            init_tracing();
            init_miette();
            init_rayon(command.runtime_options.threads);

            let result = FormatRunner::new(command)
                // Create external formatter from JS callback
                .with_external_formatter(Some(ExternalFormatter::new(
                    setup_config_cb,
                    format_embedded_cb,
                    format_file_cb,
                )))
                .run();

            ("cli".to_string(), Some(result.exit_code()))
        }
    }
}

// ---

#[napi(object)]
pub struct FormatResult {
    /// The formatted code.
    pub code: String,
    /// Parse and format errors.
    pub errors: Vec<OxcError>,
}

/// Format source code asynchronously.
#[expect(clippy::allow_attributes)]
#[allow(clippy::trailing_empty_array, clippy::unused_async)] // https://github.com/napi-rs/napi-rs/issues/2758
#[napi]
pub async fn format(
    filename: String,
    source_text: String,
    options: Option<Oxfmtrc>,
    #[napi(ts_arg_type = "(configJSON: string, numThreads: number) => Promise<string[]>")]
    setup_config_cb: JsSetupConfigCb,
    #[napi(ts_arg_type = "(tagName: string, code: string) => Promise<string>")]
    format_embedded_cb: JsFormatEmbeddedCb,
    #[napi(
        ts_arg_type = "(parserName: string, fileName: string, code: string) => Promise<string>"
    )]
    format_file_cb: JsFormatFileCb,
) -> FormatResult {
    let external_formatter =
        ExternalFormatter::new(setup_config_cb, format_embedded_cb, format_file_cb);

    // Determine format strategy from file path
    let Ok(entry) = FormatFileStrategy::try_from(PathBuf::from(&filename)) else {
        return FormatResult {
            code: source_text,
            errors: vec![OxcError::new(format!("Unsupported file type: {filename}"))],
        };
    };

    // Convert Oxfmtrc to FormatOptions and OxfmtOptions
    let oxfmtrc = options.unwrap_or_default();
    let (format_options, oxfmt_options) = match oxfmtrc.into_options() {
        Ok(opts) => opts,
        Err(err) => {
            return FormatResult {
                code: source_text,
                errors: vec![OxcError::new(format!("Invalid options: {err}"))],
            };
        }
    };

    // Build external config JSON for Prettier (with defaults applied)
    let mut external_config = Value::Object(serde_json::Map::new());
    Oxfmtrc::populate_prettier_config(&format_options, &mut external_config);

    let num_of_threads = 1;

    // Setup external formatter config (calls JS callback to initialize Prettier)
    if let Err(err) = external_formatter.setup_config(&external_config.to_string(), num_of_threads)
    {
        return FormatResult {
            code: source_text,
            errors: vec![OxcError::new(format!("Failed to setup external formatter: {err}"))],
        };
    }

    // Create source formatter with external formatter
    let formatter = SourceFormatter::new(num_of_threads, format_options)
        .with_external_formatter(Some(external_formatter), oxfmt_options.sort_package_json);

    // Run formatting in a blocking task within tokio runtime
    // This is needed because external formatter uses `tokio::runtime::Handle::current()`
    match tokio::task::block_in_place(|| formatter.format(&entry, &source_text)) {
        CoreFormatResult::Success { code, .. } => FormatResult { code, errors: vec![] },
        CoreFormatResult::Error(diagnostics) => {
            let errors = OxcError::from_diagnostics(&filename, &source_text, diagnostics);
            FormatResult { code: source_text, errors }
        }
    }
}
