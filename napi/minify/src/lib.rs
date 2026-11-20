#![expect(clippy::needless_pass_by_value)]

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

mod options;

use std::{collections::BTreeMap, path::PathBuf};

use napi::{Either, JsString, Task, bindgen_prelude::AsyncTask};
use napi_derive::napi;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_diagnostics::OxcDiagnostic;
use oxc_minifier::Minifier;
use oxc_napi::{OxcError, SendableSource};
use oxc_parser::Source;
use oxc_sourcemap::napi::SourceMap;
use oxc_span::SourceType;

pub use crate::options::*;

#[derive(Default)]
#[napi(object)]
pub struct MinifyResult {
    pub code: String,
    pub map: Option<SourceMap>,
    pub errors: Vec<OxcError>,
    /// Legal comments extracted from the source code.
    /// Only populated when `codegen.legalComments` is `"linked"` or `"external"`.
    pub legal_comments: Vec<String>,
    /// Updated property-name cache sorted by original name. Present when `mangleProps` ran on a
    /// parse without errors.
    #[napi(ts_type = "Record<string, string | false>")]
    pub mangle_cache: Option<BTreeMap<String, Either<String, bool>>>,
}

fn to_napi_mangle_cache(
    cache: oxc_minifier::ManglePropertyCache,
) -> BTreeMap<String, Either<String, bool>> {
    cache
        .into_iter()
        .map(|(original, target)| {
            let value = target.map_or(Either::B(false), |target| Either::A(target.to_string()));
            (original.to_string(), value)
        })
        .collect()
}

/// Get [`SourceType`] for a file, from its filename and `options`.
fn source_type_for(filename: &str, options: &MinifyOptions) -> SourceType {
    if options.module == Some(true) {
        SourceType::mjs()
    } else {
        SourceType::from_path(filename).unwrap_or_default()
    }
}

/// Parse `source`, minify it, and generate code, with options set according to `options`.
fn minify_impl(filename: &str, source: Source<'_>, options: &MinifyOptions) -> MinifyResult {
    use oxc_codegen::CodegenOptions;

    let mut minifier_options = match oxc_minifier::MinifierOptions::try_from(options) {
        Ok(options) => options,
        Err(error) => {
            // Diagnostic has no labels, so source text is not needed to render it
            return MinifyResult {
                errors: OxcError::from_diagnostics(filename, "", vec![OxcDiagnostic::error(error)]),
                ..MinifyResult::default()
            };
        }
    };

    let allocator = source.allocator();

    let parser_ret = source.parse();
    if !parser_ret.diagnostics.is_empty() {
        // A recovered AST may be incomplete, so skip property mangling and its cache.
        minifier_options.mangle_properties = None;
    }

    let mut program = parser_ret.program;
    let source_text = program.source_text;

    let minifier_ret = Minifier::new(minifier_options).minify(allocator, &mut program);
    let scoping = minifier_ret.scoping;
    let class_private_mappings = minifier_ret.class_private_mappings;
    let mangle_cache = parser_ret
        .diagnostics
        .is_empty()
        .then(|| minifier_ret.property_mangle_cache.map(to_napi_mangle_cache))
        .flatten();

    let mut codegen_options = match &options.codegen {
        // Need to remove all comments.
        Some(Either::A(false)) => CodegenOptions { minify: false, ..CodegenOptions::minify() },
        None | Some(Either::A(true)) => CodegenOptions::minify(),
        Some(Either::B(o)) => match o.to_codegen_options() {
            Ok(opts) => opts,
            Err(error) => {
                return MinifyResult {
                    errors: OxcError::from_diagnostics(
                        filename,
                        source_text,
                        vec![OxcDiagnostic::error(error)],
                    ),
                    ..MinifyResult::default()
                };
            }
        },
    };

    if options.sourcemap == Some(true) {
        codegen_options.source_map_path = Some(PathBuf::from(&filename));
    }

    let ret = Codegen::new()
        .with_options(codegen_options)
        .with_scoping(scoping)
        .with_private_member_mappings(class_private_mappings)
        .build(&program);

    let legal_comments =
        ret.legal_comments.iter().map(|c| c.span.source_text(source_text).to_string()).collect();

    MinifyResult {
        code: ret.code,
        map: ret.map.map(oxc_sourcemap::napi::SourceMap::from),
        errors: OxcError::from_diagnostics(filename, source_text, parser_ret.diagnostics),
        legal_comments,
        mangle_cache,
    }
}

/// Minify synchronously.
#[napi]
#[allow(clippy::missing_errors_doc, clippy::allow_attributes)]
pub fn minify_sync(
    filename: String,
    source_text: JsString,
    options: Option<MinifyOptions>,
) -> napi::Result<MinifyResult> {
    let options = options.unwrap_or_default();
    let source_type = source_type_for(&filename, &options);

    // Copy source text from JS string directly into the arena
    let allocator = Allocator::default();
    let source = Source::from_napi_js_string(&source_text, source_type, &allocator)?;

    Ok(minify_impl(&filename, source, &options))
}

pub struct MinifyTask {
    filename: String,
    /// Source text. `None` after `compute` has run.
    source: Option<SendableSource>,
    options: Option<MinifyOptions>,
}

#[napi]
impl Task for MinifyTask {
    type JsValue = MinifyResult;
    type Output = MinifyResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let options = &*self.options.get_or_insert_default();
        let source_type = source_type_for(&self.filename, options);

        // Take the source out of the task, so its `Allocator` is dropped at end of this method.
        // That frees its memory on this thread, not on main thread when the task is dropped.
        let mut sendable_source = self.source.take().expect("`compute` called more than once");
        let source = sendable_source.source(source_type);

        Ok(minify_impl(&self.filename, source, options))
    }

    fn resolve(&mut self, _: napi::Env, result: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(result)
    }
}

/// Minify asynchronously.
///
/// Note: This function can be slower than `minifySync` due to the overhead of spawning a thread.
#[napi]
#[allow(clippy::missing_errors_doc, clippy::allow_attributes)]
pub fn minify(
    filename: String,
    source_text: JsString,
    options: Option<MinifyOptions>,
) -> napi::Result<AsyncTask<MinifyTask>> {
    // Copy source text from JS string directly into an arena.
    // This has to happen here, on main thread, as JS values cannot be accessed on other threads.
    let source = SendableSource::from_napi_js_string(&source_text)?;

    Ok(AsyncTask::new(MinifyTask { filename, source: Some(source), options }))
}
