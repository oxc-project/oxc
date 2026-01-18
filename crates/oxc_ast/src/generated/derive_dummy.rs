// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/dummy.rs`.

#![allow(unused_variables, clippy::inline_always)]

use oxc_allocator::{Allocator, Dummy};

use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
use crate::ast::ts::*;

impl<'a> Dummy<'a> for Program<'a> {
    /// Create a dummy [`Program`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            source_type: Dummy::dummy(allocator),
            source_text: Dummy::dummy(allocator),
            comments: Dummy::dummy(allocator),
            hashbang: Dummy::dummy(allocator),
            directives: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for Expression<'a> {
    /// Create a dummy [`Expression`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::NullLiteral(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for IdentifierName<'a> {
    /// Create a dummy [`IdentifierName`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            name: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for IdentifierReference<'a> {
    /// Create a dummy [`IdentifierReference`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            name: Dummy::dummy(allocator),
            reference_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for BindingIdentifier<'a> {
    /// Create a dummy [`BindingIdentifier`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            name: Dummy::dummy(allocator),
            symbol_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for LabelIdentifier<'a> {
    /// Create a dummy [`LabelIdentifier`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            name: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ThisExpression {
    /// Create a dummy [`ThisExpression`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for ArrayExpression<'a> {
    /// Create a dummy [`ArrayExpression`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            elements: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ArrayExpressionElement<'a> {
    /// Create a dummy [`ArrayExpressionElement`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Elision(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for Elision {
    /// Create a dummy [`Elision`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for ObjectExpression<'a> {
    /// Create a dummy [`ObjectExpression`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            properties: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ObjectPropertyKind<'a> {
    /// Create a dummy [`ObjectPropertyKind`].
    ///
    /// Has cost of making 2 allocations (48 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::SpreadProperty(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for ObjectProperty<'a> {
    /// Create a dummy [`ObjectProperty`].
    ///
    /// Has cost of making 2 allocations (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            kind: Dummy::dummy(allocator),
            key: Dummy::dummy(allocator),
            value: Dummy::dummy(allocator),
            method: Dummy::dummy(allocator),
            shorthand: Dummy::dummy(allocator),
            computed: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for PropertyKey<'a> {
    /// Create a dummy [`PropertyKey`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::NullLiteral(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for PropertyKind {
    /// Create a dummy [`PropertyKind`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Init
    }
}

impl<'a> Dummy<'a> for TemplateLiteral<'a> {
    /// Create a dummy [`TemplateLiteral`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            quasis: Dummy::dummy(allocator),
            expressions: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TaggedTemplateExpression<'a> {
    /// Create a dummy [`TaggedTemplateExpression`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            tag: Dummy::dummy(allocator),
            type_arguments: Dummy::dummy(allocator),
            quasi: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TemplateElement<'a> {
    /// Create a dummy [`TemplateElement`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            value: Dummy::dummy(allocator),
            tail: Dummy::dummy(allocator),
            lone_surrogates: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TemplateElementValue<'a> {
    /// Create a dummy [`TemplateElementValue`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { raw: Dummy::dummy(allocator), cooked: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for MemberExpression<'a> {
    /// Create a dummy [`MemberExpression`].
    ///
    /// Has cost of making 2 allocations (80 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::StaticMemberExpression(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for ComputedMemberExpression<'a> {
    /// Create a dummy [`ComputedMemberExpression`].
    ///
    /// Has cost of making 2 allocations (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            object: Dummy::dummy(allocator),
            expression: Dummy::dummy(allocator),
            optional: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for StaticMemberExpression<'a> {
    /// Create a dummy [`StaticMemberExpression`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            object: Dummy::dummy(allocator),
            property: Dummy::dummy(allocator),
            optional: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for PrivateFieldExpression<'a> {
    /// Create a dummy [`PrivateFieldExpression`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            object: Dummy::dummy(allocator),
            field: Dummy::dummy(allocator),
            optional: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for CallExpression<'a> {
    /// Create a dummy [`CallExpression`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            callee: Dummy::dummy(allocator),
            type_arguments: Dummy::dummy(allocator),
            arguments: Dummy::dummy(allocator),
            optional: Dummy::dummy(allocator),
            pure: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for NewExpression<'a> {
    /// Create a dummy [`NewExpression`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            callee: Dummy::dummy(allocator),
            type_arguments: Dummy::dummy(allocator),
            arguments: Dummy::dummy(allocator),
            pure: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for MetaProperty<'a> {
    /// Create a dummy [`MetaProperty`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            meta: Dummy::dummy(allocator),
            property: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for SpreadElement<'a> {
    /// Create a dummy [`SpreadElement`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            argument: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for Argument<'a> {
    /// Create a dummy [`Argument`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::NullLiteral(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for UpdateExpression<'a> {
    /// Create a dummy [`UpdateExpression`].
    ///
    /// Has cost of making 1 allocation (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            operator: Dummy::dummy(allocator),
            prefix: Dummy::dummy(allocator),
            argument: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for UnaryExpression<'a> {
    /// Create a dummy [`UnaryExpression`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            operator: Dummy::dummy(allocator),
            argument: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for BinaryExpression<'a> {
    /// Create a dummy [`BinaryExpression`].
    ///
    /// Has cost of making 2 allocations (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            left: Dummy::dummy(allocator),
            operator: Dummy::dummy(allocator),
            right: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for PrivateInExpression<'a> {
    /// Create a dummy [`PrivateInExpression`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            left: Dummy::dummy(allocator),
            right: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for LogicalExpression<'a> {
    /// Create a dummy [`LogicalExpression`].
    ///
    /// Has cost of making 2 allocations (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            left: Dummy::dummy(allocator),
            operator: Dummy::dummy(allocator),
            right: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ConditionalExpression<'a> {
    /// Create a dummy [`ConditionalExpression`].
    ///
    /// Has cost of making 3 allocations (48 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            test: Dummy::dummy(allocator),
            consequent: Dummy::dummy(allocator),
            alternate: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for AssignmentExpression<'a> {
    /// Create a dummy [`AssignmentExpression`].
    ///
    /// Has cost of making 2 allocations (48 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            operator: Dummy::dummy(allocator),
            left: Dummy::dummy(allocator),
            right: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for AssignmentTarget<'a> {
    /// Create a dummy [`AssignmentTarget`].
    ///
    /// Has cost of making 1 allocation (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::AssignmentTargetIdentifier(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for SimpleAssignmentTarget<'a> {
    /// Create a dummy [`SimpleAssignmentTarget`].
    ///
    /// Has cost of making 1 allocation (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::AssignmentTargetIdentifier(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for AssignmentTargetPattern<'a> {
    /// Create a dummy [`AssignmentTargetPattern`].
    ///
    /// Has cost of making 1 allocation (48 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::ArrayAssignmentTarget(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for ArrayAssignmentTarget<'a> {
    /// Create a dummy [`ArrayAssignmentTarget`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            elements: Dummy::dummy(allocator),
            rest: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ObjectAssignmentTarget<'a> {
    /// Create a dummy [`ObjectAssignmentTarget`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            properties: Dummy::dummy(allocator),
            rest: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for AssignmentTargetRest<'a> {
    /// Create a dummy [`AssignmentTargetRest`].
    ///
    /// Has cost of making 1 allocation (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            target: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for AssignmentTargetMaybeDefault<'a> {
    /// Create a dummy [`AssignmentTargetMaybeDefault`].
    ///
    /// Has cost of making 1 allocation (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::AssignmentTargetIdentifier(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for AssignmentTargetWithDefault<'a> {
    /// Create a dummy [`AssignmentTargetWithDefault`].
    ///
    /// Has cost of making 2 allocations (48 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            binding: Dummy::dummy(allocator),
            init: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for AssignmentTargetProperty<'a> {
    /// Create a dummy [`AssignmentTargetProperty`].
    ///
    /// Has cost of making 1 allocation (64 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::AssignmentTargetPropertyIdentifier(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for AssignmentTargetPropertyIdentifier<'a> {
    /// Create a dummy [`AssignmentTargetPropertyIdentifier`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            binding: Dummy::dummy(allocator),
            init: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for AssignmentTargetPropertyProperty<'a> {
    /// Create a dummy [`AssignmentTargetPropertyProperty`].
    ///
    /// Has cost of making 2 allocations (48 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            name: Dummy::dummy(allocator),
            binding: Dummy::dummy(allocator),
            computed: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for SequenceExpression<'a> {
    /// Create a dummy [`SequenceExpression`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            expressions: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for Super {
    /// Create a dummy [`Super`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for AwaitExpression<'a> {
    /// Create a dummy [`AwaitExpression`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            argument: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ChainExpression<'a> {
    /// Create a dummy [`ChainExpression`].
    ///
    /// Has cost of making 2 allocations (48 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            expression: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ChainElement<'a> {
    /// Create a dummy [`ChainElement`].
    ///
    /// Has cost of making 2 allocations (48 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::TSNonNullExpression(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for ParenthesizedExpression<'a> {
    /// Create a dummy [`ParenthesizedExpression`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            expression: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for Statement<'a> {
    /// Create a dummy [`Statement`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::DebuggerStatement(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for Directive<'a> {
    /// Create a dummy [`Directive`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            expression: Dummy::dummy(allocator),
            directive: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for Hashbang<'a> {
    /// Create a dummy [`Hashbang`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            value: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for BlockStatement<'a> {
    /// Create a dummy [`BlockStatement`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for Declaration<'a> {
    /// Create a dummy [`Declaration`].
    ///
    /// Has cost of making 1 allocation (40 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::VariableDeclaration(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for VariableDeclaration<'a> {
    /// Create a dummy [`VariableDeclaration`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            kind: Dummy::dummy(allocator),
            declarations: Dummy::dummy(allocator),
            declare: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for VariableDeclarationKind {
    /// Create a dummy [`VariableDeclarationKind`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Var
    }
}

impl<'a> Dummy<'a> for VariableDeclarator<'a> {
    /// Create a dummy [`VariableDeclarator`].
    ///
    /// Has cost of making 1 allocation (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            kind: Dummy::dummy(allocator),
            id: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
            init: Dummy::dummy(allocator),
            definite: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for EmptyStatement {
    /// Create a dummy [`EmptyStatement`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for ExpressionStatement<'a> {
    /// Create a dummy [`ExpressionStatement`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            expression: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for IfStatement<'a> {
    /// Create a dummy [`IfStatement`].
    ///
    /// Has cost of making 2 allocations (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            test: Dummy::dummy(allocator),
            consequent: Dummy::dummy(allocator),
            alternate: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for DoWhileStatement<'a> {
    /// Create a dummy [`DoWhileStatement`].
    ///
    /// Has cost of making 2 allocations (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
            test: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for WhileStatement<'a> {
    /// Create a dummy [`WhileStatement`].
    ///
    /// Has cost of making 2 allocations (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            test: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ForStatement<'a> {
    /// Create a dummy [`ForStatement`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            init: Dummy::dummy(allocator),
            test: Dummy::dummy(allocator),
            update: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ForStatementInit<'a> {
    /// Create a dummy [`ForStatementInit`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::NullLiteral(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for ForInStatement<'a> {
    /// Create a dummy [`ForInStatement`].
    ///
    /// Has cost of making 3 allocations (64 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            left: Dummy::dummy(allocator),
            right: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ForStatementLeft<'a> {
    /// Create a dummy [`ForStatementLeft`].
    ///
    /// Has cost of making 1 allocation (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::AssignmentTargetIdentifier(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for ForOfStatement<'a> {
    /// Create a dummy [`ForOfStatement`].
    ///
    /// Has cost of making 3 allocations (64 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            r#await: Dummy::dummy(allocator),
            left: Dummy::dummy(allocator),
            right: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ContinueStatement<'a> {
    /// Create a dummy [`ContinueStatement`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            label: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for BreakStatement<'a> {
    /// Create a dummy [`BreakStatement`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            label: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ReturnStatement<'a> {
    /// Create a dummy [`ReturnStatement`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            argument: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for WithStatement<'a> {
    /// Create a dummy [`WithStatement`].
    ///
    /// Has cost of making 2 allocations (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            object: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for SwitchStatement<'a> {
    /// Create a dummy [`SwitchStatement`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            discriminant: Dummy::dummy(allocator),
            cases: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for SwitchCase<'a> {
    /// Create a dummy [`SwitchCase`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            test: Dummy::dummy(allocator),
            consequent: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for LabeledStatement<'a> {
    /// Create a dummy [`LabeledStatement`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            label: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ThrowStatement<'a> {
    /// Create a dummy [`ThrowStatement`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            argument: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TryStatement<'a> {
    /// Create a dummy [`TryStatement`].
    ///
    /// Has cost of making 1 allocation (40 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            block: Dummy::dummy(allocator),
            handler: Dummy::dummy(allocator),
            finalizer: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for CatchClause<'a> {
    /// Create a dummy [`CatchClause`].
    ///
    /// Has cost of making 1 allocation (40 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            param: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for CatchParameter<'a> {
    /// Create a dummy [`CatchParameter`].
    ///
    /// Has cost of making 1 allocation (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            pattern: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for DebuggerStatement {
    /// Create a dummy [`DebuggerStatement`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for BindingPattern<'a> {
    /// Create a dummy [`BindingPattern`].
    ///
    /// Has cost of making 1 allocation (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::BindingIdentifier(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for AssignmentPattern<'a> {
    /// Create a dummy [`AssignmentPattern`].
    ///
    /// Has cost of making 2 allocations (48 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            left: Dummy::dummy(allocator),
            right: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ObjectPattern<'a> {
    /// Create a dummy [`ObjectPattern`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            properties: Dummy::dummy(allocator),
            rest: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for BindingProperty<'a> {
    /// Create a dummy [`BindingProperty`].
    ///
    /// Has cost of making 2 allocations (48 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            key: Dummy::dummy(allocator),
            value: Dummy::dummy(allocator),
            shorthand: Dummy::dummy(allocator),
            computed: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ArrayPattern<'a> {
    /// Create a dummy [`ArrayPattern`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            elements: Dummy::dummy(allocator),
            rest: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for BindingRestElement<'a> {
    /// Create a dummy [`BindingRestElement`].
    ///
    /// Has cost of making 1 allocation (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            argument: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for Function<'a> {
    /// Create a dummy [`Function`].
    ///
    /// Has cost of making 1 allocation (48 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            r#type: Dummy::dummy(allocator),
            id: Dummy::dummy(allocator),
            generator: Dummy::dummy(allocator),
            r#async: Dummy::dummy(allocator),
            declare: Dummy::dummy(allocator),
            type_parameters: Dummy::dummy(allocator),
            this_param: Dummy::dummy(allocator),
            params: Dummy::dummy(allocator),
            return_type: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
            pure: Dummy::dummy(allocator),
            pife: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for FunctionType {
    /// Create a dummy [`FunctionType`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::FunctionDeclaration
    }
}

impl<'a> Dummy<'a> for FormalParameters<'a> {
    /// Create a dummy [`FormalParameters`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            kind: Dummy::dummy(allocator),
            items: Dummy::dummy(allocator),
            rest: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for FormalParameter<'a> {
    /// Create a dummy [`FormalParameter`].
    ///
    /// Has cost of making 1 allocation (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            decorators: Dummy::dummy(allocator),
            pattern: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
            initializer: Dummy::dummy(allocator),
            optional: Dummy::dummy(allocator),
            accessibility: Dummy::dummy(allocator),
            readonly: Dummy::dummy(allocator),
            r#override: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for FormalParameterKind {
    /// Create a dummy [`FormalParameterKind`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::FormalParameter
    }
}

impl<'a> Dummy<'a> for FormalParameterRest<'a> {
    /// Create a dummy [`FormalParameterRest`].
    ///
    /// Has cost of making 1 allocation (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            rest: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for FunctionBody<'a> {
    /// Create a dummy [`FunctionBody`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            directives: Dummy::dummy(allocator),
            statements: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ArrowFunctionExpression<'a> {
    /// Create a dummy [`ArrowFunctionExpression`].
    ///
    /// Has cost of making 2 allocations (112 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            expression: Dummy::dummy(allocator),
            r#async: Dummy::dummy(allocator),
            type_parameters: Dummy::dummy(allocator),
            params: Dummy::dummy(allocator),
            return_type: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
            pure: Dummy::dummy(allocator),
            pife: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for YieldExpression<'a> {
    /// Create a dummy [`YieldExpression`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            delegate: Dummy::dummy(allocator),
            argument: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for Class<'a> {
    /// Create a dummy [`Class`].
    ///
    /// Has cost of making 1 allocation (40 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            r#type: Dummy::dummy(allocator),
            decorators: Dummy::dummy(allocator),
            id: Dummy::dummy(allocator),
            type_parameters: Dummy::dummy(allocator),
            super_class: Dummy::dummy(allocator),
            super_type_arguments: Dummy::dummy(allocator),
            implements: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
            r#abstract: Dummy::dummy(allocator),
            declare: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ClassType {
    /// Create a dummy [`ClassType`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::ClassDeclaration
    }
}

impl<'a> Dummy<'a> for ClassBody<'a> {
    /// Create a dummy [`ClassBody`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ClassElement<'a> {
    /// Create a dummy [`ClassElement`].
    ///
    /// Has cost of making 1 allocation (40 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::StaticBlock(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for MethodDefinition<'a> {
    /// Create a dummy [`MethodDefinition`].
    ///
    /// Has cost of making 3 allocations (160 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            r#type: Dummy::dummy(allocator),
            decorators: Dummy::dummy(allocator),
            key: Dummy::dummy(allocator),
            value: Dummy::dummy(allocator),
            kind: Dummy::dummy(allocator),
            computed: Dummy::dummy(allocator),
            r#static: Dummy::dummy(allocator),
            r#override: Dummy::dummy(allocator),
            optional: Dummy::dummy(allocator),
            accessibility: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for MethodDefinitionType {
    /// Create a dummy [`MethodDefinitionType`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::MethodDefinition
    }
}

impl<'a> Dummy<'a> for PropertyDefinition<'a> {
    /// Create a dummy [`PropertyDefinition`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            r#type: Dummy::dummy(allocator),
            decorators: Dummy::dummy(allocator),
            key: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
            value: Dummy::dummy(allocator),
            computed: Dummy::dummy(allocator),
            r#static: Dummy::dummy(allocator),
            declare: Dummy::dummy(allocator),
            r#override: Dummy::dummy(allocator),
            optional: Dummy::dummy(allocator),
            definite: Dummy::dummy(allocator),
            readonly: Dummy::dummy(allocator),
            accessibility: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for PropertyDefinitionType {
    /// Create a dummy [`PropertyDefinitionType`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::PropertyDefinition
    }
}

impl<'a> Dummy<'a> for MethodDefinitionKind {
    /// Create a dummy [`MethodDefinitionKind`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Constructor
    }
}

impl<'a> Dummy<'a> for PrivateIdentifier<'a> {
    /// Create a dummy [`PrivateIdentifier`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            name: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for StaticBlock<'a> {
    /// Create a dummy [`StaticBlock`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ModuleDeclaration<'a> {
    /// Create a dummy [`ModuleDeclaration`].
    ///
    /// Has cost of making 1 allocation (48 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::TSNamespaceExportDeclaration(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for AccessorPropertyType {
    /// Create a dummy [`AccessorPropertyType`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::AccessorProperty
    }
}

impl<'a> Dummy<'a> for AccessorProperty<'a> {
    /// Create a dummy [`AccessorProperty`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            r#type: Dummy::dummy(allocator),
            decorators: Dummy::dummy(allocator),
            key: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
            value: Dummy::dummy(allocator),
            computed: Dummy::dummy(allocator),
            r#static: Dummy::dummy(allocator),
            r#override: Dummy::dummy(allocator),
            definite: Dummy::dummy(allocator),
            accessibility: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ImportExpression<'a> {
    /// Create a dummy [`ImportExpression`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            source: Dummy::dummy(allocator),
            options: Dummy::dummy(allocator),
            phase: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ImportDeclaration<'a> {
    /// Create a dummy [`ImportDeclaration`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            specifiers: Dummy::dummy(allocator),
            source: Dummy::dummy(allocator),
            phase: Dummy::dummy(allocator),
            with_clause: Dummy::dummy(allocator),
            import_kind: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ImportPhase {
    /// Create a dummy [`ImportPhase`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Source
    }
}

impl<'a> Dummy<'a> for ImportDeclarationSpecifier<'a> {
    /// Create a dummy [`ImportDeclarationSpecifier`].
    ///
    /// Has cost of making 1 allocation (48 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::ImportDefaultSpecifier(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for ImportSpecifier<'a> {
    /// Create a dummy [`ImportSpecifier`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            imported: Dummy::dummy(allocator),
            local: Dummy::dummy(allocator),
            import_kind: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ImportDefaultSpecifier<'a> {
    /// Create a dummy [`ImportDefaultSpecifier`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            local: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ImportNamespaceSpecifier<'a> {
    /// Create a dummy [`ImportNamespaceSpecifier`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            local: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for WithClause<'a> {
    /// Create a dummy [`WithClause`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            keyword: Dummy::dummy(allocator),
            with_entries: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for WithClauseKeyword {
    /// Create a dummy [`WithClauseKeyword`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::With
    }
}

impl<'a> Dummy<'a> for ImportAttribute<'a> {
    /// Create a dummy [`ImportAttribute`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            key: Dummy::dummy(allocator),
            value: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ImportAttributeKey<'a> {
    /// Create a dummy [`ImportAttributeKey`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Identifier(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for ExportNamedDeclaration<'a> {
    /// Create a dummy [`ExportNamedDeclaration`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            declaration: Dummy::dummy(allocator),
            specifiers: Dummy::dummy(allocator),
            source: Dummy::dummy(allocator),
            export_kind: Dummy::dummy(allocator),
            with_clause: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ExportDefaultDeclaration<'a> {
    /// Create a dummy [`ExportDefaultDeclaration`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            declaration: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ExportAllDeclaration<'a> {
    /// Create a dummy [`ExportAllDeclaration`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            exported: Dummy::dummy(allocator),
            source: Dummy::dummy(allocator),
            with_clause: Dummy::dummy(allocator),
            export_kind: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ExportSpecifier<'a> {
    /// Create a dummy [`ExportSpecifier`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            local: Dummy::dummy(allocator),
            exported: Dummy::dummy(allocator),
            export_kind: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ExportDefaultDeclarationKind<'a> {
    /// Create a dummy [`ExportDefaultDeclarationKind`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::NullLiteral(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for ModuleExportName<'a> {
    /// Create a dummy [`ModuleExportName`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::IdentifierName(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for V8IntrinsicExpression<'a> {
    /// Create a dummy [`V8IntrinsicExpression`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            name: Dummy::dummy(allocator),
            arguments: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for BooleanLiteral {
    /// Create a dummy [`BooleanLiteral`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            value: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for NullLiteral {
    /// Create a dummy [`NullLiteral`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for NumericLiteral<'a> {
    /// Create a dummy [`NumericLiteral`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            value: Dummy::dummy(allocator),
            raw: Dummy::dummy(allocator),
            base: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for StringLiteral<'a> {
    /// Create a dummy [`StringLiteral`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            value: Dummy::dummy(allocator),
            raw: Dummy::dummy(allocator),
            lone_surrogates: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for BigIntLiteral<'a> {
    /// Create a dummy [`BigIntLiteral`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            value: Dummy::dummy(allocator),
            raw: Dummy::dummy(allocator),
            base: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for RegExpLiteral<'a> {
    /// Create a dummy [`RegExpLiteral`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            regex: Dummy::dummy(allocator),
            raw: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for RegExp<'a> {
    /// Create a dummy [`RegExp`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { pattern: Dummy::dummy(allocator), flags: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for RegExpPattern<'a> {
    /// Create a dummy [`RegExpPattern`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { text: Dummy::dummy(allocator), pattern: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for JSXElement<'a> {
    /// Create a dummy [`JSXElement`].
    ///
    /// Has cost of making 2 allocations (80 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            opening_element: Dummy::dummy(allocator),
            children: Dummy::dummy(allocator),
            closing_element: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for JSXOpeningElement<'a> {
    /// Create a dummy [`JSXOpeningElement`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            name: Dummy::dummy(allocator),
            type_arguments: Dummy::dummy(allocator),
            attributes: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for JSXClosingElement<'a> {
    /// Create a dummy [`JSXClosingElement`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            name: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for JSXFragment<'a> {
    /// Create a dummy [`JSXFragment`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            opening_fragment: Dummy::dummy(allocator),
            children: Dummy::dummy(allocator),
            closing_fragment: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for JSXOpeningFragment {
    /// Create a dummy [`JSXOpeningFragment`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for JSXClosingFragment {
    /// Create a dummy [`JSXClosingFragment`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for JSXElementName<'a> {
    /// Create a dummy [`JSXElementName`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::ThisExpression(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for JSXNamespacedName<'a> {
    /// Create a dummy [`JSXNamespacedName`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            namespace: Dummy::dummy(allocator),
            name: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for JSXMemberExpression<'a> {
    /// Create a dummy [`JSXMemberExpression`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            object: Dummy::dummy(allocator),
            property: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for JSXMemberExpressionObject<'a> {
    /// Create a dummy [`JSXMemberExpressionObject`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::ThisExpression(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for JSXExpressionContainer<'a> {
    /// Create a dummy [`JSXExpressionContainer`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            expression: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for JSXExpression<'a> {
    /// Create a dummy [`JSXExpression`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::EmptyExpression(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for JSXEmptyExpression {
    /// Create a dummy [`JSXEmptyExpression`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for JSXAttributeItem<'a> {
    /// Create a dummy [`JSXAttributeItem`].
    ///
    /// Has cost of making 2 allocations (48 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::SpreadAttribute(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for JSXAttribute<'a> {
    /// Create a dummy [`JSXAttribute`].
    ///
    /// Has cost of making 1 allocation (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            name: Dummy::dummy(allocator),
            value: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for JSXSpreadAttribute<'a> {
    /// Create a dummy [`JSXSpreadAttribute`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            argument: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for JSXAttributeName<'a> {
    /// Create a dummy [`JSXAttributeName`].
    ///
    /// Has cost of making 1 allocation (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Identifier(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for JSXAttributeValue<'a> {
    /// Create a dummy [`JSXAttributeValue`].
    ///
    /// Has cost of making 1 allocation (40 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::ExpressionContainer(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for JSXIdentifier<'a> {
    /// Create a dummy [`JSXIdentifier`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            name: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for JSXChild<'a> {
    /// Create a dummy [`JSXChild`].
    ///
    /// Has cost of making 1 allocation (40 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::ExpressionContainer(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for JSXSpreadChild<'a> {
    /// Create a dummy [`JSXSpreadChild`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            expression: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for JSXText<'a> {
    /// Create a dummy [`JSXText`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            value: Dummy::dummy(allocator),
            raw: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSThisParameter<'a> {
    /// Create a dummy [`TSThisParameter`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            this_span: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSEnumDeclaration<'a> {
    /// Create a dummy [`TSEnumDeclaration`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            id: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
            r#const: Dummy::dummy(allocator),
            declare: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSEnumBody<'a> {
    /// Create a dummy [`TSEnumBody`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            members: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSEnumMember<'a> {
    /// Create a dummy [`TSEnumMember`].
    ///
    /// Has cost of making 1 allocation (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            id: Dummy::dummy(allocator),
            initializer: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSEnumMemberName<'a> {
    /// Create a dummy [`TSEnumMemberName`].
    ///
    /// Has cost of making 1 allocation (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Identifier(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for TSTypeAnnotation<'a> {
    /// Create a dummy [`TSTypeAnnotation`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSLiteralType<'a> {
    /// Create a dummy [`TSLiteralType`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            literal: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSLiteral<'a> {
    /// Create a dummy [`TSLiteral`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::BooleanLiteral(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for TSType<'a> {
    /// Create a dummy [`TSType`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::TSAnyKeyword(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for TSConditionalType<'a> {
    /// Create a dummy [`TSConditionalType`].
    ///
    /// Has cost of making 4 allocations (64 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            check_type: Dummy::dummy(allocator),
            extends_type: Dummy::dummy(allocator),
            true_type: Dummy::dummy(allocator),
            false_type: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSUnionType<'a> {
    /// Create a dummy [`TSUnionType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            types: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSIntersectionType<'a> {
    /// Create a dummy [`TSIntersectionType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            types: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSParenthesizedType<'a> {
    /// Create a dummy [`TSParenthesizedType`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSTypeOperator<'a> {
    /// Create a dummy [`TSTypeOperator`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            operator: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSTypeOperatorOperator {
    /// Create a dummy [`TSTypeOperatorOperator`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Keyof
    }
}

impl<'a> Dummy<'a> for TSArrayType<'a> {
    /// Create a dummy [`TSArrayType`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            element_type: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSIndexedAccessType<'a> {
    /// Create a dummy [`TSIndexedAccessType`].
    ///
    /// Has cost of making 2 allocations (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            object_type: Dummy::dummy(allocator),
            index_type: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSTupleType<'a> {
    /// Create a dummy [`TSTupleType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            element_types: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSNamedTupleMember<'a> {
    /// Create a dummy [`TSNamedTupleMember`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            label: Dummy::dummy(allocator),
            element_type: Dummy::dummy(allocator),
            optional: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSOptionalType<'a> {
    /// Create a dummy [`TSOptionalType`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSRestType<'a> {
    /// Create a dummy [`TSRestType`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSTupleElement<'a> {
    /// Create a dummy [`TSTupleElement`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::TSAnyKeyword(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for TSAnyKeyword {
    /// Create a dummy [`TSAnyKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for TSStringKeyword {
    /// Create a dummy [`TSStringKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for TSBooleanKeyword {
    /// Create a dummy [`TSBooleanKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for TSNumberKeyword {
    /// Create a dummy [`TSNumberKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for TSNeverKeyword {
    /// Create a dummy [`TSNeverKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for TSIntrinsicKeyword {
    /// Create a dummy [`TSIntrinsicKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for TSUnknownKeyword {
    /// Create a dummy [`TSUnknownKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for TSNullKeyword {
    /// Create a dummy [`TSNullKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for TSUndefinedKeyword {
    /// Create a dummy [`TSUndefinedKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for TSVoidKeyword {
    /// Create a dummy [`TSVoidKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for TSSymbolKeyword {
    /// Create a dummy [`TSSymbolKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for TSThisType {
    /// Create a dummy [`TSThisType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for TSObjectKeyword {
    /// Create a dummy [`TSObjectKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for TSBigIntKeyword {
    /// Create a dummy [`TSBigIntKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}

impl<'a> Dummy<'a> for TSTypeReference<'a> {
    /// Create a dummy [`TSTypeReference`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            type_name: Dummy::dummy(allocator),
            type_arguments: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSTypeName<'a> {
    /// Create a dummy [`TSTypeName`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::ThisExpression(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for TSQualifiedName<'a> {
    /// Create a dummy [`TSQualifiedName`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            left: Dummy::dummy(allocator),
            right: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSTypeParameterInstantiation<'a> {
    /// Create a dummy [`TSTypeParameterInstantiation`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            params: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSTypeParameter<'a> {
    /// Create a dummy [`TSTypeParameter`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            name: Dummy::dummy(allocator),
            constraint: Dummy::dummy(allocator),
            default: Dummy::dummy(allocator),
            r#in: Dummy::dummy(allocator),
            out: Dummy::dummy(allocator),
            r#const: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSTypeParameterDeclaration<'a> {
    /// Create a dummy [`TSTypeParameterDeclaration`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            params: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSTypeAliasDeclaration<'a> {
    /// Create a dummy [`TSTypeAliasDeclaration`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            id: Dummy::dummy(allocator),
            type_parameters: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
            declare: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSAccessibility {
    /// Create a dummy [`TSAccessibility`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Private
    }
}

impl<'a> Dummy<'a> for TSClassImplements<'a> {
    /// Create a dummy [`TSClassImplements`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            expression: Dummy::dummy(allocator),
            type_arguments: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSInterfaceDeclaration<'a> {
    /// Create a dummy [`TSInterfaceDeclaration`].
    ///
    /// Has cost of making 1 allocation (40 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            id: Dummy::dummy(allocator),
            type_parameters: Dummy::dummy(allocator),
            extends: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
            declare: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSInterfaceBody<'a> {
    /// Create a dummy [`TSInterfaceBody`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSPropertySignature<'a> {
    /// Create a dummy [`TSPropertySignature`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            computed: Dummy::dummy(allocator),
            optional: Dummy::dummy(allocator),
            readonly: Dummy::dummy(allocator),
            key: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSSignature<'a> {
    /// Create a dummy [`TSSignature`].
    ///
    /// Has cost of making 2 allocations (56 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::TSPropertySignature(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for TSIndexSignature<'a> {
    /// Create a dummy [`TSIndexSignature`].
    ///
    /// Has cost of making 2 allocations (48 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            parameters: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
            readonly: Dummy::dummy(allocator),
            r#static: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSCallSignatureDeclaration<'a> {
    /// Create a dummy [`TSCallSignatureDeclaration`].
    ///
    /// Has cost of making 1 allocation (48 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            type_parameters: Dummy::dummy(allocator),
            this_param: Dummy::dummy(allocator),
            params: Dummy::dummy(allocator),
            return_type: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSMethodSignatureKind {
    /// Create a dummy [`TSMethodSignatureKind`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Method
    }
}

impl<'a> Dummy<'a> for TSMethodSignature<'a> {
    /// Create a dummy [`TSMethodSignature`].
    ///
    /// Has cost of making 2 allocations (64 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            key: Dummy::dummy(allocator),
            computed: Dummy::dummy(allocator),
            optional: Dummy::dummy(allocator),
            kind: Dummy::dummy(allocator),
            type_parameters: Dummy::dummy(allocator),
            this_param: Dummy::dummy(allocator),
            params: Dummy::dummy(allocator),
            return_type: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSConstructSignatureDeclaration<'a> {
    /// Create a dummy [`TSConstructSignatureDeclaration`].
    ///
    /// Has cost of making 1 allocation (48 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            type_parameters: Dummy::dummy(allocator),
            params: Dummy::dummy(allocator),
            return_type: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSIndexSignatureName<'a> {
    /// Create a dummy [`TSIndexSignatureName`].
    ///
    /// Has cost of making 2 allocations (48 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            name: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSInterfaceHeritage<'a> {
    /// Create a dummy [`TSInterfaceHeritage`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            expression: Dummy::dummy(allocator),
            type_arguments: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSTypePredicate<'a> {
    /// Create a dummy [`TSTypePredicate`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            parameter_name: Dummy::dummy(allocator),
            asserts: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSTypePredicateName<'a> {
    /// Create a dummy [`TSTypePredicateName`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::This(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for TSModuleDeclaration<'a> {
    /// Create a dummy [`TSModuleDeclaration`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            id: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
            kind: Dummy::dummy(allocator),
            declare: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSModuleDeclarationKind {
    /// Create a dummy [`TSModuleDeclarationKind`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Module
    }
}

impl<'a> Dummy<'a> for TSModuleDeclarationName<'a> {
    /// Create a dummy [`TSModuleDeclarationName`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Identifier(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for TSModuleDeclarationBody<'a> {
    /// Create a dummy [`TSModuleDeclarationBody`].
    ///
    /// Has cost of making 1 allocation (64 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::TSModuleBlock(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for TSGlobalDeclaration<'a> {
    /// Create a dummy [`TSGlobalDeclaration`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            global_span: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
            declare: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSModuleBlock<'a> {
    /// Create a dummy [`TSModuleBlock`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            directives: Dummy::dummy(allocator),
            body: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSTypeLiteral<'a> {
    /// Create a dummy [`TSTypeLiteral`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            members: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSInferType<'a> {
    /// Create a dummy [`TSInferType`].
    ///
    /// Has cost of making 1 allocation (80 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            type_parameter: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSTypeQuery<'a> {
    /// Create a dummy [`TSTypeQuery`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            expr_name: Dummy::dummy(allocator),
            type_arguments: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSTypeQueryExprName<'a> {
    /// Create a dummy [`TSTypeQueryExprName`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::ThisExpression(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for TSImportType<'a> {
    /// Create a dummy [`TSImportType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            source: Dummy::dummy(allocator),
            options: Dummy::dummy(allocator),
            qualifier: Dummy::dummy(allocator),
            type_arguments: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSImportTypeQualifier<'a> {
    /// Create a dummy [`TSImportTypeQualifier`].
    ///
    /// Has cost of making 1 allocation (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Identifier(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for TSImportTypeQualifiedName<'a> {
    /// Create a dummy [`TSImportTypeQualifiedName`].
    ///
    /// Has cost of making 1 allocation (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            left: Dummy::dummy(allocator),
            right: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSFunctionType<'a> {
    /// Create a dummy [`TSFunctionType`].
    ///
    /// Has cost of making 3 allocations (96 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            type_parameters: Dummy::dummy(allocator),
            this_param: Dummy::dummy(allocator),
            params: Dummy::dummy(allocator),
            return_type: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSConstructorType<'a> {
    /// Create a dummy [`TSConstructorType`].
    ///
    /// Has cost of making 3 allocations (96 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            r#abstract: Dummy::dummy(allocator),
            type_parameters: Dummy::dummy(allocator),
            params: Dummy::dummy(allocator),
            return_type: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSMappedType<'a> {
    /// Create a dummy [`TSMappedType`].
    ///
    /// Has cost of making 1 allocation (80 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            type_parameter: Dummy::dummy(allocator),
            name_type: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
            optional: Dummy::dummy(allocator),
            readonly: Dummy::dummy(allocator),
            scope_id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSMappedTypeModifierOperator {
    /// Create a dummy [`TSMappedTypeModifierOperator`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::True
    }
}

impl<'a> Dummy<'a> for TSTemplateLiteralType<'a> {
    /// Create a dummy [`TSTemplateLiteralType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            quasis: Dummy::dummy(allocator),
            types: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSAsExpression<'a> {
    /// Create a dummy [`TSAsExpression`].
    ///
    /// Has cost of making 2 allocations (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            expression: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSSatisfiesExpression<'a> {
    /// Create a dummy [`TSSatisfiesExpression`].
    ///
    /// Has cost of making 2 allocations (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            expression: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSTypeAssertion<'a> {
    /// Create a dummy [`TSTypeAssertion`].
    ///
    /// Has cost of making 2 allocations (32 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
            expression: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSImportEqualsDeclaration<'a> {
    /// Create a dummy [`TSImportEqualsDeclaration`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            id: Dummy::dummy(allocator),
            module_reference: Dummy::dummy(allocator),
            import_kind: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSModuleReference<'a> {
    /// Create a dummy [`TSModuleReference`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::ThisExpression(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for TSExternalModuleReference<'a> {
    /// Create a dummy [`TSExternalModuleReference`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            expression: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSNonNullExpression<'a> {
    /// Create a dummy [`TSNonNullExpression`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            expression: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for Decorator<'a> {
    /// Create a dummy [`Decorator`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            expression: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSExportAssignment<'a> {
    /// Create a dummy [`TSExportAssignment`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            expression: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSNamespaceExportDeclaration<'a> {
    /// Create a dummy [`TSNamespaceExportDeclaration`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            id: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for TSInstantiationExpression<'a> {
    /// Create a dummy [`TSInstantiationExpression`].
    ///
    /// Has cost of making 2 allocations (56 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            expression: Dummy::dummy(allocator),
            type_arguments: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for ImportOrExportKind {
    /// Create a dummy [`ImportOrExportKind`].
    ///
    /// Does not allocate any data into arena.
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::Value
    }
}

impl<'a> Dummy<'a> for JSDocNullableType<'a> {
    /// Create a dummy [`JSDocNullableType`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
            postfix: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for JSDocNonNullableType<'a> {
    /// Create a dummy [`JSDocNonNullableType`].
    ///
    /// Has cost of making 1 allocation (16 bytes).
    fn dummy(allocator: &'a Allocator) -> Self {
        Self {
            node_id: oxc_syntax::node::NodeId::DUMMY,
            span: Dummy::dummy(allocator),
            type_annotation: Dummy::dummy(allocator),
            postfix: Dummy::dummy(allocator),
        }
    }
}

impl<'a> Dummy<'a> for JSDocUnknownType {
    /// Create a dummy [`JSDocUnknownType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy(allocator: &'a Allocator) -> Self {
        Self { node_id: oxc_syntax::node::NodeId::DUMMY, span: Dummy::dummy(allocator) }
    }
}
