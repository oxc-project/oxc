// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/get_span.rs`

#![allow(clippy::match_same_arms)]

use oxc_span::{GetSpan, Span};

use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
use crate::ast::ts::*;

impl GetSpan for BooleanLiteral {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for NullLiteral {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for NumericLiteral<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for StringLiteral<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for BigIntLiteral<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for RegExpLiteral<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Program<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Expression<'_> {
    fn span(&self) -> Span {
        match self {
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::Identifier(it) => it.span(),
            Self::MetaProperty(it) => it.span(),
            Self::Super(it) => it.span(),
            Self::ArrayExpression(it) => it.span(),
            Self::ArrowFunctionExpression(it) => it.span(),
            Self::AssignmentExpression(it) => it.span(),
            Self::AwaitExpression(it) => it.span(),
            Self::BinaryExpression(it) => it.span(),
            Self::CallExpression(it) => it.span(),
            Self::ChainExpression(it) => it.span(),
            Self::ClassExpression(it) => it.span(),
            Self::ConditionalExpression(it) => it.span(),
            Self::FunctionExpression(it) => it.span(),
            Self::ImportExpression(it) => it.span(),
            Self::LogicalExpression(it) => it.span(),
            Self::NewExpression(it) => it.span(),
            Self::ObjectExpression(it) => it.span(),
            Self::ParenthesizedExpression(it) => it.span(),
            Self::SequenceExpression(it) => it.span(),
            Self::TaggedTemplateExpression(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
            Self::UpdateExpression(it) => it.span(),
            Self::YieldExpression(it) => it.span(),
            Self::PrivateInExpression(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl GetSpan for IdentifierName<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for IdentifierReference<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for BindingIdentifier<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for LabelIdentifier<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ThisExpression {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ArrayExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ArrayExpressionElement<'_> {
    fn span(&self) -> Span {
        match self {
            Self::SpreadElement(it) => it.span(),
            Self::Elision(it) => it.span(),
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::Identifier(it) => it.span(),
            Self::MetaProperty(it) => it.span(),
            Self::Super(it) => it.span(),
            Self::ArrayExpression(it) => it.span(),
            Self::ArrowFunctionExpression(it) => it.span(),
            Self::AssignmentExpression(it) => it.span(),
            Self::AwaitExpression(it) => it.span(),
            Self::BinaryExpression(it) => it.span(),
            Self::CallExpression(it) => it.span(),
            Self::ChainExpression(it) => it.span(),
            Self::ClassExpression(it) => it.span(),
            Self::ConditionalExpression(it) => it.span(),
            Self::FunctionExpression(it) => it.span(),
            Self::ImportExpression(it) => it.span(),
            Self::LogicalExpression(it) => it.span(),
            Self::NewExpression(it) => it.span(),
            Self::ObjectExpression(it) => it.span(),
            Self::ParenthesizedExpression(it) => it.span(),
            Self::SequenceExpression(it) => it.span(),
            Self::TaggedTemplateExpression(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
            Self::UpdateExpression(it) => it.span(),
            Self::YieldExpression(it) => it.span(),
            Self::PrivateInExpression(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl GetSpan for Elision {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ObjectExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ObjectPropertyKind<'_> {
    fn span(&self) -> Span {
        match self {
            Self::ObjectProperty(it) => it.span(),
            Self::SpreadProperty(it) => it.span(),
        }
    }
}

impl GetSpan for ObjectProperty<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for PropertyKey<'_> {
    fn span(&self) -> Span {
        match self {
            Self::StaticIdentifier(it) => it.span(),
            Self::PrivateIdentifier(it) => it.span(),
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::Identifier(it) => it.span(),
            Self::MetaProperty(it) => it.span(),
            Self::Super(it) => it.span(),
            Self::ArrayExpression(it) => it.span(),
            Self::ArrowFunctionExpression(it) => it.span(),
            Self::AssignmentExpression(it) => it.span(),
            Self::AwaitExpression(it) => it.span(),
            Self::BinaryExpression(it) => it.span(),
            Self::CallExpression(it) => it.span(),
            Self::ChainExpression(it) => it.span(),
            Self::ClassExpression(it) => it.span(),
            Self::ConditionalExpression(it) => it.span(),
            Self::FunctionExpression(it) => it.span(),
            Self::ImportExpression(it) => it.span(),
            Self::LogicalExpression(it) => it.span(),
            Self::NewExpression(it) => it.span(),
            Self::ObjectExpression(it) => it.span(),
            Self::ParenthesizedExpression(it) => it.span(),
            Self::SequenceExpression(it) => it.span(),
            Self::TaggedTemplateExpression(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
            Self::UpdateExpression(it) => it.span(),
            Self::YieldExpression(it) => it.span(),
            Self::PrivateInExpression(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl GetSpan for TemplateLiteral<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TaggedTemplateExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TemplateElement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for MemberExpression<'_> {
    fn span(&self) -> Span {
        match self {
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl GetSpan for ComputedMemberExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for StaticMemberExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for PrivateFieldExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for CallExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for NewExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for MetaProperty<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for SpreadElement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Argument<'_> {
    fn span(&self) -> Span {
        match self {
            Self::SpreadElement(it) => it.span(),
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::Identifier(it) => it.span(),
            Self::MetaProperty(it) => it.span(),
            Self::Super(it) => it.span(),
            Self::ArrayExpression(it) => it.span(),
            Self::ArrowFunctionExpression(it) => it.span(),
            Self::AssignmentExpression(it) => it.span(),
            Self::AwaitExpression(it) => it.span(),
            Self::BinaryExpression(it) => it.span(),
            Self::CallExpression(it) => it.span(),
            Self::ChainExpression(it) => it.span(),
            Self::ClassExpression(it) => it.span(),
            Self::ConditionalExpression(it) => it.span(),
            Self::FunctionExpression(it) => it.span(),
            Self::ImportExpression(it) => it.span(),
            Self::LogicalExpression(it) => it.span(),
            Self::NewExpression(it) => it.span(),
            Self::ObjectExpression(it) => it.span(),
            Self::ParenthesizedExpression(it) => it.span(),
            Self::SequenceExpression(it) => it.span(),
            Self::TaggedTemplateExpression(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
            Self::UpdateExpression(it) => it.span(),
            Self::YieldExpression(it) => it.span(),
            Self::PrivateInExpression(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl GetSpan for UpdateExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for UnaryExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for BinaryExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for PrivateInExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for LogicalExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ConditionalExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for AssignmentExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for AssignmentTarget<'_> {
    fn span(&self) -> Span {
        match self {
            Self::AssignmentTargetIdentifier(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
            Self::ArrayAssignmentTarget(it) => it.span(),
            Self::ObjectAssignmentTarget(it) => it.span(),
        }
    }
}

impl GetSpan for SimpleAssignmentTarget<'_> {
    fn span(&self) -> Span {
        match self {
            Self::AssignmentTargetIdentifier(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl GetSpan for AssignmentTargetPattern<'_> {
    fn span(&self) -> Span {
        match self {
            Self::ArrayAssignmentTarget(it) => it.span(),
            Self::ObjectAssignmentTarget(it) => it.span(),
        }
    }
}

impl GetSpan for ArrayAssignmentTarget<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ObjectAssignmentTarget<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for AssignmentTargetRest<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for AssignmentTargetMaybeDefault<'_> {
    fn span(&self) -> Span {
        match self {
            Self::AssignmentTargetWithDefault(it) => it.span(),
            Self::AssignmentTargetIdentifier(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
            Self::ArrayAssignmentTarget(it) => it.span(),
            Self::ObjectAssignmentTarget(it) => it.span(),
        }
    }
}

impl GetSpan for AssignmentTargetWithDefault<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for AssignmentTargetProperty<'_> {
    fn span(&self) -> Span {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => it.span(),
            Self::AssignmentTargetPropertyProperty(it) => it.span(),
        }
    }
}

impl GetSpan for AssignmentTargetPropertyIdentifier<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for AssignmentTargetPropertyProperty<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for SequenceExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Super {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for AwaitExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ChainExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ChainElement<'_> {
    fn span(&self) -> Span {
        match self {
            Self::CallExpression(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl GetSpan for ParenthesizedExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Statement<'_> {
    fn span(&self) -> Span {
        match self {
            Self::BlockStatement(it) => it.span(),
            Self::BreakStatement(it) => it.span(),
            Self::ContinueStatement(it) => it.span(),
            Self::DebuggerStatement(it) => it.span(),
            Self::DoWhileStatement(it) => it.span(),
            Self::EmptyStatement(it) => it.span(),
            Self::ExpressionStatement(it) => it.span(),
            Self::ForInStatement(it) => it.span(),
            Self::ForOfStatement(it) => it.span(),
            Self::ForStatement(it) => it.span(),
            Self::IfStatement(it) => it.span(),
            Self::LabeledStatement(it) => it.span(),
            Self::ReturnStatement(it) => it.span(),
            Self::SwitchStatement(it) => it.span(),
            Self::ThrowStatement(it) => it.span(),
            Self::TryStatement(it) => it.span(),
            Self::WhileStatement(it) => it.span(),
            Self::WithStatement(it) => it.span(),
            Self::VariableDeclaration(it) => it.span(),
            Self::FunctionDeclaration(it) => it.span(),
            Self::ClassDeclaration(it) => it.span(),
            Self::TSTypeAliasDeclaration(it) => it.span(),
            Self::TSInterfaceDeclaration(it) => it.span(),
            Self::TSEnumDeclaration(it) => it.span(),
            Self::TSModuleDeclaration(it) => it.span(),
            Self::TSImportEqualsDeclaration(it) => it.span(),
            Self::ImportDeclaration(it) => it.span(),
            Self::ExportAllDeclaration(it) => it.span(),
            Self::ExportDefaultDeclaration(it) => it.span(),
            Self::ExportNamedDeclaration(it) => it.span(),
            Self::TSExportAssignment(it) => it.span(),
            Self::TSNamespaceExportDeclaration(it) => it.span(),
        }
    }
}

impl GetSpan for Directive<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Hashbang<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for BlockStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Declaration<'_> {
    fn span(&self) -> Span {
        match self {
            Self::VariableDeclaration(it) => it.span(),
            Self::FunctionDeclaration(it) => it.span(),
            Self::ClassDeclaration(it) => it.span(),
            Self::TSTypeAliasDeclaration(it) => it.span(),
            Self::TSInterfaceDeclaration(it) => it.span(),
            Self::TSEnumDeclaration(it) => it.span(),
            Self::TSModuleDeclaration(it) => it.span(),
            Self::TSImportEqualsDeclaration(it) => it.span(),
        }
    }
}

impl GetSpan for VariableDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for VariableDeclarator<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for EmptyStatement {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ExpressionStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for IfStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for DoWhileStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for WhileStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ForStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ForStatementInit<'_> {
    fn span(&self) -> Span {
        match self {
            Self::VariableDeclaration(it) => it.span(),
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::Identifier(it) => it.span(),
            Self::MetaProperty(it) => it.span(),
            Self::Super(it) => it.span(),
            Self::ArrayExpression(it) => it.span(),
            Self::ArrowFunctionExpression(it) => it.span(),
            Self::AssignmentExpression(it) => it.span(),
            Self::AwaitExpression(it) => it.span(),
            Self::BinaryExpression(it) => it.span(),
            Self::CallExpression(it) => it.span(),
            Self::ChainExpression(it) => it.span(),
            Self::ClassExpression(it) => it.span(),
            Self::ConditionalExpression(it) => it.span(),
            Self::FunctionExpression(it) => it.span(),
            Self::ImportExpression(it) => it.span(),
            Self::LogicalExpression(it) => it.span(),
            Self::NewExpression(it) => it.span(),
            Self::ObjectExpression(it) => it.span(),
            Self::ParenthesizedExpression(it) => it.span(),
            Self::SequenceExpression(it) => it.span(),
            Self::TaggedTemplateExpression(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
            Self::UpdateExpression(it) => it.span(),
            Self::YieldExpression(it) => it.span(),
            Self::PrivateInExpression(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl GetSpan for ForInStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ForStatementLeft<'_> {
    fn span(&self) -> Span {
        match self {
            Self::VariableDeclaration(it) => it.span(),
            Self::AssignmentTargetIdentifier(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
            Self::ArrayAssignmentTarget(it) => it.span(),
            Self::ObjectAssignmentTarget(it) => it.span(),
        }
    }
}

impl GetSpan for ForOfStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ContinueStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for BreakStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ReturnStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for WithStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for SwitchStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for SwitchCase<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for LabeledStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ThrowStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TryStatement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for CatchClause<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for CatchParameter<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for DebuggerStatement {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for BindingPattern<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.kind.span()
    }
}

impl GetSpan for BindingPatternKind<'_> {
    fn span(&self) -> Span {
        match self {
            Self::BindingIdentifier(it) => it.span(),
            Self::ObjectPattern(it) => it.span(),
            Self::ArrayPattern(it) => it.span(),
            Self::AssignmentPattern(it) => it.span(),
        }
    }
}

impl GetSpan for AssignmentPattern<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ObjectPattern<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for BindingProperty<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ArrayPattern<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for BindingRestElement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Function<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for FormalParameters<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for FormalParameter<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for FunctionBody<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ArrowFunctionExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for YieldExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Class<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ClassBody<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ClassElement<'_> {
    fn span(&self) -> Span {
        match self {
            Self::StaticBlock(it) => it.span(),
            Self::MethodDefinition(it) => it.span(),
            Self::PropertyDefinition(it) => it.span(),
            Self::AccessorProperty(it) => it.span(),
            Self::TSIndexSignature(it) => it.span(),
        }
    }
}

impl GetSpan for MethodDefinition<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for PropertyDefinition<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for PrivateIdentifier<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for StaticBlock<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ModuleDeclaration<'_> {
    fn span(&self) -> Span {
        match self {
            Self::ImportDeclaration(it) => it.span(),
            Self::ExportAllDeclaration(it) => it.span(),
            Self::ExportDefaultDeclaration(it) => it.span(),
            Self::ExportNamedDeclaration(it) => it.span(),
            Self::TSExportAssignment(it) => it.span(),
            Self::TSNamespaceExportDeclaration(it) => it.span(),
        }
    }
}

impl GetSpan for AccessorProperty<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ImportExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ImportDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ImportDeclarationSpecifier<'_> {
    fn span(&self) -> Span {
        match self {
            Self::ImportSpecifier(it) => it.span(),
            Self::ImportDefaultSpecifier(it) => it.span(),
            Self::ImportNamespaceSpecifier(it) => it.span(),
        }
    }
}

impl GetSpan for ImportSpecifier<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ImportDefaultSpecifier<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ImportNamespaceSpecifier<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for WithClause<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ImportAttribute<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ImportAttributeKey<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
        }
    }
}

impl GetSpan for ExportNamedDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ExportDefaultDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ExportAllDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ExportSpecifier<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for ExportDefaultDeclarationKind<'_> {
    fn span(&self) -> Span {
        match self {
            Self::FunctionDeclaration(it) => it.span(),
            Self::ClassDeclaration(it) => it.span(),
            Self::TSInterfaceDeclaration(it) => it.span(),
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::Identifier(it) => it.span(),
            Self::MetaProperty(it) => it.span(),
            Self::Super(it) => it.span(),
            Self::ArrayExpression(it) => it.span(),
            Self::ArrowFunctionExpression(it) => it.span(),
            Self::AssignmentExpression(it) => it.span(),
            Self::AwaitExpression(it) => it.span(),
            Self::BinaryExpression(it) => it.span(),
            Self::CallExpression(it) => it.span(),
            Self::ChainExpression(it) => it.span(),
            Self::ClassExpression(it) => it.span(),
            Self::ConditionalExpression(it) => it.span(),
            Self::FunctionExpression(it) => it.span(),
            Self::ImportExpression(it) => it.span(),
            Self::LogicalExpression(it) => it.span(),
            Self::NewExpression(it) => it.span(),
            Self::ObjectExpression(it) => it.span(),
            Self::ParenthesizedExpression(it) => it.span(),
            Self::SequenceExpression(it) => it.span(),
            Self::TaggedTemplateExpression(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
            Self::UpdateExpression(it) => it.span(),
            Self::YieldExpression(it) => it.span(),
            Self::PrivateInExpression(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl GetSpan for ModuleExportName<'_> {
    fn span(&self) -> Span {
        match self {
            Self::IdentifierName(it) => it.span(),
            Self::IdentifierReference(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
        }
    }
}

impl GetSpan for TSThisParameter<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSEnumDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSEnumMember<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSEnumMemberName<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => it.span(),
            Self::String(it) => it.span(),
        }
    }
}

impl GetSpan for TSTypeAnnotation<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSLiteralType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSLiteral<'_> {
    fn span(&self) -> Span {
        match self {
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
        }
    }
}

impl GetSpan for TSType<'_> {
    fn span(&self) -> Span {
        match self {
            Self::TSAnyKeyword(it) => it.span(),
            Self::TSBigIntKeyword(it) => it.span(),
            Self::TSBooleanKeyword(it) => it.span(),
            Self::TSIntrinsicKeyword(it) => it.span(),
            Self::TSNeverKeyword(it) => it.span(),
            Self::TSNullKeyword(it) => it.span(),
            Self::TSNumberKeyword(it) => it.span(),
            Self::TSObjectKeyword(it) => it.span(),
            Self::TSStringKeyword(it) => it.span(),
            Self::TSSymbolKeyword(it) => it.span(),
            Self::TSUndefinedKeyword(it) => it.span(),
            Self::TSUnknownKeyword(it) => it.span(),
            Self::TSVoidKeyword(it) => it.span(),
            Self::TSArrayType(it) => it.span(),
            Self::TSConditionalType(it) => it.span(),
            Self::TSConstructorType(it) => it.span(),
            Self::TSFunctionType(it) => it.span(),
            Self::TSImportType(it) => it.span(),
            Self::TSIndexedAccessType(it) => it.span(),
            Self::TSInferType(it) => it.span(),
            Self::TSIntersectionType(it) => it.span(),
            Self::TSLiteralType(it) => it.span(),
            Self::TSMappedType(it) => it.span(),
            Self::TSNamedTupleMember(it) => it.span(),
            Self::TSQualifiedName(it) => it.span(),
            Self::TSTemplateLiteralType(it) => it.span(),
            Self::TSThisType(it) => it.span(),
            Self::TSTupleType(it) => it.span(),
            Self::TSTypeLiteral(it) => it.span(),
            Self::TSTypeOperatorType(it) => it.span(),
            Self::TSTypePredicate(it) => it.span(),
            Self::TSTypeQuery(it) => it.span(),
            Self::TSTypeReference(it) => it.span(),
            Self::TSUnionType(it) => it.span(),
            Self::TSParenthesizedType(it) => it.span(),
            Self::JSDocNullableType(it) => it.span(),
            Self::JSDocNonNullableType(it) => it.span(),
            Self::JSDocUnknownType(it) => it.span(),
        }
    }
}

impl GetSpan for TSConditionalType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSUnionType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSIntersectionType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSParenthesizedType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeOperator<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSArrayType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSIndexedAccessType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTupleType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSNamedTupleMember<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSOptionalType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSRestType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTupleElement<'_> {
    fn span(&self) -> Span {
        match self {
            Self::TSOptionalType(it) => it.span(),
            Self::TSRestType(it) => it.span(),
            Self::TSAnyKeyword(it) => it.span(),
            Self::TSBigIntKeyword(it) => it.span(),
            Self::TSBooleanKeyword(it) => it.span(),
            Self::TSIntrinsicKeyword(it) => it.span(),
            Self::TSNeverKeyword(it) => it.span(),
            Self::TSNullKeyword(it) => it.span(),
            Self::TSNumberKeyword(it) => it.span(),
            Self::TSObjectKeyword(it) => it.span(),
            Self::TSStringKeyword(it) => it.span(),
            Self::TSSymbolKeyword(it) => it.span(),
            Self::TSUndefinedKeyword(it) => it.span(),
            Self::TSUnknownKeyword(it) => it.span(),
            Self::TSVoidKeyword(it) => it.span(),
            Self::TSArrayType(it) => it.span(),
            Self::TSConditionalType(it) => it.span(),
            Self::TSConstructorType(it) => it.span(),
            Self::TSFunctionType(it) => it.span(),
            Self::TSImportType(it) => it.span(),
            Self::TSIndexedAccessType(it) => it.span(),
            Self::TSInferType(it) => it.span(),
            Self::TSIntersectionType(it) => it.span(),
            Self::TSLiteralType(it) => it.span(),
            Self::TSMappedType(it) => it.span(),
            Self::TSNamedTupleMember(it) => it.span(),
            Self::TSQualifiedName(it) => it.span(),
            Self::TSTemplateLiteralType(it) => it.span(),
            Self::TSThisType(it) => it.span(),
            Self::TSTupleType(it) => it.span(),
            Self::TSTypeLiteral(it) => it.span(),
            Self::TSTypeOperatorType(it) => it.span(),
            Self::TSTypePredicate(it) => it.span(),
            Self::TSTypeQuery(it) => it.span(),
            Self::TSTypeReference(it) => it.span(),
            Self::TSUnionType(it) => it.span(),
            Self::TSParenthesizedType(it) => it.span(),
            Self::JSDocNullableType(it) => it.span(),
            Self::JSDocNonNullableType(it) => it.span(),
            Self::JSDocUnknownType(it) => it.span(),
        }
    }
}

impl GetSpan for TSAnyKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSStringKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSBooleanKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSNumberKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSNeverKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSIntrinsicKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSUnknownKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSNullKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSUndefinedKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSVoidKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSSymbolKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSThisType {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSObjectKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSBigIntKeyword {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeReference<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeName<'_> {
    fn span(&self) -> Span {
        match self {
            Self::IdentifierReference(it) => it.span(),
            Self::QualifiedName(it) => it.span(),
        }
    }
}

impl GetSpan for TSQualifiedName<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeParameterInstantiation<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeParameter<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeParameterDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeAliasDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSClassImplements<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSInterfaceDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSInterfaceBody<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSPropertySignature<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSSignature<'_> {
    fn span(&self) -> Span {
        match self {
            Self::TSIndexSignature(it) => it.span(),
            Self::TSPropertySignature(it) => it.span(),
            Self::TSCallSignatureDeclaration(it) => it.span(),
            Self::TSConstructSignatureDeclaration(it) => it.span(),
            Self::TSMethodSignature(it) => it.span(),
        }
    }
}

impl GetSpan for TSIndexSignature<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSCallSignatureDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSMethodSignature<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSConstructSignatureDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSIndexSignatureName<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSInterfaceHeritage<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypePredicate<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypePredicateName<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => it.span(),
            Self::This(it) => it.span(),
        }
    }
}

impl GetSpan for TSModuleDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSModuleDeclarationName<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
        }
    }
}

impl GetSpan for TSModuleDeclarationBody<'_> {
    fn span(&self) -> Span {
        match self {
            Self::TSModuleDeclaration(it) => it.span(),
            Self::TSModuleBlock(it) => it.span(),
        }
    }
}

impl GetSpan for TSModuleBlock<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeLiteral<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSInferType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeQuery<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeQueryExprName<'_> {
    fn span(&self) -> Span {
        match self {
            Self::TSImportType(it) => it.span(),
            Self::IdentifierReference(it) => it.span(),
            Self::QualifiedName(it) => it.span(),
        }
    }
}

impl GetSpan for TSImportType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSImportAttributes<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSImportAttribute<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSImportAttributeName<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
        }
    }
}

impl GetSpan for TSFunctionType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSConstructorType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSMappedType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTemplateLiteralType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSAsExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSSatisfiesExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSTypeAssertion<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSImportEqualsDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSModuleReference<'_> {
    fn span(&self) -> Span {
        match self {
            Self::ExternalModuleReference(it) => it.span(),
            Self::IdentifierReference(it) => it.span(),
            Self::QualifiedName(it) => it.span(),
        }
    }
}

impl GetSpan for TSExternalModuleReference<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSNonNullExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for Decorator<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSExportAssignment<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSNamespaceExportDeclaration<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for TSInstantiationExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSDocNullableType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSDocNonNullableType<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSDocUnknownType {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXElement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXOpeningElement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXClosingElement<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXFragment<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXOpeningFragment {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXClosingFragment {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXElementName<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => it.span(),
            Self::IdentifierReference(it) => it.span(),
            Self::NamespacedName(it) => it.span(),
            Self::MemberExpression(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
        }
    }
}

impl GetSpan for JSXNamespacedName<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXMemberExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXMemberExpressionObject<'_> {
    fn span(&self) -> Span {
        match self {
            Self::IdentifierReference(it) => it.span(),
            Self::MemberExpression(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
        }
    }
}

impl GetSpan for JSXExpressionContainer<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXExpression<'_> {
    fn span(&self) -> Span {
        match self {
            Self::EmptyExpression(it) => it.span(),
            Self::BooleanLiteral(it) => it.span(),
            Self::NullLiteral(it) => it.span(),
            Self::NumericLiteral(it) => it.span(),
            Self::BigIntLiteral(it) => it.span(),
            Self::RegExpLiteral(it) => it.span(),
            Self::StringLiteral(it) => it.span(),
            Self::TemplateLiteral(it) => it.span(),
            Self::Identifier(it) => it.span(),
            Self::MetaProperty(it) => it.span(),
            Self::Super(it) => it.span(),
            Self::ArrayExpression(it) => it.span(),
            Self::ArrowFunctionExpression(it) => it.span(),
            Self::AssignmentExpression(it) => it.span(),
            Self::AwaitExpression(it) => it.span(),
            Self::BinaryExpression(it) => it.span(),
            Self::CallExpression(it) => it.span(),
            Self::ChainExpression(it) => it.span(),
            Self::ClassExpression(it) => it.span(),
            Self::ConditionalExpression(it) => it.span(),
            Self::FunctionExpression(it) => it.span(),
            Self::ImportExpression(it) => it.span(),
            Self::LogicalExpression(it) => it.span(),
            Self::NewExpression(it) => it.span(),
            Self::ObjectExpression(it) => it.span(),
            Self::ParenthesizedExpression(it) => it.span(),
            Self::SequenceExpression(it) => it.span(),
            Self::TaggedTemplateExpression(it) => it.span(),
            Self::ThisExpression(it) => it.span(),
            Self::UnaryExpression(it) => it.span(),
            Self::UpdateExpression(it) => it.span(),
            Self::YieldExpression(it) => it.span(),
            Self::PrivateInExpression(it) => it.span(),
            Self::JSXElement(it) => it.span(),
            Self::JSXFragment(it) => it.span(),
            Self::TSAsExpression(it) => it.span(),
            Self::TSSatisfiesExpression(it) => it.span(),
            Self::TSTypeAssertion(it) => it.span(),
            Self::TSNonNullExpression(it) => it.span(),
            Self::TSInstantiationExpression(it) => it.span(),
            Self::ComputedMemberExpression(it) => it.span(),
            Self::StaticMemberExpression(it) => it.span(),
            Self::PrivateFieldExpression(it) => it.span(),
        }
    }
}

impl GetSpan for JSXEmptyExpression {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXAttributeItem<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Attribute(it) => it.span(),
            Self::SpreadAttribute(it) => it.span(),
        }
    }
}

impl GetSpan for JSXAttribute<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXSpreadAttribute<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXAttributeName<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Identifier(it) => it.span(),
            Self::NamespacedName(it) => it.span(),
        }
    }
}

impl GetSpan for JSXAttributeValue<'_> {
    fn span(&self) -> Span {
        match self {
            Self::StringLiteral(it) => it.span(),
            Self::ExpressionContainer(it) => it.span(),
            Self::Element(it) => it.span(),
            Self::Fragment(it) => it.span(),
        }
    }
}

impl GetSpan for JSXIdentifier<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXChild<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Text(it) => it.span(),
            Self::Element(it) => it.span(),
            Self::Fragment(it) => it.span(),
            Self::ExpressionContainer(it) => it.span(),
            Self::Spread(it) => it.span(),
        }
    }
}

impl GetSpan for JSXSpreadChild<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl GetSpan for JSXText<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
