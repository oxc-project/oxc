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

use std::path::PathBuf;

use napi::{Either, Task, bindgen_prelude::AsyncTask};
use napi_derive::napi;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_diagnostics::OxcDiagnostic;
use oxc_minifier::{Minifier, PropertyMangleBail, PropertyMangleBailKind};
use oxc_napi::OxcError;
use oxc_parser::Parser;
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
}

fn minify_impl(filename: &str, source_text: &str, options: Option<MinifyOptions>) -> MinifyResult {
    use oxc_codegen::CodegenOptions;
    let options = options.unwrap_or_default();

    let minifier_options = match oxc_minifier::MinifierOptions::try_from(&options) {
        Ok(options) => options,
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
    };

    let allocator = Allocator::default();

    let source_type = if options.module == Some(true) {
        SourceType::mjs()
    } else {
        SourceType::from_path(filename).unwrap_or_default()
    };

    let parser_ret = Parser::new(&allocator, source_text, source_type).parse();
    let mut program = parser_ret.program;

    let minifier_ret = Minifier::new(minifier_options).minify(&allocator, &mut program);
    let scoping = minifier_ret.scoping;
    // When property mangling bailed for the whole file, no property name was renamed; surface a
    // warning so callers are not surprised that names they expected mangled were left intact.
    let property_mangle_bail = minifier_ret.property_mangle_bail;

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

    let ret = Codegen::new().with_options(codegen_options).with_scoping(scoping).build(&program);

    let legal_comments =
        ret.legal_comments.iter().map(|c| c.span.source_text(source_text).to_string()).collect();

    // Report parser diagnostics plus a property-mangling bail warning (if any).
    let mut diagnostics = parser_ret.diagnostics;
    if let Some(bail) = property_mangle_bail {
        diagnostics.push(property_mangle_bail_warning(bail));
    }

    MinifyResult {
        code: ret.code,
        map: ret.map.map(oxc_sourcemap::napi::SourceMap::from),
        errors: OxcError::from_diagnostics(filename, source_text, diagnostics),
        legal_comments,
    }
}

/// Build a warning diagnostic for a whole-file property-mangling bail.
fn property_mangle_bail_warning(bail: PropertyMangleBail) -> OxcDiagnostic {
    let reason = match bail.kind {
        PropertyMangleBailKind::With => "a `with` statement",
        PropertyMangleBailKind::DirectEval => "a direct `eval(...)` call",
        PropertyMangleBailKind::FunctionConstructor => "the `Function` constructor",
    };
    OxcDiagnostic::warn(format!(
        "Property mangling was skipped for the whole file because it contains {reason}, \
         which can reference property names dynamically."
    ))
    .with_label(bail.span)
}

/// Minify synchronously.
#[napi]
pub fn minify_sync(
    filename: String,
    source_text: String,
    options: Option<MinifyOptions>,
) -> MinifyResult {
    minify_impl(&filename, &source_text, options)
}

pub struct MinifyTask {
    filename: String,
    source_text: String,
    options: Option<MinifyOptions>,
}

#[napi]
impl Task for MinifyTask {
    type JsValue = MinifyResult;
    type Output = MinifyResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        Ok(minify_impl(&self.filename, &self.source_text, self.options.take()))
    }

    fn resolve(&mut self, _: napi::Env, result: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(result)
    }
}

/// Minify asynchronously.
///
/// Note: This function can be slower than `minifySync` due to the overhead of spawning a thread.
#[napi]
pub fn minify(
    filename: String,
    source_text: String,
    options: Option<MinifyOptions>,
) -> AsyncTask<MinifyTask> {
    AsyncTask::new(MinifyTask { filename, source_text, options })
}
