//! Visitor to count nodes, scopes, symbols and references in AST.
//! These counts can be used to pre-allocate sufficient capacity in `AstNodes`,
//! `ScopeTree`, and `SymbolTable` to store info for all these items.

use std::cell::Cell;

use oxc_ast::{
    ast::{BindingIdentifier, IdentifierReference, TSEnumMemberName, TSModuleDeclarationName},
    visit::walk::{walk_ts_enum_member_name, walk_ts_module_declaration_name},
    AstKind, Visit,
};
use oxc_syntax::scope::{ScopeFlags, ScopeId};

#[derive(Default, Debug)]
pub(crate) struct Counts {
    pub nodes: usize,
    pub scopes: usize,
    pub symbols: usize,
    pub references: usize,
}

impl<'a> Visit<'a> for Counts {
    #[inline]
    fn enter_node(&mut self, _: AstKind<'a>) {
        self.nodes += 1;
    }

    #[inline]
    fn enter_scope(&mut self, _: ScopeFlags, _: &Cell<Option<ScopeId>>) {
        self.scopes += 1;
    }

    #[inline]
    fn visit_binding_identifier(&mut self, _: &BindingIdentifier<'a>) {
        self.nodes += 1;
        self.symbols += 1;
    }

    #[inline]
    fn visit_identifier_reference(&mut self, _: &IdentifierReference<'a>) {
        self.nodes += 1;
        self.references += 1;
    }

    #[inline]
    fn visit_ts_enum_member_name(&mut self, it: &TSEnumMemberName<'a>) {
        if !it.is_expression() {
            self.symbols += 1;
        }
        walk_ts_enum_member_name(self, it);
    }

    #[inline]
    fn visit_ts_module_declaration_name(&mut self, it: &TSModuleDeclarationName<'a>) {
        self.symbols += 1;
        walk_ts_module_declaration_name(self, it);
    }
}
