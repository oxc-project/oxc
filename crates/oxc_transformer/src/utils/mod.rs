pub mod ast_builder;

use oxc_ast::ast::Function;
use oxc_syntax::symbol::SymbolFlags;

use crate::context::TraverseCtx;

/// Keep the `SymbolFlags` that record a function's syntactic kind in sync with the function's
/// current shape, after a transform has rewritten it.
///
/// The semantic builder sets [`SymbolFlags::AsyncOrGeneratorFunction`] and
/// [`SymbolFlags::FunctionExpression`] from the original source. When a transform changes a
/// function's shape — downleveling `async`/generator to a plain function, or turning a declaration
/// into a named function expression (or vice versa) — it must call this so the semantic data stays
/// consistent with the rewritten AST.
///
/// No-op for anonymous functions (no symbol to update).
pub fn sync_function_symbol_flags<'a>(func: &Function<'a>, ctx: &mut TraverseCtx<'a>) {
    let Some(symbol_id) = func.id.as_ref().and_then(|id| id.symbol_id.get()) else {
        return;
    };
    let flags = ctx.scoping_mut().symbol_flags_mut(symbol_id);
    flags.set(SymbolFlags::AsyncOrGeneratorFunction, func.r#async || func.generator);
    flags.set(SymbolFlags::FunctionExpression, func.is_expression());
}
