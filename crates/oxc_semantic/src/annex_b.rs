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

use rustc_hash::FxHashSet;

use oxc_str::Ident;
use oxc_syntax::{
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};

use crate::{Scoping, SemanticBuilder};

/// A function declaration in a block that may get a var binding.
#[expect(clippy::struct_field_names)] // The ids of a scope, a scope and a symbol
pub struct AnnexBFunction {
    var_scope_id: ScopeId,
    block_scope_id: ScopeId,
    symbol_id: SymbolId,
}

impl SemanticBuilder<'_> {
    /// Must be called after binding a sloppy-mode plain function declaration in a block.
    ///
    /// `redeclares_simple_catch_parameter`: see [`Self::is_first_function_in_simple_catch_parameter`].
    pub(crate) fn declare_annex_b_function(
        &mut self,
        symbol_id: SymbolId,
        is_if_statement: bool,
        redeclares_simple_catch_parameter: bool,
    ) {
        // Only if the declaration created the symbol, rather than redeclaring another declaration.
        if !redeclares_simple_catch_parameter
            && self.scoping.symbol_declaration(symbol_id) != self.node_store.current_node_id
        {
            return;
        }
        let block_scope_id = self.current_scope_id;
        let var_scope_id = self
            .scoping
            .scope_ancestors(block_scope_id)
            .find(|&scope_id| self.scoping.scope_flags(scope_id).is_var())
            .expect("the root scope is a var scope");
        self.annex_b_functions.push(AnnexBFunction { var_scope_id, block_scope_id, symbol_id });
        if is_if_statement {
            self.annex_b_if_statement_functions.insert(symbol_id);
        }
    }

    /// Must be called before binding an `if` statement function. Whether it will redeclare a
    /// simple catch parameter of its block that no other function has redeclared:
    /// `catch (g) { if (x) function g() {} }`.
    ///
    /// Annex B.3.3 places the function in its own implicit block, which is not modeled, so it
    /// shares the catch parameter's symbol. A simple catch parameter does not prevent the var
    /// binding (Annex B.3.4), so that symbol is an Annex B function.
    pub(crate) fn is_first_function_in_simple_catch_parameter(&self, name: Ident<'_>) -> bool {
        self.scoping.get_binding(self.current_scope_id, name).is_some_and(|symbol_id| {
            let flags = self.scoping.symbol_flags(symbol_id);
            flags.contains(SymbolFlags::CatchVariable | SymbolFlags::FunctionScopedVariable)
                && !flags.contains(SymbolFlags::Function)
        })
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
            if !is_annex_b_var_binding_prevented(
                &self.scoping,
                &self.annex_b_if_statement_functions,
                function,
            ) {
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
/// declaration. Annex B.3.3 places such a function in its own implicit block, which is not
/// modeled, so it is bound in the block that contains the `if` statement.
fn is_annex_b_var_binding_prevented(
    scoping: &Scoping,
    if_statement_functions: &FxHashSet<SymbolId>,
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
        if !is_simple_catch_parameter && !if_statement_functions.contains(&symbol_id) {
            return true;
        }
    }
    unreachable!("the var scope is an ancestor of the block scope")
}
