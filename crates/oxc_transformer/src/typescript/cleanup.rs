//! Record semantic data in erased syntax before its AST nodes are discarded.

use oxc_allocator::{BitSet, GetAllocator};
use oxc_ast::ast::{BindingIdentifier, IdentifierReference, TSInterfaceHeritage, TSTypeReference};
use oxc_ast_visit::Visit;
use oxc_span::Span;
use oxc_syntax::{reference::ReferenceId, symbol::SymbolId};
use rustc_hash::FxHashSet;

use crate::context::TraverseCtx;

#[derive(Default)]
pub struct TypeScriptCleanup {
    declarations: FxHashSet<(SymbolId, Span)>,
    references: FxHashSet<ReferenceId>,
}

impl TypeScriptCleanup {
    pub fn finish(self, ctx: &mut TraverseCtx<'_>) {
        let allocator = ctx.allocator();
        let scoping = ctx.scoping_mut();
        let mut erased_symbols = BitSet::new_in(
            if self.declarations.is_empty() { 0 } else { scoping.symbols_len() },
            allocator,
        );
        for (id, _) in &self.declarations {
            erased_symbols.set_bit(id.index());
        }
        let is_erased = |id: SymbolId, span| {
            erased_symbols.contains(id.index()) && self.declarations.contains(&(id, span))
        };
        if self.references.is_empty() {
            // Most files erase only type references, which scoping already filters.
            // Specialize this path to avoid an extra lookup for every value reference.
            scoping.delete_typescript_bindings_with(is_erased, |_| false);
        } else {
            let mut erased_references = BitSet::new_in(scoping.references_len(), allocator);
            for id in self.references {
                erased_references.set_bit(id.index());
            }
            scoping.delete_typescript_bindings_with(is_erased, |id| {
                erased_references.contains(id.index())
            });
        }
    }
}

/// All removals are deferred until import/export retention has finished.
pub(super) struct Erase<'c, 'a>(pub &'c mut TraverseCtx<'a>);

impl<'a> Visit<'a> for Erase<'_, 'a> {
    // These names contain only type references, which the final filter already
    // removes. Still walk type arguments: computed keys and signature parameters
    // can contain value references and bindings that need explicit erasure.
    // Do not skip TSTypeName generally: import-equals uses it for value references.
    fn visit_ts_type_reference(&mut self, ty: &TSTypeReference<'a>) {
        if let Some(arguments) = &ty.type_arguments {
            self.visit_ts_type_parameter_instantiation(arguments);
        }
    }

    fn visit_ts_interface_heritage(&mut self, heritage: &TSInterfaceHeritage<'a>) {
        if let Some(arguments) = &heritage.type_arguments {
            self.visit_ts_type_parameter_instantiation(arguments);
        }
    }

    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        if let Some(id) = ident.symbol_id.get() {
            self.0.state.typescript_cleanup.declarations.insert((id, ident.span));
        }
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if let Some(id) = ident.reference_id.get() {
            self.0.state.typescript_cleanup.references.insert(id);
        }
    }
}
