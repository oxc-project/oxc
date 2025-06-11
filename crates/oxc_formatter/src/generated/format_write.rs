// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/formatter/format_write.rs`.

#![expect(clippy::match_same_arms)]
use oxc_ast::ast::*;

use crate::{
    formatter::{
        Buffer, Format, FormatResult, Formatter,
        trivia::{format_leading_comments, format_trailing_comments},
    },
    generated::ast_nodes::{AstNode, AstNodes, transmute_self},
    parentheses::NeedsParentheses,
    write::FormatWrite,
};

impl<'a> FormatWrite<'a> for AstNode<'a, Expression<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            Expression::BooleanLiteral(inner) => {
                allocator.alloc(AstNode::<BooleanLiteral> { inner, parent, allocator }).fmt(f)
            }
            Expression::NullLiteral(inner) => {
                allocator.alloc(AstNode::<NullLiteral> { inner, parent, allocator }).fmt(f)
            }
            Expression::NumericLiteral(inner) => {
                allocator.alloc(AstNode::<NumericLiteral> { inner, parent, allocator }).fmt(f)
            }
            Expression::BigIntLiteral(inner) => {
                allocator.alloc(AstNode::<BigIntLiteral> { inner, parent, allocator }).fmt(f)
            }
            Expression::RegExpLiteral(inner) => {
                allocator.alloc(AstNode::<RegExpLiteral> { inner, parent, allocator }).fmt(f)
            }
            Expression::StringLiteral(inner) => {
                allocator.alloc(AstNode::<StringLiteral> { inner, parent, allocator }).fmt(f)
            }
            Expression::TemplateLiteral(inner) => {
                allocator.alloc(AstNode::<TemplateLiteral> { inner, parent, allocator }).fmt(f)
            }
            Expression::Identifier(inner) => {
                allocator.alloc(AstNode::<IdentifierReference> { inner, parent, allocator }).fmt(f)
            }
            Expression::MetaProperty(inner) => {
                allocator.alloc(AstNode::<MetaProperty> { inner, parent, allocator }).fmt(f)
            }
            Expression::Super(inner) => {
                allocator.alloc(AstNode::<Super> { inner, parent, allocator }).fmt(f)
            }
            Expression::ArrayExpression(inner) => {
                allocator.alloc(AstNode::<ArrayExpression> { inner, parent, allocator }).fmt(f)
            }
            Expression::ArrowFunctionExpression(inner) => allocator
                .alloc(AstNode::<ArrowFunctionExpression> { inner, parent, allocator })
                .fmt(f),
            Expression::AssignmentExpression(inner) => {
                allocator.alloc(AstNode::<AssignmentExpression> { inner, parent, allocator }).fmt(f)
            }
            Expression::AwaitExpression(inner) => {
                allocator.alloc(AstNode::<AwaitExpression> { inner, parent, allocator }).fmt(f)
            }
            Expression::BinaryExpression(inner) => {
                allocator.alloc(AstNode::<BinaryExpression> { inner, parent, allocator }).fmt(f)
            }
            Expression::CallExpression(inner) => {
                allocator.alloc(AstNode::<CallExpression> { inner, parent, allocator }).fmt(f)
            }
            Expression::ChainExpression(inner) => {
                allocator.alloc(AstNode::<ChainExpression> { inner, parent, allocator }).fmt(f)
            }
            Expression::ClassExpression(inner) => {
                allocator.alloc(AstNode::<Class> { inner, parent, allocator }).fmt(f)
            }
            Expression::ConditionalExpression(inner) => allocator
                .alloc(AstNode::<ConditionalExpression> { inner, parent, allocator })
                .fmt(f),
            Expression::FunctionExpression(inner) => {
                allocator.alloc(AstNode::<Function> { inner, parent, allocator }).fmt(f)
            }
            Expression::ImportExpression(inner) => {
                allocator.alloc(AstNode::<ImportExpression> { inner, parent, allocator }).fmt(f)
            }
            Expression::LogicalExpression(inner) => {
                allocator.alloc(AstNode::<LogicalExpression> { inner, parent, allocator }).fmt(f)
            }
            Expression::NewExpression(inner) => {
                allocator.alloc(AstNode::<NewExpression> { inner, parent, allocator }).fmt(f)
            }
            Expression::ObjectExpression(inner) => {
                allocator.alloc(AstNode::<ObjectExpression> { inner, parent, allocator }).fmt(f)
            }
            Expression::ParenthesizedExpression(inner) => allocator
                .alloc(AstNode::<ParenthesizedExpression> { inner, parent, allocator })
                .fmt(f),
            Expression::SequenceExpression(inner) => {
                allocator.alloc(AstNode::<SequenceExpression> { inner, parent, allocator }).fmt(f)
            }
            Expression::TaggedTemplateExpression(inner) => allocator
                .alloc(AstNode::<TaggedTemplateExpression> { inner, parent, allocator })
                .fmt(f),
            Expression::ThisExpression(inner) => {
                allocator.alloc(AstNode::<ThisExpression> { inner, parent, allocator }).fmt(f)
            }
            Expression::UnaryExpression(inner) => {
                allocator.alloc(AstNode::<UnaryExpression> { inner, parent, allocator }).fmt(f)
            }
            Expression::UpdateExpression(inner) => {
                allocator.alloc(AstNode::<UpdateExpression> { inner, parent, allocator }).fmt(f)
            }
            Expression::YieldExpression(inner) => {
                allocator.alloc(AstNode::<YieldExpression> { inner, parent, allocator }).fmt(f)
            }
            Expression::PrivateInExpression(inner) => {
                allocator.alloc(AstNode::<PrivateInExpression> { inner, parent, allocator }).fmt(f)
            }
            Expression::JSXElement(inner) => {
                allocator.alloc(AstNode::<JSXElement> { inner, parent, allocator }).fmt(f)
            }
            Expression::JSXFragment(inner) => {
                allocator.alloc(AstNode::<JSXFragment> { inner, parent, allocator }).fmt(f)
            }
            Expression::TSAsExpression(inner) => {
                allocator.alloc(AstNode::<TSAsExpression> { inner, parent, allocator }).fmt(f)
            }
            Expression::TSSatisfiesExpression(inner) => allocator
                .alloc(AstNode::<TSSatisfiesExpression> { inner, parent, allocator })
                .fmt(f),
            Expression::TSTypeAssertion(inner) => {
                allocator.alloc(AstNode::<TSTypeAssertion> { inner, parent, allocator }).fmt(f)
            }
            Expression::TSNonNullExpression(inner) => {
                allocator.alloc(AstNode::<TSNonNullExpression> { inner, parent, allocator }).fmt(f)
            }
            Expression::TSInstantiationExpression(inner) => allocator
                .alloc(AstNode::<TSInstantiationExpression> { inner, parent, allocator })
                .fmt(f),
            Expression::V8IntrinsicExpression(inner) => allocator
                .alloc(AstNode::<V8IntrinsicExpression> { inner, parent, allocator })
                .fmt(f),
            it @ match_member_expression!(Expression) => {
                let inner = it.to_member_expression();
                allocator.alloc(AstNode::<'a, MemberExpression> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ArrayExpressionElement<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = allocator.alloc(AstNodes::ArrayExpressionElement(transmute_self(self)));
        match self.inner {
            ArrayExpressionElement::SpreadElement(inner) => {
                allocator.alloc(AstNode::<SpreadElement> { inner, parent, allocator }).fmt(f)
            }
            ArrayExpressionElement::Elision(inner) => {
                allocator.alloc(AstNode::<Elision> { inner, parent, allocator }).fmt(f)
            }
            it @ match_expression!(ArrayExpressionElement) => {
                let inner = it.to_expression();
                allocator.alloc(AstNode::<'a, Expression> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ObjectPropertyKind<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            ObjectPropertyKind::ObjectProperty(inner) => {
                allocator.alloc(AstNode::<ObjectProperty> { inner, parent, allocator }).fmt(f)
            }
            ObjectPropertyKind::SpreadProperty(inner) => {
                allocator.alloc(AstNode::<SpreadElement> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, PropertyKey<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = allocator.alloc(AstNodes::PropertyKey(transmute_self(self)));
        match self.inner {
            PropertyKey::StaticIdentifier(inner) => {
                allocator.alloc(AstNode::<IdentifierName> { inner, parent, allocator }).fmt(f)
            }
            PropertyKey::PrivateIdentifier(inner) => {
                allocator.alloc(AstNode::<PrivateIdentifier> { inner, parent, allocator }).fmt(f)
            }
            it @ match_expression!(PropertyKey) => {
                let inner = it.to_expression();
                allocator.alloc(AstNode::<'a, Expression> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, MemberExpression<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = allocator.alloc(AstNodes::MemberExpression(transmute_self(self)));
        match self.inner {
            MemberExpression::ComputedMemberExpression(inner) => allocator
                .alloc(AstNode::<ComputedMemberExpression> { inner, parent, allocator })
                .fmt(f),
            MemberExpression::StaticMemberExpression(inner) => allocator
                .alloc(AstNode::<StaticMemberExpression> { inner, parent, allocator })
                .fmt(f),
            MemberExpression::PrivateFieldExpression(inner) => allocator
                .alloc(AstNode::<PrivateFieldExpression> { inner, parent, allocator })
                .fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, Argument<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = allocator.alloc(AstNodes::Argument(transmute_self(self)));
        match self.inner {
            Argument::SpreadElement(inner) => {
                allocator.alloc(AstNode::<SpreadElement> { inner, parent, allocator }).fmt(f)
            }
            it @ match_expression!(Argument) => {
                let inner = it.to_expression();
                allocator.alloc(AstNode::<'a, Expression> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, AssignmentTarget<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = allocator.alloc(AstNodes::AssignmentTarget(transmute_self(self)));
        match self.inner {
            it @ match_simple_assignment_target!(AssignmentTarget) => {
                let inner = it.to_simple_assignment_target();
                allocator
                    .alloc(AstNode::<'a, SimpleAssignmentTarget> { inner, parent, allocator })
                    .fmt(f)
            }
            it @ match_assignment_target_pattern!(AssignmentTarget) => {
                let inner = it.to_assignment_target_pattern();
                allocator
                    .alloc(AstNode::<'a, AssignmentTargetPattern> { inner, parent, allocator })
                    .fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, SimpleAssignmentTarget<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = allocator.alloc(AstNodes::SimpleAssignmentTarget(transmute_self(self)));
        match self.inner {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(inner) => {
                allocator.alloc(AstNode::<IdentifierReference> { inner, parent, allocator }).fmt(f)
            }
            SimpleAssignmentTarget::TSAsExpression(inner) => {
                allocator.alloc(AstNode::<TSAsExpression> { inner, parent, allocator }).fmt(f)
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(inner) => allocator
                .alloc(AstNode::<TSSatisfiesExpression> { inner, parent, allocator })
                .fmt(f),
            SimpleAssignmentTarget::TSNonNullExpression(inner) => {
                allocator.alloc(AstNode::<TSNonNullExpression> { inner, parent, allocator }).fmt(f)
            }
            SimpleAssignmentTarget::TSTypeAssertion(inner) => {
                allocator.alloc(AstNode::<TSTypeAssertion> { inner, parent, allocator }).fmt(f)
            }
            it @ match_member_expression!(SimpleAssignmentTarget) => {
                let inner = it.to_member_expression();
                allocator.alloc(AstNode::<'a, MemberExpression> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, AssignmentTargetPattern<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = allocator.alloc(AstNodes::AssignmentTargetPattern(transmute_self(self)));
        match self.inner {
            AssignmentTargetPattern::ArrayAssignmentTarget(inner) => allocator
                .alloc(AstNode::<ArrayAssignmentTarget> { inner, parent, allocator })
                .fmt(f),
            AssignmentTargetPattern::ObjectAssignmentTarget(inner) => allocator
                .alloc(AstNode::<ObjectAssignmentTarget> { inner, parent, allocator })
                .fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, AssignmentTargetMaybeDefault<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(inner) => allocator
                .alloc(AstNode::<AssignmentTargetWithDefault> { inner, parent, allocator })
                .fmt(f),
            it @ match_assignment_target!(AssignmentTargetMaybeDefault) => {
                let inner = it.to_assignment_target();
                allocator.alloc(AstNode::<'a, AssignmentTarget> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, AssignmentTargetProperty<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(inner) => allocator
                .alloc(AstNode::<AssignmentTargetPropertyIdentifier> { inner, parent, allocator })
                .fmt(f),
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(inner) => allocator
                .alloc(AstNode::<AssignmentTargetPropertyProperty> { inner, parent, allocator })
                .fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ChainElement<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            ChainElement::CallExpression(inner) => {
                allocator.alloc(AstNode::<CallExpression> { inner, parent, allocator }).fmt(f)
            }
            ChainElement::TSNonNullExpression(inner) => {
                allocator.alloc(AstNode::<TSNonNullExpression> { inner, parent, allocator }).fmt(f)
            }
            it @ match_member_expression!(ChainElement) => {
                let inner = it.to_member_expression();
                allocator.alloc(AstNode::<'a, MemberExpression> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, Statement<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            Statement::BlockStatement(inner) => {
                allocator.alloc(AstNode::<BlockStatement> { inner, parent, allocator }).fmt(f)
            }
            Statement::BreakStatement(inner) => {
                allocator.alloc(AstNode::<BreakStatement> { inner, parent, allocator }).fmt(f)
            }
            Statement::ContinueStatement(inner) => {
                allocator.alloc(AstNode::<ContinueStatement> { inner, parent, allocator }).fmt(f)
            }
            Statement::DebuggerStatement(inner) => {
                allocator.alloc(AstNode::<DebuggerStatement> { inner, parent, allocator }).fmt(f)
            }
            Statement::DoWhileStatement(inner) => {
                allocator.alloc(AstNode::<DoWhileStatement> { inner, parent, allocator }).fmt(f)
            }
            Statement::EmptyStatement(inner) => {
                allocator.alloc(AstNode::<EmptyStatement> { inner, parent, allocator }).fmt(f)
            }
            Statement::ExpressionStatement(inner) => {
                allocator.alloc(AstNode::<ExpressionStatement> { inner, parent, allocator }).fmt(f)
            }
            Statement::ForInStatement(inner) => {
                allocator.alloc(AstNode::<ForInStatement> { inner, parent, allocator }).fmt(f)
            }
            Statement::ForOfStatement(inner) => {
                allocator.alloc(AstNode::<ForOfStatement> { inner, parent, allocator }).fmt(f)
            }
            Statement::ForStatement(inner) => {
                allocator.alloc(AstNode::<ForStatement> { inner, parent, allocator }).fmt(f)
            }
            Statement::IfStatement(inner) => {
                allocator.alloc(AstNode::<IfStatement> { inner, parent, allocator }).fmt(f)
            }
            Statement::LabeledStatement(inner) => {
                allocator.alloc(AstNode::<LabeledStatement> { inner, parent, allocator }).fmt(f)
            }
            Statement::ReturnStatement(inner) => {
                allocator.alloc(AstNode::<ReturnStatement> { inner, parent, allocator }).fmt(f)
            }
            Statement::SwitchStatement(inner) => {
                allocator.alloc(AstNode::<SwitchStatement> { inner, parent, allocator }).fmt(f)
            }
            Statement::ThrowStatement(inner) => {
                allocator.alloc(AstNode::<ThrowStatement> { inner, parent, allocator }).fmt(f)
            }
            Statement::TryStatement(inner) => {
                allocator.alloc(AstNode::<TryStatement> { inner, parent, allocator }).fmt(f)
            }
            Statement::WhileStatement(inner) => {
                allocator.alloc(AstNode::<WhileStatement> { inner, parent, allocator }).fmt(f)
            }
            Statement::WithStatement(inner) => {
                allocator.alloc(AstNode::<WithStatement> { inner, parent, allocator }).fmt(f)
            }
            it @ match_declaration!(Statement) => {
                let inner = it.to_declaration();
                allocator.alloc(AstNode::<'a, Declaration> { inner, parent, allocator }).fmt(f)
            }
            it @ match_module_declaration!(Statement) => {
                let inner = it.to_module_declaration();
                allocator
                    .alloc(AstNode::<'a, ModuleDeclaration> { inner, parent, allocator })
                    .fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, Declaration<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            Declaration::VariableDeclaration(inner) => {
                allocator.alloc(AstNode::<VariableDeclaration> { inner, parent, allocator }).fmt(f)
            }
            Declaration::FunctionDeclaration(inner) => {
                allocator.alloc(AstNode::<Function> { inner, parent, allocator }).fmt(f)
            }
            Declaration::ClassDeclaration(inner) => {
                allocator.alloc(AstNode::<Class> { inner, parent, allocator }).fmt(f)
            }
            Declaration::TSTypeAliasDeclaration(inner) => allocator
                .alloc(AstNode::<TSTypeAliasDeclaration> { inner, parent, allocator })
                .fmt(f),
            Declaration::TSInterfaceDeclaration(inner) => allocator
                .alloc(AstNode::<TSInterfaceDeclaration> { inner, parent, allocator })
                .fmt(f),
            Declaration::TSEnumDeclaration(inner) => {
                allocator.alloc(AstNode::<TSEnumDeclaration> { inner, parent, allocator }).fmt(f)
            }
            Declaration::TSModuleDeclaration(inner) => {
                allocator.alloc(AstNode::<TSModuleDeclaration> { inner, parent, allocator }).fmt(f)
            }
            Declaration::TSImportEqualsDeclaration(inner) => allocator
                .alloc(AstNode::<TSImportEqualsDeclaration> { inner, parent, allocator })
                .fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ForStatementInit<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = allocator.alloc(AstNodes::ForStatementInit(transmute_self(self)));
        match self.inner {
            ForStatementInit::VariableDeclaration(inner) => {
                allocator.alloc(AstNode::<VariableDeclaration> { inner, parent, allocator }).fmt(f)
            }
            it @ match_expression!(ForStatementInit) => {
                let inner = it.to_expression();
                allocator.alloc(AstNode::<'a, Expression> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ForStatementLeft<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            ForStatementLeft::VariableDeclaration(inner) => {
                allocator.alloc(AstNode::<VariableDeclaration> { inner, parent, allocator }).fmt(f)
            }
            it @ match_assignment_target!(ForStatementLeft) => {
                let inner = it.to_assignment_target();
                allocator.alloc(AstNode::<'a, AssignmentTarget> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, BindingPatternKind<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            BindingPatternKind::BindingIdentifier(inner) => {
                allocator.alloc(AstNode::<BindingIdentifier> { inner, parent, allocator }).fmt(f)
            }
            BindingPatternKind::ObjectPattern(inner) => {
                allocator.alloc(AstNode::<ObjectPattern> { inner, parent, allocator }).fmt(f)
            }
            BindingPatternKind::ArrayPattern(inner) => {
                allocator.alloc(AstNode::<ArrayPattern> { inner, parent, allocator }).fmt(f)
            }
            BindingPatternKind::AssignmentPattern(inner) => {
                allocator.alloc(AstNode::<AssignmentPattern> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ClassElement<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            ClassElement::StaticBlock(inner) => {
                allocator.alloc(AstNode::<StaticBlock> { inner, parent, allocator }).fmt(f)
            }
            ClassElement::MethodDefinition(inner) => {
                allocator.alloc(AstNode::<MethodDefinition> { inner, parent, allocator }).fmt(f)
            }
            ClassElement::PropertyDefinition(inner) => {
                allocator.alloc(AstNode::<PropertyDefinition> { inner, parent, allocator }).fmt(f)
            }
            ClassElement::AccessorProperty(inner) => {
                allocator.alloc(AstNode::<AccessorProperty> { inner, parent, allocator }).fmt(f)
            }
            ClassElement::TSIndexSignature(inner) => {
                allocator.alloc(AstNode::<TSIndexSignature> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ModuleDeclaration<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = allocator.alloc(AstNodes::ModuleDeclaration(transmute_self(self)));
        match self.inner {
            ModuleDeclaration::ImportDeclaration(inner) => {
                allocator.alloc(AstNode::<ImportDeclaration> { inner, parent, allocator }).fmt(f)
            }
            ModuleDeclaration::ExportAllDeclaration(inner) => {
                allocator.alloc(AstNode::<ExportAllDeclaration> { inner, parent, allocator }).fmt(f)
            }
            ModuleDeclaration::ExportDefaultDeclaration(inner) => allocator
                .alloc(AstNode::<ExportDefaultDeclaration> { inner, parent, allocator })
                .fmt(f),
            ModuleDeclaration::ExportNamedDeclaration(inner) => allocator
                .alloc(AstNode::<ExportNamedDeclaration> { inner, parent, allocator })
                .fmt(f),
            ModuleDeclaration::TSExportAssignment(inner) => {
                allocator.alloc(AstNode::<TSExportAssignment> { inner, parent, allocator }).fmt(f)
            }
            ModuleDeclaration::TSNamespaceExportDeclaration(inner) => allocator
                .alloc(AstNode::<TSNamespaceExportDeclaration> { inner, parent, allocator })
                .fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportDeclarationSpecifier<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            ImportDeclarationSpecifier::ImportSpecifier(inner) => {
                allocator.alloc(AstNode::<ImportSpecifier> { inner, parent, allocator }).fmt(f)
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(inner) => allocator
                .alloc(AstNode::<ImportDefaultSpecifier> { inner, parent, allocator })
                .fmt(f),
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(inner) => allocator
                .alloc(AstNode::<ImportNamespaceSpecifier> { inner, parent, allocator })
                .fmt(f),
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportAttributeKey<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            ImportAttributeKey::Identifier(inner) => {
                allocator.alloc(AstNode::<IdentifierName> { inner, parent, allocator }).fmt(f)
            }
            ImportAttributeKey::StringLiteral(inner) => {
                allocator.alloc(AstNode::<StringLiteral> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ExportDefaultDeclarationKind<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            ExportDefaultDeclarationKind::FunctionDeclaration(inner) => {
                allocator.alloc(AstNode::<Function> { inner, parent, allocator }).fmt(f)
            }
            ExportDefaultDeclarationKind::ClassDeclaration(inner) => {
                allocator.alloc(AstNode::<Class> { inner, parent, allocator }).fmt(f)
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(inner) => allocator
                .alloc(AstNode::<TSInterfaceDeclaration> { inner, parent, allocator })
                .fmt(f),
            it @ match_expression!(ExportDefaultDeclarationKind) => {
                let inner = it.to_expression();
                allocator.alloc(AstNode::<'a, Expression> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ModuleExportName<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            ModuleExportName::IdentifierName(inner) => {
                allocator.alloc(AstNode::<IdentifierName> { inner, parent, allocator }).fmt(f)
            }
            ModuleExportName::IdentifierReference(inner) => {
                allocator.alloc(AstNode::<IdentifierReference> { inner, parent, allocator }).fmt(f)
            }
            ModuleExportName::StringLiteral(inner) => {
                allocator.alloc(AstNode::<StringLiteral> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXElementName<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            JSXElementName::Identifier(inner) => {
                allocator.alloc(AstNode::<JSXIdentifier> { inner, parent, allocator }).fmt(f)
            }
            JSXElementName::IdentifierReference(inner) => {
                allocator.alloc(AstNode::<IdentifierReference> { inner, parent, allocator }).fmt(f)
            }
            JSXElementName::NamespacedName(inner) => {
                allocator.alloc(AstNode::<JSXNamespacedName> { inner, parent, allocator }).fmt(f)
            }
            JSXElementName::MemberExpression(inner) => {
                allocator.alloc(AstNode::<JSXMemberExpression> { inner, parent, allocator }).fmt(f)
            }
            JSXElementName::ThisExpression(inner) => {
                allocator.alloc(AstNode::<ThisExpression> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXMemberExpressionObject<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            JSXMemberExpressionObject::IdentifierReference(inner) => {
                allocator.alloc(AstNode::<IdentifierReference> { inner, parent, allocator }).fmt(f)
            }
            JSXMemberExpressionObject::MemberExpression(inner) => {
                allocator.alloc(AstNode::<JSXMemberExpression> { inner, parent, allocator }).fmt(f)
            }
            JSXMemberExpressionObject::ThisExpression(inner) => {
                allocator.alloc(AstNode::<ThisExpression> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXExpression<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            JSXExpression::EmptyExpression(inner) => {
                allocator.alloc(AstNode::<JSXEmptyExpression> { inner, parent, allocator }).fmt(f)
            }
            it @ match_expression!(JSXExpression) => {
                let inner = it.to_expression();
                allocator.alloc(AstNode::<'a, Expression> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXAttributeItem<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            JSXAttributeItem::Attribute(inner) => {
                allocator.alloc(AstNode::<JSXAttribute> { inner, parent, allocator }).fmt(f)
            }
            JSXAttributeItem::SpreadAttribute(inner) => {
                allocator.alloc(AstNode::<JSXSpreadAttribute> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXAttributeName<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            JSXAttributeName::Identifier(inner) => {
                allocator.alloc(AstNode::<JSXIdentifier> { inner, parent, allocator }).fmt(f)
            }
            JSXAttributeName::NamespacedName(inner) => {
                allocator.alloc(AstNode::<JSXNamespacedName> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXAttributeValue<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            JSXAttributeValue::StringLiteral(inner) => {
                allocator.alloc(AstNode::<StringLiteral> { inner, parent, allocator }).fmt(f)
            }
            JSXAttributeValue::ExpressionContainer(inner) => allocator
                .alloc(AstNode::<JSXExpressionContainer> { inner, parent, allocator })
                .fmt(f),
            JSXAttributeValue::Element(inner) => {
                allocator.alloc(AstNode::<JSXElement> { inner, parent, allocator }).fmt(f)
            }
            JSXAttributeValue::Fragment(inner) => {
                allocator.alloc(AstNode::<JSXFragment> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXChild<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            JSXChild::Text(inner) => {
                allocator.alloc(AstNode::<JSXText> { inner, parent, allocator }).fmt(f)
            }
            JSXChild::Element(inner) => {
                allocator.alloc(AstNode::<JSXElement> { inner, parent, allocator }).fmt(f)
            }
            JSXChild::Fragment(inner) => {
                allocator.alloc(AstNode::<JSXFragment> { inner, parent, allocator }).fmt(f)
            }
            JSXChild::ExpressionContainer(inner) => allocator
                .alloc(AstNode::<JSXExpressionContainer> { inner, parent, allocator })
                .fmt(f),
            JSXChild::Spread(inner) => {
                allocator.alloc(AstNode::<JSXSpreadChild> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSEnumMemberName<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            TSEnumMemberName::Identifier(inner) => {
                allocator.alloc(AstNode::<IdentifierName> { inner, parent, allocator }).fmt(f)
            }
            TSEnumMemberName::String(inner) => {
                allocator.alloc(AstNode::<StringLiteral> { inner, parent, allocator }).fmt(f)
            }
            TSEnumMemberName::ComputedString(inner) => {
                allocator.alloc(AstNode::<StringLiteral> { inner, parent, allocator }).fmt(f)
            }
            TSEnumMemberName::ComputedTemplateString(inner) => {
                allocator.alloc(AstNode::<TemplateLiteral> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSLiteral<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            TSLiteral::BooleanLiteral(inner) => {
                allocator.alloc(AstNode::<BooleanLiteral> { inner, parent, allocator }).fmt(f)
            }
            TSLiteral::NumericLiteral(inner) => {
                allocator.alloc(AstNode::<NumericLiteral> { inner, parent, allocator }).fmt(f)
            }
            TSLiteral::BigIntLiteral(inner) => {
                allocator.alloc(AstNode::<BigIntLiteral> { inner, parent, allocator }).fmt(f)
            }
            TSLiteral::StringLiteral(inner) => {
                allocator.alloc(AstNode::<StringLiteral> { inner, parent, allocator }).fmt(f)
            }
            TSLiteral::TemplateLiteral(inner) => {
                allocator.alloc(AstNode::<TemplateLiteral> { inner, parent, allocator }).fmt(f)
            }
            TSLiteral::UnaryExpression(inner) => {
                allocator.alloc(AstNode::<UnaryExpression> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSType<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            TSType::TSAnyKeyword(inner) => {
                allocator.alloc(AstNode::<TSAnyKeyword> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSBigIntKeyword(inner) => {
                allocator.alloc(AstNode::<TSBigIntKeyword> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSBooleanKeyword(inner) => {
                allocator.alloc(AstNode::<TSBooleanKeyword> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSIntrinsicKeyword(inner) => {
                allocator.alloc(AstNode::<TSIntrinsicKeyword> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSNeverKeyword(inner) => {
                allocator.alloc(AstNode::<TSNeverKeyword> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSNullKeyword(inner) => {
                allocator.alloc(AstNode::<TSNullKeyword> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSNumberKeyword(inner) => {
                allocator.alloc(AstNode::<TSNumberKeyword> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSObjectKeyword(inner) => {
                allocator.alloc(AstNode::<TSObjectKeyword> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSStringKeyword(inner) => {
                allocator.alloc(AstNode::<TSStringKeyword> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSSymbolKeyword(inner) => {
                allocator.alloc(AstNode::<TSSymbolKeyword> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSUndefinedKeyword(inner) => {
                allocator.alloc(AstNode::<TSUndefinedKeyword> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSUnknownKeyword(inner) => {
                allocator.alloc(AstNode::<TSUnknownKeyword> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSVoidKeyword(inner) => {
                allocator.alloc(AstNode::<TSVoidKeyword> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSArrayType(inner) => {
                allocator.alloc(AstNode::<TSArrayType> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSConditionalType(inner) => {
                allocator.alloc(AstNode::<TSConditionalType> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSConstructorType(inner) => {
                allocator.alloc(AstNode::<TSConstructorType> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSFunctionType(inner) => {
                allocator.alloc(AstNode::<TSFunctionType> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSImportType(inner) => {
                allocator.alloc(AstNode::<TSImportType> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSIndexedAccessType(inner) => {
                allocator.alloc(AstNode::<TSIndexedAccessType> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSInferType(inner) => {
                allocator.alloc(AstNode::<TSInferType> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSIntersectionType(inner) => {
                allocator.alloc(AstNode::<TSIntersectionType> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSLiteralType(inner) => {
                allocator.alloc(AstNode::<TSLiteralType> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSMappedType(inner) => {
                allocator.alloc(AstNode::<TSMappedType> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSNamedTupleMember(inner) => {
                allocator.alloc(AstNode::<TSNamedTupleMember> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSTemplateLiteralType(inner) => allocator
                .alloc(AstNode::<TSTemplateLiteralType> { inner, parent, allocator })
                .fmt(f),
            TSType::TSThisType(inner) => {
                allocator.alloc(AstNode::<TSThisType> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSTupleType(inner) => {
                allocator.alloc(AstNode::<TSTupleType> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSTypeLiteral(inner) => {
                allocator.alloc(AstNode::<TSTypeLiteral> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSTypeOperatorType(inner) => {
                allocator.alloc(AstNode::<TSTypeOperator> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSTypePredicate(inner) => {
                allocator.alloc(AstNode::<TSTypePredicate> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSTypeQuery(inner) => {
                allocator.alloc(AstNode::<TSTypeQuery> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSTypeReference(inner) => {
                allocator.alloc(AstNode::<TSTypeReference> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSUnionType(inner) => {
                allocator.alloc(AstNode::<TSUnionType> { inner, parent, allocator }).fmt(f)
            }
            TSType::TSParenthesizedType(inner) => {
                allocator.alloc(AstNode::<TSParenthesizedType> { inner, parent, allocator }).fmt(f)
            }
            TSType::JSDocNullableType(inner) => {
                allocator.alloc(AstNode::<JSDocNullableType> { inner, parent, allocator }).fmt(f)
            }
            TSType::JSDocNonNullableType(inner) => {
                allocator.alloc(AstNode::<JSDocNonNullableType> { inner, parent, allocator }).fmt(f)
            }
            TSType::JSDocUnknownType(inner) => {
                allocator.alloc(AstNode::<JSDocUnknownType> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTupleElement<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            TSTupleElement::TSOptionalType(inner) => {
                allocator.alloc(AstNode::<TSOptionalType> { inner, parent, allocator }).fmt(f)
            }
            TSTupleElement::TSRestType(inner) => {
                allocator.alloc(AstNode::<TSRestType> { inner, parent, allocator }).fmt(f)
            }
            it @ match_ts_type!(TSTupleElement) => {
                let inner = it.to_ts_type();
                allocator.alloc(AstNode::<'a, TSType> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeName<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = allocator.alloc(AstNodes::TSTypeName(transmute_self(self)));
        match self.inner {
            TSTypeName::IdentifierReference(inner) => {
                allocator.alloc(AstNode::<IdentifierReference> { inner, parent, allocator }).fmt(f)
            }
            TSTypeName::QualifiedName(inner) => {
                allocator.alloc(AstNode::<TSQualifiedName> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSSignature<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            TSSignature::TSIndexSignature(inner) => {
                allocator.alloc(AstNode::<TSIndexSignature> { inner, parent, allocator }).fmt(f)
            }
            TSSignature::TSPropertySignature(inner) => {
                allocator.alloc(AstNode::<TSPropertySignature> { inner, parent, allocator }).fmt(f)
            }
            TSSignature::TSCallSignatureDeclaration(inner) => allocator
                .alloc(AstNode::<TSCallSignatureDeclaration> { inner, parent, allocator })
                .fmt(f),
            TSSignature::TSConstructSignatureDeclaration(inner) => allocator
                .alloc(AstNode::<TSConstructSignatureDeclaration> { inner, parent, allocator })
                .fmt(f),
            TSSignature::TSMethodSignature(inner) => {
                allocator.alloc(AstNode::<TSMethodSignature> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypePredicateName<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            TSTypePredicateName::Identifier(inner) => {
                allocator.alloc(AstNode::<IdentifierName> { inner, parent, allocator }).fmt(f)
            }
            TSTypePredicateName::This(inner) => {
                allocator.alloc(AstNode::<TSThisType> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSModuleDeclarationName<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            TSModuleDeclarationName::Identifier(inner) => {
                allocator.alloc(AstNode::<BindingIdentifier> { inner, parent, allocator }).fmt(f)
            }
            TSModuleDeclarationName::StringLiteral(inner) => {
                allocator.alloc(AstNode::<StringLiteral> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSModuleDeclarationBody<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            TSModuleDeclarationBody::TSModuleDeclaration(inner) => {
                allocator.alloc(AstNode::<TSModuleDeclaration> { inner, parent, allocator }).fmt(f)
            }
            TSModuleDeclarationBody::TSModuleBlock(inner) => {
                allocator.alloc(AstNode::<TSModuleBlock> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSTypeQueryExprName<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = self.parent;
        match self.inner {
            TSTypeQueryExprName::TSImportType(inner) => {
                allocator.alloc(AstNode::<TSImportType> { inner, parent, allocator }).fmt(f)
            }
            it @ match_ts_type_name!(TSTypeQueryExprName) => {
                let inner = it.to_ts_type_name();
                allocator.alloc(AstNode::<'a, TSTypeName> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSModuleReference<'a>> {
    #[inline]
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let allocator = self.allocator;
        let parent = allocator.alloc(AstNodes::TSModuleReference(transmute_self(self)));
        match self.inner {
            TSModuleReference::ExternalModuleReference(inner) => allocator
                .alloc(AstNode::<TSExternalModuleReference> { inner, parent, allocator })
                .fmt(f),
            it @ match_ts_type_name!(TSModuleReference) => {
                let inner = it.to_ts_type_name();
                allocator.alloc(AstNode::<'a, TSTypeName> { inner, parent, allocator }).fmt(f)
            }
        }
    }
}
