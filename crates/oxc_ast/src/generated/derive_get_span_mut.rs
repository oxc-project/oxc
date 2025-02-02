// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/get_span.rs`

#![allow(clippy::match_same_arms)]

use oxc_span::{GetSpanMut, Span};

use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
use crate::ast::ts::*;

impl GetSpanMut for BooleanLiteral {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for NullLiteral {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for NumericLiteral<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for StringLiteral<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for BigIntLiteral<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for RegExpLiteral<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Program<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Expression<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::BooleanLiteral(it) => it.span_mut(),
            Self::NullLiteral(it) => it.span_mut(),
            Self::NumericLiteral(it) => it.span_mut(),
            Self::BigIntLiteral(it) => it.span_mut(),
            Self::RegExpLiteral(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
            Self::TemplateLiteral(it) => it.span_mut(),
            Self::Identifier(it) => it.span_mut(),
            Self::MetaProperty(it) => it.span_mut(),
            Self::Super(it) => it.span_mut(),
            Self::ArrayExpression(it) => it.span_mut(),
            Self::ArrowFunctionExpression(it) => it.span_mut(),
            Self::AssignmentExpression(it) => it.span_mut(),
            Self::AwaitExpression(it) => it.span_mut(),
            Self::BinaryExpression(it) => it.span_mut(),
            Self::CallExpression(it) => it.span_mut(),
            Self::ChainExpression(it) => it.span_mut(),
            Self::ClassExpression(it) => it.span_mut(),
            Self::ConditionalExpression(it) => it.span_mut(),
            Self::FunctionExpression(it) => it.span_mut(),
            Self::ImportExpression(it) => it.span_mut(),
            Self::LogicalExpression(it) => it.span_mut(),
            Self::NewExpression(it) => it.span_mut(),
            Self::ObjectExpression(it) => it.span_mut(),
            Self::ParenthesizedExpression(it) => it.span_mut(),
            Self::SequenceExpression(it) => it.span_mut(),
            Self::TaggedTemplateExpression(it) => it.span_mut(),
            Self::ThisExpression(it) => it.span_mut(),
            Self::UnaryExpression(it) => it.span_mut(),
            Self::UpdateExpression(it) => it.span_mut(),
            Self::YieldExpression(it) => it.span_mut(),
            Self::PrivateInExpression(it) => it.span_mut(),
            Self::JSXElement(it) => it.span_mut(),
            Self::JSXFragment(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for IdentifierName<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for IdentifierReference<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for BindingIdentifier<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for LabelIdentifier<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ThisExpression {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ArrayExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ArrayExpressionElement<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::SpreadElement(it) => it.span_mut(),
            Self::Elision(it) => it.span_mut(),
            Self::BooleanLiteral(it) => it.span_mut(),
            Self::NullLiteral(it) => it.span_mut(),
            Self::NumericLiteral(it) => it.span_mut(),
            Self::BigIntLiteral(it) => it.span_mut(),
            Self::RegExpLiteral(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
            Self::TemplateLiteral(it) => it.span_mut(),
            Self::Identifier(it) => it.span_mut(),
            Self::MetaProperty(it) => it.span_mut(),
            Self::Super(it) => it.span_mut(),
            Self::ArrayExpression(it) => it.span_mut(),
            Self::ArrowFunctionExpression(it) => it.span_mut(),
            Self::AssignmentExpression(it) => it.span_mut(),
            Self::AwaitExpression(it) => it.span_mut(),
            Self::BinaryExpression(it) => it.span_mut(),
            Self::CallExpression(it) => it.span_mut(),
            Self::ChainExpression(it) => it.span_mut(),
            Self::ClassExpression(it) => it.span_mut(),
            Self::ConditionalExpression(it) => it.span_mut(),
            Self::FunctionExpression(it) => it.span_mut(),
            Self::ImportExpression(it) => it.span_mut(),
            Self::LogicalExpression(it) => it.span_mut(),
            Self::NewExpression(it) => it.span_mut(),
            Self::ObjectExpression(it) => it.span_mut(),
            Self::ParenthesizedExpression(it) => it.span_mut(),
            Self::SequenceExpression(it) => it.span_mut(),
            Self::TaggedTemplateExpression(it) => it.span_mut(),
            Self::ThisExpression(it) => it.span_mut(),
            Self::UnaryExpression(it) => it.span_mut(),
            Self::UpdateExpression(it) => it.span_mut(),
            Self::YieldExpression(it) => it.span_mut(),
            Self::PrivateInExpression(it) => it.span_mut(),
            Self::JSXElement(it) => it.span_mut(),
            Self::JSXFragment(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for Elision {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ObjectExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ObjectPropertyKind<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ObjectProperty(it) => it.span_mut(),
            Self::SpreadProperty(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for ObjectProperty<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for PropertyKey<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::StaticIdentifier(it) => it.span_mut(),
            Self::PrivateIdentifier(it) => it.span_mut(),
            Self::BooleanLiteral(it) => it.span_mut(),
            Self::NullLiteral(it) => it.span_mut(),
            Self::NumericLiteral(it) => it.span_mut(),
            Self::BigIntLiteral(it) => it.span_mut(),
            Self::RegExpLiteral(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
            Self::TemplateLiteral(it) => it.span_mut(),
            Self::Identifier(it) => it.span_mut(),
            Self::MetaProperty(it) => it.span_mut(),
            Self::Super(it) => it.span_mut(),
            Self::ArrayExpression(it) => it.span_mut(),
            Self::ArrowFunctionExpression(it) => it.span_mut(),
            Self::AssignmentExpression(it) => it.span_mut(),
            Self::AwaitExpression(it) => it.span_mut(),
            Self::BinaryExpression(it) => it.span_mut(),
            Self::CallExpression(it) => it.span_mut(),
            Self::ChainExpression(it) => it.span_mut(),
            Self::ClassExpression(it) => it.span_mut(),
            Self::ConditionalExpression(it) => it.span_mut(),
            Self::FunctionExpression(it) => it.span_mut(),
            Self::ImportExpression(it) => it.span_mut(),
            Self::LogicalExpression(it) => it.span_mut(),
            Self::NewExpression(it) => it.span_mut(),
            Self::ObjectExpression(it) => it.span_mut(),
            Self::ParenthesizedExpression(it) => it.span_mut(),
            Self::SequenceExpression(it) => it.span_mut(),
            Self::TaggedTemplateExpression(it) => it.span_mut(),
            Self::ThisExpression(it) => it.span_mut(),
            Self::UnaryExpression(it) => it.span_mut(),
            Self::UpdateExpression(it) => it.span_mut(),
            Self::YieldExpression(it) => it.span_mut(),
            Self::PrivateInExpression(it) => it.span_mut(),
            Self::JSXElement(it) => it.span_mut(),
            Self::JSXFragment(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for TemplateLiteral<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TaggedTemplateExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TemplateElement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for MemberExpression<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for ComputedMemberExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for StaticMemberExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for PrivateFieldExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for CallExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for NewExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for MetaProperty<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for SpreadElement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Argument<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::SpreadElement(it) => it.span_mut(),
            Self::BooleanLiteral(it) => it.span_mut(),
            Self::NullLiteral(it) => it.span_mut(),
            Self::NumericLiteral(it) => it.span_mut(),
            Self::BigIntLiteral(it) => it.span_mut(),
            Self::RegExpLiteral(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
            Self::TemplateLiteral(it) => it.span_mut(),
            Self::Identifier(it) => it.span_mut(),
            Self::MetaProperty(it) => it.span_mut(),
            Self::Super(it) => it.span_mut(),
            Self::ArrayExpression(it) => it.span_mut(),
            Self::ArrowFunctionExpression(it) => it.span_mut(),
            Self::AssignmentExpression(it) => it.span_mut(),
            Self::AwaitExpression(it) => it.span_mut(),
            Self::BinaryExpression(it) => it.span_mut(),
            Self::CallExpression(it) => it.span_mut(),
            Self::ChainExpression(it) => it.span_mut(),
            Self::ClassExpression(it) => it.span_mut(),
            Self::ConditionalExpression(it) => it.span_mut(),
            Self::FunctionExpression(it) => it.span_mut(),
            Self::ImportExpression(it) => it.span_mut(),
            Self::LogicalExpression(it) => it.span_mut(),
            Self::NewExpression(it) => it.span_mut(),
            Self::ObjectExpression(it) => it.span_mut(),
            Self::ParenthesizedExpression(it) => it.span_mut(),
            Self::SequenceExpression(it) => it.span_mut(),
            Self::TaggedTemplateExpression(it) => it.span_mut(),
            Self::ThisExpression(it) => it.span_mut(),
            Self::UnaryExpression(it) => it.span_mut(),
            Self::UpdateExpression(it) => it.span_mut(),
            Self::YieldExpression(it) => it.span_mut(),
            Self::PrivateInExpression(it) => it.span_mut(),
            Self::JSXElement(it) => it.span_mut(),
            Self::JSXFragment(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for UpdateExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for UnaryExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for BinaryExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for PrivateInExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for LogicalExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ConditionalExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for AssignmentExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for AssignmentTarget<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::AssignmentTargetIdentifier(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
            Self::ArrayAssignmentTarget(it) => it.span_mut(),
            Self::ObjectAssignmentTarget(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for SimpleAssignmentTarget<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::AssignmentTargetIdentifier(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for AssignmentTargetPattern<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ArrayAssignmentTarget(it) => it.span_mut(),
            Self::ObjectAssignmentTarget(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for ArrayAssignmentTarget<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ObjectAssignmentTarget<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for AssignmentTargetRest<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for AssignmentTargetMaybeDefault<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::AssignmentTargetWithDefault(it) => it.span_mut(),
            Self::AssignmentTargetIdentifier(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
            Self::ArrayAssignmentTarget(it) => it.span_mut(),
            Self::ObjectAssignmentTarget(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for AssignmentTargetWithDefault<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for AssignmentTargetProperty<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => it.span_mut(),
            Self::AssignmentTargetPropertyProperty(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for AssignmentTargetPropertyIdentifier<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for AssignmentTargetPropertyProperty<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for SequenceExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Super {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for AwaitExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ChainExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ChainElement<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::CallExpression(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for ParenthesizedExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Statement<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::BlockStatement(it) => it.span_mut(),
            Self::BreakStatement(it) => it.span_mut(),
            Self::ContinueStatement(it) => it.span_mut(),
            Self::DebuggerStatement(it) => it.span_mut(),
            Self::DoWhileStatement(it) => it.span_mut(),
            Self::EmptyStatement(it) => it.span_mut(),
            Self::ExpressionStatement(it) => it.span_mut(),
            Self::ForInStatement(it) => it.span_mut(),
            Self::ForOfStatement(it) => it.span_mut(),
            Self::ForStatement(it) => it.span_mut(),
            Self::IfStatement(it) => it.span_mut(),
            Self::LabeledStatement(it) => it.span_mut(),
            Self::ReturnStatement(it) => it.span_mut(),
            Self::SwitchStatement(it) => it.span_mut(),
            Self::ThrowStatement(it) => it.span_mut(),
            Self::TryStatement(it) => it.span_mut(),
            Self::WhileStatement(it) => it.span_mut(),
            Self::WithStatement(it) => it.span_mut(),
            Self::VariableDeclaration(it) => it.span_mut(),
            Self::FunctionDeclaration(it) => it.span_mut(),
            Self::ClassDeclaration(it) => it.span_mut(),
            Self::TSTypeAliasDeclaration(it) => it.span_mut(),
            Self::TSInterfaceDeclaration(it) => it.span_mut(),
            Self::TSEnumDeclaration(it) => it.span_mut(),
            Self::TSModuleDeclaration(it) => it.span_mut(),
            Self::TSImportEqualsDeclaration(it) => it.span_mut(),
            Self::ImportDeclaration(it) => it.span_mut(),
            Self::ExportAllDeclaration(it) => it.span_mut(),
            Self::ExportDefaultDeclaration(it) => it.span_mut(),
            Self::ExportNamedDeclaration(it) => it.span_mut(),
            Self::TSExportAssignment(it) => it.span_mut(),
            Self::TSNamespaceExportDeclaration(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for Directive<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Hashbang<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for BlockStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Declaration<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::VariableDeclaration(it) => it.span_mut(),
            Self::FunctionDeclaration(it) => it.span_mut(),
            Self::ClassDeclaration(it) => it.span_mut(),
            Self::TSTypeAliasDeclaration(it) => it.span_mut(),
            Self::TSInterfaceDeclaration(it) => it.span_mut(),
            Self::TSEnumDeclaration(it) => it.span_mut(),
            Self::TSModuleDeclaration(it) => it.span_mut(),
            Self::TSImportEqualsDeclaration(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for VariableDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for VariableDeclarator<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for EmptyStatement {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ExpressionStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for IfStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for DoWhileStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for WhileStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ForStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ForStatementInit<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::VariableDeclaration(it) => it.span_mut(),
            Self::BooleanLiteral(it) => it.span_mut(),
            Self::NullLiteral(it) => it.span_mut(),
            Self::NumericLiteral(it) => it.span_mut(),
            Self::BigIntLiteral(it) => it.span_mut(),
            Self::RegExpLiteral(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
            Self::TemplateLiteral(it) => it.span_mut(),
            Self::Identifier(it) => it.span_mut(),
            Self::MetaProperty(it) => it.span_mut(),
            Self::Super(it) => it.span_mut(),
            Self::ArrayExpression(it) => it.span_mut(),
            Self::ArrowFunctionExpression(it) => it.span_mut(),
            Self::AssignmentExpression(it) => it.span_mut(),
            Self::AwaitExpression(it) => it.span_mut(),
            Self::BinaryExpression(it) => it.span_mut(),
            Self::CallExpression(it) => it.span_mut(),
            Self::ChainExpression(it) => it.span_mut(),
            Self::ClassExpression(it) => it.span_mut(),
            Self::ConditionalExpression(it) => it.span_mut(),
            Self::FunctionExpression(it) => it.span_mut(),
            Self::ImportExpression(it) => it.span_mut(),
            Self::LogicalExpression(it) => it.span_mut(),
            Self::NewExpression(it) => it.span_mut(),
            Self::ObjectExpression(it) => it.span_mut(),
            Self::ParenthesizedExpression(it) => it.span_mut(),
            Self::SequenceExpression(it) => it.span_mut(),
            Self::TaggedTemplateExpression(it) => it.span_mut(),
            Self::ThisExpression(it) => it.span_mut(),
            Self::UnaryExpression(it) => it.span_mut(),
            Self::UpdateExpression(it) => it.span_mut(),
            Self::YieldExpression(it) => it.span_mut(),
            Self::PrivateInExpression(it) => it.span_mut(),
            Self::JSXElement(it) => it.span_mut(),
            Self::JSXFragment(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for ForInStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ForStatementLeft<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::VariableDeclaration(it) => it.span_mut(),
            Self::AssignmentTargetIdentifier(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
            Self::ArrayAssignmentTarget(it) => it.span_mut(),
            Self::ObjectAssignmentTarget(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for ForOfStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ContinueStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for BreakStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ReturnStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for WithStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for SwitchStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for SwitchCase<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for LabeledStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ThrowStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TryStatement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for CatchClause<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for CatchParameter<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for DebuggerStatement {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for BindingPattern<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        self.kind.span_mut()
    }
}

impl GetSpanMut for BindingPatternKind<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::BindingIdentifier(it) => it.span_mut(),
            Self::ObjectPattern(it) => it.span_mut(),
            Self::ArrayPattern(it) => it.span_mut(),
            Self::AssignmentPattern(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for AssignmentPattern<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ObjectPattern<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for BindingProperty<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ArrayPattern<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for BindingRestElement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Function<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for FormalParameters<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for FormalParameter<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for FunctionBody<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ArrowFunctionExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for YieldExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Class<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ClassBody<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ClassElement<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::StaticBlock(it) => it.span_mut(),
            Self::MethodDefinition(it) => it.span_mut(),
            Self::PropertyDefinition(it) => it.span_mut(),
            Self::AccessorProperty(it) => it.span_mut(),
            Self::TSIndexSignature(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for MethodDefinition<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for PropertyDefinition<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for PrivateIdentifier<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for StaticBlock<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ModuleDeclaration<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ImportDeclaration(it) => it.span_mut(),
            Self::ExportAllDeclaration(it) => it.span_mut(),
            Self::ExportDefaultDeclaration(it) => it.span_mut(),
            Self::ExportNamedDeclaration(it) => it.span_mut(),
            Self::TSExportAssignment(it) => it.span_mut(),
            Self::TSNamespaceExportDeclaration(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for AccessorProperty<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ImportExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ImportDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ImportDeclarationSpecifier<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ImportSpecifier(it) => it.span_mut(),
            Self::ImportDefaultSpecifier(it) => it.span_mut(),
            Self::ImportNamespaceSpecifier(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for ImportSpecifier<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ImportDefaultSpecifier<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ImportNamespaceSpecifier<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for WithClause<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ImportAttribute<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ImportAttributeKey<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for ExportNamedDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ExportDefaultDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ExportAllDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ExportSpecifier<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for ExportDefaultDeclarationKind<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::FunctionDeclaration(it) => it.span_mut(),
            Self::ClassDeclaration(it) => it.span_mut(),
            Self::TSInterfaceDeclaration(it) => it.span_mut(),
            Self::BooleanLiteral(it) => it.span_mut(),
            Self::NullLiteral(it) => it.span_mut(),
            Self::NumericLiteral(it) => it.span_mut(),
            Self::BigIntLiteral(it) => it.span_mut(),
            Self::RegExpLiteral(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
            Self::TemplateLiteral(it) => it.span_mut(),
            Self::Identifier(it) => it.span_mut(),
            Self::MetaProperty(it) => it.span_mut(),
            Self::Super(it) => it.span_mut(),
            Self::ArrayExpression(it) => it.span_mut(),
            Self::ArrowFunctionExpression(it) => it.span_mut(),
            Self::AssignmentExpression(it) => it.span_mut(),
            Self::AwaitExpression(it) => it.span_mut(),
            Self::BinaryExpression(it) => it.span_mut(),
            Self::CallExpression(it) => it.span_mut(),
            Self::ChainExpression(it) => it.span_mut(),
            Self::ClassExpression(it) => it.span_mut(),
            Self::ConditionalExpression(it) => it.span_mut(),
            Self::FunctionExpression(it) => it.span_mut(),
            Self::ImportExpression(it) => it.span_mut(),
            Self::LogicalExpression(it) => it.span_mut(),
            Self::NewExpression(it) => it.span_mut(),
            Self::ObjectExpression(it) => it.span_mut(),
            Self::ParenthesizedExpression(it) => it.span_mut(),
            Self::SequenceExpression(it) => it.span_mut(),
            Self::TaggedTemplateExpression(it) => it.span_mut(),
            Self::ThisExpression(it) => it.span_mut(),
            Self::UnaryExpression(it) => it.span_mut(),
            Self::UpdateExpression(it) => it.span_mut(),
            Self::YieldExpression(it) => it.span_mut(),
            Self::PrivateInExpression(it) => it.span_mut(),
            Self::JSXElement(it) => it.span_mut(),
            Self::JSXFragment(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for ModuleExportName<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::IdentifierName(it) => it.span_mut(),
            Self::IdentifierReference(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for TSThisParameter<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSEnumDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSEnumMember<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSEnumMemberName<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::String(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for TSTypeAnnotation<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSLiteralType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSLiteral<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::BooleanLiteral(it) => it.span_mut(),
            Self::NullLiteral(it) => it.span_mut(),
            Self::NumericLiteral(it) => it.span_mut(),
            Self::BigIntLiteral(it) => it.span_mut(),
            Self::RegExpLiteral(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
            Self::TemplateLiteral(it) => it.span_mut(),
            Self::UnaryExpression(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for TSType<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSAnyKeyword(it) => it.span_mut(),
            Self::TSBigIntKeyword(it) => it.span_mut(),
            Self::TSBooleanKeyword(it) => it.span_mut(),
            Self::TSIntrinsicKeyword(it) => it.span_mut(),
            Self::TSNeverKeyword(it) => it.span_mut(),
            Self::TSNullKeyword(it) => it.span_mut(),
            Self::TSNumberKeyword(it) => it.span_mut(),
            Self::TSObjectKeyword(it) => it.span_mut(),
            Self::TSStringKeyword(it) => it.span_mut(),
            Self::TSSymbolKeyword(it) => it.span_mut(),
            Self::TSUndefinedKeyword(it) => it.span_mut(),
            Self::TSUnknownKeyword(it) => it.span_mut(),
            Self::TSVoidKeyword(it) => it.span_mut(),
            Self::TSArrayType(it) => it.span_mut(),
            Self::TSConditionalType(it) => it.span_mut(),
            Self::TSConstructorType(it) => it.span_mut(),
            Self::TSFunctionType(it) => it.span_mut(),
            Self::TSImportType(it) => it.span_mut(),
            Self::TSIndexedAccessType(it) => it.span_mut(),
            Self::TSInferType(it) => it.span_mut(),
            Self::TSIntersectionType(it) => it.span_mut(),
            Self::TSLiteralType(it) => it.span_mut(),
            Self::TSMappedType(it) => it.span_mut(),
            Self::TSNamedTupleMember(it) => it.span_mut(),
            Self::TSQualifiedName(it) => it.span_mut(),
            Self::TSTemplateLiteralType(it) => it.span_mut(),
            Self::TSThisType(it) => it.span_mut(),
            Self::TSTupleType(it) => it.span_mut(),
            Self::TSTypeLiteral(it) => it.span_mut(),
            Self::TSTypeOperatorType(it) => it.span_mut(),
            Self::TSTypePredicate(it) => it.span_mut(),
            Self::TSTypeQuery(it) => it.span_mut(),
            Self::TSTypeReference(it) => it.span_mut(),
            Self::TSUnionType(it) => it.span_mut(),
            Self::TSParenthesizedType(it) => it.span_mut(),
            Self::JSDocNullableType(it) => it.span_mut(),
            Self::JSDocNonNullableType(it) => it.span_mut(),
            Self::JSDocUnknownType(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for TSConditionalType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSUnionType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSIntersectionType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSParenthesizedType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeOperator<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSArrayType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSIndexedAccessType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTupleType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSNamedTupleMember<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSOptionalType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSRestType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTupleElement<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSOptionalType(it) => it.span_mut(),
            Self::TSRestType(it) => it.span_mut(),
            Self::TSAnyKeyword(it) => it.span_mut(),
            Self::TSBigIntKeyword(it) => it.span_mut(),
            Self::TSBooleanKeyword(it) => it.span_mut(),
            Self::TSIntrinsicKeyword(it) => it.span_mut(),
            Self::TSNeverKeyword(it) => it.span_mut(),
            Self::TSNullKeyword(it) => it.span_mut(),
            Self::TSNumberKeyword(it) => it.span_mut(),
            Self::TSObjectKeyword(it) => it.span_mut(),
            Self::TSStringKeyword(it) => it.span_mut(),
            Self::TSSymbolKeyword(it) => it.span_mut(),
            Self::TSUndefinedKeyword(it) => it.span_mut(),
            Self::TSUnknownKeyword(it) => it.span_mut(),
            Self::TSVoidKeyword(it) => it.span_mut(),
            Self::TSArrayType(it) => it.span_mut(),
            Self::TSConditionalType(it) => it.span_mut(),
            Self::TSConstructorType(it) => it.span_mut(),
            Self::TSFunctionType(it) => it.span_mut(),
            Self::TSImportType(it) => it.span_mut(),
            Self::TSIndexedAccessType(it) => it.span_mut(),
            Self::TSInferType(it) => it.span_mut(),
            Self::TSIntersectionType(it) => it.span_mut(),
            Self::TSLiteralType(it) => it.span_mut(),
            Self::TSMappedType(it) => it.span_mut(),
            Self::TSNamedTupleMember(it) => it.span_mut(),
            Self::TSQualifiedName(it) => it.span_mut(),
            Self::TSTemplateLiteralType(it) => it.span_mut(),
            Self::TSThisType(it) => it.span_mut(),
            Self::TSTupleType(it) => it.span_mut(),
            Self::TSTypeLiteral(it) => it.span_mut(),
            Self::TSTypeOperatorType(it) => it.span_mut(),
            Self::TSTypePredicate(it) => it.span_mut(),
            Self::TSTypeQuery(it) => it.span_mut(),
            Self::TSTypeReference(it) => it.span_mut(),
            Self::TSUnionType(it) => it.span_mut(),
            Self::TSParenthesizedType(it) => it.span_mut(),
            Self::JSDocNullableType(it) => it.span_mut(),
            Self::JSDocNonNullableType(it) => it.span_mut(),
            Self::JSDocUnknownType(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for TSAnyKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSStringKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSBooleanKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSNumberKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSNeverKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSIntrinsicKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSUnknownKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSNullKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSUndefinedKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSVoidKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSSymbolKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSThisType {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSObjectKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSBigIntKeyword {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeReference<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeName<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::IdentifierReference(it) => it.span_mut(),
            Self::QualifiedName(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for TSQualifiedName<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeParameterInstantiation<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeParameter<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeParameterDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeAliasDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSClassImplements<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSInterfaceDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSInterfaceBody<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSPropertySignature<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSSignature<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSIndexSignature(it) => it.span_mut(),
            Self::TSPropertySignature(it) => it.span_mut(),
            Self::TSCallSignatureDeclaration(it) => it.span_mut(),
            Self::TSConstructSignatureDeclaration(it) => it.span_mut(),
            Self::TSMethodSignature(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for TSIndexSignature<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSCallSignatureDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSMethodSignature<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSConstructSignatureDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSIndexSignatureName<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSInterfaceHeritage<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypePredicate<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypePredicateName<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::This(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for TSModuleDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSModuleDeclarationName<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for TSModuleDeclarationBody<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSModuleDeclaration(it) => it.span_mut(),
            Self::TSModuleBlock(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for TSModuleBlock<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeLiteral<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSInferType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeQuery<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeQueryExprName<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSImportType(it) => it.span_mut(),
            Self::IdentifierReference(it) => it.span_mut(),
            Self::QualifiedName(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for TSImportType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSImportAttributes<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSImportAttribute<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSImportAttributeName<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for TSFunctionType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSConstructorType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSMappedType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTemplateLiteralType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSAsExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSSatisfiesExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSTypeAssertion<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSImportEqualsDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSModuleReference<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ExternalModuleReference(it) => it.span_mut(),
            Self::IdentifierReference(it) => it.span_mut(),
            Self::QualifiedName(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for TSExternalModuleReference<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSNonNullExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for Decorator<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSExportAssignment<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSNamespaceExportDeclaration<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for TSInstantiationExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSDocNullableType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSDocNonNullableType<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSDocUnknownType {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXElement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXOpeningElement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXClosingElement<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXFragment<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXOpeningFragment {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXClosingFragment {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXElementName<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::IdentifierReference(it) => it.span_mut(),
            Self::NamespacedName(it) => it.span_mut(),
            Self::MemberExpression(it) => it.span_mut(),
            Self::ThisExpression(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for JSXNamespacedName<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXMemberExpression<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXMemberExpressionObject<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::IdentifierReference(it) => it.span_mut(),
            Self::MemberExpression(it) => it.span_mut(),
            Self::ThisExpression(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for JSXExpressionContainer<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXExpression<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::EmptyExpression(it) => it.span_mut(),
            Self::BooleanLiteral(it) => it.span_mut(),
            Self::NullLiteral(it) => it.span_mut(),
            Self::NumericLiteral(it) => it.span_mut(),
            Self::BigIntLiteral(it) => it.span_mut(),
            Self::RegExpLiteral(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
            Self::TemplateLiteral(it) => it.span_mut(),
            Self::Identifier(it) => it.span_mut(),
            Self::MetaProperty(it) => it.span_mut(),
            Self::Super(it) => it.span_mut(),
            Self::ArrayExpression(it) => it.span_mut(),
            Self::ArrowFunctionExpression(it) => it.span_mut(),
            Self::AssignmentExpression(it) => it.span_mut(),
            Self::AwaitExpression(it) => it.span_mut(),
            Self::BinaryExpression(it) => it.span_mut(),
            Self::CallExpression(it) => it.span_mut(),
            Self::ChainExpression(it) => it.span_mut(),
            Self::ClassExpression(it) => it.span_mut(),
            Self::ConditionalExpression(it) => it.span_mut(),
            Self::FunctionExpression(it) => it.span_mut(),
            Self::ImportExpression(it) => it.span_mut(),
            Self::LogicalExpression(it) => it.span_mut(),
            Self::NewExpression(it) => it.span_mut(),
            Self::ObjectExpression(it) => it.span_mut(),
            Self::ParenthesizedExpression(it) => it.span_mut(),
            Self::SequenceExpression(it) => it.span_mut(),
            Self::TaggedTemplateExpression(it) => it.span_mut(),
            Self::ThisExpression(it) => it.span_mut(),
            Self::UnaryExpression(it) => it.span_mut(),
            Self::UpdateExpression(it) => it.span_mut(),
            Self::YieldExpression(it) => it.span_mut(),
            Self::PrivateInExpression(it) => it.span_mut(),
            Self::JSXElement(it) => it.span_mut(),
            Self::JSXFragment(it) => it.span_mut(),
            Self::TSAsExpression(it) => it.span_mut(),
            Self::TSSatisfiesExpression(it) => it.span_mut(),
            Self::TSTypeAssertion(it) => it.span_mut(),
            Self::TSNonNullExpression(it) => it.span_mut(),
            Self::TSInstantiationExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for JSXEmptyExpression {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXAttributeItem<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Attribute(it) => it.span_mut(),
            Self::SpreadAttribute(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for JSXAttribute<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXSpreadAttribute<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXAttributeName<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::NamespacedName(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for JSXAttributeValue<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::StringLiteral(it) => it.span_mut(),
            Self::ExpressionContainer(it) => it.span_mut(),
            Self::Element(it) => it.span_mut(),
            Self::Fragment(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for JSXIdentifier<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXChild<'_> {
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Text(it) => it.span_mut(),
            Self::Element(it) => it.span_mut(),
            Self::Fragment(it) => it.span_mut(),
            Self::ExpressionContainer(it) => it.span_mut(),
            Self::Spread(it) => it.span_mut(),
        }
    }
}

impl GetSpanMut for JSXSpreadChild<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl GetSpanMut for JSXText<'_> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}
