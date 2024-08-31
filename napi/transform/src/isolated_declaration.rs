use napi_derive::napi;

use oxc_allocator::Allocator;
use oxc_codegen::CodegenReturn;
use oxc_isolated_declarations::IsolatedDeclarations;
use oxc_span::SourceType;

use crate::{context::TransformContext, SourceMap, TransformOptions};

#[napi(object)]
pub struct IsolatedDeclarationsResult {
    pub errors: Vec<String>,
    pub source_text: String,
    pub source_map: Option<SourceMap>,
}

#[napi(object)]
pub struct IsolatedDeclarationsOptions {
    pub sourcemap: bool,
}

/// TypeScript Isolated Declarations for Standalone DTS Emit
#[allow(clippy::needless_pass_by_value)]
#[napi]
pub fn isolated_declaration(
    filename: String,
    source_text: String,
    options: IsolatedDeclarationsOptions,
) -> IsolatedDeclarationsResult {
    let source_type = SourceType::from_path(&filename).unwrap_or_default().with_typescript(true);
    let allocator = Allocator::default();
    let ctx = TransformContext::new(
        &allocator,
        &filename,
        &source_text,
        source_type,
        Some(TransformOptions { sourcemap: Some(options.sourcemap), ..Default::default() }),
    );
    let transformed_ret = build_declarations(&ctx);

    IsolatedDeclarationsResult {
        errors: ctx.take_and_render_reports(),
        source_text: transformed_ret.source_text,
        source_map: options.sourcemap.then(|| transformed_ret.source_map.map(Into::into)).flatten(),
    }
}

pub(crate) fn build_declarations(ctx: &TransformContext<'_>) -> CodegenReturn {
    let transformed_ret = IsolatedDeclarations::new(ctx.allocator).build(&ctx.program());
    ctx.add_diagnostics(transformed_ret.errors);
    ctx.codegen().build(&transformed_ret.program)
}
