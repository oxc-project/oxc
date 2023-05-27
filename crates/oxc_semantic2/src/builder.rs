use oxc_span::{Atom, SourceType, Span};
use rustc_hash::FxHashMap;

use crate::{
    reference::ReferenceId,
    scope::{ScopeFlags, ScopeId, ScopeTree},
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
        let mut scope_tree = ScopeTree::default();
        let scope_flags = ScopeFlags::Top
            .with_strict_mode(source_type.is_module() || source_type.always_strict());
        let current_scope_id = scope_tree.add_scope(None, scope_flags);
        Self { scope_tree, symbol_table: SymbolTable::new(), current_scope_id }
    }

    pub fn build(mut self) -> Semantic {
        self.leave_scope();
        Semantic { scope_tree: self.scope_tree, symbol_table: self.symbol_table }
    }

    fn enter_scope(&mut self, flags: ScopeFlags) {
        let mut flags = flags;
        // Inherit strict mode for functions
        // https://tc39.es/ecma262/#sec-strict-mode-code
        let mut strict_mode = self.scope_tree.root_flags().is_strict_mode();
        let parent_scope_id = self.current_scope_id;
        let parent_scope_flags = self.scope_tree.get_flags(parent_scope_id);

        if !strict_mode && parent_scope_flags.is_function() && parent_scope_flags.is_strict_mode() {
            strict_mode = true;
        }

        // inherit flags for non-function scopes
        if !flags.contains(ScopeFlags::Function) {
            flags |= parent_scope_flags & ScopeFlags::Modifiers;
        };

        if strict_mode {
            flags |= ScopeFlags::StrictMode;
        }

        self.current_scope_id = self.scope_tree.add_scope(Some(self.current_scope_id), flags);
    }

    pub fn leave_scope(&mut self) {
        self.resolve_references_for_current_scope();
        if let Some(parent_id) = self.scope_tree.get_parent_id(self.current_scope_id) {
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
            && !self.scope_tree.get_flags(self.current_scope_id).is_var()
            && !includes.is_function()
        {
            self.get_var_hosisting_scope_id()
        } else {
            self.current_scope_id
        };

        if let Some(symbol_id) = self.check_redeclaration(scope_id, span, name, excludes) {
            return symbol_id;
        }

        let symbol_id = self.symbol_table.create_symbol(span, name.clone(), includes, scope_id);
        if includes.is_variable() {
            self.scope_tree.add_binding(scope_id, name.clone(), symbol_id);
        }
        symbol_id
    }

    fn get_var_hosisting_scope_id(&self) -> ScopeId {
        let mut top_scope_id = self.current_scope_id;
        for scope_id in self.scope_tree.ancestors(self.current_scope_id).skip(1) {
            if self.scope_tree.get_flags(scope_id).is_var() {
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
        self.scope_tree.get_binding(scope_id, name)
        // TODO: redeclaration error
        // if self.symbol_table.get_flag(symbol_id).intersects(excludes) {
        // self.error(Redeclaration(name.clone(), symbol.span(), span));
        // }
    }

    fn declare_reference(&mut self, span: Span, name: &Atom) -> ReferenceId {
        let reference_id = self.symbol_table.create_reference(span, name.clone());
        self.scope_tree.add_unresolved_reference(self.current_scope_id, name.clone(), reference_id);
        reference_id
    }

    fn resolve_references_for_current_scope(&mut self) {
        let all_references = self
            .scope_tree
            .unresolved_references_mut(self.current_scope_id)
            .drain()
            .collect::<Vec<(Atom, Vec<ReferenceId>)>>();

        let mut unresolved_references: FxHashMap<Atom, Vec<ReferenceId>> = FxHashMap::default();
        let mut resolved_references: Vec<(SymbolId, Vec<ReferenceId>)> = vec![];

        for (name, reference_ids) in all_references {
            if let Some(symbol_id) = self.scope_tree.get_binding(self.current_scope_id, &name) {
                resolved_references.push((symbol_id, reference_ids));
            } else {
                unresolved_references.insert(name, reference_ids);
            }
        }

        let scope_id =
            self.scope_tree.get_parent_id(self.current_scope_id).unwrap_or(self.current_scope_id);

        for (name, reference_ids) in unresolved_references {
            self.scope_tree.extend_unresolved_reference(scope_id, name, reference_ids);
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

    // ForStatement : for ( LexicalDeclaration Expressionopt ; Expressionopt ) Statement
    //   1. Let oldEnv be the running execution context's LexicalEnvironment.
    //   2. Let loopEnv be NewDeclarativeEnvironment(oldEnv).
    pub fn enter_for_statement(&mut self, is_lexical_declaration: bool) {
        if is_lexical_declaration {
            self.enter_scope(ScopeFlags::empty());
        }
    }

    pub fn leave_for_statement(&mut self, is_lexical_declaration: bool) {
        if is_lexical_declaration {
            self.leave_scope();
        }
    }

    pub fn enter_for_in_of_statement(&mut self, is_lexical_declaration: bool) {
        if is_lexical_declaration {
            self.enter_scope(ScopeFlags::empty());
        }
    }

    pub fn leave_for_in_of_statement(&mut self, is_lexical_declaration: bool) {
        if is_lexical_declaration {
            self.leave_scope();
        }
    }
}
