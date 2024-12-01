// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/get_id.rs`

use oxc_syntax::{reference::ReferenceId, scope::ScopeId, symbol::SymbolId};

use crate::ast::*;

impl Program<'_> {
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

impl IdentifierReference<'_> {
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

impl BindingIdentifier<'_> {
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

impl BlockStatement<'_> {
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

impl ForStatement<'_> {
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

impl ForInStatement<'_> {
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

impl ForOfStatement<'_> {
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

impl SwitchStatement<'_> {
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

impl CatchClause<'_> {
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

impl Function<'_> {
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

impl ArrowFunctionExpression<'_> {
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

impl Class<'_> {
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

impl StaticBlock<'_> {
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

impl TSEnumDeclaration<'_> {
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

impl TSConditionalType<'_> {
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

impl TSTypeAliasDeclaration<'_> {
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

impl TSInterfaceDeclaration<'_> {
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

impl TSMethodSignature<'_> {
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

impl TSConstructSignatureDeclaration<'_> {
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

impl TSModuleDeclaration<'_> {
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

impl TSMappedType<'_> {
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
