//! Annex B.3.2 var bindings of function declarations in blocks.
//!
//! In sloppy mode, a plain function declaration in a block may also create a `var` binding in the
//! enclosing function or script (its var scope). It does not if replacing the declaration with
//! `var name` would be an early error. This is modeled by moving the function's binding to the var
//! scope, unless that scope already has a binding of the same name.
//!
//! Whether the var binding is prevented depends on every declaration in the var scope, including
//! later ones, so the decision is deferred until the var scope has been bound. Until then, the
//! function is bound in its block. References are resolved after that, so they see the final
//! placement.
//!
//! <https://tc39.es/ecma262/#sec-block-level-function-declarations-web-legacy-compatibility-semantics>

use oxc_syntax::{
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};

use crate::{Scoping, SemanticBuilder};

/// A function declaration in a block that may get a var binding.
pub struct AnnexBFunction {
    var_scope_id: ScopeId,
    block_scope_id: ScopeId,
    symbol_id: SymbolId,
    /// The function is the body of an `if` statement. Annex B.3.3 places it in its own implicit
    /// block, so it does not prevent the var binding of a function in a nested block.
    is_if_statement: bool,
}

impl SemanticBuilder<'_> {
    /// Must be called after binding a sloppy-mode plain function declaration in a block.
    pub(crate) fn declare_annex_b_function(&mut self, symbol_id: SymbolId, is_if_statement: bool) {
        // Only if the declaration created the symbol, rather than redeclaring another declaration.
        if self.scoping.symbol_declaration(symbol_id) != self.node_store.current_node_id {
            return;
        }
        let block_scope_id = self.current_scope_id;
        let var_scope_id = self
            .scoping
            .scope_ancestors(block_scope_id)
            .find(|&scope_id| self.scoping.scope_flags(scope_id).is_var())
            .expect("the root scope is a var scope");
        self.annex_b_functions.push(AnnexBFunction {
            var_scope_id,
            block_scope_id,
            symbol_id,
            is_if_statement,
        });
    }

    /// Whether leaving the current scope finishes binding a var scope with Annex B functions.
    #[inline]
    pub(crate) fn has_annex_b_functions_in_current_scope(&self) -> bool {
        self.annex_b_functions.last().is_some_and(|f| f.var_scope_id == self.current_scope_id)
    }

    /// Move the binding of each Annex B function of the current var scope to the var scope,
    /// unless it is prevented. Must be called when the var scope has been fully bound, before
    /// resolving its references.
    #[cold]
    pub(crate) fn bind_annex_b_functions(&mut self) {
        let var_scope_id = self.current_scope_id;
        // Functions of nested var scopes were removed when leaving those, so the functions of this
        // var scope are at the end, in source order.
        let start = self
            .annex_b_functions
            .iter()
            .rposition(|f| f.var_scope_id != var_scope_id)
            .map_or(0, |i| i + 1);
        let functions = &self.annex_b_functions[start..];
        for function in functions {
            if !is_annex_b_var_binding_prevented(&self.scoping, functions, function) {
                debug_assert_eq!(
                    self.scoping.symbol_scope_id(function.symbol_id),
                    function.block_scope_id
                );
                self.scoping.move_binding_by_symbol_id(
                    function.block_scope_id,
                    var_scope_id,
                    function.symbol_id,
                );
            }
        }
        self.annex_b_functions.truncate(start);
    }
}

/// Whether a binding of the function's name prevents its var binding: any binding in the var
/// scope, or a declaration in a block between the function and its var scope. A simple catch
/// parameter (Annex B.3.4) or a function that is the body of an `if` statement is not such a
/// declaration.
///
/// `functions` are the Annex B functions of the var scope, which are sorted by symbol id because
/// they are in source order.
fn is_annex_b_var_binding_prevented(
    scoping: &Scoping,
    functions: &[AnnexBFunction],
    function: &AnnexBFunction,
) -> bool {
    let name = scoping.symbol_ident(function.symbol_id);
    for scope_id in scoping.scope_ancestors(function.block_scope_id).skip(1) {
        let binding = scoping.get_binding(scope_id, name);
        if scope_id == function.var_scope_id {
            return binding.is_some();
        }
        let Some(symbol_id) = binding else { continue };
        let is_simple_catch_parameter = scoping
            .symbol_flags(symbol_id)
            .contains(SymbolFlags::CatchVariable | SymbolFlags::FunctionScopedVariable);
        let is_if_statement_function = functions
            .binary_search_by_key(&symbol_id, |f| f.symbol_id)
            .is_ok_and(|i| functions[i].is_if_statement);
        if !is_simple_catch_parameter && !is_if_statement_function {
            return true;
        }
    }
    unreachable!("the var scope is an ancestor of the block scope")
}
