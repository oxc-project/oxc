// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/take_in.rs`

#![allow(unused_imports, unused_variables)]

use std::cell::Cell;

use oxc_allocator::{Allocator, Box, TakeIn, Vec};

use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
use crate::ast::ts::*;

impl<'a> TakeIn<'a> for Program<'a> {
    /// Create a dummy [`Program`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            source_type: TakeIn::dummy_in(allocator),
            source_text: "",
            comments: Vec::new_in(allocator),
            hashbang: None,
            directives: Vec::new_in(allocator),
            body: Vec::new_in(allocator),
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for Expression<'a> {
    /// Create a dummy [`Expression`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::NullLiteral(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for IdentifierName<'a> {
    /// Create a dummy [`IdentifierName`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), name: oxc_span::Atom::from("") }
    }
}

impl<'a> TakeIn<'a> for IdentifierReference<'a> {
    /// Create a dummy [`IdentifierReference`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            name: oxc_span::Atom::from(""),
            reference_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for BindingIdentifier<'a> {
    /// Create a dummy [`BindingIdentifier`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            name: oxc_span::Atom::from(""),
            symbol_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for LabelIdentifier<'a> {
    /// Create a dummy [`LabelIdentifier`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), name: oxc_span::Atom::from("") }
    }
}

impl<'a> TakeIn<'a> for ThisExpression {
    /// Create a dummy [`ThisExpression`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for ArrayExpression<'a> {
    /// Create a dummy [`ArrayExpression`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            elements: Vec::new_in(allocator),
            trailing_comma: None,
        }
    }
}

impl<'a> TakeIn<'a> for ArrayExpressionElement<'a> {
    /// Create a dummy [`ArrayExpressionElement`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Elision(TakeIn::dummy_in(allocator))
    }
}

impl<'a> TakeIn<'a> for Elision {
    /// Create a dummy [`Elision`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for ObjectExpression<'a> {
    /// Create a dummy [`ObjectExpression`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            properties: Vec::new_in(allocator),
            trailing_comma: None,
        }
    }
}

impl<'a> TakeIn<'a> for ObjectPropertyKind<'a> {
    /// Create a dummy [`ObjectPropertyKind`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::SpreadProperty(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for ObjectProperty<'a> {
    /// Create a dummy [`ObjectProperty`].
    ///
    /// Has cost of allocating 16 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            kind: TakeIn::dummy_in(allocator),
            key: TakeIn::dummy_in(allocator),
            value: TakeIn::dummy_in(allocator),
            method: false,
            shorthand: false,
            computed: false,
        }
    }
}

impl<'a> TakeIn<'a> for PropertyKey<'a> {
    /// Create a dummy [`PropertyKey`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::NullLiteral(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for PropertyKind {
    /// Create a dummy [`PropertyKind`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Init
    }
}

impl<'a> TakeIn<'a> for TemplateLiteral<'a> {
    /// Create a dummy [`TemplateLiteral`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            quasis: Vec::new_in(allocator),
            expressions: Vec::new_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for TaggedTemplateExpression<'a> {
    /// Create a dummy [`TaggedTemplateExpression`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            tag: TakeIn::dummy_in(allocator),
            quasi: TakeIn::dummy_in(allocator),
            type_arguments: None,
        }
    }
}

impl<'a> TakeIn<'a> for TemplateElement<'a> {
    /// Create a dummy [`TemplateElement`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), value: TakeIn::dummy_in(allocator), tail: false }
    }
}

impl<'a> TakeIn<'a> for TemplateElementValue<'a> {
    /// Create a dummy [`TemplateElementValue`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { raw: oxc_span::Atom::from(""), cooked: None }
    }
}

impl<'a> TakeIn<'a> for MemberExpression<'a> {
    /// Create a dummy [`MemberExpression`].
    ///
    /// Has cost of allocating 64 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::ComputedMemberExpression(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for ComputedMemberExpression<'a> {
    /// Create a dummy [`ComputedMemberExpression`].
    ///
    /// Has cost of allocating 16 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            object: TakeIn::dummy_in(allocator),
            expression: TakeIn::dummy_in(allocator),
            optional: false,
        }
    }
}

impl<'a> TakeIn<'a> for StaticMemberExpression<'a> {
    /// Create a dummy [`StaticMemberExpression`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            object: TakeIn::dummy_in(allocator),
            property: TakeIn::dummy_in(allocator),
            optional: false,
        }
    }
}

impl<'a> TakeIn<'a> for PrivateFieldExpression<'a> {
    /// Create a dummy [`PrivateFieldExpression`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            object: TakeIn::dummy_in(allocator),
            field: TakeIn::dummy_in(allocator),
            optional: false,
        }
    }
}

impl<'a> TakeIn<'a> for CallExpression<'a> {
    /// Create a dummy [`CallExpression`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            callee: TakeIn::dummy_in(allocator),
            type_arguments: None,
            arguments: Vec::new_in(allocator),
            optional: false,
            pure: false,
        }
    }
}

impl<'a> TakeIn<'a> for NewExpression<'a> {
    /// Create a dummy [`NewExpression`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            callee: TakeIn::dummy_in(allocator),
            arguments: Vec::new_in(allocator),
            type_arguments: None,
            pure: false,
        }
    }
}

impl<'a> TakeIn<'a> for MetaProperty<'a> {
    /// Create a dummy [`MetaProperty`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            meta: TakeIn::dummy_in(allocator),
            property: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for SpreadElement<'a> {
    /// Create a dummy [`SpreadElement`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), argument: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for Argument<'a> {
    /// Create a dummy [`Argument`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::NullLiteral(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for UpdateExpression<'a> {
    /// Create a dummy [`UpdateExpression`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            operator: TakeIn::dummy_in(allocator),
            prefix: false,
            argument: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for UnaryExpression<'a> {
    /// Create a dummy [`UnaryExpression`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            operator: TakeIn::dummy_in(allocator),
            argument: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for BinaryExpression<'a> {
    /// Create a dummy [`BinaryExpression`].
    ///
    /// Has cost of allocating 16 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            left: TakeIn::dummy_in(allocator),
            operator: TakeIn::dummy_in(allocator),
            right: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for PrivateInExpression<'a> {
    /// Create a dummy [`PrivateInExpression`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            left: TakeIn::dummy_in(allocator),
            right: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for LogicalExpression<'a> {
    /// Create a dummy [`LogicalExpression`].
    ///
    /// Has cost of allocating 16 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            left: TakeIn::dummy_in(allocator),
            operator: TakeIn::dummy_in(allocator),
            right: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for ConditionalExpression<'a> {
    /// Create a dummy [`ConditionalExpression`].
    ///
    /// Has cost of allocating 24 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            test: TakeIn::dummy_in(allocator),
            consequent: TakeIn::dummy_in(allocator),
            alternate: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for AssignmentExpression<'a> {
    /// Create a dummy [`AssignmentExpression`].
    ///
    /// Has cost of allocating 40 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            operator: TakeIn::dummy_in(allocator),
            left: TakeIn::dummy_in(allocator),
            right: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for AssignmentTarget<'a> {
    /// Create a dummy [`AssignmentTarget`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::AssignmentTargetIdentifier(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for SimpleAssignmentTarget<'a> {
    /// Create a dummy [`SimpleAssignmentTarget`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::AssignmentTargetIdentifier(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for AssignmentTargetPattern<'a> {
    /// Create a dummy [`AssignmentTargetPattern`].
    ///
    /// Has cost of allocating 64 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::ObjectAssignmentTarget(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for ArrayAssignmentTarget<'a> {
    /// Create a dummy [`ArrayAssignmentTarget`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            elements: Vec::new_in(allocator),
            rest: None,
            trailing_comma: None,
        }
    }
}

impl<'a> TakeIn<'a> for ObjectAssignmentTarget<'a> {
    /// Create a dummy [`ObjectAssignmentTarget`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), properties: Vec::new_in(allocator), rest: None }
    }
}

impl<'a> TakeIn<'a> for AssignmentTargetRest<'a> {
    /// Create a dummy [`AssignmentTargetRest`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), target: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for AssignmentTargetMaybeDefault<'a> {
    /// Create a dummy [`AssignmentTargetMaybeDefault`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::AssignmentTargetIdentifier(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for AssignmentTargetWithDefault<'a> {
    /// Create a dummy [`AssignmentTargetWithDefault`].
    ///
    /// Has cost of allocating 40 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            binding: TakeIn::dummy_in(allocator),
            init: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for AssignmentTargetProperty<'a> {
    /// Create a dummy [`AssignmentTargetProperty`].
    ///
    /// Has cost of allocating 56 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::AssignmentTargetPropertyIdentifier(Box::new_in(
            TakeIn::dummy_in(allocator),
            allocator,
        ))
    }
}

impl<'a> TakeIn<'a> for AssignmentTargetPropertyIdentifier<'a> {
    /// Create a dummy [`AssignmentTargetPropertyIdentifier`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), binding: TakeIn::dummy_in(allocator), init: None }
    }
}

impl<'a> TakeIn<'a> for AssignmentTargetPropertyProperty<'a> {
    /// Create a dummy [`AssignmentTargetPropertyProperty`].
    ///
    /// Has cost of allocating 40 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            name: TakeIn::dummy_in(allocator),
            binding: TakeIn::dummy_in(allocator),
            computed: false,
        }
    }
}

impl<'a> TakeIn<'a> for SequenceExpression<'a> {
    /// Create a dummy [`SequenceExpression`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), expressions: Vec::new_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for Super {
    /// Create a dummy [`Super`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for AwaitExpression<'a> {
    /// Create a dummy [`AwaitExpression`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), argument: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for ChainExpression<'a> {
    /// Create a dummy [`ChainExpression`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), expression: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for ChainElement<'a> {
    /// Create a dummy [`ChainElement`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::TSNonNullExpression(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for ParenthesizedExpression<'a> {
    /// Create a dummy [`ParenthesizedExpression`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), expression: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for Statement<'a> {
    /// Create a dummy [`Statement`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::DebuggerStatement(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for Directive<'a> {
    /// Create a dummy [`Directive`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            expression: TakeIn::dummy_in(allocator),
            directive: oxc_span::Atom::from(""),
        }
    }
}

impl<'a> TakeIn<'a> for Hashbang<'a> {
    /// Create a dummy [`Hashbang`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), value: oxc_span::Atom::from("") }
    }
}

impl<'a> TakeIn<'a> for BlockStatement<'a> {
    /// Create a dummy [`BlockStatement`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            body: Vec::new_in(allocator),
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for Declaration<'a> {
    /// Create a dummy [`Declaration`].
    ///
    /// Has cost of allocating 56 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::VariableDeclaration(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for VariableDeclaration<'a> {
    /// Create a dummy [`VariableDeclaration`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            kind: TakeIn::dummy_in(allocator),
            declarations: Vec::new_in(allocator),
            declare: false,
        }
    }
}

impl<'a> TakeIn<'a> for VariableDeclarationKind {
    /// Create a dummy [`VariableDeclarationKind`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Var
    }
}

impl<'a> TakeIn<'a> for VariableDeclarator<'a> {
    /// Create a dummy [`VariableDeclarator`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            kind: TakeIn::dummy_in(allocator),
            id: TakeIn::dummy_in(allocator),
            init: None,
            definite: false,
        }
    }
}

impl<'a> TakeIn<'a> for EmptyStatement {
    /// Create a dummy [`EmptyStatement`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for ExpressionStatement<'a> {
    /// Create a dummy [`ExpressionStatement`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), expression: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for IfStatement<'a> {
    /// Create a dummy [`IfStatement`].
    ///
    /// Has cost of allocating 16 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            test: TakeIn::dummy_in(allocator),
            consequent: TakeIn::dummy_in(allocator),
            alternate: None,
        }
    }
}

impl<'a> TakeIn<'a> for DoWhileStatement<'a> {
    /// Create a dummy [`DoWhileStatement`].
    ///
    /// Has cost of allocating 16 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            body: TakeIn::dummy_in(allocator),
            test: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for WhileStatement<'a> {
    /// Create a dummy [`WhileStatement`].
    ///
    /// Has cost of allocating 16 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            test: TakeIn::dummy_in(allocator),
            body: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for ForStatement<'a> {
    /// Create a dummy [`ForStatement`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            init: None,
            test: None,
            update: None,
            body: TakeIn::dummy_in(allocator),
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for ForStatementInit<'a> {
    /// Create a dummy [`ForStatementInit`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::NullLiteral(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for ForInStatement<'a> {
    /// Create a dummy [`ForInStatement`].
    ///
    /// Has cost of allocating 48 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            left: TakeIn::dummy_in(allocator),
            right: TakeIn::dummy_in(allocator),
            body: TakeIn::dummy_in(allocator),
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for ForStatementLeft<'a> {
    /// Create a dummy [`ForStatementLeft`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::AssignmentTargetIdentifier(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for ForOfStatement<'a> {
    /// Create a dummy [`ForOfStatement`].
    ///
    /// Has cost of allocating 48 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            r#await: false,
            left: TakeIn::dummy_in(allocator),
            right: TakeIn::dummy_in(allocator),
            body: TakeIn::dummy_in(allocator),
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for ContinueStatement<'a> {
    /// Create a dummy [`ContinueStatement`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), label: None }
    }
}

impl<'a> TakeIn<'a> for BreakStatement<'a> {
    /// Create a dummy [`BreakStatement`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), label: None }
    }
}

impl<'a> TakeIn<'a> for ReturnStatement<'a> {
    /// Create a dummy [`ReturnStatement`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), argument: None }
    }
}

impl<'a> TakeIn<'a> for WithStatement<'a> {
    /// Create a dummy [`WithStatement`].
    ///
    /// Has cost of allocating 16 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            object: TakeIn::dummy_in(allocator),
            body: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for SwitchStatement<'a> {
    /// Create a dummy [`SwitchStatement`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            discriminant: TakeIn::dummy_in(allocator),
            cases: Vec::new_in(allocator),
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for SwitchCase<'a> {
    /// Create a dummy [`SwitchCase`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), test: None, consequent: Vec::new_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for LabeledStatement<'a> {
    /// Create a dummy [`LabeledStatement`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            label: TakeIn::dummy_in(allocator),
            body: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for ThrowStatement<'a> {
    /// Create a dummy [`ThrowStatement`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), argument: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TryStatement<'a> {
    /// Create a dummy [`TryStatement`].
    ///
    /// Has cost of allocating 48 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            block: Box::new_in(TakeIn::dummy_in(allocator), allocator),
            handler: None,
            finalizer: None,
        }
    }
}

impl<'a> TakeIn<'a> for CatchClause<'a> {
    /// Create a dummy [`CatchClause`].
    ///
    /// Has cost of allocating 48 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            param: None,
            body: Box::new_in(TakeIn::dummy_in(allocator), allocator),
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for CatchParameter<'a> {
    /// Create a dummy [`CatchParameter`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), pattern: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for DebuggerStatement {
    /// Create a dummy [`DebuggerStatement`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for BindingPattern<'a> {
    /// Create a dummy [`BindingPattern`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { kind: TakeIn::dummy_in(allocator), type_annotation: None, optional: false }
    }
}

impl<'a> TakeIn<'a> for BindingPatternKind<'a> {
    /// Create a dummy [`BindingPatternKind`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::BindingIdentifier(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for AssignmentPattern<'a> {
    /// Create a dummy [`AssignmentPattern`].
    ///
    /// Has cost of allocating 40 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            left: TakeIn::dummy_in(allocator),
            right: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for ObjectPattern<'a> {
    /// Create a dummy [`ObjectPattern`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), properties: Vec::new_in(allocator), rest: None }
    }
}

impl<'a> TakeIn<'a> for BindingProperty<'a> {
    /// Create a dummy [`BindingProperty`].
    ///
    /// Has cost of allocating 40 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            key: TakeIn::dummy_in(allocator),
            value: TakeIn::dummy_in(allocator),
            shorthand: false,
            computed: false,
        }
    }
}

impl<'a> TakeIn<'a> for ArrayPattern<'a> {
    /// Create a dummy [`ArrayPattern`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), elements: Vec::new_in(allocator), rest: None }
    }
}

impl<'a> TakeIn<'a> for BindingRestElement<'a> {
    /// Create a dummy [`BindingRestElement`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), argument: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for Function<'a> {
    /// Create a dummy [`Function`].
    ///
    /// Has cost of allocating 56 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            r#type: TakeIn::dummy_in(allocator),
            id: None,
            generator: false,
            r#async: false,
            declare: false,
            type_parameters: None,
            this_param: None,
            params: Box::new_in(TakeIn::dummy_in(allocator), allocator),
            return_type: None,
            body: None,
            scope_id: Cell::new(None),
            pure: false,
        }
    }
}

impl<'a> TakeIn<'a> for FunctionType {
    /// Create a dummy [`FunctionType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::FunctionDeclaration
    }
}

impl<'a> TakeIn<'a> for FormalParameters<'a> {
    /// Create a dummy [`FormalParameters`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            kind: TakeIn::dummy_in(allocator),
            items: Vec::new_in(allocator),
            rest: None,
        }
    }
}

impl<'a> TakeIn<'a> for FormalParameter<'a> {
    /// Create a dummy [`FormalParameter`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            decorators: Vec::new_in(allocator),
            pattern: TakeIn::dummy_in(allocator),
            accessibility: None,
            readonly: false,
            r#override: false,
        }
    }
}

impl<'a> TakeIn<'a> for FormalParameterKind {
    /// Create a dummy [`FormalParameterKind`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::FormalParameter
    }
}

impl<'a> TakeIn<'a> for FunctionBody<'a> {
    /// Create a dummy [`FunctionBody`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            directives: Vec::new_in(allocator),
            statements: Vec::new_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for ArrowFunctionExpression<'a> {
    /// Create a dummy [`ArrowFunctionExpression`].
    ///
    /// Has cost of allocating 128 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            expression: false,
            r#async: false,
            type_parameters: None,
            params: Box::new_in(TakeIn::dummy_in(allocator), allocator),
            return_type: None,
            body: Box::new_in(TakeIn::dummy_in(allocator), allocator),
            scope_id: Cell::new(None),
            pure: false,
        }
    }
}

impl<'a> TakeIn<'a> for YieldExpression<'a> {
    /// Create a dummy [`YieldExpression`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), delegate: false, argument: None }
    }
}

impl<'a> TakeIn<'a> for Class<'a> {
    /// Create a dummy [`Class`].
    ///
    /// Has cost of allocating 40 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            r#type: TakeIn::dummy_in(allocator),
            decorators: Vec::new_in(allocator),
            id: None,
            type_parameters: None,
            super_class: None,
            super_type_arguments: None,
            implements: None,
            body: Box::new_in(TakeIn::dummy_in(allocator), allocator),
            r#abstract: false,
            declare: false,
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for ClassType {
    /// Create a dummy [`ClassType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::ClassDeclaration
    }
}

impl<'a> TakeIn<'a> for ClassBody<'a> {
    /// Create a dummy [`ClassBody`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), body: Vec::new_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for ClassElement<'a> {
    /// Create a dummy [`ClassElement`].
    ///
    /// Has cost of allocating 48 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::StaticBlock(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for MethodDefinition<'a> {
    /// Create a dummy [`MethodDefinition`].
    ///
    /// Has cost of allocating 168 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            r#type: TakeIn::dummy_in(allocator),
            decorators: Vec::new_in(allocator),
            key: TakeIn::dummy_in(allocator),
            value: Box::new_in(TakeIn::dummy_in(allocator), allocator),
            kind: TakeIn::dummy_in(allocator),
            computed: false,
            r#static: false,
            r#override: false,
            optional: false,
            accessibility: None,
        }
    }
}

impl<'a> TakeIn<'a> for MethodDefinitionType {
    /// Create a dummy [`MethodDefinitionType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::MethodDefinition
    }
}

impl<'a> TakeIn<'a> for PropertyDefinition<'a> {
    /// Create a dummy [`PropertyDefinition`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            r#type: TakeIn::dummy_in(allocator),
            decorators: Vec::new_in(allocator),
            key: TakeIn::dummy_in(allocator),
            value: None,
            computed: false,
            r#static: false,
            declare: false,
            r#override: false,
            optional: false,
            definite: false,
            readonly: false,
            type_annotation: None,
            accessibility: None,
        }
    }
}

impl<'a> TakeIn<'a> for PropertyDefinitionType {
    /// Create a dummy [`PropertyDefinitionType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::PropertyDefinition
    }
}

impl<'a> TakeIn<'a> for MethodDefinitionKind {
    /// Create a dummy [`MethodDefinitionKind`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Constructor
    }
}

impl<'a> TakeIn<'a> for PrivateIdentifier<'a> {
    /// Create a dummy [`PrivateIdentifier`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), name: oxc_span::Atom::from("") }
    }
}

impl<'a> TakeIn<'a> for StaticBlock<'a> {
    /// Create a dummy [`StaticBlock`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            body: Vec::new_in(allocator),
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for ModuleDeclaration<'a> {
    /// Create a dummy [`ModuleDeclaration`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::TSExportAssignment(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for AccessorPropertyType {
    /// Create a dummy [`AccessorPropertyType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::AccessorProperty
    }
}

impl<'a> TakeIn<'a> for AccessorProperty<'a> {
    /// Create a dummy [`AccessorProperty`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            r#type: TakeIn::dummy_in(allocator),
            decorators: Vec::new_in(allocator),
            key: TakeIn::dummy_in(allocator),
            value: None,
            computed: false,
            r#static: false,
            definite: false,
            type_annotation: None,
            accessibility: None,
        }
    }
}

impl<'a> TakeIn<'a> for ImportExpression<'a> {
    /// Create a dummy [`ImportExpression`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            source: TakeIn::dummy_in(allocator),
            options: Vec::new_in(allocator),
            phase: None,
        }
    }
}

impl<'a> TakeIn<'a> for ImportDeclaration<'a> {
    /// Create a dummy [`ImportDeclaration`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            specifiers: None,
            source: TakeIn::dummy_in(allocator),
            phase: None,
            with_clause: None,
            import_kind: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for ImportPhase {
    /// Create a dummy [`ImportPhase`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Source
    }
}

impl<'a> TakeIn<'a> for ImportDeclarationSpecifier<'a> {
    /// Create a dummy [`ImportDeclarationSpecifier`].
    ///
    /// Has cost of allocating 40 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::ImportDefaultSpecifier(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for ImportSpecifier<'a> {
    /// Create a dummy [`ImportSpecifier`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            imported: TakeIn::dummy_in(allocator),
            local: TakeIn::dummy_in(allocator),
            import_kind: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for ImportDefaultSpecifier<'a> {
    /// Create a dummy [`ImportDefaultSpecifier`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), local: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for ImportNamespaceSpecifier<'a> {
    /// Create a dummy [`ImportNamespaceSpecifier`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), local: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for WithClause<'a> {
    /// Create a dummy [`WithClause`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            attributes_keyword: TakeIn::dummy_in(allocator),
            with_entries: Vec::new_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for ImportAttribute<'a> {
    /// Create a dummy [`ImportAttribute`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            key: TakeIn::dummy_in(allocator),
            value: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for ImportAttributeKey<'a> {
    /// Create a dummy [`ImportAttributeKey`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Identifier(TakeIn::dummy_in(allocator))
    }
}

impl<'a> TakeIn<'a> for ExportNamedDeclaration<'a> {
    /// Create a dummy [`ExportNamedDeclaration`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            declaration: None,
            specifiers: Vec::new_in(allocator),
            source: None,
            export_kind: TakeIn::dummy_in(allocator),
            with_clause: None,
        }
    }
}

impl<'a> TakeIn<'a> for ExportDefaultDeclaration<'a> {
    /// Create a dummy [`ExportDefaultDeclaration`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            exported: TakeIn::dummy_in(allocator),
            declaration: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for ExportAllDeclaration<'a> {
    /// Create a dummy [`ExportAllDeclaration`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            exported: None,
            source: TakeIn::dummy_in(allocator),
            with_clause: None,
            export_kind: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for ExportSpecifier<'a> {
    /// Create a dummy [`ExportSpecifier`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            local: TakeIn::dummy_in(allocator),
            exported: TakeIn::dummy_in(allocator),
            export_kind: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for ExportDefaultDeclarationKind<'a> {
    /// Create a dummy [`ExportDefaultDeclarationKind`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::NullLiteral(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for ModuleExportName<'a> {
    /// Create a dummy [`ModuleExportName`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::IdentifierName(TakeIn::dummy_in(allocator))
    }
}

impl<'a> TakeIn<'a> for V8IntrinsicExpression<'a> {
    /// Create a dummy [`V8IntrinsicExpression`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            name: TakeIn::dummy_in(allocator),
            arguments: Vec::new_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for BooleanLiteral {
    /// Create a dummy [`BooleanLiteral`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), value: false }
    }
}

impl<'a> TakeIn<'a> for NullLiteral {
    /// Create a dummy [`NullLiteral`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for NumericLiteral<'a> {
    /// Create a dummy [`NumericLiteral`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            value: 0.0,
            raw: None,
            base: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for StringLiteral<'a> {
    /// Create a dummy [`StringLiteral`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            value: oxc_span::Atom::from(""),
            raw: None,
            lossy: false,
        }
    }
}

impl<'a> TakeIn<'a> for BigIntLiteral<'a> {
    /// Create a dummy [`BigIntLiteral`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            raw: oxc_span::Atom::from(""),
            base: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for RegExpLiteral<'a> {
    /// Create a dummy [`RegExpLiteral`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), regex: TakeIn::dummy_in(allocator), raw: None }
    }
}

impl<'a> TakeIn<'a> for RegExp<'a> {
    /// Create a dummy [`RegExp`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { pattern: TakeIn::dummy_in(allocator), flags: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for RegExpPattern<'a> {
    /// Create a dummy [`RegExpPattern`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Raw("")
    }
}

impl<'a> TakeIn<'a> for JSXElement<'a> {
    /// Create a dummy [`JSXElement`].
    ///
    /// Has cost of allocating 80 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            opening_element: Box::new_in(TakeIn::dummy_in(allocator), allocator),
            closing_element: None,
            children: Vec::new_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for JSXOpeningElement<'a> {
    /// Create a dummy [`JSXOpeningElement`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            self_closing: false,
            name: TakeIn::dummy_in(allocator),
            attributes: Vec::new_in(allocator),
            type_arguments: None,
        }
    }
}

impl<'a> TakeIn<'a> for JSXClosingElement<'a> {
    /// Create a dummy [`JSXClosingElement`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), name: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for JSXFragment<'a> {
    /// Create a dummy [`JSXFragment`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            opening_fragment: TakeIn::dummy_in(allocator),
            closing_fragment: TakeIn::dummy_in(allocator),
            children: Vec::new_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for JSXOpeningFragment {
    /// Create a dummy [`JSXOpeningFragment`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for JSXClosingFragment {
    /// Create a dummy [`JSXClosingFragment`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for JSXElementName<'a> {
    /// Create a dummy [`JSXElementName`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::ThisExpression(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for JSXNamespacedName<'a> {
    /// Create a dummy [`JSXNamespacedName`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            namespace: TakeIn::dummy_in(allocator),
            name: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for JSXMemberExpression<'a> {
    /// Create a dummy [`JSXMemberExpression`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            object: TakeIn::dummy_in(allocator),
            property: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for JSXMemberExpressionObject<'a> {
    /// Create a dummy [`JSXMemberExpressionObject`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::ThisExpression(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for JSXExpressionContainer<'a> {
    /// Create a dummy [`JSXExpressionContainer`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), expression: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for JSXExpression<'a> {
    /// Create a dummy [`JSXExpression`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::EmptyExpression(TakeIn::dummy_in(allocator))
    }
}

impl<'a> TakeIn<'a> for JSXEmptyExpression {
    /// Create a dummy [`JSXEmptyExpression`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for JSXAttributeItem<'a> {
    /// Create a dummy [`JSXAttributeItem`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::SpreadAttribute(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for JSXAttribute<'a> {
    /// Create a dummy [`JSXAttribute`].
    ///
    /// Has cost of allocating 24 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), name: TakeIn::dummy_in(allocator), value: None }
    }
}

impl<'a> TakeIn<'a> for JSXSpreadAttribute<'a> {
    /// Create a dummy [`JSXSpreadAttribute`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), argument: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for JSXAttributeName<'a> {
    /// Create a dummy [`JSXAttributeName`].
    ///
    /// Has cost of allocating 24 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Identifier(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for JSXAttributeValue<'a> {
    /// Create a dummy [`JSXAttributeValue`].
    ///
    /// Has cost of allocating 24 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::ExpressionContainer(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for JSXIdentifier<'a> {
    /// Create a dummy [`JSXIdentifier`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), name: oxc_span::Atom::from("") }
    }
}

impl<'a> TakeIn<'a> for JSXChild<'a> {
    /// Create a dummy [`JSXChild`].
    ///
    /// Has cost of allocating 24 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::ExpressionContainer(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for JSXSpreadChild<'a> {
    /// Create a dummy [`JSXSpreadChild`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), expression: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for JSXText<'a> {
    /// Create a dummy [`JSXText`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), value: oxc_span::Atom::from(""), raw: None }
    }
}

impl<'a> TakeIn<'a> for TSThisParameter<'a> {
    /// Create a dummy [`TSThisParameter`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            this_span: TakeIn::dummy_in(allocator),
            type_annotation: None,
        }
    }
}

impl<'a> TakeIn<'a> for TSEnumDeclaration<'a> {
    /// Create a dummy [`TSEnumDeclaration`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            id: TakeIn::dummy_in(allocator),
            members: Vec::new_in(allocator),
            r#const: false,
            declare: false,
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for TSEnumMember<'a> {
    /// Create a dummy [`TSEnumMember`].
    ///
    /// Has cost of allocating 24 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            id: TakeIn::dummy_in(allocator),
            initializer: None,
        }
    }
}

impl<'a> TakeIn<'a> for TSEnumMemberName<'a> {
    /// Create a dummy [`TSEnumMemberName`].
    ///
    /// Has cost of allocating 24 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Identifier(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for TSTypeAnnotation<'a> {
    /// Create a dummy [`TSTypeAnnotation`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), type_annotation: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSLiteralType<'a> {
    /// Create a dummy [`TSLiteralType`].
    ///
    /// Has cost of allocating 16 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), literal: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSLiteral<'a> {
    /// Create a dummy [`TSLiteral`].
    ///
    /// Has cost of allocating 16 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::BooleanLiteral(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for TSType<'a> {
    /// Create a dummy [`TSType`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::TSAnyKeyword(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for TSConditionalType<'a> {
    /// Create a dummy [`TSConditionalType`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            check_type: TakeIn::dummy_in(allocator),
            extends_type: TakeIn::dummy_in(allocator),
            true_type: TakeIn::dummy_in(allocator),
            false_type: TakeIn::dummy_in(allocator),
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for TSUnionType<'a> {
    /// Create a dummy [`TSUnionType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), types: Vec::new_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSIntersectionType<'a> {
    /// Create a dummy [`TSIntersectionType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), types: Vec::new_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSParenthesizedType<'a> {
    /// Create a dummy [`TSParenthesizedType`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), type_annotation: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSTypeOperator<'a> {
    /// Create a dummy [`TSTypeOperator`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            operator: TakeIn::dummy_in(allocator),
            type_annotation: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for TSTypeOperatorOperator {
    /// Create a dummy [`TSTypeOperatorOperator`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Keyof
    }
}

impl<'a> TakeIn<'a> for TSArrayType<'a> {
    /// Create a dummy [`TSArrayType`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), element_type: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSIndexedAccessType<'a> {
    /// Create a dummy [`TSIndexedAccessType`].
    ///
    /// Has cost of allocating 16 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            object_type: TakeIn::dummy_in(allocator),
            index_type: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for TSTupleType<'a> {
    /// Create a dummy [`TSTupleType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), element_types: Vec::new_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSNamedTupleMember<'a> {
    /// Create a dummy [`TSNamedTupleMember`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            element_type: TakeIn::dummy_in(allocator),
            label: TakeIn::dummy_in(allocator),
            optional: false,
        }
    }
}

impl<'a> TakeIn<'a> for TSOptionalType<'a> {
    /// Create a dummy [`TSOptionalType`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), type_annotation: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSRestType<'a> {
    /// Create a dummy [`TSRestType`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), type_annotation: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSTupleElement<'a> {
    /// Create a dummy [`TSTupleElement`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::TSAnyKeyword(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for TSAnyKeyword {
    /// Create a dummy [`TSAnyKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSStringKeyword {
    /// Create a dummy [`TSStringKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSBooleanKeyword {
    /// Create a dummy [`TSBooleanKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSNumberKeyword {
    /// Create a dummy [`TSNumberKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSNeverKeyword {
    /// Create a dummy [`TSNeverKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSIntrinsicKeyword {
    /// Create a dummy [`TSIntrinsicKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSUnknownKeyword {
    /// Create a dummy [`TSUnknownKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSNullKeyword {
    /// Create a dummy [`TSNullKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSUndefinedKeyword {
    /// Create a dummy [`TSUndefinedKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSVoidKeyword {
    /// Create a dummy [`TSVoidKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSSymbolKeyword {
    /// Create a dummy [`TSSymbolKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSThisType {
    /// Create a dummy [`TSThisType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSObjectKeyword {
    /// Create a dummy [`TSObjectKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSBigIntKeyword {
    /// Create a dummy [`TSBigIntKeyword`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSTypeReference<'a> {
    /// Create a dummy [`TSTypeReference`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            type_name: TakeIn::dummy_in(allocator),
            type_arguments: None,
        }
    }
}

impl<'a> TakeIn<'a> for TSTypeName<'a> {
    /// Create a dummy [`TSTypeName`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::IdentifierReference(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for TSQualifiedName<'a> {
    /// Create a dummy [`TSQualifiedName`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            left: TakeIn::dummy_in(allocator),
            right: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for TSTypeParameterInstantiation<'a> {
    /// Create a dummy [`TSTypeParameterInstantiation`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), params: Vec::new_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSTypeParameter<'a> {
    /// Create a dummy [`TSTypeParameter`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            name: TakeIn::dummy_in(allocator),
            constraint: None,
            default: None,
            r#in: false,
            out: false,
            r#const: false,
        }
    }
}

impl<'a> TakeIn<'a> for TSTypeParameterDeclaration<'a> {
    /// Create a dummy [`TSTypeParameterDeclaration`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), params: Vec::new_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSTypeAliasDeclaration<'a> {
    /// Create a dummy [`TSTypeAliasDeclaration`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            id: TakeIn::dummy_in(allocator),
            type_parameters: None,
            type_annotation: TakeIn::dummy_in(allocator),
            declare: false,
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for TSAccessibility {
    /// Create a dummy [`TSAccessibility`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Private
    }
}

impl<'a> TakeIn<'a> for TSClassImplements<'a> {
    /// Create a dummy [`TSClassImplements`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            expression: TakeIn::dummy_in(allocator),
            type_arguments: None,
        }
    }
}

impl<'a> TakeIn<'a> for TSInterfaceDeclaration<'a> {
    /// Create a dummy [`TSInterfaceDeclaration`].
    ///
    /// Has cost of allocating 40 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            id: TakeIn::dummy_in(allocator),
            extends: None,
            type_parameters: None,
            body: Box::new_in(TakeIn::dummy_in(allocator), allocator),
            declare: false,
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for TSInterfaceBody<'a> {
    /// Create a dummy [`TSInterfaceBody`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), body: Vec::new_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSPropertySignature<'a> {
    /// Create a dummy [`TSPropertySignature`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            computed: false,
            optional: false,
            readonly: false,
            key: TakeIn::dummy_in(allocator),
            type_annotation: None,
        }
    }
}

impl<'a> TakeIn<'a> for TSSignature<'a> {
    /// Create a dummy [`TSSignature`].
    ///
    /// Has cost of allocating 48 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::TSPropertySignature(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for TSIndexSignature<'a> {
    /// Create a dummy [`TSIndexSignature`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            parameters: Vec::new_in(allocator),
            type_annotation: Box::new_in(TakeIn::dummy_in(allocator), allocator),
            readonly: false,
            r#static: false,
        }
    }
}

impl<'a> TakeIn<'a> for TSCallSignatureDeclaration<'a> {
    /// Create a dummy [`TSCallSignatureDeclaration`].
    ///
    /// Has cost of allocating 56 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            type_parameters: None,
            this_param: None,
            params: Box::new_in(TakeIn::dummy_in(allocator), allocator),
            return_type: None,
        }
    }
}

impl<'a> TakeIn<'a> for TSMethodSignatureKind {
    /// Create a dummy [`TSMethodSignatureKind`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Method
    }
}

impl<'a> TakeIn<'a> for TSMethodSignature<'a> {
    /// Create a dummy [`TSMethodSignature`].
    ///
    /// Has cost of allocating 64 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            key: TakeIn::dummy_in(allocator),
            computed: false,
            optional: false,
            kind: TakeIn::dummy_in(allocator),
            type_parameters: None,
            this_param: None,
            params: Box::new_in(TakeIn::dummy_in(allocator), allocator),
            return_type: None,
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for TSConstructSignatureDeclaration<'a> {
    /// Create a dummy [`TSConstructSignatureDeclaration`].
    ///
    /// Has cost of allocating 56 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            type_parameters: None,
            params: Box::new_in(TakeIn::dummy_in(allocator), allocator),
            return_type: None,
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for TSIndexSignatureName<'a> {
    /// Create a dummy [`TSIndexSignatureName`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            name: oxc_span::Atom::from(""),
            type_annotation: Box::new_in(TakeIn::dummy_in(allocator), allocator),
        }
    }
}

impl<'a> TakeIn<'a> for TSInterfaceHeritage<'a> {
    /// Create a dummy [`TSInterfaceHeritage`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            expression: TakeIn::dummy_in(allocator),
            type_arguments: None,
        }
    }
}

impl<'a> TakeIn<'a> for TSTypePredicate<'a> {
    /// Create a dummy [`TSTypePredicate`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            parameter_name: TakeIn::dummy_in(allocator),
            asserts: false,
            type_annotation: None,
        }
    }
}

impl<'a> TakeIn<'a> for TSTypePredicateName<'a> {
    /// Create a dummy [`TSTypePredicateName`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::This(TakeIn::dummy_in(allocator))
    }
}

impl<'a> TakeIn<'a> for TSModuleDeclaration<'a> {
    /// Create a dummy [`TSModuleDeclaration`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            id: TakeIn::dummy_in(allocator),
            body: None,
            kind: TakeIn::dummy_in(allocator),
            declare: false,
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for TSModuleDeclarationKind {
    /// Create a dummy [`TSModuleDeclarationKind`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Global
    }
}

impl<'a> TakeIn<'a> for TSModuleDeclarationName<'a> {
    /// Create a dummy [`TSModuleDeclarationName`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Identifier(TakeIn::dummy_in(allocator))
    }
}

impl<'a> TakeIn<'a> for TSModuleDeclarationBody<'a> {
    /// Create a dummy [`TSModuleDeclarationBody`].
    ///
    /// Has cost of allocating 72 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::TSModuleBlock(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for TSModuleBlock<'a> {
    /// Create a dummy [`TSModuleBlock`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            directives: Vec::new_in(allocator),
            body: Vec::new_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for TSTypeLiteral<'a> {
    /// Create a dummy [`TSTypeLiteral`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), members: Vec::new_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSInferType<'a> {
    /// Create a dummy [`TSInferType`].
    ///
    /// Has cost of allocating 80 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            type_parameter: Box::new_in(TakeIn::dummy_in(allocator), allocator),
        }
    }
}

impl<'a> TakeIn<'a> for TSTypeQuery<'a> {
    /// Create a dummy [`TSTypeQuery`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            expr_name: TakeIn::dummy_in(allocator),
            type_arguments: None,
        }
    }
}

impl<'a> TakeIn<'a> for TSTypeQueryExprName<'a> {
    /// Create a dummy [`TSTypeQueryExprName`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::IdentifierReference(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for TSImportType<'a> {
    /// Create a dummy [`TSImportType`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            argument: TakeIn::dummy_in(allocator),
            options: None,
            qualifier: None,
            type_arguments: None,
            is_type_of: false,
        }
    }
}

impl<'a> TakeIn<'a> for TSFunctionType<'a> {
    /// Create a dummy [`TSFunctionType`].
    ///
    /// Has cost of allocating 88 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            type_parameters: None,
            this_param: None,
            params: Box::new_in(TakeIn::dummy_in(allocator), allocator),
            return_type: Box::new_in(TakeIn::dummy_in(allocator), allocator),
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for TSConstructorType<'a> {
    /// Create a dummy [`TSConstructorType`].
    ///
    /// Has cost of allocating 88 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            r#abstract: false,
            type_parameters: None,
            params: Box::new_in(TakeIn::dummy_in(allocator), allocator),
            return_type: Box::new_in(TakeIn::dummy_in(allocator), allocator),
        }
    }
}

impl<'a> TakeIn<'a> for TSMappedType<'a> {
    /// Create a dummy [`TSMappedType`].
    ///
    /// Has cost of allocating 80 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            type_parameter: Box::new_in(TakeIn::dummy_in(allocator), allocator),
            name_type: None,
            type_annotation: None,
            optional: TakeIn::dummy_in(allocator),
            readonly: TakeIn::dummy_in(allocator),
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TakeIn<'a> for TSMappedTypeModifierOperator {
    /// Create a dummy [`TSMappedTypeModifierOperator`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::True
    }
}

impl<'a> TakeIn<'a> for TSTemplateLiteralType<'a> {
    /// Create a dummy [`TSTemplateLiteralType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            quasis: Vec::new_in(allocator),
            types: Vec::new_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for TSAsExpression<'a> {
    /// Create a dummy [`TSAsExpression`].
    ///
    /// Has cost of allocating 16 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            expression: TakeIn::dummy_in(allocator),
            type_annotation: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for TSSatisfiesExpression<'a> {
    /// Create a dummy [`TSSatisfiesExpression`].
    ///
    /// Has cost of allocating 16 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            expression: TakeIn::dummy_in(allocator),
            type_annotation: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for TSTypeAssertion<'a> {
    /// Create a dummy [`TSTypeAssertion`].
    ///
    /// Has cost of allocating 16 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            expression: TakeIn::dummy_in(allocator),
            type_annotation: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for TSImportEqualsDeclaration<'a> {
    /// Create a dummy [`TSImportEqualsDeclaration`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            id: TakeIn::dummy_in(allocator),
            module_reference: TakeIn::dummy_in(allocator),
            import_kind: TakeIn::dummy_in(allocator),
        }
    }
}

impl<'a> TakeIn<'a> for TSModuleReference<'a> {
    /// Create a dummy [`TSModuleReference`].
    ///
    /// Has cost of allocating 32 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::IdentifierReference(Box::new_in(TakeIn::dummy_in(allocator), allocator))
    }
}

impl<'a> TakeIn<'a> for TSExternalModuleReference<'a> {
    /// Create a dummy [`TSExternalModuleReference`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), expression: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSNonNullExpression<'a> {
    /// Create a dummy [`TSNonNullExpression`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), expression: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for Decorator<'a> {
    /// Create a dummy [`Decorator`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), expression: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSExportAssignment<'a> {
    /// Create a dummy [`TSExportAssignment`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), expression: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSNamespaceExportDeclaration<'a> {
    /// Create a dummy [`TSNamespaceExportDeclaration`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator), id: TakeIn::dummy_in(allocator) }
    }
}

impl<'a> TakeIn<'a> for TSInstantiationExpression<'a> {
    /// Create a dummy [`TSInstantiationExpression`].
    ///
    /// Has cost of allocating 48 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            expression: TakeIn::dummy_in(allocator),
            type_parameters: Box::new_in(TakeIn::dummy_in(allocator), allocator),
        }
    }
}

impl<'a> TakeIn<'a> for ImportOrExportKind {
    /// Create a dummy [`ImportOrExportKind`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self::Value
    }
}

impl<'a> TakeIn<'a> for JSDocNullableType<'a> {
    /// Create a dummy [`JSDocNullableType`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            type_annotation: TakeIn::dummy_in(allocator),
            postfix: false,
        }
    }
}

impl<'a> TakeIn<'a> for JSDocNonNullableType<'a> {
    /// Create a dummy [`JSDocNonNullableType`].
    ///
    /// Has cost of allocating 8 bytes into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self {
            span: TakeIn::dummy_in(allocator),
            type_annotation: TakeIn::dummy_in(allocator),
            postfix: false,
        }
    }
}

impl<'a> TakeIn<'a> for JSDocUnknownType {
    /// Create a dummy [`JSDocUnknownType`].
    ///
    /// Does not allocate any data into arena.
    fn dummy_in(allocator: &'a Allocator) -> Self {
        Self { span: TakeIn::dummy_in(allocator) }
    }
}
