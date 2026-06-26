// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/get_id.rs`.

#![expect(clippy::inline_always)]
#![expect(clippy::match_same_arms)]
use oxc_syntax::{node::NodeId, reference::ReferenceId, scope::ScopeId, symbol::SymbolId};

use crate::ast::*;

impl Program<'_> {
    /// Get [`NodeId`] of [`Program`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`Program`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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

impl IdentifierName<'_> {
    /// Get [`NodeId`] of [`IdentifierName`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    /// Get [`NodeId`] of [`IdentifierReference`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`IdentifierReference`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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
    /// Get [`NodeId`] of [`BindingIdentifier`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`BindingIdentifier`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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

impl LabelIdentifier<'_> {
    /// Get [`NodeId`] of [`LabelIdentifier`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    /// Get [`NodeId`] of [`BlockStatement`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`BlockStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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

impl VariableDeclaration<'_> {
    /// Get [`NodeId`] of [`VariableDeclaration`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    /// Get [`NodeId`] of [`ForStatement`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ForStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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
    /// Get [`NodeId`] of [`ForInStatement`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ForInStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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
    /// Get [`NodeId`] of [`ForOfStatement`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ForOfStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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

impl ContinueStatement<'_> {
    /// Get [`NodeId`] of [`ContinueStatement`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    /// Get [`NodeId`] of [`WithStatement`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`WithStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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
}

impl SwitchStatement<'_> {
    /// Get [`NodeId`] of [`SwitchStatement`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`SwitchStatement`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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

impl SwitchCase<'_> {
    /// Get [`NodeId`] of [`SwitchCase`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    /// Get [`NodeId`] of [`CatchClause`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`CatchClause`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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

impl CatchParameter<'_> {
    /// Get [`NodeId`] of [`CatchParameter`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    /// Get [`NodeId`] of [`Function`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`Function`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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

impl FormalParameters<'_> {
    /// Get [`NodeId`] of [`FormalParameters`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    /// Get [`NodeId`] of [`ArrowFunctionExpression`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`ArrowFunctionExpression`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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

impl YieldExpression<'_> {
    /// Get [`NodeId`] of [`YieldExpression`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    /// Get [`NodeId`] of [`Class`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`Class`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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

impl ClassBody<'_> {
    /// Get [`NodeId`] of [`ClassBody`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    /// Get [`NodeId`] of [`StaticBlock`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`StaticBlock`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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

impl AccessorProperty<'_> {
    /// Get [`NodeId`] of [`AccessorProperty`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    /// Get [`NodeId`] of [`TSEnumBody`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSEnumBody`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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
}

impl TSEnumMember<'_> {
    /// Get [`NodeId`] of [`TSEnumMember`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    /// Get [`NodeId`] of [`TSConditionalType`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSConditionalType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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

impl TSUnionType<'_> {
    /// Get [`NodeId`] of [`TSUnionType`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    /// Get [`NodeId`] of [`TSTypeAliasDeclaration`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSTypeAliasDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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

impl TSClassImplements<'_> {
    /// Get [`NodeId`] of [`TSClassImplements`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    /// Get [`NodeId`] of [`TSInterfaceDeclaration`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSInterfaceDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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

impl TSInterfaceBody<'_> {
    /// Get [`NodeId`] of [`TSInterfaceBody`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    /// Get [`NodeId`] of [`TSCallSignatureDeclaration`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSCallSignatureDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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
}

impl TSMethodSignature<'_> {
    /// Get [`NodeId`] of [`TSMethodSignature`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSMethodSignature`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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
    /// Get [`NodeId`] of [`TSConstructSignatureDeclaration`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSConstructSignatureDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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

impl TSIndexSignatureName<'_> {
    /// Get [`NodeId`] of [`TSIndexSignatureName`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    /// Get [`NodeId`] of [`TSModuleDeclaration`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSModuleDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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

impl TSGlobalDeclaration<'_> {
    /// Get [`NodeId`] of [`TSGlobalDeclaration`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSGlobalDeclaration`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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
}

impl TSModuleBlock<'_> {
    /// Get [`NodeId`] of [`TSModuleBlock`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    /// Get [`NodeId`] of [`TSFunctionType`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSFunctionType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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
}

impl TSConstructorType<'_> {
    /// Get [`NodeId`] of [`TSConstructorType`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSConstructorType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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
}

impl TSMappedType<'_> {
    /// Get [`NodeId`] of [`TSMappedType`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
    #[inline]
    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    /// Set [`NodeId`] of [`TSMappedType`].
    #[inline]
    pub fn set_node_id(&self, node_id: NodeId) {
        self.node_id.set(node_id);
    }

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

impl TSTemplateLiteralType<'_> {
    /// Get [`NodeId`] of [`TSTemplateLiteralType`].
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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
    ///
    /// Only use this method on a post-semantic AST where [`NodeId`]s are always defined.
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

impl Expression<'_> {
    /// Get [`NodeId`] of [`Expression`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::BooleanLiteral(it) => it.node_id(),
            Self::NullLiteral(it) => it.node_id(),
            Self::NumericLiteral(it) => it.node_id(),
            Self::BigIntLiteral(it) => it.node_id(),
            Self::RegExpLiteral(it) => it.node_id(),
            Self::StringLiteral(it) => it.node_id(),
            Self::TemplateLiteral(it) => it.node_id(),
            Self::Identifier(it) => it.node_id(),
            Self::MetaProperty(it) => it.node_id(),
            Self::Super(it) => it.node_id(),
            Self::ArrayExpression(it) => it.node_id(),
            Self::ArrowFunctionExpression(it) => it.node_id(),
            Self::AssignmentExpression(it) => it.node_id(),
            Self::AwaitExpression(it) => it.node_id(),
            Self::BinaryExpression(it) => it.node_id(),
            Self::CallExpression(it) => it.node_id(),
            Self::ChainExpression(it) => it.node_id(),
            Self::ClassExpression(it) => it.node_id(),
            Self::ConditionalExpression(it) => it.node_id(),
            Self::FunctionExpression(it) => it.node_id(),
            Self::ImportExpression(it) => it.node_id(),
            Self::LogicalExpression(it) => it.node_id(),
            Self::NewExpression(it) => it.node_id(),
            Self::ObjectExpression(it) => it.node_id(),
            Self::ParenthesizedExpression(it) => it.node_id(),
            Self::SequenceExpression(it) => it.node_id(),
            Self::TaggedTemplateExpression(it) => it.node_id(),
            Self::ThisExpression(it) => it.node_id(),
            Self::UnaryExpression(it) => it.node_id(),
            Self::UpdateExpression(it) => it.node_id(),
            Self::YieldExpression(it) => it.node_id(),
            Self::PrivateInExpression(it) => it.node_id(),
            Self::JSXElement(it) => it.node_id(),
            Self::JSXFragment(it) => it.node_id(),
            Self::TSAsExpression(it) => it.node_id(),
            Self::TSSatisfiesExpression(it) => it.node_id(),
            Self::TSTypeAssertion(it) => it.node_id(),
            Self::TSNonNullExpression(it) => it.node_id(),
            Self::TSInstantiationExpression(it) => it.node_id(),
            Self::V8IntrinsicExpression(it) => it.node_id(),
            Self::ComputedMemberExpression(it) => it.node_id(),
            Self::StaticMemberExpression(it) => it.node_id(),
            Self::PrivateFieldExpression(it) => it.node_id(),
        }
    }
}

impl ArrayExpressionElement<'_> {
    /// Get [`NodeId`] of [`ArrayExpressionElement`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::SpreadElement(it) => it.node_id(),
            Self::Elision(it) => it.node_id(),
            Self::BooleanLiteral(it) => it.node_id(),
            Self::NullLiteral(it) => it.node_id(),
            Self::NumericLiteral(it) => it.node_id(),
            Self::BigIntLiteral(it) => it.node_id(),
            Self::RegExpLiteral(it) => it.node_id(),
            Self::StringLiteral(it) => it.node_id(),
            Self::TemplateLiteral(it) => it.node_id(),
            Self::Identifier(it) => it.node_id(),
            Self::MetaProperty(it) => it.node_id(),
            Self::Super(it) => it.node_id(),
            Self::ArrayExpression(it) => it.node_id(),
            Self::ArrowFunctionExpression(it) => it.node_id(),
            Self::AssignmentExpression(it) => it.node_id(),
            Self::AwaitExpression(it) => it.node_id(),
            Self::BinaryExpression(it) => it.node_id(),
            Self::CallExpression(it) => it.node_id(),
            Self::ChainExpression(it) => it.node_id(),
            Self::ClassExpression(it) => it.node_id(),
            Self::ConditionalExpression(it) => it.node_id(),
            Self::FunctionExpression(it) => it.node_id(),
            Self::ImportExpression(it) => it.node_id(),
            Self::LogicalExpression(it) => it.node_id(),
            Self::NewExpression(it) => it.node_id(),
            Self::ObjectExpression(it) => it.node_id(),
            Self::ParenthesizedExpression(it) => it.node_id(),
            Self::SequenceExpression(it) => it.node_id(),
            Self::TaggedTemplateExpression(it) => it.node_id(),
            Self::ThisExpression(it) => it.node_id(),
            Self::UnaryExpression(it) => it.node_id(),
            Self::UpdateExpression(it) => it.node_id(),
            Self::YieldExpression(it) => it.node_id(),
            Self::PrivateInExpression(it) => it.node_id(),
            Self::JSXElement(it) => it.node_id(),
            Self::JSXFragment(it) => it.node_id(),
            Self::TSAsExpression(it) => it.node_id(),
            Self::TSSatisfiesExpression(it) => it.node_id(),
            Self::TSTypeAssertion(it) => it.node_id(),
            Self::TSNonNullExpression(it) => it.node_id(),
            Self::TSInstantiationExpression(it) => it.node_id(),
            Self::V8IntrinsicExpression(it) => it.node_id(),
            Self::ComputedMemberExpression(it) => it.node_id(),
            Self::StaticMemberExpression(it) => it.node_id(),
            Self::PrivateFieldExpression(it) => it.node_id(),
        }
    }
}

impl ObjectPropertyKind<'_> {
    /// Get [`NodeId`] of [`ObjectPropertyKind`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::ObjectProperty(it) => it.node_id(),
            Self::SpreadProperty(it) => it.node_id(),
        }
    }
}

impl PropertyKey<'_> {
    /// Get [`NodeId`] of [`PropertyKey`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::StaticIdentifier(it) => it.node_id(),
            Self::PrivateIdentifier(it) => it.node_id(),
            Self::BooleanLiteral(it) => it.node_id(),
            Self::NullLiteral(it) => it.node_id(),
            Self::NumericLiteral(it) => it.node_id(),
            Self::BigIntLiteral(it) => it.node_id(),
            Self::RegExpLiteral(it) => it.node_id(),
            Self::StringLiteral(it) => it.node_id(),
            Self::TemplateLiteral(it) => it.node_id(),
            Self::Identifier(it) => it.node_id(),
            Self::MetaProperty(it) => it.node_id(),
            Self::Super(it) => it.node_id(),
            Self::ArrayExpression(it) => it.node_id(),
            Self::ArrowFunctionExpression(it) => it.node_id(),
            Self::AssignmentExpression(it) => it.node_id(),
            Self::AwaitExpression(it) => it.node_id(),
            Self::BinaryExpression(it) => it.node_id(),
            Self::CallExpression(it) => it.node_id(),
            Self::ChainExpression(it) => it.node_id(),
            Self::ClassExpression(it) => it.node_id(),
            Self::ConditionalExpression(it) => it.node_id(),
            Self::FunctionExpression(it) => it.node_id(),
            Self::ImportExpression(it) => it.node_id(),
            Self::LogicalExpression(it) => it.node_id(),
            Self::NewExpression(it) => it.node_id(),
            Self::ObjectExpression(it) => it.node_id(),
            Self::ParenthesizedExpression(it) => it.node_id(),
            Self::SequenceExpression(it) => it.node_id(),
            Self::TaggedTemplateExpression(it) => it.node_id(),
            Self::ThisExpression(it) => it.node_id(),
            Self::UnaryExpression(it) => it.node_id(),
            Self::UpdateExpression(it) => it.node_id(),
            Self::YieldExpression(it) => it.node_id(),
            Self::PrivateInExpression(it) => it.node_id(),
            Self::JSXElement(it) => it.node_id(),
            Self::JSXFragment(it) => it.node_id(),
            Self::TSAsExpression(it) => it.node_id(),
            Self::TSSatisfiesExpression(it) => it.node_id(),
            Self::TSTypeAssertion(it) => it.node_id(),
            Self::TSNonNullExpression(it) => it.node_id(),
            Self::TSInstantiationExpression(it) => it.node_id(),
            Self::V8IntrinsicExpression(it) => it.node_id(),
            Self::ComputedMemberExpression(it) => it.node_id(),
            Self::StaticMemberExpression(it) => it.node_id(),
            Self::PrivateFieldExpression(it) => it.node_id(),
        }
    }
}

impl MemberExpression<'_> {
    /// Get [`NodeId`] of [`MemberExpression`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::ComputedMemberExpression(it) => it.node_id(),
            Self::StaticMemberExpression(it) => it.node_id(),
            Self::PrivateFieldExpression(it) => it.node_id(),
        }
    }
}

impl Argument<'_> {
    /// Get [`NodeId`] of [`Argument`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::SpreadElement(it) => it.node_id(),
            Self::BooleanLiteral(it) => it.node_id(),
            Self::NullLiteral(it) => it.node_id(),
            Self::NumericLiteral(it) => it.node_id(),
            Self::BigIntLiteral(it) => it.node_id(),
            Self::RegExpLiteral(it) => it.node_id(),
            Self::StringLiteral(it) => it.node_id(),
            Self::TemplateLiteral(it) => it.node_id(),
            Self::Identifier(it) => it.node_id(),
            Self::MetaProperty(it) => it.node_id(),
            Self::Super(it) => it.node_id(),
            Self::ArrayExpression(it) => it.node_id(),
            Self::ArrowFunctionExpression(it) => it.node_id(),
            Self::AssignmentExpression(it) => it.node_id(),
            Self::AwaitExpression(it) => it.node_id(),
            Self::BinaryExpression(it) => it.node_id(),
            Self::CallExpression(it) => it.node_id(),
            Self::ChainExpression(it) => it.node_id(),
            Self::ClassExpression(it) => it.node_id(),
            Self::ConditionalExpression(it) => it.node_id(),
            Self::FunctionExpression(it) => it.node_id(),
            Self::ImportExpression(it) => it.node_id(),
            Self::LogicalExpression(it) => it.node_id(),
            Self::NewExpression(it) => it.node_id(),
            Self::ObjectExpression(it) => it.node_id(),
            Self::ParenthesizedExpression(it) => it.node_id(),
            Self::SequenceExpression(it) => it.node_id(),
            Self::TaggedTemplateExpression(it) => it.node_id(),
            Self::ThisExpression(it) => it.node_id(),
            Self::UnaryExpression(it) => it.node_id(),
            Self::UpdateExpression(it) => it.node_id(),
            Self::YieldExpression(it) => it.node_id(),
            Self::PrivateInExpression(it) => it.node_id(),
            Self::JSXElement(it) => it.node_id(),
            Self::JSXFragment(it) => it.node_id(),
            Self::TSAsExpression(it) => it.node_id(),
            Self::TSSatisfiesExpression(it) => it.node_id(),
            Self::TSTypeAssertion(it) => it.node_id(),
            Self::TSNonNullExpression(it) => it.node_id(),
            Self::TSInstantiationExpression(it) => it.node_id(),
            Self::V8IntrinsicExpression(it) => it.node_id(),
            Self::ComputedMemberExpression(it) => it.node_id(),
            Self::StaticMemberExpression(it) => it.node_id(),
            Self::PrivateFieldExpression(it) => it.node_id(),
        }
    }
}

impl AssignmentTarget<'_> {
    /// Get [`NodeId`] of [`AssignmentTarget`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::AssignmentTargetIdentifier(it) => it.node_id(),
            Self::TSAsExpression(it) => it.node_id(),
            Self::TSSatisfiesExpression(it) => it.node_id(),
            Self::TSNonNullExpression(it) => it.node_id(),
            Self::TSTypeAssertion(it) => it.node_id(),
            Self::ComputedMemberExpression(it) => it.node_id(),
            Self::StaticMemberExpression(it) => it.node_id(),
            Self::PrivateFieldExpression(it) => it.node_id(),
            Self::ArrayAssignmentTarget(it) => it.node_id(),
            Self::ObjectAssignmentTarget(it) => it.node_id(),
        }
    }
}

impl SimpleAssignmentTarget<'_> {
    /// Get [`NodeId`] of [`SimpleAssignmentTarget`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::AssignmentTargetIdentifier(it) => it.node_id(),
            Self::TSAsExpression(it) => it.node_id(),
            Self::TSSatisfiesExpression(it) => it.node_id(),
            Self::TSNonNullExpression(it) => it.node_id(),
            Self::TSTypeAssertion(it) => it.node_id(),
            Self::ComputedMemberExpression(it) => it.node_id(),
            Self::StaticMemberExpression(it) => it.node_id(),
            Self::PrivateFieldExpression(it) => it.node_id(),
        }
    }
}

impl AssignmentTargetPattern<'_> {
    /// Get [`NodeId`] of [`AssignmentTargetPattern`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::ArrayAssignmentTarget(it) => it.node_id(),
            Self::ObjectAssignmentTarget(it) => it.node_id(),
        }
    }
}

impl AssignmentTargetMaybeDefault<'_> {
    /// Get [`NodeId`] of [`AssignmentTargetMaybeDefault`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::AssignmentTargetWithDefault(it) => it.node_id(),
            Self::AssignmentTargetIdentifier(it) => it.node_id(),
            Self::TSAsExpression(it) => it.node_id(),
            Self::TSSatisfiesExpression(it) => it.node_id(),
            Self::TSNonNullExpression(it) => it.node_id(),
            Self::TSTypeAssertion(it) => it.node_id(),
            Self::ComputedMemberExpression(it) => it.node_id(),
            Self::StaticMemberExpression(it) => it.node_id(),
            Self::PrivateFieldExpression(it) => it.node_id(),
            Self::ArrayAssignmentTarget(it) => it.node_id(),
            Self::ObjectAssignmentTarget(it) => it.node_id(),
        }
    }
}

impl AssignmentTargetProperty<'_> {
    /// Get [`NodeId`] of [`AssignmentTargetProperty`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => it.node_id(),
            Self::AssignmentTargetPropertyProperty(it) => it.node_id(),
        }
    }
}

impl ChainElement<'_> {
    /// Get [`NodeId`] of [`ChainElement`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::CallExpression(it) => it.node_id(),
            Self::TSNonNullExpression(it) => it.node_id(),
            Self::ComputedMemberExpression(it) => it.node_id(),
            Self::StaticMemberExpression(it) => it.node_id(),
            Self::PrivateFieldExpression(it) => it.node_id(),
        }
    }
}

impl Statement<'_> {
    /// Get [`NodeId`] of [`Statement`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::BlockStatement(it) => it.node_id(),
            Self::BreakStatement(it) => it.node_id(),
            Self::ContinueStatement(it) => it.node_id(),
            Self::DebuggerStatement(it) => it.node_id(),
            Self::DoWhileStatement(it) => it.node_id(),
            Self::EmptyStatement(it) => it.node_id(),
            Self::ExpressionStatement(it) => it.node_id(),
            Self::ForInStatement(it) => it.node_id(),
            Self::ForOfStatement(it) => it.node_id(),
            Self::ForStatement(it) => it.node_id(),
            Self::IfStatement(it) => it.node_id(),
            Self::LabeledStatement(it) => it.node_id(),
            Self::ReturnStatement(it) => it.node_id(),
            Self::SwitchStatement(it) => it.node_id(),
            Self::ThrowStatement(it) => it.node_id(),
            Self::TryStatement(it) => it.node_id(),
            Self::WhileStatement(it) => it.node_id(),
            Self::WithStatement(it) => it.node_id(),
            Self::VariableDeclaration(it) => it.node_id(),
            Self::FunctionDeclaration(it) => it.node_id(),
            Self::ClassDeclaration(it) => it.node_id(),
            Self::TSTypeAliasDeclaration(it) => it.node_id(),
            Self::TSInterfaceDeclaration(it) => it.node_id(),
            Self::TSEnumDeclaration(it) => it.node_id(),
            Self::TSModuleDeclaration(it) => it.node_id(),
            Self::TSGlobalDeclaration(it) => it.node_id(),
            Self::TSImportEqualsDeclaration(it) => it.node_id(),
            Self::ImportDeclaration(it) => it.node_id(),
            Self::ExportAllDeclaration(it) => it.node_id(),
            Self::ExportDefaultDeclaration(it) => it.node_id(),
            Self::ExportNamedDeclaration(it) => it.node_id(),
            Self::TSExportAssignment(it) => it.node_id(),
            Self::TSNamespaceExportDeclaration(it) => it.node_id(),
        }
    }
}

impl Declaration<'_> {
    /// Get [`NodeId`] of [`Declaration`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::VariableDeclaration(it) => it.node_id(),
            Self::FunctionDeclaration(it) => it.node_id(),
            Self::ClassDeclaration(it) => it.node_id(),
            Self::TSTypeAliasDeclaration(it) => it.node_id(),
            Self::TSInterfaceDeclaration(it) => it.node_id(),
            Self::TSEnumDeclaration(it) => it.node_id(),
            Self::TSModuleDeclaration(it) => it.node_id(),
            Self::TSGlobalDeclaration(it) => it.node_id(),
            Self::TSImportEqualsDeclaration(it) => it.node_id(),
        }
    }
}

impl ForStatementInit<'_> {
    /// Get [`NodeId`] of [`ForStatementInit`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::VariableDeclaration(it) => it.node_id(),
            Self::BooleanLiteral(it) => it.node_id(),
            Self::NullLiteral(it) => it.node_id(),
            Self::NumericLiteral(it) => it.node_id(),
            Self::BigIntLiteral(it) => it.node_id(),
            Self::RegExpLiteral(it) => it.node_id(),
            Self::StringLiteral(it) => it.node_id(),
            Self::TemplateLiteral(it) => it.node_id(),
            Self::Identifier(it) => it.node_id(),
            Self::MetaProperty(it) => it.node_id(),
            Self::Super(it) => it.node_id(),
            Self::ArrayExpression(it) => it.node_id(),
            Self::ArrowFunctionExpression(it) => it.node_id(),
            Self::AssignmentExpression(it) => it.node_id(),
            Self::AwaitExpression(it) => it.node_id(),
            Self::BinaryExpression(it) => it.node_id(),
            Self::CallExpression(it) => it.node_id(),
            Self::ChainExpression(it) => it.node_id(),
            Self::ClassExpression(it) => it.node_id(),
            Self::ConditionalExpression(it) => it.node_id(),
            Self::FunctionExpression(it) => it.node_id(),
            Self::ImportExpression(it) => it.node_id(),
            Self::LogicalExpression(it) => it.node_id(),
            Self::NewExpression(it) => it.node_id(),
            Self::ObjectExpression(it) => it.node_id(),
            Self::ParenthesizedExpression(it) => it.node_id(),
            Self::SequenceExpression(it) => it.node_id(),
            Self::TaggedTemplateExpression(it) => it.node_id(),
            Self::ThisExpression(it) => it.node_id(),
            Self::UnaryExpression(it) => it.node_id(),
            Self::UpdateExpression(it) => it.node_id(),
            Self::YieldExpression(it) => it.node_id(),
            Self::PrivateInExpression(it) => it.node_id(),
            Self::JSXElement(it) => it.node_id(),
            Self::JSXFragment(it) => it.node_id(),
            Self::TSAsExpression(it) => it.node_id(),
            Self::TSSatisfiesExpression(it) => it.node_id(),
            Self::TSTypeAssertion(it) => it.node_id(),
            Self::TSNonNullExpression(it) => it.node_id(),
            Self::TSInstantiationExpression(it) => it.node_id(),
            Self::V8IntrinsicExpression(it) => it.node_id(),
            Self::ComputedMemberExpression(it) => it.node_id(),
            Self::StaticMemberExpression(it) => it.node_id(),
            Self::PrivateFieldExpression(it) => it.node_id(),
        }
    }
}

impl ForStatementLeft<'_> {
    /// Get [`NodeId`] of [`ForStatementLeft`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::VariableDeclaration(it) => it.node_id(),
            Self::AssignmentTargetIdentifier(it) => it.node_id(),
            Self::TSAsExpression(it) => it.node_id(),
            Self::TSSatisfiesExpression(it) => it.node_id(),
            Self::TSNonNullExpression(it) => it.node_id(),
            Self::TSTypeAssertion(it) => it.node_id(),
            Self::ComputedMemberExpression(it) => it.node_id(),
            Self::StaticMemberExpression(it) => it.node_id(),
            Self::PrivateFieldExpression(it) => it.node_id(),
            Self::ArrayAssignmentTarget(it) => it.node_id(),
            Self::ObjectAssignmentTarget(it) => it.node_id(),
        }
    }
}

impl BindingPattern<'_> {
    /// Get [`NodeId`] of [`BindingPattern`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::BindingIdentifier(it) => it.node_id(),
            Self::ObjectPattern(it) => it.node_id(),
            Self::ArrayPattern(it) => it.node_id(),
            Self::AssignmentPattern(it) => it.node_id(),
        }
    }
}

impl ClassElement<'_> {
    /// Get [`NodeId`] of [`ClassElement`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::StaticBlock(it) => it.node_id(),
            Self::MethodDefinition(it) => it.node_id(),
            Self::PropertyDefinition(it) => it.node_id(),
            Self::AccessorProperty(it) => it.node_id(),
            Self::TSIndexSignature(it) => it.node_id(),
        }
    }
}

impl ModuleDeclaration<'_> {
    /// Get [`NodeId`] of [`ModuleDeclaration`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::ImportDeclaration(it) => it.node_id(),
            Self::ExportAllDeclaration(it) => it.node_id(),
            Self::ExportDefaultDeclaration(it) => it.node_id(),
            Self::ExportNamedDeclaration(it) => it.node_id(),
            Self::TSExportAssignment(it) => it.node_id(),
            Self::TSNamespaceExportDeclaration(it) => it.node_id(),
        }
    }
}

impl ImportDeclarationSpecifier<'_> {
    /// Get [`NodeId`] of [`ImportDeclarationSpecifier`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::ImportSpecifier(it) => it.node_id(),
            Self::ImportDefaultSpecifier(it) => it.node_id(),
            Self::ImportNamespaceSpecifier(it) => it.node_id(),
        }
    }
}

impl ImportAttributeKey<'_> {
    /// Get [`NodeId`] of [`ImportAttributeKey`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::Identifier(it) => it.node_id(),
            Self::StringLiteral(it) => it.node_id(),
        }
    }
}

impl ExportDefaultDeclarationKind<'_> {
    /// Get [`NodeId`] of [`ExportDefaultDeclarationKind`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::FunctionDeclaration(it) => it.node_id(),
            Self::ClassDeclaration(it) => it.node_id(),
            Self::TSInterfaceDeclaration(it) => it.node_id(),
            Self::BooleanLiteral(it) => it.node_id(),
            Self::NullLiteral(it) => it.node_id(),
            Self::NumericLiteral(it) => it.node_id(),
            Self::BigIntLiteral(it) => it.node_id(),
            Self::RegExpLiteral(it) => it.node_id(),
            Self::StringLiteral(it) => it.node_id(),
            Self::TemplateLiteral(it) => it.node_id(),
            Self::Identifier(it) => it.node_id(),
            Self::MetaProperty(it) => it.node_id(),
            Self::Super(it) => it.node_id(),
            Self::ArrayExpression(it) => it.node_id(),
            Self::ArrowFunctionExpression(it) => it.node_id(),
            Self::AssignmentExpression(it) => it.node_id(),
            Self::AwaitExpression(it) => it.node_id(),
            Self::BinaryExpression(it) => it.node_id(),
            Self::CallExpression(it) => it.node_id(),
            Self::ChainExpression(it) => it.node_id(),
            Self::ClassExpression(it) => it.node_id(),
            Self::ConditionalExpression(it) => it.node_id(),
            Self::FunctionExpression(it) => it.node_id(),
            Self::ImportExpression(it) => it.node_id(),
            Self::LogicalExpression(it) => it.node_id(),
            Self::NewExpression(it) => it.node_id(),
            Self::ObjectExpression(it) => it.node_id(),
            Self::ParenthesizedExpression(it) => it.node_id(),
            Self::SequenceExpression(it) => it.node_id(),
            Self::TaggedTemplateExpression(it) => it.node_id(),
            Self::ThisExpression(it) => it.node_id(),
            Self::UnaryExpression(it) => it.node_id(),
            Self::UpdateExpression(it) => it.node_id(),
            Self::YieldExpression(it) => it.node_id(),
            Self::PrivateInExpression(it) => it.node_id(),
            Self::JSXElement(it) => it.node_id(),
            Self::JSXFragment(it) => it.node_id(),
            Self::TSAsExpression(it) => it.node_id(),
            Self::TSSatisfiesExpression(it) => it.node_id(),
            Self::TSTypeAssertion(it) => it.node_id(),
            Self::TSNonNullExpression(it) => it.node_id(),
            Self::TSInstantiationExpression(it) => it.node_id(),
            Self::V8IntrinsicExpression(it) => it.node_id(),
            Self::ComputedMemberExpression(it) => it.node_id(),
            Self::StaticMemberExpression(it) => it.node_id(),
            Self::PrivateFieldExpression(it) => it.node_id(),
        }
    }
}

impl ModuleExportName<'_> {
    /// Get [`NodeId`] of [`ModuleExportName`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::IdentifierName(it) => it.node_id(),
            Self::IdentifierReference(it) => it.node_id(),
            Self::StringLiteral(it) => it.node_id(),
        }
    }
}

impl JSXElementName<'_> {
    /// Get [`NodeId`] of [`JSXElementName`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::Identifier(it) => it.node_id(),
            Self::IdentifierReference(it) => it.node_id(),
            Self::NamespacedName(it) => it.node_id(),
            Self::MemberExpression(it) => it.node_id(),
            Self::ThisExpression(it) => it.node_id(),
        }
    }
}

impl JSXMemberExpressionObject<'_> {
    /// Get [`NodeId`] of [`JSXMemberExpressionObject`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::IdentifierReference(it) => it.node_id(),
            Self::MemberExpression(it) => it.node_id(),
            Self::ThisExpression(it) => it.node_id(),
        }
    }
}

impl JSXExpression<'_> {
    /// Get [`NodeId`] of [`JSXExpression`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::EmptyExpression(it) => it.node_id(),
            Self::BooleanLiteral(it) => it.node_id(),
            Self::NullLiteral(it) => it.node_id(),
            Self::NumericLiteral(it) => it.node_id(),
            Self::BigIntLiteral(it) => it.node_id(),
            Self::RegExpLiteral(it) => it.node_id(),
            Self::StringLiteral(it) => it.node_id(),
            Self::TemplateLiteral(it) => it.node_id(),
            Self::Identifier(it) => it.node_id(),
            Self::MetaProperty(it) => it.node_id(),
            Self::Super(it) => it.node_id(),
            Self::ArrayExpression(it) => it.node_id(),
            Self::ArrowFunctionExpression(it) => it.node_id(),
            Self::AssignmentExpression(it) => it.node_id(),
            Self::AwaitExpression(it) => it.node_id(),
            Self::BinaryExpression(it) => it.node_id(),
            Self::CallExpression(it) => it.node_id(),
            Self::ChainExpression(it) => it.node_id(),
            Self::ClassExpression(it) => it.node_id(),
            Self::ConditionalExpression(it) => it.node_id(),
            Self::FunctionExpression(it) => it.node_id(),
            Self::ImportExpression(it) => it.node_id(),
            Self::LogicalExpression(it) => it.node_id(),
            Self::NewExpression(it) => it.node_id(),
            Self::ObjectExpression(it) => it.node_id(),
            Self::ParenthesizedExpression(it) => it.node_id(),
            Self::SequenceExpression(it) => it.node_id(),
            Self::TaggedTemplateExpression(it) => it.node_id(),
            Self::ThisExpression(it) => it.node_id(),
            Self::UnaryExpression(it) => it.node_id(),
            Self::UpdateExpression(it) => it.node_id(),
            Self::YieldExpression(it) => it.node_id(),
            Self::PrivateInExpression(it) => it.node_id(),
            Self::JSXElement(it) => it.node_id(),
            Self::JSXFragment(it) => it.node_id(),
            Self::TSAsExpression(it) => it.node_id(),
            Self::TSSatisfiesExpression(it) => it.node_id(),
            Self::TSTypeAssertion(it) => it.node_id(),
            Self::TSNonNullExpression(it) => it.node_id(),
            Self::TSInstantiationExpression(it) => it.node_id(),
            Self::V8IntrinsicExpression(it) => it.node_id(),
            Self::ComputedMemberExpression(it) => it.node_id(),
            Self::StaticMemberExpression(it) => it.node_id(),
            Self::PrivateFieldExpression(it) => it.node_id(),
        }
    }
}

impl JSXAttributeItem<'_> {
    /// Get [`NodeId`] of [`JSXAttributeItem`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::Attribute(it) => it.node_id(),
            Self::SpreadAttribute(it) => it.node_id(),
        }
    }
}

impl JSXAttributeName<'_> {
    /// Get [`NodeId`] of [`JSXAttributeName`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::Identifier(it) => it.node_id(),
            Self::NamespacedName(it) => it.node_id(),
        }
    }
}

impl JSXAttributeValue<'_> {
    /// Get [`NodeId`] of [`JSXAttributeValue`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::StringLiteral(it) => it.node_id(),
            Self::ExpressionContainer(it) => it.node_id(),
            Self::Element(it) => it.node_id(),
            Self::Fragment(it) => it.node_id(),
        }
    }
}

impl JSXChild<'_> {
    /// Get [`NodeId`] of [`JSXChild`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::Text(it) => it.node_id(),
            Self::Element(it) => it.node_id(),
            Self::Fragment(it) => it.node_id(),
            Self::ExpressionContainer(it) => it.node_id(),
            Self::Spread(it) => it.node_id(),
        }
    }
}

impl TSEnumMemberName<'_> {
    /// Get [`NodeId`] of [`TSEnumMemberName`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::Identifier(it) => it.node_id(),
            Self::String(it) => it.node_id(),
            Self::ComputedString(it) => it.node_id(),
            Self::ComputedTemplateString(it) => it.node_id(),
        }
    }
}

impl TSLiteral<'_> {
    /// Get [`NodeId`] of [`TSLiteral`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::BooleanLiteral(it) => it.node_id(),
            Self::NumericLiteral(it) => it.node_id(),
            Self::BigIntLiteral(it) => it.node_id(),
            Self::StringLiteral(it) => it.node_id(),
            Self::TemplateLiteral(it) => it.node_id(),
            Self::UnaryExpression(it) => it.node_id(),
        }
    }
}

impl TSType<'_> {
    /// Get [`NodeId`] of [`TSType`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::TSAnyKeyword(it) => it.node_id(),
            Self::TSBigIntKeyword(it) => it.node_id(),
            Self::TSBooleanKeyword(it) => it.node_id(),
            Self::TSIntrinsicKeyword(it) => it.node_id(),
            Self::TSNeverKeyword(it) => it.node_id(),
            Self::TSNullKeyword(it) => it.node_id(),
            Self::TSNumberKeyword(it) => it.node_id(),
            Self::TSObjectKeyword(it) => it.node_id(),
            Self::TSStringKeyword(it) => it.node_id(),
            Self::TSSymbolKeyword(it) => it.node_id(),
            Self::TSUndefinedKeyword(it) => it.node_id(),
            Self::TSUnknownKeyword(it) => it.node_id(),
            Self::TSVoidKeyword(it) => it.node_id(),
            Self::TSArrayType(it) => it.node_id(),
            Self::TSConditionalType(it) => it.node_id(),
            Self::TSConstructorType(it) => it.node_id(),
            Self::TSFunctionType(it) => it.node_id(),
            Self::TSImportType(it) => it.node_id(),
            Self::TSIndexedAccessType(it) => it.node_id(),
            Self::TSInferType(it) => it.node_id(),
            Self::TSIntersectionType(it) => it.node_id(),
            Self::TSLiteralType(it) => it.node_id(),
            Self::TSMappedType(it) => it.node_id(),
            Self::TSNamedTupleMember(it) => it.node_id(),
            Self::TSTemplateLiteralType(it) => it.node_id(),
            Self::TSThisType(it) => it.node_id(),
            Self::TSTupleType(it) => it.node_id(),
            Self::TSTypeLiteral(it) => it.node_id(),
            Self::TSTypeOperatorType(it) => it.node_id(),
            Self::TSTypePredicate(it) => it.node_id(),
            Self::TSTypeQuery(it) => it.node_id(),
            Self::TSTypeReference(it) => it.node_id(),
            Self::TSUnionType(it) => it.node_id(),
            Self::TSParenthesizedType(it) => it.node_id(),
            Self::JSDocNullableType(it) => it.node_id(),
            Self::JSDocNonNullableType(it) => it.node_id(),
            Self::JSDocUnknownType(it) => it.node_id(),
        }
    }
}

impl TSTupleElement<'_> {
    /// Get [`NodeId`] of [`TSTupleElement`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::TSOptionalType(it) => it.node_id(),
            Self::TSRestType(it) => it.node_id(),
            Self::TSAnyKeyword(it) => it.node_id(),
            Self::TSBigIntKeyword(it) => it.node_id(),
            Self::TSBooleanKeyword(it) => it.node_id(),
            Self::TSIntrinsicKeyword(it) => it.node_id(),
            Self::TSNeverKeyword(it) => it.node_id(),
            Self::TSNullKeyword(it) => it.node_id(),
            Self::TSNumberKeyword(it) => it.node_id(),
            Self::TSObjectKeyword(it) => it.node_id(),
            Self::TSStringKeyword(it) => it.node_id(),
            Self::TSSymbolKeyword(it) => it.node_id(),
            Self::TSUndefinedKeyword(it) => it.node_id(),
            Self::TSUnknownKeyword(it) => it.node_id(),
            Self::TSVoidKeyword(it) => it.node_id(),
            Self::TSArrayType(it) => it.node_id(),
            Self::TSConditionalType(it) => it.node_id(),
            Self::TSConstructorType(it) => it.node_id(),
            Self::TSFunctionType(it) => it.node_id(),
            Self::TSImportType(it) => it.node_id(),
            Self::TSIndexedAccessType(it) => it.node_id(),
            Self::TSInferType(it) => it.node_id(),
            Self::TSIntersectionType(it) => it.node_id(),
            Self::TSLiteralType(it) => it.node_id(),
            Self::TSMappedType(it) => it.node_id(),
            Self::TSNamedTupleMember(it) => it.node_id(),
            Self::TSTemplateLiteralType(it) => it.node_id(),
            Self::TSThisType(it) => it.node_id(),
            Self::TSTupleType(it) => it.node_id(),
            Self::TSTypeLiteral(it) => it.node_id(),
            Self::TSTypeOperatorType(it) => it.node_id(),
            Self::TSTypePredicate(it) => it.node_id(),
            Self::TSTypeQuery(it) => it.node_id(),
            Self::TSTypeReference(it) => it.node_id(),
            Self::TSUnionType(it) => it.node_id(),
            Self::TSParenthesizedType(it) => it.node_id(),
            Self::JSDocNullableType(it) => it.node_id(),
            Self::JSDocNonNullableType(it) => it.node_id(),
            Self::JSDocUnknownType(it) => it.node_id(),
        }
    }
}

impl TSTypeName<'_> {
    /// Get [`NodeId`] of [`TSTypeName`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::IdentifierReference(it) => it.node_id(),
            Self::QualifiedName(it) => it.node_id(),
            Self::ThisExpression(it) => it.node_id(),
        }
    }
}

impl TSSignature<'_> {
    /// Get [`NodeId`] of [`TSSignature`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::TSIndexSignature(it) => it.node_id(),
            Self::TSPropertySignature(it) => it.node_id(),
            Self::TSCallSignatureDeclaration(it) => it.node_id(),
            Self::TSConstructSignatureDeclaration(it) => it.node_id(),
            Self::TSMethodSignature(it) => it.node_id(),
        }
    }
}

impl TSTypePredicateName<'_> {
    /// Get [`NodeId`] of [`TSTypePredicateName`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::Identifier(it) => it.node_id(),
            Self::This(it) => it.node_id(),
        }
    }
}

impl TSModuleDeclarationName<'_> {
    /// Get [`NodeId`] of [`TSModuleDeclarationName`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::Identifier(it) => it.node_id(),
            Self::StringLiteral(it) => it.node_id(),
        }
    }
}

impl TSModuleDeclarationBody<'_> {
    /// Get [`NodeId`] of [`TSModuleDeclarationBody`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::TSModuleDeclaration(it) => it.node_id(),
            Self::TSModuleBlock(it) => it.node_id(),
        }
    }
}

impl TSTypeQueryExprName<'_> {
    /// Get [`NodeId`] of [`TSTypeQueryExprName`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::TSImportType(it) => it.node_id(),
            Self::IdentifierReference(it) => it.node_id(),
            Self::QualifiedName(it) => it.node_id(),
            Self::ThisExpression(it) => it.node_id(),
        }
    }
}

impl TSImportTypeQualifier<'_> {
    /// Get [`NodeId`] of [`TSImportTypeQualifier`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::Identifier(it) => it.node_id(),
            Self::QualifiedName(it) => it.node_id(),
        }
    }
}

impl TSModuleReference<'_> {
    /// Get [`NodeId`] of [`TSModuleReference`].
    // `#[inline(always)]` because this should boil down to a single instruction.
    #[inline(always)]
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::ExternalModuleReference(it) => it.node_id(),
            Self::IdentifierReference(it) => it.node_id(),
            Self::QualifiedName(it) => it.node_id(),
        }
    }
}
