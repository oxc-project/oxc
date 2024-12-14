use std::path::Path;

use napi_derive::napi;

use oxc::{
    allocator::Allocator,
    codegen::{CodeGenerator, CodegenOptions},
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

/// TypeScript Isolated Declarations for Standalone DTS Emit
#[allow(clippy::needless_pass_by_value)]
#[napi]
pub fn isolated_declaration(
    filename: String,
    source_text: String,
    options: Option<IsolatedDeclarationsOptions>,
) -> IsolatedDeclarationsResult {
    let source_path = Path::new(&filename);
    let source_type = SourceType::from_path(source_path).unwrap_or_default().with_typescript(true);
    let allocator = Allocator::default();
    let options = options.unwrap_or_default();

    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    let transformed_ret = IsolatedDeclarations::new(
        &allocator,
        oxc::isolated_declarations::IsolatedDeclarationsOptions {
            strip_internal: options.strip_internal.unwrap_or(false),
        },
    )
    .build(&ret.program);

    let source_map_path = match options.sourcemap {
        Some(true) => Some(source_path.to_path_buf()),
        _ => None,
    };
    let codegen_ret = CodeGenerator::new()
        .with_options(CodegenOptions { source_map_path, ..CodegenOptions::default() })
        .build(&transformed_ret.program);

    let errors = ret.errors.into_iter().chain(transformed_ret.errors).map(OxcError::from).collect();

    IsolatedDeclarationsResult {
        code: codegen_ret.code,
        map: codegen_ret.map.map(SourceMap::from),
        errors,
    }
}
