use std::ffi::OsString;
use std::path::PathBuf;

use napi_derive::napi;

use oxc_napi::OxcError;
use serde_json::Value;

use crate::{
    cli::{FormatRunner, Mode, format_command, init_miette, init_rayon},
    core::{
        ConfigResolver, ExternalFormatter, FormatFileStrategy, FormatResult as CoreFormatResult,
        JsFormatEmbeddedCb, JsFormatFileCb, JsInitExternalFormatterCb, JsSortTailwindClassesCb,
        SourceFormatter, utils, wrap_format_embedded_only, wrap_sort_tailwind_for_doc,
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
        Mode::Migrate(_) => {
            return ("migrate:prettier".to_string(), None);
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
    match command.mode {
        Mode::Lsp => {
            run_lsp(external_formatter).await;

            ("lsp".to_string(), Some(0))
        }
        Mode::Stdin(_) => {
            init_miette();

            let result = StdinRunner::new(command, external_formatter).run();

            ("stdin".to_string(), Some(result.exit_code()))
        }
        Mode::Cli(_) => {
            init_miette();
            init_rayon(command.runtime_options.threads);

            let result =
                FormatRunner::new(command).with_external_formatter(Some(external_formatter)).run();

            ("cli".to_string(), Some(result.exit_code()))
        }
        _ => unreachable!("All other modes must have been handled above match arm"),
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

    // Create resolver from options and resolve format options
    let mut config_resolver = ConfigResolver::from_value(options.unwrap_or_default());
    match config_resolver.build_and_validate() {
        Ok(_) => {}
        Err(err) => {
            return FormatResult {
                code: source_text,
                errors: vec![OxcError::new(format!("Failed to parse configuration: {err}"))],
            };
        }
    }

    // Use `block_in_place()` to avoid nested async runtime access
    match tokio::task::block_in_place(|| external_formatter.init(num_of_threads)) {
        // TODO: Plugins support
        Ok(_) => {}
        Err(err) => {
            return FormatResult {
                code: source_text,
                errors: vec![OxcError::new(format!("Failed to setup external formatter: {err}"))],
            };
        }
    }

    // Determine format strategy from file path
    let Ok(strategy) = FormatFileStrategy::try_from(PathBuf::from(&filename)) else {
        return FormatResult {
            code: source_text,
            errors: vec![OxcError::new(format!("Unsupported file type: {filename}"))],
        };
    };

    let resolved_options = config_resolver.resolve(&strategy);

    // Create formatter and format
    let formatter =
        SourceFormatter::new(num_of_threads).with_external_formatter(Some(external_formatter));

    // Use `block_in_place()` to avoid nested async runtime access
    match tokio::task::block_in_place(|| {
        formatter.format(&strategy, &source_text, resolved_options)
    }) {
        CoreFormatResult::Success { code, .. } => FormatResult { code, errors: vec![] },
        CoreFormatResult::Error(diagnostics) => {
            let errors = OxcError::from_diagnostics(&filename, &source_text, diagnostics);
            FormatResult { code: source_text, errors }
        }
    }
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
    source_text: String,
    #[napi(ts_arg_type = "'js' | 'ts' | 'jsx' | 'tsx'")] source_type: String,
    filepath: String,
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
    use oxc_formatter::{ExternalCallbacks, Formatter, enable_jsx_source_type, get_parse_options};
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    // Parse source type
    let source_type = match source_type.as_str() {
        "js" => SourceType::mjs(),
        "jsx" => SourceType::jsx(),
        "ts" => SourceType::ts(),
        "tsx" => SourceType::tsx(),
        _ => {
            return Err(napi::Error::from_reason(format!(
                "Invalid source type: {source_type}. Expected 'js', 'ts', 'jsx', or 'tsx'"
            )));
        }
    };
    let source_type = enable_jsx_source_type(source_type);

    // Convert Prettier-style options to FormatOptions
    let external_options = options.unwrap_or_default();
    let format_options = convert_prettier_options_to_format_options(&external_options);

    // Create external callbacks for embedded language formatting and Tailwind sorting
    let embedded_callback =
        format_embedded_cb.map(|cb| wrap_format_embedded_only(cb, external_options.clone()));

    let tailwind_callback = sort_tailwind_classes_cb
        .filter(|_| format_options.experimental_tailwindcss.is_some())
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

        let formatter = Formatter::new(&allocator, format_options);
        let formatted = formatter.format_with_external_callbacks(&ret.program, external_callbacks);

        let code = formatted.print().map_err(|err| {
            napi::Error::from_reason(format!("Failed to print formatted code: {err}"))
        })?;

        Ok(code.into_code())
    })?;

    Ok(code)
}

/// Convert Prettier-style options to oxc_formatter's FormatOptions.
fn convert_prettier_options_to_format_options(options: &Value) -> oxc_formatter::FormatOptions {
    use oxc_formatter::{
        FormatOptions, IndentStyle, IndentWidth, LineEnding, LineWidth, QuoteStyle, Semicolons,
        SortImportsOptions, TailwindcssOptions,
    };

    let Some(obj) = options.as_object() else {
        return FormatOptions::default();
    };

    let mut format_options = FormatOptions::default();

    // useTabs -> indent_style
    if let Some(use_tabs) = obj.get("useTabs").and_then(Value::as_bool) {
        format_options.indent_style = if use_tabs { IndentStyle::Tab } else { IndentStyle::Space };
    }

    // tabWidth -> indent_width
    if let Some(tab_width) = obj.get("tabWidth").and_then(Value::as_u64) {
        format_options.indent_width = IndentWidth::try_from(tab_width as u8).unwrap_or_default();
    }

    // printWidth -> line_width
    if let Some(print_width) = obj.get("printWidth").and_then(Value::as_u64) {
        format_options.line_width = LineWidth::try_from(print_width as u16).unwrap_or_default();
    }

    // singleQuote -> quote_style
    if let Some(single_quote) = obj.get("singleQuote").and_then(Value::as_bool) {
        format_options.quote_style =
            if single_quote { QuoteStyle::Single } else { QuoteStyle::Double };
    }

    // jsxSingleQuote -> jsx_quote_style
    if let Some(jsx_single_quote) = obj.get("jsxSingleQuote").and_then(Value::as_bool) {
        format_options.jsx_quote_style =
            if jsx_single_quote { QuoteStyle::Single } else { QuoteStyle::Double };
    }

    // semi -> semicolons
    if let Some(semi) = obj.get("semi").and_then(Value::as_bool) {
        format_options.semicolons = if semi { Semicolons::Always } else { Semicolons::AsNeeded };
    }

    // endOfLine -> line_ending
    if let Some(end_of_line) = obj.get("endOfLine").and_then(Value::as_str) {
        format_options.line_ending = match end_of_line {
            "lf" => LineEnding::Lf,
            "crlf" => LineEnding::Crlf,
            "cr" => LineEnding::Cr,
            _ => LineEnding::Lf,
        };
    }

    // Check for Tailwind plugin enabled flag or experimentalTailwindcss option
    let tailwind_enabled =
        obj.get("_tailwindPluginEnabled").and_then(Value::as_bool).unwrap_or(false);
    if tailwind_enabled || obj.contains_key("experimentalTailwindcss") {
        format_options.experimental_tailwindcss = Some(TailwindcssOptions::default());
    }

    // experimentalSortImports -> experimental_sort_imports
    if obj.contains_key("experimentalSortImports") {
        format_options.experimental_sort_imports = Some(SortImportsOptions::default());
    }

    format_options
}
