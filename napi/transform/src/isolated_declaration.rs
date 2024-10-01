use napi_derive::napi;
use oxc_allocator::Allocator;
use oxc_codegen::{CodegenReturn, CommentOptions};
use oxc_isolated_declarations::IsolatedDeclarations;
use oxc_span::SourceType;

use crate::{context::TransformContext, SourceMap, TransformOptions};

#[napi(object)]
pub struct IsolatedDeclarationsResult {
    pub code: String,
    pub map: Option<SourceMap>,
    pub errors: Vec<String>,
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

/// TypeScript Isolated Declarations for Standalone DTS Emit
#[allow(clippy::needless_pass_by_value)]
#[napi]
pub fn isolated_declaration(
    filename: String,
    source_text: String,
    options: Option<IsolatedDeclarationsOptions>,
) -> IsolatedDeclarationsResult {
    let source_type = SourceType::from_path(&filename).unwrap_or_default().with_typescript(true);
    let allocator = Allocator::default();
    let options = options.unwrap_or_default();
    let ctx = TransformContext::new(
        &allocator,
        &filename,
        &source_text,
        source_type,
        Some(&TransformOptions { sourcemap: options.sourcemap, ..Default::default() }),
    );
    let transformed_ret = build_declarations(&ctx, options);

    IsolatedDeclarationsResult {
        code: transformed_ret.source_text,
        map: options.sourcemap.and_then(|_| transformed_ret.source_map.map(Into::into)),
        errors: ctx.take_and_render_reports(),
    }
}

pub(crate) fn build_declarations(
    ctx: &TransformContext<'_>,
    options: IsolatedDeclarationsOptions,
) -> CodegenReturn {
    let transformed_ret = IsolatedDeclarations::new(
        ctx.allocator,
        ctx.source_text(),
        &ctx.trivias,
        oxc_isolated_declarations::IsolatedDeclarationsOptions {
            strip_internal: options.strip_internal.unwrap_or(false),
        },
    )
    .build(&ctx.program());
    ctx.add_diagnostics(transformed_ret.errors);
    ctx.codegen()
        .enable_comment(
            ctx.source_text(),
            ctx.trivias.clone(),
            CommentOptions { preserve_annotate_comments: false },
        )
        .build(&transformed_ret.program)
}
