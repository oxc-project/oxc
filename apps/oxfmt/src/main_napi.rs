use std::ffi::OsString;

use napi_derive::napi;

use oxc_napi::OxcError;
use serde_json::Value;

use crate::{
    api::{format_api, text_to_doc_api},
    cli::{FormatRunner, MigrateSource, Mode, format_command, init_miette, init_rayon},
    core::{
        ExternalFormatter, JsFormatEmbeddedCb, JsFormatFileCb, JsInitExternalFormatterCb,
        JsSortTailwindClassesCb, utils,
    },
    lsp::run_lsp,
    stdin::StdinRunner,
};

/// NAPI based JS CLI entry point.
/// For pure Rust CLI entry point, see `main.rs`.
///
/// JS side passes in:
/// 1. `args`: Command line arguments (process.argv.slice(2))
/// 2. `init_external_formatter_cb`: Callback to initialize external formatter
/// 3. `format_embedded_cb`: Callback to format embedded code in templates
/// 4. `format_file_cb`: Callback to format files
/// 5. `sort_tailwindcss_classes_cb`: Callback to sort Tailwind classes
///
/// Returns a tuple of `[mode, exitCode]`:
/// - `mode`: If main logic will run in JS side, use this to indicate which mode
/// - `exitCode`: If main logic already ran in Rust side, return the exit code
#[expect(clippy::allow_attributes)]
#[allow(clippy::trailing_empty_array, clippy::unused_async)] // https://github.com/napi-rs/napi-rs/issues/2758
#[napi]
pub async fn run_cli(
    args: Vec<String>,
    #[napi(ts_arg_type = "(numThreads: number) => Promise<string[]>")]
    init_external_formatter_cb: JsInitExternalFormatterCb,
    #[napi(ts_arg_type = "(options: Record<string, any>, code: string) => Promise<string>")]
    format_embedded_cb: JsFormatEmbeddedCb,
    #[napi(ts_arg_type = "(options: Record<string, any>, code: string) => Promise<string>")]
    format_file_cb: JsFormatFileCb,
    #[napi(ts_arg_type = "(options: Record<string, any>, classes: string[]) => Promise<string[]>")]
    sort_tailwindcss_classes_cb: JsSortTailwindClassesCb,
) -> (String, Option<u8>) {
    // Convert `String` args to `OsString` for compatibility with `bpaf`
    let args: Vec<OsString> = args.into_iter().map(OsString::from).collect();

    // Use `run_inner()` to report errors instead of panicking
    let command = match format_command().run_inner(&*args) {
        Ok(cmd) => cmd,
        Err(e) => {
            e.print_message(100);
            // `bpaf` returns exit_code 0 for --help/--version, non-0 for parse errors
            let exit_code = u8::from(e.exit_code() != 0);
            return ("cli".to_string(), Some(exit_code));
        }
    };

    // Early return for modes that handle everything in JS side
    match command.mode {
        Mode::Init => {
            return ("init".to_string(), None);
        }
        Mode::Migrate(source) => {
            let mode_str = match source {
                MigrateSource::Prettier => "migrate:prettier",
                MigrateSource::Biome => "migrate:biome",
            };
            return (mode_str.to_string(), None);
        }
        _ => {}
    }

    // Otherwise, handle modes that require Rust side processing

    let external_formatter = ExternalFormatter::new(
        init_external_formatter_cb,
        format_embedded_cb,
        format_file_cb,
        sort_tailwindcss_classes_cb,
    );

    utils::init_tracing();
    let result = match command.mode {
        Mode::Lsp => {
            run_lsp(external_formatter.clone()).await;

            ("lsp".to_string(), Some(0))
        }
        Mode::Stdin(_) => {
            init_miette();

            let result = StdinRunner::new(command, external_formatter.clone()).run();

            ("stdin".to_string(), Some(result.exit_code()))
        }
        Mode::Cli(_) => {
            init_miette();
            init_rayon(command.runtime_options.threads);

            let result = FormatRunner::new(command)
                .with_external_formatter(Some(external_formatter.clone()))
                .run();

            ("cli".to_string(), Some(result.exit_code()))
        }
        _ => unreachable!("All other modes must have been handled above match arm"),
    };

    // Explicitly drop ThreadsafeFunctions before returning to prevent
    // use-after-free during V8 cleanup (Node.js issue with TSFN cleanup timing)
    external_formatter.cleanup();

    result
}

// ---

#[napi(object)]
pub struct FormatResult {
    /// The formatted code.
    pub code: String,
    /// Parse and format errors.
    pub errors: Vec<OxcError>,
}

/// NAPI based format API entry point.
///
/// Since it internally uses `await prettier.format()` in JS side, `formatSync()` cannot be provided.
///
/// # Panics
/// Panics if the current working directory cannot be determined.
#[expect(clippy::allow_attributes)]
#[allow(clippy::trailing_empty_array, clippy::unused_async)] // https://github.com/napi-rs/napi-rs/issues/2758
#[napi]
pub async fn format(
    filename: String,
    source_text: String,
    options: Option<Value>,
    #[napi(ts_arg_type = "(numThreads: number) => Promise<string[]>")]
    init_external_formatter_cb: JsInitExternalFormatterCb,
    #[napi(ts_arg_type = "(options: Record<string, any>, code: string) => Promise<string>")]
    format_embedded_cb: JsFormatEmbeddedCb,
    #[napi(ts_arg_type = "(options: Record<string, any>, code: string) => Promise<string>")]
    format_file_cb: JsFormatFileCb,
    #[napi(ts_arg_type = "(options: Record<string, any>, classes: string[]) => Promise<string[]>")]
    sort_tailwind_classes_cb: JsSortTailwindClassesCb,
) -> FormatResult {
    let format_api::ApiFormatResult { code, errors } = format_api::run(
        &filename,
        source_text,
        options,
        init_external_formatter_cb,
        format_embedded_cb,
        format_file_cb,
        sort_tailwind_classes_cb,
    );

    FormatResult { code, errors }
}

// ---

#[napi(object)]
pub struct TextToDocResult {
    /// The formatted code.
    pub doc: String,
    /// Parse and format errors.
    pub errors: Vec<OxcError>,
}

/// NAPI based `textToDoc` API entry point for `prettier-plugin-oxfmt`.
///
/// This API is specialized for JS/TS snippets embedded in non-JS files.
/// Unlike `format()`, it is called only for JS/TS-in-xxx `textToDoc` flow.
///
/// # Panics
/// Panics if the current working directory cannot be determined.
#[expect(clippy::allow_attributes)]
#[allow(clippy::trailing_empty_array, clippy::unused_async)] // https://github.com/napi-rs/napi-rs/issues/2758
#[napi]
pub async fn js_text_to_doc(
    filename: String,
    source_text: String,
    oxfmt_plugin_options_json: String,
    parent_context: String,
    #[napi(ts_arg_type = "(numThreads: number) => Promise<string[]>")]
    init_external_formatter_cb: JsInitExternalFormatterCb,
    #[napi(ts_arg_type = "(options: Record<string, any>, code: string) => Promise<string>")]
    format_embedded_cb: JsFormatEmbeddedCb,
    #[napi(ts_arg_type = "(options: Record<string, any>, code: string) => Promise<string>")]
    format_file_cb: JsFormatFileCb,
    #[napi(ts_arg_type = "(options: Record<string, any>, classes: string[]) => Promise<string[]>")]
    sort_tailwind_classes_cb: JsSortTailwindClassesCb,
) -> TextToDocResult {
    match text_to_doc_api::run(
        &filename,
        &source_text,
        &oxfmt_plugin_options_json,
        &parent_context,
        init_external_formatter_cb,
        format_embedded_cb,
        format_file_cb,
        sort_tailwind_classes_cb,
    ) {
        Ok(doc) => TextToDocResult { doc, errors: vec![] },
        Err(errors) => TextToDocResult { doc: String::new(), errors },
    }
}
