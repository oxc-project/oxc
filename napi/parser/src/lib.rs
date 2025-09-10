// Napi value need to be passed as value
#![expect(clippy::needless_pass_by_value)]

#[cfg(all(
    feature = "allocator",
    not(any(target_arch = "arm", target_os = "freebsd", target_family = "wasm"))
))]
#[global_allocator]
static ALLOC: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;

use std::mem;

use napi::{Task, bindgen_prelude::AsyncTask};
use napi_derive::napi;

use oxc::{
    allocator::Allocator,
    parser::{ParseOptions, Parser, ParserReturn},
    semantic::SemanticBuilder,
    span::SourceType,
};
use oxc_napi::{Comment, OxcError, convert_utf8_to_utf16, get_source_type};

mod convert;
mod types;
pub use types::{EcmaScriptModule, ParseResult, ParserOptions};

// Raw transfer is only supported on 64-bit little-endian systems.
// Don't include raw transfer code on other platforms (notably WASM32).
// `raw_transfer_types` still needs to be compiled, as `assert_layouts` refers to those types,
// but it's all dead code on unsupported platforms, and will be excluded from binary.
#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
mod raw_transfer;
mod raw_transfer_types;
#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
pub use raw_transfer::{
    get_buffer_offset, parse_async_raw, parse_sync_raw, raw_transfer_supported,
};

// Fallback for 32-bit or big-endian platforms.
/// Returns `true` if raw transfer is supported on this platform.
#[cfg(not(all(target_pointer_width = "64", target_endian = "little")))]
#[napi]
pub fn raw_transfer_supported() -> bool {
    false
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

fn parse<'a>(
    allocator: &'a Allocator,
    source_type: SourceType,
    source_text: &'a str,
    options: &ParserOptions,
) -> ParserReturn<'a> {
    Parser::new(allocator, source_text, source_type)
        .with_options(ParseOptions {
            preserve_parens: options.preserve_parens.unwrap_or(true),
            ..ParseOptions::default()
        })
        .parse()
}

fn parse_with_return(filename: &str, source_text: String, options: &ParserOptions) -> ParseResult {
    let allocator = Allocator::default();
    let source_type =
        get_source_type(filename, options.lang.as_deref(), options.source_type.as_deref());
    let ast_type = get_ast_type(source_type, options);
    let ranges = options.range.unwrap_or(false);
    let ret = parse(&allocator, source_type, &source_text, options);

    let mut program = ret.program;
    let mut module_record = ret.module_record;
    let mut diagnostics = ret.errors;

    if options.show_semantic_errors == Some(true) {
        let semantic_ret = SemanticBuilder::new().with_check_syntax_error(true).build(&program);
        diagnostics.extend(semantic_ret.errors);
    }

    let mut errors = OxcError::from_diagnostics(filename, &source_text, diagnostics);

    let mut comments =
        convert_utf8_to_utf16(&source_text, &mut program, &mut module_record, &mut errors);

    let program_and_fixes = match ast_type {
        AstType::JavaScript => {
            // Add hashbang to start of comments
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

            program.to_estree_js_json_with_fixes(ranges)
        }
        AstType::TypeScript => {
            // Note: `@typescript-eslint/parser` ignores hashbangs,
            // despite appearances to the contrary in AST explorers.
            // So we ignore them too.
            // See: https://github.com/typescript-eslint/typescript-eslint/issues/6500
            program.to_estree_ts_json_with_fixes(ranges)
        }
    };

    let module = EcmaScriptModule::from(&module_record);

    ParseResult { program_and_fixes, module, comments, errors }
}

/// Parse synchronously.
#[napi]
pub fn parse_sync(
    filename: String,
    source_text: String,
    options: Option<ParserOptions>,
) -> ParseResult {
    let options = options.unwrap_or_default();
    parse_with_return(&filename, source_text, &options)
}

pub struct ResolveTask {
    filename: String,
    source_text: String,
    options: ParserOptions,
}

#[napi]
impl Task for ResolveTask {
    type JsValue = ParseResult;
    type Output = ParseResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let source_text = mem::take(&mut self.source_text);
        Ok(parse_with_return(&self.filename, source_text, &self.options))
    }

    fn resolve(&mut self, _: napi::Env, result: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(result)
    }
}

/// Parse asynchronously.
///
/// Note: This function can be slower than `parseSync` due to the overhead of spawning a thread.
#[napi]
pub fn parse_async(
    filename: String,
    source_text: String,
    options: Option<ParserOptions>,
) -> AsyncTask<ResolveTask> {
    let options = options.unwrap_or_default();
    AsyncTask::new(ResolveTask { filename, source_text, options })
}
