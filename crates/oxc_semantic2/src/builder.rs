use oxc_span::{Atom, SourceType, Span};
use rustc_hash::FxHashMap;

use crate::{
    reference::ReferenceId,
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

    pub fn build(mut self) -> Semantic {
        self.leave_scope();
        Semantic { scope_tree: self.scope_tree, symbol_table: self.symbol_table }
    }

    fn current_scope(&self) -> &Scope {
        self.scope_tree.get_scope(self.current_scope_id)
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        self.scope_tree.get_scope_mut(self.current_scope_id)
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
        self.resolve_references_for_current_scope();
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
        excludes: SymbolFlags,
    ) -> SymbolId {
        let scope_id = if includes.is_function_scoped_declaration()
            && !self.current_scope().is_var_scope()
            && !includes.is_function()
        {
            self.get_var_hosisting_scope_id()
        } else {
            self.current_scope_id
        };

        if let Some(symbol_id) = self.check_redeclaration(scope_id, span, name, excludes) {
            return symbol_id;
        }

        let symbol_id = self.symbol_table.create_symbol(span, name.clone(), includes);
        if includes.is_variable() {
            self.scope_tree.get_scope_mut(scope_id).add_binding(name.clone(), symbol_id);
        }
        symbol_id
    }

    fn get_var_hosisting_scope_id(&self) -> ScopeId {
        let mut top_scope_id = self.current_scope_id;
        for scope_id in self.scope_tree.ancestors(self.current_scope_id).skip(1) {
            let scope = self.scope_tree.get_scope(scope_id);
            if scope.is_var_scope() {
                top_scope_id = scope_id;
                break;
            }
            top_scope_id = scope_id;
        }
        top_scope_id
    }

    fn check_redeclaration(
        &mut self,
        scope_id: ScopeId,
        _span: Span,
        name: &Atom,
        _excludes: SymbolFlags,
    ) -> Option<SymbolId> {
        self.scope_tree.get_scope(scope_id).get_binding(name)
        // TODO: redeclaration error
        // if self.symbol_table.get_flag(symbol_id).intersects(excludes) {
        // self.error(Redeclaration(name.clone(), symbol.span(), span));
        // }
    }

    fn declare_reference(&mut self, span: Span, name: &Atom) -> ReferenceId {
        let reference_id = self.symbol_table.create_reference(span, name.clone());
        self.current_scope_mut().add_unresolved_reference(name.clone(), reference_id);
        reference_id
    }

    fn resolve_references_for_current_scope(&mut self) {
        let all_references = self
            .current_scope_mut()
            .unresolved_references
            .drain()
            .collect::<Vec<(Atom, Vec<ReferenceId>)>>();

        let mut unresolved_references: FxHashMap<Atom, Vec<ReferenceId>> = FxHashMap::default();
        let mut resolved_references: Vec<(SymbolId, Vec<ReferenceId>)> = vec![];

        for (name, reference_ids) in all_references {
            if let Some(symbol_id) = self.current_scope().bindings().get(&name) {
                resolved_references.push((*symbol_id, reference_ids));
            } else {
                unresolved_references.insert(name, reference_ids);
            }
        }

        let scope = if let Some(parent_scope_id) = self.current_scope().parent_id() {
            self.scope_tree.get_scope_mut(parent_scope_id)
        } else {
            self.current_scope_mut()
        };

        for (name, references) in unresolved_references {
            scope.unresolved_references.entry(name).or_default().extend(references);
        }

        for (symbol_id, reference_ids) in resolved_references {
            for reference_id in reference_ids {
                self.symbol_table.references[reference_id].symbol_id = Some(symbol_id);
                self.symbol_table.resolved_references[symbol_id].push(reference_id);
            }
        }
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

    pub fn enter_identifier_reference(&mut self, span: Span, name: &Atom) -> ReferenceId {
        self.declare_reference(span, name)
    }

    pub fn enter_block_statement(&mut self) {
        self.enter_scope(ScopeFlags::empty());
    }

    pub fn leave_block_statement(&mut self) {
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

    pub fn enter_catch_clause(&mut self) {
        self.enter_scope(ScopeFlags::empty());
    }

    pub fn leave_catch_clause(&mut self) {
        self.leave_scope();
    }
}
