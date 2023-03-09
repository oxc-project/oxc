use oxc_ast::{AstKind, SourceType};

use super::{Scope, ScopeFlags, ScopeId, ScopeTree};

#[derive(Debug)]
pub struct ScopeBuilder {
    pub scopes: ScopeTree,

    pub current_scope_id: ScopeId,
}

impl ScopeBuilder {
    #[must_use]
    pub fn new(source_type: SourceType) -> Self {
        // Module code is always strict mode code.
        let strict_mode = source_type.is_module() || source_type.always_strict();
        let scopes = ScopeTree::new(strict_mode);
        let current_scope_id = scopes.root_scope_id();
        Self { scopes, current_scope_id }
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
        let new_scope_id = self.scopes.new_node(scope);
        self.current_scope_id.append(new_scope_id, &mut self.scopes);
        self.current_scope_id = new_scope_id.into();
    }

    pub fn leave(&mut self) {
        if let Some(parent_id) = self.scopes[self.current_scope_id.indextree_id()].parent() {
            self.current_scope_id = parent_id.into();
        }
    }

    #[must_use]
    pub fn current_scope(&self) -> &Scope {
        &self.scopes[self.current_scope_id]
    }

    #[must_use]
    pub fn scope_flags_from_ast_kind(kind: AstKind) -> Option<ScopeFlags> {
        match kind {
            AstKind::Function(_) => Some(ScopeFlags::Function),
            AstKind::ArrowExpression(_) => Some(ScopeFlags::Function | ScopeFlags::Arrow),
            AstKind::StaticBlock(_) => Some(ScopeFlags::ClassStaticBlock),
            AstKind::TSModuleBlock(_) => Some(ScopeFlags::TsModuleBlock),
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
