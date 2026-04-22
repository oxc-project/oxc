// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/get_id.rs`.

use oxc_syntax::{node::NodeId, reference::ReferenceId, scope::ScopeId, symbol::SymbolId};

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

    /// Get [`NodeId`] of [`Program`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`Program`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl IdentifierName<'_> {
    /// Get [`NodeId`] of [`IdentifierName`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`IdentifierName`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`IdentifierReference`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`IdentifierReference`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`BindingIdentifier`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`BindingIdentifier`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl LabelIdentifier<'_> {
    /// Get [`NodeId`] of [`LabelIdentifier`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`LabelIdentifier`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ThisExpression {
    /// Get [`NodeId`] of [`ThisExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ThisExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ArrayExpression<'_> {
    /// Get [`NodeId`] of [`ArrayExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ArrayExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl Elision {
    /// Get [`NodeId`] of [`Elision`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`Elision`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ObjectExpression<'_> {
    /// Get [`NodeId`] of [`ObjectExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ObjectExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ObjectProperty<'_> {
    /// Get [`NodeId`] of [`ObjectProperty`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ObjectProperty`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TemplateLiteral<'_> {
    /// Get [`NodeId`] of [`TemplateLiteral`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TemplateLiteral`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TaggedTemplateExpression<'_> {
    /// Get [`NodeId`] of [`TaggedTemplateExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TaggedTemplateExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TemplateElement<'_> {
    /// Get [`NodeId`] of [`TemplateElement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TemplateElement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ComputedMemberExpression<'_> {
    /// Get [`NodeId`] of [`ComputedMemberExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ComputedMemberExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl StaticMemberExpression<'_> {
    /// Get [`NodeId`] of [`StaticMemberExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`StaticMemberExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl PrivateFieldExpression<'_> {
    /// Get [`NodeId`] of [`PrivateFieldExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`PrivateFieldExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl CallExpression<'_> {
    /// Get [`NodeId`] of [`CallExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`CallExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl NewExpression<'_> {
    /// Get [`NodeId`] of [`NewExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`NewExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl MetaProperty<'_> {
    /// Get [`NodeId`] of [`MetaProperty`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`MetaProperty`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl SpreadElement<'_> {
    /// Get [`NodeId`] of [`SpreadElement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`SpreadElement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl UpdateExpression<'_> {
    /// Get [`NodeId`] of [`UpdateExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`UpdateExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl UnaryExpression<'_> {
    /// Get [`NodeId`] of [`UnaryExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`UnaryExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl BinaryExpression<'_> {
    /// Get [`NodeId`] of [`BinaryExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`BinaryExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl PrivateInExpression<'_> {
    /// Get [`NodeId`] of [`PrivateInExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`PrivateInExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl LogicalExpression<'_> {
    /// Get [`NodeId`] of [`LogicalExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`LogicalExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ConditionalExpression<'_> {
    /// Get [`NodeId`] of [`ConditionalExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ConditionalExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl AssignmentExpression<'_> {
    /// Get [`NodeId`] of [`AssignmentExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`AssignmentExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ArrayAssignmentTarget<'_> {
    /// Get [`NodeId`] of [`ArrayAssignmentTarget`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ArrayAssignmentTarget`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ObjectAssignmentTarget<'_> {
    /// Get [`NodeId`] of [`ObjectAssignmentTarget`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ObjectAssignmentTarget`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl AssignmentTargetRest<'_> {
    /// Get [`NodeId`] of [`AssignmentTargetRest`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`AssignmentTargetRest`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl AssignmentTargetWithDefault<'_> {
    /// Get [`NodeId`] of [`AssignmentTargetWithDefault`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`AssignmentTargetWithDefault`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl AssignmentTargetPropertyIdentifier<'_> {
    /// Get [`NodeId`] of [`AssignmentTargetPropertyIdentifier`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`AssignmentTargetPropertyIdentifier`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl AssignmentTargetPropertyProperty<'_> {
    /// Get [`NodeId`] of [`AssignmentTargetPropertyProperty`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`AssignmentTargetPropertyProperty`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl SequenceExpression<'_> {
    /// Get [`NodeId`] of [`SequenceExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`SequenceExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl Super {
    /// Get [`NodeId`] of [`Super`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`Super`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl AwaitExpression<'_> {
    /// Get [`NodeId`] of [`AwaitExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`AwaitExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ChainExpression<'_> {
    /// Get [`NodeId`] of [`ChainExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ChainExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ParenthesizedExpression<'_> {
    /// Get [`NodeId`] of [`ParenthesizedExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ParenthesizedExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl Directive<'_> {
    /// Get [`NodeId`] of [`Directive`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`Directive`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl Hashbang<'_> {
    /// Get [`NodeId`] of [`Hashbang`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`Hashbang`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`BlockStatement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`BlockStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl VariableDeclaration<'_> {
    /// Get [`NodeId`] of [`VariableDeclaration`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`VariableDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl VariableDeclarator<'_> {
    /// Get [`NodeId`] of [`VariableDeclarator`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`VariableDeclarator`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl EmptyStatement {
    /// Get [`NodeId`] of [`EmptyStatement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`EmptyStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ExpressionStatement<'_> {
    /// Get [`NodeId`] of [`ExpressionStatement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ExpressionStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl IfStatement<'_> {
    /// Get [`NodeId`] of [`IfStatement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`IfStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl DoWhileStatement<'_> {
    /// Get [`NodeId`] of [`DoWhileStatement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`DoWhileStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl WhileStatement<'_> {
    /// Get [`NodeId`] of [`WhileStatement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`WhileStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`ForStatement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ForStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`ForInStatement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ForInStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`ForOfStatement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ForOfStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ContinueStatement<'_> {
    /// Get [`NodeId`] of [`ContinueStatement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ContinueStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl BreakStatement<'_> {
    /// Get [`NodeId`] of [`BreakStatement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`BreakStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ReturnStatement<'_> {
    /// Get [`NodeId`] of [`ReturnStatement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ReturnStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl WithStatement<'_> {
    /// Get [`ScopeId`] of [`WithStatement`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`WithStatement`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }

    /// Get [`NodeId`] of [`WithStatement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`WithStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`SwitchStatement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`SwitchStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl SwitchCase<'_> {
    /// Get [`NodeId`] of [`SwitchCase`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`SwitchCase`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl LabeledStatement<'_> {
    /// Get [`NodeId`] of [`LabeledStatement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`LabeledStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ThrowStatement<'_> {
    /// Get [`NodeId`] of [`ThrowStatement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ThrowStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TryStatement<'_> {
    /// Get [`NodeId`] of [`TryStatement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TryStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`CatchClause`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`CatchClause`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl CatchParameter<'_> {
    /// Get [`NodeId`] of [`CatchParameter`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`CatchParameter`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl DebuggerStatement {
    /// Get [`NodeId`] of [`DebuggerStatement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`DebuggerStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl AssignmentPattern<'_> {
    /// Get [`NodeId`] of [`AssignmentPattern`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`AssignmentPattern`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ObjectPattern<'_> {
    /// Get [`NodeId`] of [`ObjectPattern`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ObjectPattern`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl BindingProperty<'_> {
    /// Get [`NodeId`] of [`BindingProperty`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`BindingProperty`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ArrayPattern<'_> {
    /// Get [`NodeId`] of [`ArrayPattern`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ArrayPattern`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl BindingRestElement<'_> {
    /// Get [`NodeId`] of [`BindingRestElement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`BindingRestElement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`Function`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`Function`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl FormalParameters<'_> {
    /// Get [`NodeId`] of [`FormalParameters`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`FormalParameters`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl FormalParameter<'_> {
    /// Get [`NodeId`] of [`FormalParameter`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`FormalParameter`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl FormalParameterRest<'_> {
    /// Get [`NodeId`] of [`FormalParameterRest`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`FormalParameterRest`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl FunctionBody<'_> {
    /// Get [`NodeId`] of [`FunctionBody`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`FunctionBody`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`ArrowFunctionExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ArrowFunctionExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl YieldExpression<'_> {
    /// Get [`NodeId`] of [`YieldExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`YieldExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`Class`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`Class`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ClassBody<'_> {
    /// Get [`NodeId`] of [`ClassBody`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ClassBody`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl MethodDefinition<'_> {
    /// Get [`NodeId`] of [`MethodDefinition`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`MethodDefinition`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl PropertyDefinition<'_> {
    /// Get [`NodeId`] of [`PropertyDefinition`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`PropertyDefinition`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl PrivateIdentifier<'_> {
    /// Get [`NodeId`] of [`PrivateIdentifier`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`PrivateIdentifier`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`StaticBlock`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`StaticBlock`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl AccessorProperty<'_> {
    /// Get [`NodeId`] of [`AccessorProperty`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`AccessorProperty`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ImportExpression<'_> {
    /// Get [`NodeId`] of [`ImportExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ImportExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ImportDeclaration<'_> {
    /// Get [`NodeId`] of [`ImportDeclaration`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ImportDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ImportSpecifier<'_> {
    /// Get [`NodeId`] of [`ImportSpecifier`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ImportSpecifier`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ImportDefaultSpecifier<'_> {
    /// Get [`NodeId`] of [`ImportDefaultSpecifier`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ImportDefaultSpecifier`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ImportNamespaceSpecifier<'_> {
    /// Get [`NodeId`] of [`ImportNamespaceSpecifier`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ImportNamespaceSpecifier`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl WithClause<'_> {
    /// Get [`NodeId`] of [`WithClause`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`WithClause`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ImportAttribute<'_> {
    /// Get [`NodeId`] of [`ImportAttribute`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ImportAttribute`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ExportNamedDeclaration<'_> {
    /// Get [`NodeId`] of [`ExportNamedDeclaration`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ExportNamedDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ExportDefaultDeclaration<'_> {
    /// Get [`NodeId`] of [`ExportDefaultDeclaration`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ExportDefaultDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ExportAllDeclaration<'_> {
    /// Get [`NodeId`] of [`ExportAllDeclaration`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ExportAllDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl ExportSpecifier<'_> {
    /// Get [`NodeId`] of [`ExportSpecifier`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ExportSpecifier`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl V8IntrinsicExpression<'_> {
    /// Get [`NodeId`] of [`V8IntrinsicExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`V8IntrinsicExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl BooleanLiteral {
    /// Get [`NodeId`] of [`BooleanLiteral`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`BooleanLiteral`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl NullLiteral {
    /// Get [`NodeId`] of [`NullLiteral`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`NullLiteral`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl NumericLiteral<'_> {
    /// Get [`NodeId`] of [`NumericLiteral`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`NumericLiteral`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl StringLiteral<'_> {
    /// Get [`NodeId`] of [`StringLiteral`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`StringLiteral`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl BigIntLiteral<'_> {
    /// Get [`NodeId`] of [`BigIntLiteral`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`BigIntLiteral`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl RegExpLiteral<'_> {
    /// Get [`NodeId`] of [`RegExpLiteral`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`RegExpLiteral`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl JSXElement<'_> {
    /// Get [`NodeId`] of [`JSXElement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`JSXElement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl JSXOpeningElement<'_> {
    /// Get [`NodeId`] of [`JSXOpeningElement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`JSXOpeningElement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl JSXClosingElement<'_> {
    /// Get [`NodeId`] of [`JSXClosingElement`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`JSXClosingElement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl JSXFragment<'_> {
    /// Get [`NodeId`] of [`JSXFragment`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`JSXFragment`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl JSXOpeningFragment {
    /// Get [`NodeId`] of [`JSXOpeningFragment`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`JSXOpeningFragment`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl JSXClosingFragment {
    /// Get [`NodeId`] of [`JSXClosingFragment`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`JSXClosingFragment`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl JSXNamespacedName<'_> {
    /// Get [`NodeId`] of [`JSXNamespacedName`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`JSXNamespacedName`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl JSXMemberExpression<'_> {
    /// Get [`NodeId`] of [`JSXMemberExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`JSXMemberExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl JSXExpressionContainer<'_> {
    /// Get [`NodeId`] of [`JSXExpressionContainer`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`JSXExpressionContainer`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl JSXEmptyExpression {
    /// Get [`NodeId`] of [`JSXEmptyExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`JSXEmptyExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl JSXAttribute<'_> {
    /// Get [`NodeId`] of [`JSXAttribute`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`JSXAttribute`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl JSXSpreadAttribute<'_> {
    /// Get [`NodeId`] of [`JSXSpreadAttribute`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`JSXSpreadAttribute`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl JSXIdentifier<'_> {
    /// Get [`NodeId`] of [`JSXIdentifier`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`JSXIdentifier`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl JSXSpreadChild<'_> {
    /// Get [`NodeId`] of [`JSXSpreadChild`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`JSXSpreadChild`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl JSXText<'_> {
    /// Get [`NodeId`] of [`JSXText`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`JSXText`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSThisParameter<'_> {
    /// Get [`NodeId`] of [`TSThisParameter`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSThisParameter`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSEnumDeclaration<'_> {
    /// Get [`NodeId`] of [`TSEnumDeclaration`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSEnumDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSEnumBody<'_> {
    /// Get [`ScopeId`] of [`TSEnumBody`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`TSEnumBody`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }

    /// Get [`NodeId`] of [`TSEnumBody`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSEnumBody`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSEnumMember<'_> {
    /// Get [`NodeId`] of [`TSEnumMember`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSEnumMember`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSTypeAnnotation<'_> {
    /// Get [`NodeId`] of [`TSTypeAnnotation`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSTypeAnnotation`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSLiteralType<'_> {
    /// Get [`NodeId`] of [`TSLiteralType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSLiteralType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`TSConditionalType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSConditionalType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSUnionType<'_> {
    /// Get [`NodeId`] of [`TSUnionType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSUnionType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSIntersectionType<'_> {
    /// Get [`NodeId`] of [`TSIntersectionType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSIntersectionType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSParenthesizedType<'_> {
    /// Get [`NodeId`] of [`TSParenthesizedType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSParenthesizedType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSTypeOperator<'_> {
    /// Get [`NodeId`] of [`TSTypeOperator`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSTypeOperator`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSArrayType<'_> {
    /// Get [`NodeId`] of [`TSArrayType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSArrayType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSIndexedAccessType<'_> {
    /// Get [`NodeId`] of [`TSIndexedAccessType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSIndexedAccessType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSTupleType<'_> {
    /// Get [`NodeId`] of [`TSTupleType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSTupleType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSNamedTupleMember<'_> {
    /// Get [`NodeId`] of [`TSNamedTupleMember`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSNamedTupleMember`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSOptionalType<'_> {
    /// Get [`NodeId`] of [`TSOptionalType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSOptionalType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSRestType<'_> {
    /// Get [`NodeId`] of [`TSRestType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSRestType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSAnyKeyword {
    /// Get [`NodeId`] of [`TSAnyKeyword`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSAnyKeyword`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSStringKeyword {
    /// Get [`NodeId`] of [`TSStringKeyword`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSStringKeyword`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSBooleanKeyword {
    /// Get [`NodeId`] of [`TSBooleanKeyword`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSBooleanKeyword`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSNumberKeyword {
    /// Get [`NodeId`] of [`TSNumberKeyword`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSNumberKeyword`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSNeverKeyword {
    /// Get [`NodeId`] of [`TSNeverKeyword`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSNeverKeyword`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSIntrinsicKeyword {
    /// Get [`NodeId`] of [`TSIntrinsicKeyword`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSIntrinsicKeyword`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSUnknownKeyword {
    /// Get [`NodeId`] of [`TSUnknownKeyword`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSUnknownKeyword`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSNullKeyword {
    /// Get [`NodeId`] of [`TSNullKeyword`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSNullKeyword`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSUndefinedKeyword {
    /// Get [`NodeId`] of [`TSUndefinedKeyword`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSUndefinedKeyword`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSVoidKeyword {
    /// Get [`NodeId`] of [`TSVoidKeyword`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSVoidKeyword`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSSymbolKeyword {
    /// Get [`NodeId`] of [`TSSymbolKeyword`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSSymbolKeyword`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSThisType {
    /// Get [`NodeId`] of [`TSThisType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSThisType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSObjectKeyword {
    /// Get [`NodeId`] of [`TSObjectKeyword`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSObjectKeyword`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSBigIntKeyword {
    /// Get [`NodeId`] of [`TSBigIntKeyword`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSBigIntKeyword`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSTypeReference<'_> {
    /// Get [`NodeId`] of [`TSTypeReference`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSTypeReference`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSQualifiedName<'_> {
    /// Get [`NodeId`] of [`TSQualifiedName`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSQualifiedName`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSTypeParameterInstantiation<'_> {
    /// Get [`NodeId`] of [`TSTypeParameterInstantiation`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSTypeParameterInstantiation`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSTypeParameter<'_> {
    /// Get [`NodeId`] of [`TSTypeParameter`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSTypeParameter`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSTypeParameterDeclaration<'_> {
    /// Get [`NodeId`] of [`TSTypeParameterDeclaration`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSTypeParameterDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`TSTypeAliasDeclaration`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSTypeAliasDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSClassImplements<'_> {
    /// Get [`NodeId`] of [`TSClassImplements`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSClassImplements`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`TSInterfaceDeclaration`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSInterfaceDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSInterfaceBody<'_> {
    /// Get [`NodeId`] of [`TSInterfaceBody`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSInterfaceBody`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSPropertySignature<'_> {
    /// Get [`NodeId`] of [`TSPropertySignature`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSPropertySignature`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSIndexSignature<'_> {
    /// Get [`NodeId`] of [`TSIndexSignature`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSIndexSignature`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSCallSignatureDeclaration<'_> {
    /// Get [`ScopeId`] of [`TSCallSignatureDeclaration`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`TSCallSignatureDeclaration`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }

    /// Get [`NodeId`] of [`TSCallSignatureDeclaration`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSCallSignatureDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`TSMethodSignature`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSMethodSignature`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`TSConstructSignatureDeclaration`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSConstructSignatureDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSIndexSignatureName<'_> {
    /// Get [`NodeId`] of [`TSIndexSignatureName`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSIndexSignatureName`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSInterfaceHeritage<'_> {
    /// Get [`NodeId`] of [`TSInterfaceHeritage`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSInterfaceHeritage`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSTypePredicate<'_> {
    /// Get [`NodeId`] of [`TSTypePredicate`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSTypePredicate`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`TSModuleDeclaration`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSModuleDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSGlobalDeclaration<'_> {
    /// Get [`ScopeId`] of [`TSGlobalDeclaration`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`TSGlobalDeclaration`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }

    /// Get [`NodeId`] of [`TSGlobalDeclaration`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSGlobalDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSModuleBlock<'_> {
    /// Get [`NodeId`] of [`TSModuleBlock`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSModuleBlock`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSTypeLiteral<'_> {
    /// Get [`NodeId`] of [`TSTypeLiteral`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSTypeLiteral`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSInferType<'_> {
    /// Get [`NodeId`] of [`TSInferType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSInferType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSTypeQuery<'_> {
    /// Get [`NodeId`] of [`TSTypeQuery`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSTypeQuery`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSImportType<'_> {
    /// Get [`NodeId`] of [`TSImportType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSImportType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSImportTypeQualifiedName<'_> {
    /// Get [`NodeId`] of [`TSImportTypeQualifiedName`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSImportTypeQualifiedName`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSFunctionType<'_> {
    /// Get [`ScopeId`] of [`TSFunctionType`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`TSFunctionType`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }

    /// Get [`NodeId`] of [`TSFunctionType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSFunctionType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSConstructorType<'_> {
    /// Get [`ScopeId`] of [`TSConstructorType`].
    ///
    /// Only use this method on a post-semantic AST where [`ScopeId`]s are always defined.
    ///
    /// # Panics
    /// Panics if `scope_id` is [`None`].
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.get().unwrap()
    }

    /// Set [`ScopeId`] of [`TSConstructorType`].
    #[inline]
    pub fn set_scope_id(&self, scope_id: ScopeId) {
        self.scope_id.set(Some(scope_id));
    }

    /// Get [`NodeId`] of [`TSConstructorType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSConstructorType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
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

    /// Get [`NodeId`] of [`TSMappedType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSMappedType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSTemplateLiteralType<'_> {
    /// Get [`NodeId`] of [`TSTemplateLiteralType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSTemplateLiteralType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSAsExpression<'_> {
    /// Get [`NodeId`] of [`TSAsExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSAsExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSSatisfiesExpression<'_> {
    /// Get [`NodeId`] of [`TSSatisfiesExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSSatisfiesExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSTypeAssertion<'_> {
    /// Get [`NodeId`] of [`TSTypeAssertion`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSTypeAssertion`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSImportEqualsDeclaration<'_> {
    /// Get [`NodeId`] of [`TSImportEqualsDeclaration`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSImportEqualsDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSExternalModuleReference<'_> {
    /// Get [`NodeId`] of [`TSExternalModuleReference`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSExternalModuleReference`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSNonNullExpression<'_> {
    /// Get [`NodeId`] of [`TSNonNullExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSNonNullExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl Decorator<'_> {
    /// Get [`NodeId`] of [`Decorator`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`Decorator`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSExportAssignment<'_> {
    /// Get [`NodeId`] of [`TSExportAssignment`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSExportAssignment`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSNamespaceExportDeclaration<'_> {
    /// Get [`NodeId`] of [`TSNamespaceExportDeclaration`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSNamespaceExportDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl TSInstantiationExpression<'_> {
    /// Get [`NodeId`] of [`TSInstantiationExpression`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSInstantiationExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl JSDocNullableType<'_> {
    /// Get [`NodeId`] of [`JSDocNullableType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`JSDocNullableType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl JSDocNonNullableType<'_> {
    /// Get [`NodeId`] of [`JSDocNonNullableType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`JSDocNonNullableType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}

impl JSDocUnknownType {
    /// Get [`NodeId`] of [`JSDocUnknownType`].
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`JSDocUnknownType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }
}
