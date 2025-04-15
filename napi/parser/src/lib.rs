// Napi value need to be passed as value
#![expect(clippy::needless_pass_by_value)]

#[cfg(all(feature = "allocator", not(target_arch = "arm"), not(target_family = "wasm")))]
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
use oxc_napi::{OxcError, convert_utf8_to_utf16};

mod convert;
mod raw_transfer;
mod raw_transfer_types;
mod types;
pub use raw_transfer::{get_buffer_offset, parse_sync_raw, raw_transfer_supported};
pub use types::{EcmaScriptModule, ParseResult, ParserOptions};

mod generated {
    // Note: We intentionally don't import `generated/derive_estree.rs`. It's not needed.
    #[cfg(debug_assertions)]
    pub mod assert_layouts;
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AstType {
    JavaScript,
    TypeScript,
}

fn get_source_and_ast_type(filename: &str, options: &ParserOptions) -> (SourceType, AstType) {
    let source_type = match options.lang.as_deref() {
        Some("js") => SourceType::mjs(),
        Some("jsx") => SourceType::jsx(),
        Some("ts") => SourceType::ts(),
        Some("tsx") => SourceType::tsx(),
        _ => {
            let mut source_type = SourceType::from_path(filename).unwrap_or_default();
            // Force `script` or `module`
            match options.source_type.as_deref() {
                Some("script") => source_type = source_type.with_script(true),
                Some("module") => source_type = source_type.with_module(true),
                _ => {}
            }
            source_type
        }
    };

    let ast_type = match options.ast_type.as_deref() {
        Some("js") => AstType::JavaScript,
        Some("ts") => AstType::TypeScript,
        _ => {
            if source_type.is_javascript() {
                AstType::JavaScript
            } else {
                AstType::TypeScript
            }
        }
    };

    (source_type, ast_type)
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
    let (source_type, ast_type) = get_source_and_ast_type(filename, options);
    let ret = parse(&allocator, source_type, &source_text, options);

    let mut program = ret.program;
    let mut module_record = ret.module_record;
    let mut diagnostics = ret.errors;

    if options.show_semantic_errors == Some(true) {
        let semantic_ret = SemanticBuilder::new().with_check_syntax_error(true).build(&program);
        diagnostics.extend(semantic_ret.errors);
    }

    let mut errors = OxcError::from_diagnostics(filename, &source_text, diagnostics);

    let comments =
        convert_utf8_to_utf16(&source_text, &mut program, &mut module_record, &mut errors);

    let program = match ast_type {
        AstType::JavaScript => program.to_estree_js_json(),
        AstType::TypeScript => program.to_estree_ts_json(),
    };

    let module = EcmaScriptModule::from(&module_record);

    ParseResult { program, module, comments, errors }
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

#[napi]
pub fn call_threadsafe_function(callback: napi::JsFunction) -> napi::Result<()> {
    use std::sync::Arc;

    use napi::threadsafe_function::{ThreadsafeFunction, UnknownReturnValue};

    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let tsfn: ThreadsafeFunction<u32, UnknownReturnValue, _> = callback
            .create_threadsafe_function(|ctx| ctx.env.create_uint32(ctx.value + 1).map(|v| vec![v]))
            .unwrap();
        let tsfn = Arc::new(tsfn);

        for n in 0..10 {
            let tsfn = tsfn.clone();
            tokio::spawn(async move {
                let _ = tsfn.call_async(Ok(n)).await;
            });
        }
    });

    Ok(())
}
