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
    declarations: Vec<(SymbolId, Span)>,
    references: Vec<ReferenceId>,
}

impl TypeScriptCleanup {
    pub fn finish(self, ctx: &mut TraverseCtx<'_>) {
        let allocator = ctx.allocator();
        let scoping = ctx.scoping_mut();
        let mut erased_symbols = BitSet::new_in(
            if self.declarations.is_empty() { 0 } else { scoping.symbols_len() },
            allocator,
        );
        let mut merged_symbols = BitSet::new_in(erased_symbols.capacity(), allocator);
        let mut merged_declarations = FxHashSet::default();
        for (id, span) in self.declarations {
            // Classify after import/export retention, which can promote a surviving
            // declaration or collapse a merge. Only a matching sole declaration can
            // be removed by symbol ID; other records still need their binding span.
            if scoping.symbol_redeclarations(id).is_empty() && scoping.symbol_span(id) == span {
                erased_symbols.set_bit(id.index());
            } else {
                merged_symbols.set_bit(id.index());
                merged_declarations.insert((id, span));
            }
        }
        let mut erased_references = BitSet::new_in(
            if self.references.is_empty() { 0 } else { scoping.references_len() },
            allocator,
        );
        for id in self.references {
            erased_references.set_bit(id.index());
        }
        scoping.delete_typescript_bindings_with(
            |id, span| {
                erased_symbols.contains(id.index())
                    || (merged_symbols.contains(id.index())
                        && merged_declarations.contains(&(id, span)))
            },
            |id| erased_references.contains(id.index()),
        );
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
            self.0.state.typescript_cleanup.declarations.push((id, ident.span));
        }
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if let Some(id) = ident.reference_id.get() {
            self.0.state.typescript_cleanup.references.push(id);
        }
    }
}
