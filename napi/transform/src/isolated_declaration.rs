use napi_derive::napi;

use oxc_allocator::Allocator;
use oxc_codegen::CodegenReturn;
use oxc_isolated_declarations::IsolatedDeclarations;
use oxc_span::SourceType;

use crate::context::TransformContext;

#[napi(object)]
pub struct IsolatedDeclarationsResult {
    pub source_text: String,
    // TODO: should we expose source maps?
    // pub source_map: Option<SourceMap>,
    pub errors: Vec<String>,
}

/// TypeScript Isolated Declarations for Standalone DTS Emit
#[allow(clippy::needless_pass_by_value)]
#[napi]
pub fn isolated_declaration(filename: String, source_text: String) -> IsolatedDeclarationsResult {
    let source_type = SourceType::from_path(&filename).unwrap_or_default().with_typescript(true);
    let allocator = Allocator::default();
    let ctx = TransformContext::new(&allocator, &filename, &source_text, source_type, None);
    let transformed_ret = build_declarations(&ctx);

    IsolatedDeclarationsResult {
        source_text: transformed_ret.source_text,
        errors: ctx.take_and_render_reports(),
    }
}

pub(crate) fn build_declarations(ctx: &TransformContext<'_>) -> CodegenReturn {
    let transformed_ret = IsolatedDeclarations::new(ctx.allocator).build(&ctx.program_mut());
    ctx.add_diagnostics(transformed_ret.errors);
    ctx.codegen::<false>().build(&ctx.program())
}
