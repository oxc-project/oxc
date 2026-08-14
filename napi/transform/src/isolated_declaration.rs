use std::path::Path;

use napi::{Task, bindgen_prelude::AsyncTask};
use napi_derive::napi;

use oxc::{
    allocator::Allocator,
    codegen::{Codegen, CodegenOptions, CommentOptions},
    isolated_declarations::IsolatedDeclarations,
    parser::Parser,
    span::SourceType,
};
use oxc_napi::OxcError;
use oxc_sourcemap::napi::SourceMap;

#[napi(object)]
pub struct IsolatedDeclarationsResult {
    pub code: String,
    pub map: Option<SourceMap>,
    pub errors: Vec<OxcError>,
}

#[napi(object)]
#[derive(Debug, Default, Clone, Copy)]
pub struct IsolatedDeclarationsOptions {
    /// Do not emit declarations for code that has an @internal annotation in its JSDoc comment.
    /// This is an internal compiler option; use at your own risk, because the compiler does not check that the result is valid.
    ///
    /// Default: `false`
    ///
    /// See <https://www.typescriptlang.org/tsconfig/#stripInternal>
    pub strip_internal: Option<bool>,

    pub sourcemap: Option<bool>,
}

impl From<IsolatedDeclarationsOptions> for oxc::isolated_declarations::IsolatedDeclarationsOptions {
    fn from(options: IsolatedDeclarationsOptions) -> Self {
        Self { strip_internal: options.strip_internal.unwrap_or_default() }
    }
}

fn isolated_declaration_impl(
    filename: &str,
    source_text: &str,
    options: Option<IsolatedDeclarationsOptions>,
) -> IsolatedDeclarationsResult {
    let source_path = Path::new(filename);
    let allocator = Allocator::default();
    let options = options.unwrap_or_default();
    let id_options = oxc::isolated_declarations::IsolatedDeclarationsOptions {
        strip_internal: options.strip_internal.unwrap_or(false),
    };

    let is_json = source_path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("json"));

    // JSON files are treated as modules whose default export is the JSON value,
    // matching TypeScript's isolated declarations behavior.
    let (transformed_ret, parse_diagnostics) = if is_json {
        match Parser::new(&allocator, source_text, SourceType::mjs()).parse_expression() {
            Ok(expression) => (
                IsolatedDeclarations::new(&allocator, id_options)
                    .build_json(&expression, source_text),
                oxc::diagnostics::Diagnostics::default(),
            ),
            Err(diagnostics) => {
                let errors = OxcError::from_diagnostics(filename, source_text, diagnostics);
                return IsolatedDeclarationsResult { code: String::new(), map: None, errors };
            }
        }
    } else {
        let source_type =
            SourceType::from_path(source_path).unwrap_or_default().with_typescript(true);
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        (IsolatedDeclarations::new(&allocator, id_options).build(&ret.program), ret.diagnostics)
    };

    let source_map_path = match options.sourcemap {
        Some(true) => Some(source_path.to_path_buf()),
        _ => None,
    };
    let codegen_ret = Codegen::new()
        .with_options(CodegenOptions {
            comments: CommentOptions { jsdoc: true, ..CommentOptions::disabled() },
            source_map_path,
            ..CodegenOptions::default()
        })
        .build(&transformed_ret.program);

    let diagnostics =
        parse_diagnostics.into_iter().chain(transformed_ret.diagnostics).collect::<Vec<_>>();
    let errors = OxcError::from_diagnostics(filename, source_text, diagnostics);

    IsolatedDeclarationsResult {
        code: codegen_ret.code,
        map: codegen_ret.map.map(SourceMap::from),
        errors,
    }
}

/// TypeScript Isolated Declarations for Standalone DTS Emit
#[allow(clippy::needless_pass_by_value, clippy::allow_attributes)]
#[napi]
pub fn isolated_declaration_sync(
    filename: String,
    source_text: String,
    options: Option<IsolatedDeclarationsOptions>,
) -> IsolatedDeclarationsResult {
    isolated_declaration_impl(&filename, &source_text, options)
}

pub struct IsolatedDeclarationTask {
    filename: String,
    source_text: String,
    options: Option<IsolatedDeclarationsOptions>,
}

#[napi]
impl Task for IsolatedDeclarationTask {
    type JsValue = IsolatedDeclarationsResult;
    type Output = IsolatedDeclarationsResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        Ok(isolated_declaration_impl(&self.filename, &self.source_text, self.options.take()))
    }

    fn resolve(&mut self, _: napi::Env, result: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(result)
    }
}

/// TypeScript Isolated Declarations for Standalone DTS Emit (async)
///
/// Note: This function can be slower than `isolatedDeclarationSync` due to the overhead of spawning a thread.
#[napi]
pub fn isolated_declaration(
    filename: String,
    source_text: String,
    options: Option<IsolatedDeclarationsOptions>,
) -> AsyncTask<IsolatedDeclarationTask> {
    AsyncTask::new(IsolatedDeclarationTask { filename, source_text, options })
}
