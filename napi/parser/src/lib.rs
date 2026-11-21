use napi::{JsString, Task, bindgen_prelude::AsyncTask};
use napi_derive::napi;

#[cfg(not(feature = "tokens"))]
use oxc::parser::config::NoTokensParserConfig;
#[cfg(feature = "tokens")]
use oxc::parser::config::RuntimeParserConfig;
use oxc::{
    allocator::Allocator,
    parser::{ParseOptions, ParserReturn, Source},
    semantic::SemanticBuilder,
    span::SourceType,
};
use oxc_napi::{Comment, OxcError, SendableSource, convert_utf8_to_utf16, get_source_type};

mod convert;
mod types;
pub use types::*;

#[cfg(all(
    feature = "allocator",
    not(any(
        target_arch = "arm",
        target_os = "android",
        target_os = "freebsd",
        target_os = "windows",
        target_family = "wasm"
    ))
))]
#[global_allocator]
static ALLOC: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;

// Raw transfer is only supported on 64-bit little-endian systems.
// Don't include raw transfer code on other platforms (notably WASM32).
// `raw_transfer_types` still needs to be compiled, as `assert_layouts` refers to those types,
// but it's all dead code on unsupported platforms, and will be excluded from binary.
#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
mod raw_transfer;
mod raw_transfer_types;
#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
pub use raw_transfer::{get_buffer_offset, parse_raw, parse_raw_sync};

/// Returns `true` if raw transfer is supported on this platform.
#[napi]
pub fn raw_transfer_supported() -> bool {
    cfg!(all(target_pointer_width = "64", target_endian = "little"))
}

mod generated {
    // Note: We intentionally don't import `generated/derive_estree.rs`. It's not needed.
    #[cfg(debug_assertions)]
    mod assert_layouts;
    #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
    pub mod raw_transfer_constants;
}
#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
use generated::raw_transfer_constants;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AstType {
    JavaScript,
    TypeScript,
}

fn get_ast_type(source_type: SourceType, options: &ParserOptions) -> AstType {
    match options.ast_type.as_deref() {
        Some("js") => AstType::JavaScript,
        Some("ts") => AstType::TypeScript,
        _ => {
            if source_type.is_javascript() {
                AstType::JavaScript
            } else {
                AstType::TypeScript
            }
        }
    }
}

/// Parse `source`, with parse options and parser config set according to `options`.
fn parse_impl<'a>(source: Source<'a>, options: &ParserOptions) -> ParserReturn<'a> {
    let parse_options = ParseOptions {
        preserve_parens: options.preserve_parens.unwrap_or(true),
        // Ident hashes are only consumed by semantic analysis, which this crate runs solely when
        // semantic errors are requested. Skip hashing otherwise to speed up parse-and-serialize.
        enable_ident_hashes: options.show_semantic_errors == Some(true),
        ..ParseOptions::default()
    };

    // When `tokens` feature is disabled, parser uses the default `NoTokensParserConfig`,
    // which avoids the runtime branch on whether to collect tokens, and so is faster.
    // The `experimentalTokens` option in `ParserOptions` is silently ignored in that case.
    #[cfg(feature = "tokens")]
    let config = RuntimeParserConfig::new(options.tokens.unwrap_or(false));
    #[cfg(not(feature = "tokens"))]
    let config = NoTokensParserConfig;

    source.parse_with_config(parse_options, config)
}

/// Get [`SourceType`] for a file, from its filename and `options`.
fn source_type_for(filename: &str, options: &ParserOptions) -> SourceType {
    get_source_type(filename, options.lang.as_deref(), options.source_type.as_deref())
}

/// Parse `source`, and convert the AST, module record, comments and errors to a [`ParseResult`].
fn parse_with_return(filename: &str, source: Source<'_>, options: &ParserOptions) -> ParseResult {
    let ast_type = get_ast_type(source.source_type(), options);
    let ranges = options.range.unwrap_or(false);
    let ret = parse_impl(source, options);

    let mut program = ret.program;
    let source_text = program.source_text;
    let mut module_record = ret.module_record;
    let mut diagnostics = ret.diagnostics;

    if options.show_semantic_errors == Some(true) {
        let semantic_ret = SemanticBuilder::new_compiler().build(&program);
        diagnostics.extend(semantic_ret.diagnostics);
    }

    let mut errors = OxcError::from_diagnostics(filename, source_text, diagnostics);

    let mut comments =
        convert_utf8_to_utf16(source_text, &mut program, &mut module_record, &mut errors);

    if ast_type == AstType::JavaScript {
        // Add hashbang to start of comments.
        // Note: `@typescript-eslint/parser` ignores hashbangs, despite appearances to the contrary in AST explorers.
        // So we ignore them too for TS.
        // See: https://github.com/typescript-eslint/typescript-eslint/issues/6500
        if let Some(hashbang) = &program.hashbang {
            comments.insert(
                0,
                Comment {
                    r#type: "Line".to_string(),
                    value: hashbang.value.to_string(),
                    start: hashbang.span.start,
                    end: hashbang.span.end,
                },
            );
        }
    }

    let include_ts_fields = ast_type == AstType::TypeScript;
    let program_and_fixes = program.to_estree_json_with_fixes(include_ts_fields, ranges);

    let module = EcmaScriptModule::from(&module_record);

    ParseResult { program_and_fixes, module, comments, errors }
}

/// Parse JS/TS source synchronously on current thread.
///
/// This is generally preferable over `parse` (async) as it does not have the overhead
/// of spawning a thread, and the majority of the workload cannot be parallelized anyway
/// (see `parse` documentation for details).
///
/// If you need to parallelize parsing multiple files, it is recommended to use worker threads
/// with `parseSync` rather than using `parse`.
#[napi]
#[allow(clippy::needless_pass_by_value, clippy::missing_errors_doc, clippy::allow_attributes)]
pub fn parse_sync(
    filename: String,
    source_text: JsString,
    options: Option<ParserOptions>,
) -> napi::Result<ParseResult> {
    let options = options.unwrap_or_default();
    let source_type = source_type_for(&filename, &options);

    // Copy source text from JS string directly into the arena
    let allocator = Allocator::default();
    let source = Source::from_napi_js_string(&source_text, source_type, &allocator)?;

    Ok(parse_with_return(&filename, source, &options))
}

pub struct ResolveTask {
    /// File path.
    filename: String,
    /// Source text. `None` after `compute` has run.
    source: Option<SendableSource>,
    /// Options, as passed in from JS.
    options: Option<ParserOptions>,
}

#[napi]
impl Task for ResolveTask {
    type JsValue = ParseResult;
    type Output = ParseResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let options = &*self.options.get_or_insert_default();
        let source_type = source_type_for(&self.filename, options);

        // Take the source out of the task, so its `Allocator` is dropped at end of this method.
        // That frees its memory on this thread, not on main thread when the task is dropped.
        let mut sendable_source = self.source.take().expect("`compute` called more than once");
        let source = sendable_source.source(source_type);

        Ok(parse_with_return(&self.filename, source, options))
    }

    fn resolve(&mut self, _: napi::Env, result: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(result)
    }
}

/// Parse JS/TS source asynchronously on a separate thread.
///
/// Note that not all of the workload can happen on a separate thread.
/// Parsing on Rust side does happen in a separate thread, but deserialization of the AST to JS objects
/// has to happen on current thread. This synchronous deserialization work typically outweighs
/// the asynchronous parsing by a factor of between 3 and 20.
///
/// i.e. the majority of the workload cannot be parallelized by using this method.
///
/// Generally `parseSync` is preferable to use as it does not have the overhead of spawning a thread.
/// If you need to parallelize parsing multiple files, it is recommended to use worker threads.
#[napi]
#[allow(clippy::missing_errors_doc, clippy::allow_attributes)]
pub fn parse(
    filename: String,
    source_text: JsString,
    options: Option<ParserOptions>,
) -> napi::Result<AsyncTask<ResolveTask>> {
    // Copy source text from JS string directly into an arena.
    // This has to happen here, on main thread, as JS values cannot be accessed on other threads.
    let source = SendableSource::from_napi_js_string(&source_text)?;

    Ok(AsyncTask::new(ResolveTask { filename, source: Some(source), options }))
}
