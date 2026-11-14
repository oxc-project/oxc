use oxc_ast::ast::VariableDeclarator;
use oxc_ecmascript::BoundNames;
use oxc_semantic::SymbolId;

/// Returns whether an initializer on `decl`, or the `for-in`/`for-of` loop it heads,
/// assigns to `symbol_id`.
///
/// A declarator's identifiers carry the bindings they assign. That is usually the declared
/// binding, but in `catch (e) { var e = 1 }` the identifier carries the catch parameter:
/// the `var` declares a separate enclosing binding (see `Scoping::catch_var_binding`),
/// while `= 1` assigns to `e` in the catch.
pub fn declarator_initializes(decl: &VariableDeclarator, symbol_id: SymbolId) -> bool {
    let mut initializes = false;
    decl.id.bound_names(&mut |ident| initializes |= ident.symbol_id() == symbol_id);
    initializes
}
