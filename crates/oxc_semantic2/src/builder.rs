use oxc_span::{Atom, SourceType, Span};

use crate::{
    scope::{Scope, ScopeFlags, ScopeId, ScopeTree},
    symbol::{SymbolFlags, SymbolId, SymbolTable},
    Semantic,
};

pub struct SemanticBuilder {
    scope_tree: ScopeTree,
    symbol_table: SymbolTable,

    current_scope_id: ScopeId,
}

impl SemanticBuilder {
    pub fn new(source_type: SourceType) -> Self {
        let mut scope_tree = ScopeTree::new();
        let scope_flags = ScopeFlags::Top
            .with_strict_mode(source_type.is_module() || source_type.always_strict());
        let root_scope = Scope::new(None, scope_flags);
        let current_scope_id = scope_tree.add_scope(root_scope);
        Self { scope_tree, symbol_table: SymbolTable::new(), current_scope_id }
    }

    pub fn build(self) -> Semantic {
        Semantic { scope_tree: self.scope_tree, symbol_table: self.symbol_table }
    }

    fn current_scope(&self) -> &Scope {
        self.scope_tree.get_scope(self.current_scope_id)
    }

    fn enter_scope(&mut self, flags: ScopeFlags) {
        let mut flags = flags;
        // Inherit strict mode for functions
        // https://tc39.es/ecma262/#sec-strict-mode-code
        let mut strict_mode = self.scope_tree.root_scope().is_strict_mode();
        let parent_scope = self.current_scope();
        if !strict_mode && parent_scope.is_function() && parent_scope.is_strict_mode() {
            strict_mode = true;
        }

        // inherit flags for non-function scopes
        if !flags.contains(ScopeFlags::Function) {
            flags |= parent_scope.flags() & ScopeFlags::Modifiers;
        };

        if strict_mode {
            flags |= ScopeFlags::StrictMode;
        }

        let scope = Scope::new(Some(self.current_scope_id), flags);
        self.current_scope_id = self.scope_tree.add_scope(scope);
    }

    pub fn leave_scope(&mut self) {
        if let Some(parent_id) = self.current_scope().parent_id() {
            self.current_scope_id = parent_id;
        }
    }

    /// Declares a `Symbol` for the node, adds it to symbol table, and binds it to the scope.
    ///
    /// includes: the `SymbolFlags` that node has in addition to its declaration type (eg: export, ambient, etc.)
    /// excludes: the flags which node cannot be declared alongside in a symbol table. Used to report forbidden declarations.
    ///
    /// Reports errors for conflicting identifier names.
    fn declare_symbol(
        &mut self,
        span: Span,
        name: &Atom,
        includes: SymbolFlags,
        _excludes: SymbolFlags,
    ) -> SymbolId {
        let symbol_id = self.symbol_table.add_symbol(name.clone(), span, includes);
        self.scope_tree.get_scope_mut(self.current_scope_id).add_symbol(name.clone(), symbol_id);
        symbol_id
    }

    fn resolve_reference(&mut self, name: &Atom) -> Option<SymbolId> {
        self.scope_tree
            .ancestors(self.current_scope_id)
            .find_map(|scope_id| self.scope_tree.get_scope(scope_id).get_binding(name))
    }
}

impl SemanticBuilder {
    pub fn enter_binding_identifier(
        &mut self,
        span: Span,
        name: &Atom,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> SymbolId {
        self.declare_symbol(span, name, includes, excludes)
    }

    pub fn enter_identifier_reference(&mut self, name: &Atom) -> Option<SymbolId> {
        self.resolve_reference(name)
    }

    pub fn enter_block(&mut self) {
        self.enter_scope(ScopeFlags::empty());
    }

    pub fn leave_block(&mut self) {
        self.leave_scope();
    }

    pub fn enter_function_scope(&mut self) {
        self.enter_scope(ScopeFlags::Function);
    }

    pub fn leave_function_scope(&mut self) {
        self.leave_scope();
    }

    pub fn enter_static_block(&mut self) {
        self.enter_scope(ScopeFlags::ClassStaticBlock);
    }

    pub fn leave_static_block(&mut self) {
        self.leave_scope();
    }
}
