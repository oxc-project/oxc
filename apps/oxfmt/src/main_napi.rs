use std::ffi::OsString;
use std::path::PathBuf;

use napi_derive::napi;

use oxc_napi::OxcError;
use serde_json::Value;

use crate::{
    cli::{FormatRunner, MigrateSource, Mode, format_command, init_miette, init_rayon},
    core::{
        ExternalFormatter, FormatFileStrategy, FormatResult as CoreFormatResult,
        JsFormatEmbeddedCb, JsFormatFileCb, JsInitExternalFormatterCb, JsSortTailwindClassesCb,
        SourceFormatter, oxfmtrc::FormatConfig, resolve_options_from_value, utils,
        wrap_format_embedded_only, wrap_sort_tailwind_for_doc,
    },
    lsp::run_lsp,
    oxfmtrc::populate_prettier_config,
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
    #[napi(
        ts_arg_type = "(options: Record<string, any>, parserName: string, code: string) => Promise<string>"
    )]
    format_embedded_cb: JsFormatEmbeddedCb,
    #[napi(
        ts_arg_type = "(options: Record<string, any>, parserName: string, fileName: string, code: string) => Promise<string>"
    )]
    format_file_cb: JsFormatFileCb,
    #[napi(
        ts_arg_type = "(filepath: string, options: Record<string, any>, classes: string[]) => Promise<string[]>"
    )]
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
#[expect(clippy::allow_attributes)]
#[allow(clippy::trailing_empty_array, clippy::unused_async)] // https://github.com/napi-rs/napi-rs/issues/2758
#[napi]
pub async fn format(
    filename: String,
    source_text: String,
    options: Option<Value>,
    #[napi(ts_arg_type = "(numThreads: number) => Promise<string[]>")]
    init_external_formatter_cb: JsInitExternalFormatterCb,
    #[napi(
        ts_arg_type = "(options: Record<string, any>, parserName: string, code: string) => Promise<string>"
    )]
    format_embedded_cb: JsFormatEmbeddedCb,
    #[napi(
        ts_arg_type = "(options: Record<string, any>, parserName: string, fileName: string, code: string) => Promise<string>"
    )]
    format_file_cb: JsFormatFileCb,
    #[napi(
        ts_arg_type = "(filepath: string, options: Record<string, any>, classes: string[]) => Promise<string[]>"
    )]
    sort_tailwind_classes_cb: JsSortTailwindClassesCb,
) -> FormatResult {
    let num_of_threads = 1;

    let external_formatter = ExternalFormatter::new(
        init_external_formatter_cb,
        format_embedded_cb,
        format_file_cb,
        sort_tailwind_classes_cb,
    );

    // Use `block_in_place()` to avoid nested async runtime access
    match tokio::task::block_in_place(|| external_formatter.init(num_of_threads)) {
        // TODO: Plugins support
        Ok(_) => {}
        Err(err) => {
            external_formatter.cleanup();
            return FormatResult {
                code: source_text,
                errors: vec![OxcError::new(format!("Failed to setup external formatter: {err}"))],
            };
        }
    }

    // Determine format strategy from file path
    let Ok(strategy) = FormatFileStrategy::try_from(PathBuf::from(&filename)) else {
        external_formatter.cleanup();
        return FormatResult {
            code: source_text,
            errors: vec![OxcError::new(format!("Unsupported file type: {filename}"))],
        };
    };

    // Resolve format options directly from the provided options
    let resolved_options = match resolve_options_from_value(options.unwrap_or_default(), &strategy)
    {
        Ok(options) => options,
        Err(err) => {
            external_formatter.cleanup();
            return FormatResult {
                code: source_text,
                errors: vec![OxcError::new(format!("Failed to parse configuration: {err}"))],
            };
        }
    };

    // Create formatter and format
    let formatter = SourceFormatter::new(num_of_threads)
        .with_external_formatter(Some(external_formatter.clone()));

    // Use `block_in_place()` to avoid nested async runtime access
    let result = match tokio::task::block_in_place(|| {
        formatter.format(&strategy, &source_text, resolved_options)
    }) {
        CoreFormatResult::Success { code, .. } => FormatResult { code, errors: vec![] },
        CoreFormatResult::Error(diagnostics) => {
            let errors = OxcError::from_diagnostics(&filename, &source_text, diagnostics);
            FormatResult { code: source_text, errors }
        }
    };

    // Explicitly drop ThreadsafeFunctions before returning to prevent
    // use-after-free during V8 cleanup (Node.js issue with TSFN cleanup timing)
    external_formatter.cleanup();

    result
}

// ---

/// NAPI function for Prettier plugin integration.
///
/// This function is called from the Prettier plugin's `parse()` function.
/// It formats the source code using oxc_formatter and returns the result as a string (Doc).
///
/// The `options` parameter contains Prettier-style options (useTabs, singleQuote, etc.)
/// which are converted to oxc_formatter's FormatOptions.
///
/// Returns the formatted code as a string, which Prettier treats as a Doc.
#[expect(clippy::allow_attributes)]
#[allow(
    clippy::trailing_empty_array,
    clippy::unused_async,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]
#[napi]
pub async fn format_to_doc(
    filepath: String,
    source_filename: String,
    source_text: String,
    options: Option<Value>,
    #[napi(
        ts_arg_type = "(options: Record<string, any>, parserName: string, code: string) => Promise<string>"
    )]
    format_embedded_cb: Option<JsFormatEmbeddedCb>,
    #[napi(
        ts_arg_type = "(filepath: string, options: Record<string, any>, classes: string[]) => Promise<string[]>"
    )]
    sort_tailwind_classes_cb: Option<JsSortTailwindClassesCb>,
) -> napi::Result<String> {
    use oxc_allocator::Allocator;
    use oxc_formatter::{ExternalCallbacks, Formatter, get_parse_options};
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    // Parse source type
    let Ok(source_type) = SourceType::from_path(&source_filename) else {
        return Err(napi::Error::from_reason(format!(
            "Invalid source filename: {source_filename}. Expected '.js', '.ts', '.jsx', or '.tsx'"
        )));
    };

    let Ok(format_config) = serde_json::from_value::<FormatConfig>(options.unwrap_or_default())
    else {
        return Err(napi::Error::from_reason("Failed to deserialize FormatConfig".to_string()));
    };

    let mut external_options =
        serde_json::to_value(&format_config).expect("FormatConfig serialization should not fail");

    let Ok(oxfmt_options) = format_config.into_oxfmt_options() else {
        return Err(napi::Error::from_reason("Failed to parse configuration."));
    };
    populate_prettier_config(&oxfmt_options.format_options, &mut external_options);

    // Create external callbacks for embedded language formatting and Tailwind sorting
    let embedded_callback =
        format_embedded_cb.map(|cb| wrap_format_embedded_only(cb, external_options.clone()));

    let tailwind_callback = sort_tailwind_classes_cb
        .filter(|_| oxfmt_options.format_options.experimental_tailwindcss.is_some())
        .map(|cb| wrap_sort_tailwind_for_doc(cb, filepath, external_options));

    let external_callbacks = Some(
        ExternalCallbacks::new()
            .with_embedded_formatter(embedded_callback)
            .with_tailwind(tailwind_callback),
    );

    // Parse and format
    // Use `block_in_place()` to avoid nested async runtime access when embedded callback is used
    let code = tokio::task::block_in_place(|| {
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, &source_text, source_type)
            .with_options(get_parse_options())
            .parse();

        if !ret.errors.is_empty() {
            let error = ret.errors.into_iter().next().unwrap();
            return Err(napi::Error::from_reason(format!("Parse error: {error}")));
        }

        let formatter = Formatter::new(&allocator, oxfmt_options.format_options);
        let formatted = formatter.format_with_external_callbacks(&ret.program, external_callbacks);

        let code = formatted.print().map_err(|err| {
            napi::Error::from_reason(format!("Failed to print formatted code: {err}"))
        })?;

        Ok(code.into_code())
    })?;

    Ok(code)
}
