// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_codegen/src/generators/derive_get_span.rs`

#![allow(clippy::match_same_arms)]

use crate::ast::*;
use oxc_span::{GetSpanMut, Span};

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

impl<'a> GetSpanMut for NumericLiteral<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for BigIntLiteral<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for RegExpLiteral<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for StringLiteral<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for Program<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for Expression<'a> {
    #[inline]
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

impl<'a> GetSpanMut for IdentifierName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for IdentifierReference<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for BindingIdentifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for LabelIdentifier<'a> {
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

impl<'a> GetSpanMut for ArrayExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ArrayExpressionElement<'a> {
    #[inline]
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

impl<'a> GetSpanMut for ObjectExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ObjectPropertyKind<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ObjectProperty(it) => it.span_mut(),
            Self::SpreadProperty(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for ObjectProperty<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for PropertyKey<'a> {
    #[inline]
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

impl<'a> GetSpanMut for TemplateLiteral<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TaggedTemplateExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TemplateElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for MemberExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for ComputedMemberExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for StaticMemberExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for PrivateFieldExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for CallExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for NewExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for MetaProperty<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for SpreadElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for Argument<'a> {
    #[inline]
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

impl<'a> GetSpanMut for UpdateExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for UnaryExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for BinaryExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for PrivateInExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for LogicalExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ConditionalExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for AssignmentExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for AssignmentTarget<'a> {
    #[inline]
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

impl<'a> GetSpanMut for SimpleAssignmentTarget<'a> {
    #[inline]
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

impl<'a> GetSpanMut for AssignmentTargetPattern<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ArrayAssignmentTarget(it) => it.span_mut(),
            Self::ObjectAssignmentTarget(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for ArrayAssignmentTarget<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ObjectAssignmentTarget<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for AssignmentTargetRest<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for AssignmentTargetMaybeDefault<'a> {
    #[inline]
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

impl<'a> GetSpanMut for AssignmentTargetWithDefault<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for AssignmentTargetProperty<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => it.span_mut(),
            Self::AssignmentTargetPropertyProperty(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for AssignmentTargetPropertyIdentifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for AssignmentTargetPropertyProperty<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for SequenceExpression<'a> {
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

impl<'a> GetSpanMut for AwaitExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ChainExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ChainElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::CallExpression(it) => it.span_mut(),
            Self::ComputedMemberExpression(it) => it.span_mut(),
            Self::StaticMemberExpression(it) => it.span_mut(),
            Self::PrivateFieldExpression(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for ParenthesizedExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for Statement<'a> {
    #[inline]
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
            Self::UsingDeclaration(it) => it.span_mut(),
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

impl<'a> GetSpanMut for Directive<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for Hashbang<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for BlockStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for Declaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::VariableDeclaration(it) => it.span_mut(),
            Self::FunctionDeclaration(it) => it.span_mut(),
            Self::ClassDeclaration(it) => it.span_mut(),
            Self::UsingDeclaration(it) => it.span_mut(),
            Self::TSTypeAliasDeclaration(it) => it.span_mut(),
            Self::TSInterfaceDeclaration(it) => it.span_mut(),
            Self::TSEnumDeclaration(it) => it.span_mut(),
            Self::TSModuleDeclaration(it) => it.span_mut(),
            Self::TSImportEqualsDeclaration(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for VariableDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for VariableDeclarator<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for UsingDeclaration<'a> {
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

impl<'a> GetSpanMut for ExpressionStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for IfStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for DoWhileStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for WhileStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ForStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ForStatementInit<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::VariableDeclaration(it) => it.span_mut(),
            Self::UsingDeclaration(it) => it.span_mut(),
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

impl<'a> GetSpanMut for ForInStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ForStatementLeft<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::VariableDeclaration(it) => it.span_mut(),
            Self::UsingDeclaration(it) => it.span_mut(),
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

impl<'a> GetSpanMut for ForOfStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ContinueStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for BreakStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ReturnStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for WithStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for SwitchStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for SwitchCase<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for LabeledStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ThrowStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TryStatement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for CatchClause<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for CatchParameter<'a> {
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

impl<'a> GetSpanMut for BindingPattern<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        self.kind.span_mut()
    }
}

impl<'a> GetSpanMut for BindingPatternKind<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::BindingIdentifier(it) => it.span_mut(),
            Self::ObjectPattern(it) => it.span_mut(),
            Self::ArrayPattern(it) => it.span_mut(),
            Self::AssignmentPattern(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for AssignmentPattern<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ObjectPattern<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for BindingProperty<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ArrayPattern<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for BindingRestElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for Function<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for FormalParameters<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for FormalParameter<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for FunctionBody<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ArrowFunctionExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for YieldExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for Class<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ClassBody<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ClassElement<'a> {
    #[inline]
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

impl<'a> GetSpanMut for MethodDefinition<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for PropertyDefinition<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for PrivateIdentifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for StaticBlock<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ModuleDeclaration<'a> {
    #[inline]
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

impl<'a> GetSpanMut for AccessorProperty<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ImportExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ImportDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ImportDeclarationSpecifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ImportSpecifier(it) => it.span_mut(),
            Self::ImportDefaultSpecifier(it) => it.span_mut(),
            Self::ImportNamespaceSpecifier(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for ImportSpecifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ImportDefaultSpecifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ImportNamespaceSpecifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for WithClause<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ImportAttribute<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ImportAttributeKey<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for ExportNamedDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ExportDefaultDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ExportAllDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ExportSpecifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for ExportDefaultDeclarationKind<'a> {
    #[inline]
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

impl<'a> GetSpanMut for ModuleExportName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::IdentifierName(it) => it.span_mut(),
            Self::IdentifierReference(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for TSThisParameter<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSEnumDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSEnumMember<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSEnumMemberName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::StaticIdentifier(it) => it.span_mut(),
            Self::StaticStringLiteral(it) => it.span_mut(),
            Self::StaticTemplateLiteral(it) => it.span_mut(),
            Self::StaticNumericLiteral(it) => it.span_mut(),
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

impl<'a> GetSpanMut for TSTypeAnnotation<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSLiteralType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSLiteral<'a> {
    #[inline]
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

impl<'a> GetSpanMut for TSType<'a> {
    #[inline]
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

impl<'a> GetSpanMut for TSConditionalType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSUnionType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSIntersectionType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSParenthesizedType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeOperator<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSArrayType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSIndexedAccessType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTupleType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSNamedTupleMember<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSOptionalType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSRestType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTupleElement<'a> {
    #[inline]
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

impl<'a> GetSpanMut for TSTypeReference<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::IdentifierReference(it) => it.span_mut(),
            Self::QualifiedName(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for TSQualifiedName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeParameterInstantiation<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeParameter<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeParameterDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeAliasDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSClassImplements<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSInterfaceDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSInterfaceBody<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSPropertySignature<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSSignature<'a> {
    #[inline]
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

impl<'a> GetSpanMut for TSIndexSignature<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSCallSignatureDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSMethodSignature<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSConstructSignatureDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSIndexSignatureName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSInterfaceHeritage<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypePredicate<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypePredicateName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::This(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for TSModuleDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSModuleDeclarationName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for TSModuleDeclarationBody<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSModuleDeclaration(it) => it.span_mut(),
            Self::TSModuleBlock(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for TSModuleBlock<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeLiteral<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSInferType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeQuery<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeQueryExprName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::TSImportType(it) => it.span_mut(),
            Self::IdentifierReference(it) => it.span_mut(),
            Self::QualifiedName(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for TSImportType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSImportAttributes<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSImportAttribute<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSImportAttributeName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::StringLiteral(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for TSFunctionType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSConstructorType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSMappedType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTemplateLiteralType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSAsExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSSatisfiesExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSTypeAssertion<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSImportEqualsDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSModuleReference<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::ExternalModuleReference(it) => it.span_mut(),
            Self::IdentifierReference(it) => it.span_mut(),
            Self::QualifiedName(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for TSExternalModuleReference<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSNonNullExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for Decorator<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSExportAssignment<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSNamespaceExportDeclaration<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for TSInstantiationExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSDocNullableType<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSDocNonNullableType<'a> {
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

impl<'a> GetSpanMut for JSXElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXOpeningElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXClosingElement<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXFragment<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXElementName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::NamespacedName(it) => it.span_mut(),
            Self::MemberExpression(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for JSXNamespacedName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXMemberExpression<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXMemberExpressionObject<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::MemberExpression(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for JSXExpressionContainer<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXExpression<'a> {
    #[inline]
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

impl<'a> GetSpanMut for JSXAttributeItem<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Attribute(it) => it.span_mut(),
            Self::SpreadAttribute(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for JSXAttribute<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXSpreadAttribute<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXAttributeName<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::Identifier(it) => it.span_mut(),
            Self::NamespacedName(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for JSXAttributeValue<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        match self {
            Self::StringLiteral(it) => it.span_mut(),
            Self::ExpressionContainer(it) => it.span_mut(),
            Self::Element(it) => it.span_mut(),
            Self::Fragment(it) => it.span_mut(),
        }
    }
}

impl<'a> GetSpanMut for JSXIdentifier<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXChild<'a> {
    #[inline]
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

impl<'a> GetSpanMut for JSXSpreadChild<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl<'a> GetSpanMut for JSXText<'a> {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }
}
