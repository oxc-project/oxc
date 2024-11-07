// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/get_id.rs`

use oxc_syntax::{reference::ReferenceId, scope::ScopeId, symbol::SymbolId};

use crate::ast::*;

impl<'a> Program<'a> {
    /// Get [`ScopeId`] of [`Program`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`Program`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}

impl<'a> IdentifierReference<'a> {
    /// Get [`ReferenceId`] of [`IdentifierReference`].
    ///
    /// Only use this method on a post-semantic AST where [`ReferenceId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `reference_id` is [`None`].
    #[inline]
    pub fn reference_id(&self) -> ReferenceId {
        self.reference_id.get().unwrap()
    }

    /// Set [`ReferenceId`] of [`IdentifierReference`].
    #[inline]
    pub fn set_reference_id(&self, reference_id: ReferenceId) {
        self.reference_id.set(Some(reference_id));
    }
}

impl<'a> BindingIdentifier<'a> {
    /// Get [`SymbolId`] of [`BindingIdentifier`].
    ///
    /// Only use this method on a post-semantic AST where [`SymbolId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `symbol_id` is [`None`].
    #[inline]
    pub fn symbol_id(&self) -> SymbolId {
        self.symbol_id.get().unwrap()
    }

    /// Set [`SymbolId`] of [`BindingIdentifier`].
    #[inline]
    pub fn set_symbol_id(&self, symbol_id: SymbolId) {
        self.symbol_id.set(Some(symbol_id));
    }
}

impl<'a> BlockStatement<'a> {
    /// Get [`ScopeId`] of [`BlockStatement`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`BlockStatement`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}

impl<'a> ForStatement<'a> {
    /// Get [`ScopeId`] of [`ForStatement`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`ForStatement`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}

impl<'a> ForInStatement<'a> {
    /// Get [`ScopeId`] of [`ForInStatement`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`ForInStatement`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}

impl<'a> ForOfStatement<'a> {
    /// Get [`ScopeId`] of [`ForOfStatement`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`ForOfStatement`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}

impl<'a> SwitchStatement<'a> {
    /// Get [`ScopeId`] of [`SwitchStatement`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`SwitchStatement`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}

impl<'a> CatchClause<'a> {
    /// Get [`ScopeId`] of [`CatchClause`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`CatchClause`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}

impl<'a> Function<'a> {
    /// Get [`ScopeId`] of [`Function`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`Function`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}

impl<'a> ArrowFunctionExpression<'a> {
    /// Get [`ScopeId`] of [`ArrowFunctionExpression`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`ArrowFunctionExpression`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}

impl<'a> Class<'a> {
    /// Get [`ScopeId`] of [`Class`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`Class`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}

impl<'a> StaticBlock<'a> {
    /// Get [`ScopeId`] of [`StaticBlock`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`StaticBlock`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}

impl<'a> TSEnumDeclaration<'a> {
    /// Get [`ScopeId`] of [`TSEnumDeclaration`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`TSEnumDeclaration`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}

impl<'a> TSConditionalType<'a> {
    /// Get [`ScopeId`] of [`TSConditionalType`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`TSConditionalType`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}

impl<'a> TSTypeAliasDeclaration<'a> {
    /// Get [`ScopeId`] of [`TSTypeAliasDeclaration`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`TSTypeAliasDeclaration`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}

impl<'a> TSInterfaceDeclaration<'a> {
    /// Get [`ScopeId`] of [`TSInterfaceDeclaration`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`TSInterfaceDeclaration`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}

impl<'a> TSMethodSignature<'a> {
    /// Get [`ScopeId`] of [`TSMethodSignature`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`TSMethodSignature`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}

impl<'a> TSConstructSignatureDeclaration<'a> {
    /// Get [`ScopeId`] of [`TSConstructSignatureDeclaration`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`TSConstructSignatureDeclaration`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}

impl<'a> TSModuleDeclaration<'a> {
    /// Get [`ScopeId`] of [`TSModuleDeclaration`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`TSModuleDeclaration`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}

impl<'a> TSMappedType<'a> {
    /// Get [`ScopeId`] of [`TSMappedType`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`TSMappedType`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }
}
