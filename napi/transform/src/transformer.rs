use napi_derive::napi;
use oxc_allocator::Allocator;
use oxc_codegen::CodegenReturn;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::Transformer;

use crate::{context::TransformContext, isolated_declaration, SourceMap, TransformOptions};

// NOTE: Use JSDoc syntax for all doc comments, not rustdoc.
// NOTE: Types must be aligned with [@types/babel__core](https://github.com/DefinitelyTyped/DefinitelyTyped/blob/master/types/babel__core/index.d.ts).

#[napi(object)]
pub struct TransformResult {
    /// The transformed code.
    ///
    /// If parsing failed, this will be an empty string.
    pub code: String,

    /// The source map for the transformed code.
    ///
    /// This will be set if {@link TransformOptions#sourcemap} is `true`.
    pub map: Option<SourceMap>,

    /// The `.d.ts` declaration file for the transformed code. Declarations are
    /// only generated if `declaration` is set to `true` and a TypeScript file
    /// is provided.
    ///
    /// If parsing failed and `declaration` is set, this will be an empty string.
    ///
    /// @see {@link TypeScriptBindingOptions#declaration}
    /// @see [declaration tsconfig option](https://www.typescriptlang.org/tsconfig/#declaration)
    pub declaration: Option<String>,

    /// Declaration source map. Only generated if both
    /// {@link TypeScriptBindingOptions#declaration declaration} and
    /// {@link TransformOptions#sourcemap sourcemap} are set to `true`.
    pub declaration_map: Option<SourceMap>,

    /// Parse and transformation errors.
    ///
    /// Oxc's parser recovers from common syntax errors, meaning that
    /// transformed code may still be available even if there are errors in this
    /// list.
    pub errors: Vec<String>,
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
    let source_type = {
        let mut source_type = SourceType::from_path(&filename).unwrap_or_default();
        // Force `script` or `module`
        match options.as_ref().and_then(|options| options.source_type.as_deref()) {
            Some("script") => source_type = source_type.with_script(true),
            Some("module") => source_type = source_type.with_module(true),
            _ => {}
        }
        // Force `jsx`
        if let Some(jsx) = options.as_ref().and_then(|options| options.jsx.as_ref()) {
            source_type = source_type.with_jsx(*jsx);
        }
        source_type
    };

    let allocator = Allocator::default();
    let ctx = TransformContext::new(&allocator, &filename, &source_text, source_type, options);

    let should_build_types = ctx.declarations() && source_type.is_typescript();
    let declarations_result =
        should_build_types.then(|| isolated_declaration::build_declarations(&ctx));

    let transpile_result = transpile(&ctx);

    let (declaration, declaration_map) = declarations_result
        .map_or((None, None), |d| (Some(d.source_text), d.source_map.map(Into::into)));

    TransformResult {
        code: transpile_result.source_text,
        map: transpile_result.source_map.map(Into::into),
        declaration,
        declaration_map,
        errors: ctx.take_and_render_reports(),
    }
}

fn transpile(ctx: &TransformContext<'_>) -> CodegenReturn {
    let (symbols, scopes) = SemanticBuilder::new(ctx.source_text(), ctx.source_type())
        .build(&ctx.program())
        .semantic
        .into_symbol_table_and_scope_tree();
    let ret = Transformer::new(
        ctx.allocator,
        ctx.file_path(),
        ctx.source_type(),
        ctx.source_text(),
        ctx.trivias.clone(),
        ctx.oxc_options(),
    )
    .build_with_symbols_and_scopes(symbols, scopes, &mut ctx.program_mut());

    ctx.add_diagnostics(ret.errors);
    ctx.codegen().build(&ctx.program())
}
