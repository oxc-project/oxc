use std::path::Path;

use napi::Either;
use napi_derive::napi;

use oxc::{
    codegen::CodegenReturn,
    diagnostics::OxcDiagnostic,
    napi::{
        source_map::SourceMap,
        transform::{TransformOptions, TransformResult},
    },
    span::SourceType,
    transformer::{InjectGlobalVariablesConfig, InjectImport, ReplaceGlobalDefinesConfig},
    CompilerInterface,
};

use crate::errors::wrap_diagnostics;

#[derive(Default)]
struct Compiler {
    transform_options: oxc::transformer::TransformOptions,
    sourcemap: bool,

    printed: String,
    printed_sourcemap: Option<SourceMap>,

    declaration: Option<String>,
    declaration_map: Option<SourceMap>,

    define: Option<ReplaceGlobalDefinesConfig>,
    inject: Option<InjectGlobalVariablesConfig>,

    errors: Vec<OxcDiagnostic>,
}

impl Compiler {
    fn new(options: Option<TransformOptions>) -> Result<Self, Vec<OxcDiagnostic>> {
        let mut options = options;
        let sourcemap = options.as_ref().and_then(|o| o.sourcemap).unwrap_or_default();

        let define = options
            .as_mut()
            .and_then(|options| options.define.take())
            .map(|map| {
                let define = map.into_iter().collect::<Vec<_>>();
                ReplaceGlobalDefinesConfig::new(&define)
            })
            .transpose()?;

        let inject = options
            .as_mut()
            .and_then(|options| options.inject.take())
            .map(|map| {
                map.into_iter()
                    .map(|(local, value)| match value {
                        Either::A(source) => Ok(InjectImport::default_specifier(&source, &local)),
                        Either::B(v) => {
                            if v.len() != 2 {
                                return Err(vec![OxcDiagnostic::error(
                                    "Inject plugin did not receive a tuple [string, string].",
                                )]);
                            }
                            let source = v[0].to_string();
                            Ok(if v[1] == "*" {
                                InjectImport::namespace_specifier(&source, &local)
                            } else {
                                InjectImport::named_specifier(&source, Some(&v[1]), &local)
                            })
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .map(InjectGlobalVariablesConfig::new);

        let transform_options =
            options.map(oxc::transformer::TransformOptions::from).unwrap_or_default();
        Ok(Self {
            transform_options,
            sourcemap,
            printed: String::default(),
            printed_sourcemap: None,
            declaration: None,
            declaration_map: None,
            define,
            inject,
            errors: vec![],
        })
    }
}

impl CompilerInterface for Compiler {
    fn handle_errors(&mut self, errors: Vec<OxcDiagnostic>) {
        self.errors.extend(errors);
    }

    fn enable_sourcemap(&self) -> bool {
        self.sourcemap
    }

    fn transform_options(&self) -> Option<oxc::transformer::TransformOptions> {
        Some(self.transform_options.clone())
    }

    fn define_options(&self) -> Option<ReplaceGlobalDefinesConfig> {
        self.define.clone()
    }

    fn inject_options(&self) -> Option<InjectGlobalVariablesConfig> {
        self.inject.clone()
    }

    fn after_codegen(&mut self, ret: CodegenReturn) {
        self.printed = ret.code;
        self.printed_sourcemap = ret.map.map(SourceMap::from);
    }

    fn after_isolated_declarations(&mut self, ret: CodegenReturn) {
        self.declaration.replace(ret.code);
        self.declaration_map = ret.map.map(SourceMap::from);
    }
}

/// Transpile a JavaScript or TypeScript into a target ECMAScript version.
///
/// @param filename The name of the file being transformed. If this is a
/// relative path, consider setting the {@link TransformOptions#cwd} option..
/// @param sourceText the source code itself
/// @param options The options for the transformation. See {@link
/// TransformOptions} for more information.
///
/// @returns an object containing the transformed code, source maps, and any
/// errors that occurred during parsing or transformation.
#[allow(clippy::needless_pass_by_value)]
#[napi]
pub fn transform(
    filename: String,
    source_text: String,
    options: Option<TransformOptions>,
) -> TransformResult {
    let source_path = Path::new(&filename);

    let source_type = match options.as_ref().and_then(|options| options.lang.as_deref()) {
        Some("js") => SourceType::mjs(),
        Some("jsx") => SourceType::jsx(),
        Some("ts") => SourceType::ts(),
        Some("tsx") => SourceType::tsx(),
        Some(lang) => {
            return TransformResult {
                errors: vec![format!("Incorrect lang '{lang}'")],
                ..Default::default()
            }
        }
        None => {
            let mut source_type = SourceType::from_path(source_path).unwrap_or_default();
            // Force `script` or `module`
            match options.as_ref().and_then(|options| options.source_type.as_deref()) {
                Some("script") => source_type = source_type.with_script(true),
                Some("module") => source_type = source_type.with_module(true),
                _ => {}
            }
            source_type
        }
    };

    let mut compiler = match Compiler::new(options) {
        Ok(compiler) => compiler,
        Err(errors) => {
            return TransformResult {
                errors: wrap_diagnostics(source_path, source_type, &source_text, errors),
                ..Default::default()
            }
        }
    };
    compiler.compile(&source_text, source_type, source_path);

    TransformResult {
        code: compiler.printed,
        map: compiler.printed_sourcemap,
        declaration: compiler.declaration,
        declaration_map: compiler.declaration_map,
        errors: wrap_diagnostics(source_path, source_type, &source_text, compiler.errors),
    }
}
