//! Record semantic data in erased syntax before its AST nodes are discarded.

use oxc_allocator::{BitSet, GetAllocator};
use oxc_ast::ast::{
    BindingIdentifier, IdentifierReference, TSInterfaceDeclaration, TSInterfaceHeritage,
    TSTypeAliasDeclaration, TSTypeParameter, TSTypeReference,
};
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
    pub fn finish(mut self, ctx: &mut TraverseCtx<'_>) {
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
        // Type-only references are already removed by scoping. Explicit value
        // erasures are sparse, so remove them from the affected reference lists
        // instead of adding a membership test to every reference in the program.
        self.references.retain(|id| {
            let flags = scoping.get_reference(*id).flags();
            (flags.is_value() || !flags.is_type()) && !flags.is_value_as_type()
        });
        scoping.remove_references(&self.references);
        scoping.delete_typescript_bindings_with(allocator, is_erased, |_| false);
    }
}

/// All removals are deferred until import/export retention has finished.
pub(super) struct Erase<'c, 'a>(pub &'c mut TraverseCtx<'a>);

impl<'a> Visit<'a> for Erase<'_, 'a> {
    // Scoping removes these declarations using their individual declaration flags,
    // including when they are merged with runtime declarations. Only their contents
    // need visiting here; recording the binding again adds redundant cleanup work.
    fn visit_ts_type_alias_declaration(&mut self, decl: &TSTypeAliasDeclaration<'a>) {
        if let Some(parameters) = &decl.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        self.visit_ts_type(&decl.type_annotation);
    }

    fn visit_ts_interface_declaration(&mut self, decl: &TSInterfaceDeclaration<'a>) {
        if let Some(parameters) = &decl.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        self.visit_ts_interface_heritages(&decl.extends);
        self.visit_ts_interface_body(&decl.body);
    }

    fn visit_ts_type_parameter(&mut self, parameter: &TSTypeParameter<'a>) {
        if let Some(constraint) = &parameter.constraint {
            self.visit_ts_type(constraint);
        }
        if let Some(default) = &parameter.default {
            self.visit_ts_type(default);
        }
    }

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
