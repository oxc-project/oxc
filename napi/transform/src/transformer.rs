use napi::Either;
use napi_derive::napi;
use rustc_hash::FxHashMap;

use oxc::{
    allocator::Allocator,
    codegen::CodegenReturn,
    napi::{source_map::SourceMap, transform::TransformOptions},
    semantic::{ScopeTree, SemanticBuilder, SymbolTable},
    span::SourceType,
    transformer::{
        InjectGlobalVariables, InjectGlobalVariablesConfig, InjectImport, ReplaceGlobalDefines,
        ReplaceGlobalDefinesConfig, Transformer,
    },
};

use crate::{context::TransformContext, isolated_declaration};

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
    /// @see {@link TypeScriptOptions#declaration}
    /// @see [declaration tsconfig option](https://www.typescriptlang.org/tsconfig/#declaration)
    pub declaration: Option<String>,

    /// Declaration source map. Only generated if both
    /// {@link TypeScriptOptions#declaration declaration} and
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
        source_type
    };

    let allocator = Allocator::default();
    let ctx =
        TransformContext::new(&allocator, &filename, &source_text, source_type, options.as_ref());

    let declarations_result = source_type
        .is_typescript()
        .then(|| ctx.declarations())
        .flatten()
        .map(|options| isolated_declaration::build_declarations(&ctx, *options));

    let transpile_result = transpile(&ctx, options);

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

fn transpile(ctx: &TransformContext<'_>, options: Option<TransformOptions>) -> CodegenReturn {
    let semantic_ret = SemanticBuilder::new(ctx.source_text())
        // Estimate transformer will triple scopes, symbols, references
        .with_excess_capacity(2.0)
        .with_check_syntax_error(true)
        .build(&ctx.program());
    ctx.add_diagnostics(semantic_ret.errors);

    let mut options = options;
    let define = options.as_mut().and_then(|options| options.define.take());
    let inject = options.as_mut().and_then(|options| options.inject.take());

    let options = options.map(oxc::transformer::TransformOptions::from).unwrap_or_default();

    let (mut symbols, mut scopes) = semantic_ret.semantic.into_symbol_table_and_scope_tree();

    let ret = Transformer::new(
        ctx.allocator,
        ctx.file_path(),
        ctx.source_text(),
        ctx.trivias.clone(),
        options,
    )
    .build_with_symbols_and_scopes(symbols, scopes, &mut ctx.program_mut());
    ctx.add_diagnostics(ret.errors);
    symbols = ret.symbols;
    scopes = ret.scopes;

    if let Some(define) = define {
        (symbols, scopes) = define_plugin(ctx, define, symbols, scopes);
    }

    if let Some(inject) = inject {
        _ = inject_plugin(ctx, inject, symbols, scopes);
    }

    ctx.codegen().build(&ctx.program())
}

fn define_plugin(
    ctx: &TransformContext<'_>,
    define: FxHashMap<String, String>,
    symbols: SymbolTable,
    scopes: ScopeTree,
) -> (SymbolTable, ScopeTree) {
    let define = define.into_iter().collect::<Vec<_>>();
    match ReplaceGlobalDefinesConfig::new(&define) {
        Ok(config) => {
            let ret = ReplaceGlobalDefines::new(ctx.allocator, config).build(
                symbols,
                scopes,
                &mut ctx.program_mut(),
            );
            (ret.symbols, ret.scopes)
        }
        Err(errors) => {
            ctx.add_diagnostics(errors);
            (symbols, scopes)
        }
    }
}

fn inject_plugin(
    ctx: &TransformContext<'_>,
    inject: FxHashMap<String, Either<String, Vec<String>>>,
    symbols: SymbolTable,
    scopes: ScopeTree,
) -> (SymbolTable, ScopeTree) {
    let Ok(injects) = inject
        .into_iter()
        .map(|(local, value)| match value {
            Either::A(source) => Ok(InjectImport::default_specifier(&source, &local)),
            Either::B(v) => {
                if v.len() != 2 {
                    return Err(());
                }
                let source = v[0].to_string();
                Ok(if v[1] == "*" {
                    InjectImport::namespace_specifier(&source, &local)
                } else {
                    InjectImport::named_specifier(&source, Some(&v[1]), &local)
                })
            }
        })
        .collect::<Result<Vec<_>, ()>>()
    else {
        return (symbols, scopes);
    };

    let config = InjectGlobalVariablesConfig::new(injects);
    let ret = InjectGlobalVariables::new(ctx.allocator, config).build(
        symbols,
        scopes,
        &mut ctx.program_mut(),
    );

    (ret.symbols, ret.scopes)
}
