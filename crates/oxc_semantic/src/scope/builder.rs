use oxc_ast::{ast::ClassType, AstKind};
use oxc_span::{Atom, SourceType};
use rustc_hash::FxHashMap;

use super::{Scope, ScopeFlags, ScopeId, ScopeTree};
use crate::{
    node::AstNode,
    symbol::{Reference, SymbolTableBuilder},
};

pub struct ScopeBuilder {
    pub scopes: ScopeTree,

    pub current_scope_id: ScopeId,
}

impl ScopeBuilder {
    pub fn new(source_type: SourceType) -> Self {
        // Module code is always strict mode code.
        let strict_mode = source_type.is_module() || source_type.always_strict();
        let scopes = ScopeTree::new(strict_mode);
        let current_scope_id = scopes.root_scope_id();
        Self { scopes, current_scope_id }
    }

    pub fn current_scope(&self) -> &Scope {
        &self.scopes[self.current_scope_id]
    }

    pub fn current_scope_mut(&mut self) -> &mut Scope {
        &mut self.scopes[self.current_scope_id]
    }

    pub fn node_scope(&self, node: &AstNode) -> &Scope {
        self.scopes[node.get().scope_id().indextree_id()].get()
    }

    pub fn enter(&mut self, flags: ScopeFlags) {
        // Inherit strict mode for functions
        // https://tc39.es/ecma262/#sec-strict-mode-code
        let mut strict_mode = self.scopes[self.scopes.root_scope_id()].strict_mode;
        let parent_scope = self.current_scope();
        if !strict_mode && parent_scope.is_function() && parent_scope.strict_mode {
            strict_mode = true;
        }

        // inherit flags for non-function scopes
        let flags = if flags.contains(ScopeFlags::Function) {
            flags
        } else {
            flags | (parent_scope.flags & ScopeFlags::MODIFIERS)
        };

        let scope = Scope::new(flags, strict_mode);
        let new_scope_id = self.current_scope_id.append_value(scope, &mut self.scopes);
        self.current_scope_id = new_scope_id.into();
    }

    pub fn leave(&mut self) {
        if let Some(parent_id) = self.scopes[self.current_scope_id.indextree_id()].parent() {
            self.current_scope_id = parent_id.into();
        }
    }

    pub fn resolve_reference(&mut self, symbol_table: &mut SymbolTableBuilder) {
        // At the initial stage, all references are unresolved.
        let all_references = {
            let current_scope = self.current_scope_mut();
            std::mem::take(&mut current_scope.unresolved_references)
        };
        let mut unresolved_references = FxHashMap::default();

        'outer: for (variable, reference) in all_references {
            // The reference resolves to the first matching symbol in the scope chain
            let scope_chain = self.scopes.ancestors(self.current_scope_id);
            for scope in scope_chain {
                let scope = &self.scopes[scope];
                if let Some(symbol_id) = scope.get().get_variable_symbol_id(&variable) {
                    // We have resolved this reference.
                    symbol_table.resolve_reference(reference, symbol_id);
                    continue 'outer;
                }
            }

            unresolved_references.insert(variable, reference);
        }

        let current_scope = self.current_scope_mut();
        current_scope.unresolved_references = unresolved_references;
    }

    pub fn reference_identifier(&mut self, name: &Atom, reference: Reference) {
        self.current_scope_mut()
            .unresolved_references
            .entry(name.clone())
            .or_default()
            .push(reference);
    }

    pub fn scope_flags_from_ast_kind(kind: AstKind) -> Option<ScopeFlags> {
        match kind {
            AstKind::Function(_) => Some(ScopeFlags::Function),
            AstKind::ArrowExpression(_) => Some(ScopeFlags::Function | ScopeFlags::Arrow),
            AstKind::StaticBlock(_) => Some(ScopeFlags::ClassStaticBlock),
            AstKind::TSModuleBlock(_) => Some(ScopeFlags::TsModuleBlock),
            AstKind::Class(class) if matches!(class.r#type, ClassType::ClassExpression) => {
                // Class expression creates a temporary scope with the class name as its only variable
                // E.g., `let c = class A { foo() { console.log(A) } }`
                Some(ScopeFlags::empty())
            }
            AstKind::BlockStatement(_)
            | AstKind::CatchClause(_)
            | AstKind::ForStatement(_)
            | AstKind::ForInStatement(_)
            | AstKind::ForOfStatement(_)
            | AstKind::SwitchStatement(_) => Some(ScopeFlags::empty()),
            _ => None,
        }
    }
}
