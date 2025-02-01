// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/get_address.rs`

#![allow(clippy::match_same_arms)]

use oxc_allocator::{Address, GetAddress};

use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::ts::*;

impl GetAddress for Expression<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::BooleanLiteral(it) => GetAddress::address(it),
            Self::NullLiteral(it) => GetAddress::address(it),
            Self::NumericLiteral(it) => GetAddress::address(it),
            Self::BigIntLiteral(it) => GetAddress::address(it),
            Self::RegExpLiteral(it) => GetAddress::address(it),
            Self::StringLiteral(it) => GetAddress::address(it),
            Self::TemplateLiteral(it) => GetAddress::address(it),
            Self::Identifier(it) => GetAddress::address(it),
            Self::MetaProperty(it) => GetAddress::address(it),
            Self::Super(it) => GetAddress::address(it),
            Self::ArrayExpression(it) => GetAddress::address(it),
            Self::ArrowFunctionExpression(it) => GetAddress::address(it),
            Self::AssignmentExpression(it) => GetAddress::address(it),
            Self::AwaitExpression(it) => GetAddress::address(it),
            Self::BinaryExpression(it) => GetAddress::address(it),
            Self::CallExpression(it) => GetAddress::address(it),
            Self::ChainExpression(it) => GetAddress::address(it),
            Self::ClassExpression(it) => GetAddress::address(it),
            Self::ConditionalExpression(it) => GetAddress::address(it),
            Self::FunctionExpression(it) => GetAddress::address(it),
            Self::ImportExpression(it) => GetAddress::address(it),
            Self::LogicalExpression(it) => GetAddress::address(it),
            Self::NewExpression(it) => GetAddress::address(it),
            Self::ObjectExpression(it) => GetAddress::address(it),
            Self::ParenthesizedExpression(it) => GetAddress::address(it),
            Self::SequenceExpression(it) => GetAddress::address(it),
            Self::TaggedTemplateExpression(it) => GetAddress::address(it),
            Self::ThisExpression(it) => GetAddress::address(it),
            Self::UnaryExpression(it) => GetAddress::address(it),
            Self::UpdateExpression(it) => GetAddress::address(it),
            Self::YieldExpression(it) => GetAddress::address(it),
            Self::PrivateInExpression(it) => GetAddress::address(it),
            Self::JSXElement(it) => GetAddress::address(it),
            Self::JSXFragment(it) => GetAddress::address(it),
            Self::TSAsExpression(it) => GetAddress::address(it),
            Self::TSSatisfiesExpression(it) => GetAddress::address(it),
            Self::TSTypeAssertion(it) => GetAddress::address(it),
            Self::TSNonNullExpression(it) => GetAddress::address(it),
            Self::TSInstantiationExpression(it) => GetAddress::address(it),
            Self::ComputedMemberExpression(it) => GetAddress::address(it),
            Self::StaticMemberExpression(it) => GetAddress::address(it),
            Self::PrivateFieldExpression(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for ObjectPropertyKind<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::ObjectProperty(it) => GetAddress::address(it),
            Self::SpreadProperty(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for PropertyKey<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::StaticIdentifier(it) => GetAddress::address(it),
            Self::PrivateIdentifier(it) => GetAddress::address(it),
            Self::BooleanLiteral(it) => GetAddress::address(it),
            Self::NullLiteral(it) => GetAddress::address(it),
            Self::NumericLiteral(it) => GetAddress::address(it),
            Self::BigIntLiteral(it) => GetAddress::address(it),
            Self::RegExpLiteral(it) => GetAddress::address(it),
            Self::StringLiteral(it) => GetAddress::address(it),
            Self::TemplateLiteral(it) => GetAddress::address(it),
            Self::Identifier(it) => GetAddress::address(it),
            Self::MetaProperty(it) => GetAddress::address(it),
            Self::Super(it) => GetAddress::address(it),
            Self::ArrayExpression(it) => GetAddress::address(it),
            Self::ArrowFunctionExpression(it) => GetAddress::address(it),
            Self::AssignmentExpression(it) => GetAddress::address(it),
            Self::AwaitExpression(it) => GetAddress::address(it),
            Self::BinaryExpression(it) => GetAddress::address(it),
            Self::CallExpression(it) => GetAddress::address(it),
            Self::ChainExpression(it) => GetAddress::address(it),
            Self::ClassExpression(it) => GetAddress::address(it),
            Self::ConditionalExpression(it) => GetAddress::address(it),
            Self::FunctionExpression(it) => GetAddress::address(it),
            Self::ImportExpression(it) => GetAddress::address(it),
            Self::LogicalExpression(it) => GetAddress::address(it),
            Self::NewExpression(it) => GetAddress::address(it),
            Self::ObjectExpression(it) => GetAddress::address(it),
            Self::ParenthesizedExpression(it) => GetAddress::address(it),
            Self::SequenceExpression(it) => GetAddress::address(it),
            Self::TaggedTemplateExpression(it) => GetAddress::address(it),
            Self::ThisExpression(it) => GetAddress::address(it),
            Self::UnaryExpression(it) => GetAddress::address(it),
            Self::UpdateExpression(it) => GetAddress::address(it),
            Self::YieldExpression(it) => GetAddress::address(it),
            Self::PrivateInExpression(it) => GetAddress::address(it),
            Self::JSXElement(it) => GetAddress::address(it),
            Self::JSXFragment(it) => GetAddress::address(it),
            Self::TSAsExpression(it) => GetAddress::address(it),
            Self::TSSatisfiesExpression(it) => GetAddress::address(it),
            Self::TSTypeAssertion(it) => GetAddress::address(it),
            Self::TSNonNullExpression(it) => GetAddress::address(it),
            Self::TSInstantiationExpression(it) => GetAddress::address(it),
            Self::ComputedMemberExpression(it) => GetAddress::address(it),
            Self::StaticMemberExpression(it) => GetAddress::address(it),
            Self::PrivateFieldExpression(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for MemberExpression<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::ComputedMemberExpression(it) => GetAddress::address(it),
            Self::StaticMemberExpression(it) => GetAddress::address(it),
            Self::PrivateFieldExpression(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for Argument<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::SpreadElement(it) => GetAddress::address(it),
            Self::BooleanLiteral(it) => GetAddress::address(it),
            Self::NullLiteral(it) => GetAddress::address(it),
            Self::NumericLiteral(it) => GetAddress::address(it),
            Self::BigIntLiteral(it) => GetAddress::address(it),
            Self::RegExpLiteral(it) => GetAddress::address(it),
            Self::StringLiteral(it) => GetAddress::address(it),
            Self::TemplateLiteral(it) => GetAddress::address(it),
            Self::Identifier(it) => GetAddress::address(it),
            Self::MetaProperty(it) => GetAddress::address(it),
            Self::Super(it) => GetAddress::address(it),
            Self::ArrayExpression(it) => GetAddress::address(it),
            Self::ArrowFunctionExpression(it) => GetAddress::address(it),
            Self::AssignmentExpression(it) => GetAddress::address(it),
            Self::AwaitExpression(it) => GetAddress::address(it),
            Self::BinaryExpression(it) => GetAddress::address(it),
            Self::CallExpression(it) => GetAddress::address(it),
            Self::ChainExpression(it) => GetAddress::address(it),
            Self::ClassExpression(it) => GetAddress::address(it),
            Self::ConditionalExpression(it) => GetAddress::address(it),
            Self::FunctionExpression(it) => GetAddress::address(it),
            Self::ImportExpression(it) => GetAddress::address(it),
            Self::LogicalExpression(it) => GetAddress::address(it),
            Self::NewExpression(it) => GetAddress::address(it),
            Self::ObjectExpression(it) => GetAddress::address(it),
            Self::ParenthesizedExpression(it) => GetAddress::address(it),
            Self::SequenceExpression(it) => GetAddress::address(it),
            Self::TaggedTemplateExpression(it) => GetAddress::address(it),
            Self::ThisExpression(it) => GetAddress::address(it),
            Self::UnaryExpression(it) => GetAddress::address(it),
            Self::UpdateExpression(it) => GetAddress::address(it),
            Self::YieldExpression(it) => GetAddress::address(it),
            Self::PrivateInExpression(it) => GetAddress::address(it),
            Self::JSXElement(it) => GetAddress::address(it),
            Self::JSXFragment(it) => GetAddress::address(it),
            Self::TSAsExpression(it) => GetAddress::address(it),
            Self::TSSatisfiesExpression(it) => GetAddress::address(it),
            Self::TSTypeAssertion(it) => GetAddress::address(it),
            Self::TSNonNullExpression(it) => GetAddress::address(it),
            Self::TSInstantiationExpression(it) => GetAddress::address(it),
            Self::ComputedMemberExpression(it) => GetAddress::address(it),
            Self::StaticMemberExpression(it) => GetAddress::address(it),
            Self::PrivateFieldExpression(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for AssignmentTarget<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::AssignmentTargetIdentifier(it) => GetAddress::address(it),
            Self::TSAsExpression(it) => GetAddress::address(it),
            Self::TSSatisfiesExpression(it) => GetAddress::address(it),
            Self::TSNonNullExpression(it) => GetAddress::address(it),
            Self::TSTypeAssertion(it) => GetAddress::address(it),
            Self::TSInstantiationExpression(it) => GetAddress::address(it),
            Self::ComputedMemberExpression(it) => GetAddress::address(it),
            Self::StaticMemberExpression(it) => GetAddress::address(it),
            Self::PrivateFieldExpression(it) => GetAddress::address(it),
            Self::ArrayAssignmentTarget(it) => GetAddress::address(it),
            Self::ObjectAssignmentTarget(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for SimpleAssignmentTarget<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::AssignmentTargetIdentifier(it) => GetAddress::address(it),
            Self::TSAsExpression(it) => GetAddress::address(it),
            Self::TSSatisfiesExpression(it) => GetAddress::address(it),
            Self::TSNonNullExpression(it) => GetAddress::address(it),
            Self::TSTypeAssertion(it) => GetAddress::address(it),
            Self::TSInstantiationExpression(it) => GetAddress::address(it),
            Self::ComputedMemberExpression(it) => GetAddress::address(it),
            Self::StaticMemberExpression(it) => GetAddress::address(it),
            Self::PrivateFieldExpression(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for AssignmentTargetPattern<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::ArrayAssignmentTarget(it) => GetAddress::address(it),
            Self::ObjectAssignmentTarget(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for AssignmentTargetMaybeDefault<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::AssignmentTargetWithDefault(it) => GetAddress::address(it),
            Self::AssignmentTargetIdentifier(it) => GetAddress::address(it),
            Self::TSAsExpression(it) => GetAddress::address(it),
            Self::TSSatisfiesExpression(it) => GetAddress::address(it),
            Self::TSNonNullExpression(it) => GetAddress::address(it),
            Self::TSTypeAssertion(it) => GetAddress::address(it),
            Self::TSInstantiationExpression(it) => GetAddress::address(it),
            Self::ComputedMemberExpression(it) => GetAddress::address(it),
            Self::StaticMemberExpression(it) => GetAddress::address(it),
            Self::PrivateFieldExpression(it) => GetAddress::address(it),
            Self::ArrayAssignmentTarget(it) => GetAddress::address(it),
            Self::ObjectAssignmentTarget(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for AssignmentTargetProperty<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => GetAddress::address(it),
            Self::AssignmentTargetPropertyProperty(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for ChainElement<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::CallExpression(it) => GetAddress::address(it),
            Self::TSNonNullExpression(it) => GetAddress::address(it),
            Self::ComputedMemberExpression(it) => GetAddress::address(it),
            Self::StaticMemberExpression(it) => GetAddress::address(it),
            Self::PrivateFieldExpression(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for Statement<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::BlockStatement(it) => GetAddress::address(it),
            Self::BreakStatement(it) => GetAddress::address(it),
            Self::ContinueStatement(it) => GetAddress::address(it),
            Self::DebuggerStatement(it) => GetAddress::address(it),
            Self::DoWhileStatement(it) => GetAddress::address(it),
            Self::EmptyStatement(it) => GetAddress::address(it),
            Self::ExpressionStatement(it) => GetAddress::address(it),
            Self::ForInStatement(it) => GetAddress::address(it),
            Self::ForOfStatement(it) => GetAddress::address(it),
            Self::ForStatement(it) => GetAddress::address(it),
            Self::IfStatement(it) => GetAddress::address(it),
            Self::LabeledStatement(it) => GetAddress::address(it),
            Self::ReturnStatement(it) => GetAddress::address(it),
            Self::SwitchStatement(it) => GetAddress::address(it),
            Self::ThrowStatement(it) => GetAddress::address(it),
            Self::TryStatement(it) => GetAddress::address(it),
            Self::WhileStatement(it) => GetAddress::address(it),
            Self::WithStatement(it) => GetAddress::address(it),
            Self::VariableDeclaration(it) => GetAddress::address(it),
            Self::FunctionDeclaration(it) => GetAddress::address(it),
            Self::ClassDeclaration(it) => GetAddress::address(it),
            Self::TSTypeAliasDeclaration(it) => GetAddress::address(it),
            Self::TSInterfaceDeclaration(it) => GetAddress::address(it),
            Self::TSEnumDeclaration(it) => GetAddress::address(it),
            Self::TSModuleDeclaration(it) => GetAddress::address(it),
            Self::TSImportEqualsDeclaration(it) => GetAddress::address(it),
            Self::ImportDeclaration(it) => GetAddress::address(it),
            Self::ExportAllDeclaration(it) => GetAddress::address(it),
            Self::ExportDefaultDeclaration(it) => GetAddress::address(it),
            Self::ExportNamedDeclaration(it) => GetAddress::address(it),
            Self::TSExportAssignment(it) => GetAddress::address(it),
            Self::TSNamespaceExportDeclaration(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for Declaration<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::VariableDeclaration(it) => GetAddress::address(it),
            Self::FunctionDeclaration(it) => GetAddress::address(it),
            Self::ClassDeclaration(it) => GetAddress::address(it),
            Self::TSTypeAliasDeclaration(it) => GetAddress::address(it),
            Self::TSInterfaceDeclaration(it) => GetAddress::address(it),
            Self::TSEnumDeclaration(it) => GetAddress::address(it),
            Self::TSModuleDeclaration(it) => GetAddress::address(it),
            Self::TSImportEqualsDeclaration(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for ForStatementInit<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::VariableDeclaration(it) => GetAddress::address(it),
            Self::BooleanLiteral(it) => GetAddress::address(it),
            Self::NullLiteral(it) => GetAddress::address(it),
            Self::NumericLiteral(it) => GetAddress::address(it),
            Self::BigIntLiteral(it) => GetAddress::address(it),
            Self::RegExpLiteral(it) => GetAddress::address(it),
            Self::StringLiteral(it) => GetAddress::address(it),
            Self::TemplateLiteral(it) => GetAddress::address(it),
            Self::Identifier(it) => GetAddress::address(it),
            Self::MetaProperty(it) => GetAddress::address(it),
            Self::Super(it) => GetAddress::address(it),
            Self::ArrayExpression(it) => GetAddress::address(it),
            Self::ArrowFunctionExpression(it) => GetAddress::address(it),
            Self::AssignmentExpression(it) => GetAddress::address(it),
            Self::AwaitExpression(it) => GetAddress::address(it),
            Self::BinaryExpression(it) => GetAddress::address(it),
            Self::CallExpression(it) => GetAddress::address(it),
            Self::ChainExpression(it) => GetAddress::address(it),
            Self::ClassExpression(it) => GetAddress::address(it),
            Self::ConditionalExpression(it) => GetAddress::address(it),
            Self::FunctionExpression(it) => GetAddress::address(it),
            Self::ImportExpression(it) => GetAddress::address(it),
            Self::LogicalExpression(it) => GetAddress::address(it),
            Self::NewExpression(it) => GetAddress::address(it),
            Self::ObjectExpression(it) => GetAddress::address(it),
            Self::ParenthesizedExpression(it) => GetAddress::address(it),
            Self::SequenceExpression(it) => GetAddress::address(it),
            Self::TaggedTemplateExpression(it) => GetAddress::address(it),
            Self::ThisExpression(it) => GetAddress::address(it),
            Self::UnaryExpression(it) => GetAddress::address(it),
            Self::UpdateExpression(it) => GetAddress::address(it),
            Self::YieldExpression(it) => GetAddress::address(it),
            Self::PrivateInExpression(it) => GetAddress::address(it),
            Self::JSXElement(it) => GetAddress::address(it),
            Self::JSXFragment(it) => GetAddress::address(it),
            Self::TSAsExpression(it) => GetAddress::address(it),
            Self::TSSatisfiesExpression(it) => GetAddress::address(it),
            Self::TSTypeAssertion(it) => GetAddress::address(it),
            Self::TSNonNullExpression(it) => GetAddress::address(it),
            Self::TSInstantiationExpression(it) => GetAddress::address(it),
            Self::ComputedMemberExpression(it) => GetAddress::address(it),
            Self::StaticMemberExpression(it) => GetAddress::address(it),
            Self::PrivateFieldExpression(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for ForStatementLeft<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::VariableDeclaration(it) => GetAddress::address(it),
            Self::AssignmentTargetIdentifier(it) => GetAddress::address(it),
            Self::TSAsExpression(it) => GetAddress::address(it),
            Self::TSSatisfiesExpression(it) => GetAddress::address(it),
            Self::TSNonNullExpression(it) => GetAddress::address(it),
            Self::TSTypeAssertion(it) => GetAddress::address(it),
            Self::TSInstantiationExpression(it) => GetAddress::address(it),
            Self::ComputedMemberExpression(it) => GetAddress::address(it),
            Self::StaticMemberExpression(it) => GetAddress::address(it),
            Self::PrivateFieldExpression(it) => GetAddress::address(it),
            Self::ArrayAssignmentTarget(it) => GetAddress::address(it),
            Self::ObjectAssignmentTarget(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for BindingPatternKind<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::BindingIdentifier(it) => GetAddress::address(it),
            Self::ObjectPattern(it) => GetAddress::address(it),
            Self::ArrayPattern(it) => GetAddress::address(it),
            Self::AssignmentPattern(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for ClassElement<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::StaticBlock(it) => GetAddress::address(it),
            Self::MethodDefinition(it) => GetAddress::address(it),
            Self::PropertyDefinition(it) => GetAddress::address(it),
            Self::AccessorProperty(it) => GetAddress::address(it),
            Self::TSIndexSignature(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for ModuleDeclaration<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::ImportDeclaration(it) => GetAddress::address(it),
            Self::ExportAllDeclaration(it) => GetAddress::address(it),
            Self::ExportDefaultDeclaration(it) => GetAddress::address(it),
            Self::ExportNamedDeclaration(it) => GetAddress::address(it),
            Self::TSExportAssignment(it) => GetAddress::address(it),
            Self::TSNamespaceExportDeclaration(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for ImportDeclarationSpecifier<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::ImportSpecifier(it) => GetAddress::address(it),
            Self::ImportDefaultSpecifier(it) => GetAddress::address(it),
            Self::ImportNamespaceSpecifier(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for ExportDefaultDeclarationKind<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::FunctionDeclaration(it) => GetAddress::address(it),
            Self::ClassDeclaration(it) => GetAddress::address(it),
            Self::TSInterfaceDeclaration(it) => GetAddress::address(it),
            Self::BooleanLiteral(it) => GetAddress::address(it),
            Self::NullLiteral(it) => GetAddress::address(it),
            Self::NumericLiteral(it) => GetAddress::address(it),
            Self::BigIntLiteral(it) => GetAddress::address(it),
            Self::RegExpLiteral(it) => GetAddress::address(it),
            Self::StringLiteral(it) => GetAddress::address(it),
            Self::TemplateLiteral(it) => GetAddress::address(it),
            Self::Identifier(it) => GetAddress::address(it),
            Self::MetaProperty(it) => GetAddress::address(it),
            Self::Super(it) => GetAddress::address(it),
            Self::ArrayExpression(it) => GetAddress::address(it),
            Self::ArrowFunctionExpression(it) => GetAddress::address(it),
            Self::AssignmentExpression(it) => GetAddress::address(it),
            Self::AwaitExpression(it) => GetAddress::address(it),
            Self::BinaryExpression(it) => GetAddress::address(it),
            Self::CallExpression(it) => GetAddress::address(it),
            Self::ChainExpression(it) => GetAddress::address(it),
            Self::ClassExpression(it) => GetAddress::address(it),
            Self::ConditionalExpression(it) => GetAddress::address(it),
            Self::FunctionExpression(it) => GetAddress::address(it),
            Self::ImportExpression(it) => GetAddress::address(it),
            Self::LogicalExpression(it) => GetAddress::address(it),
            Self::NewExpression(it) => GetAddress::address(it),
            Self::ObjectExpression(it) => GetAddress::address(it),
            Self::ParenthesizedExpression(it) => GetAddress::address(it),
            Self::SequenceExpression(it) => GetAddress::address(it),
            Self::TaggedTemplateExpression(it) => GetAddress::address(it),
            Self::ThisExpression(it) => GetAddress::address(it),
            Self::UnaryExpression(it) => GetAddress::address(it),
            Self::UpdateExpression(it) => GetAddress::address(it),
            Self::YieldExpression(it) => GetAddress::address(it),
            Self::PrivateInExpression(it) => GetAddress::address(it),
            Self::JSXElement(it) => GetAddress::address(it),
            Self::JSXFragment(it) => GetAddress::address(it),
            Self::TSAsExpression(it) => GetAddress::address(it),
            Self::TSSatisfiesExpression(it) => GetAddress::address(it),
            Self::TSTypeAssertion(it) => GetAddress::address(it),
            Self::TSNonNullExpression(it) => GetAddress::address(it),
            Self::TSInstantiationExpression(it) => GetAddress::address(it),
            Self::ComputedMemberExpression(it) => GetAddress::address(it),
            Self::StaticMemberExpression(it) => GetAddress::address(it),
            Self::PrivateFieldExpression(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for TSEnumMemberName<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::Identifier(it) => GetAddress::address(it),
            Self::String(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for TSLiteral<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::BooleanLiteral(it) => GetAddress::address(it),
            Self::NullLiteral(it) => GetAddress::address(it),
            Self::NumericLiteral(it) => GetAddress::address(it),
            Self::BigIntLiteral(it) => GetAddress::address(it),
            Self::RegExpLiteral(it) => GetAddress::address(it),
            Self::StringLiteral(it) => GetAddress::address(it),
            Self::TemplateLiteral(it) => GetAddress::address(it),
            Self::UnaryExpression(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for TSType<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::TSAnyKeyword(it) => GetAddress::address(it),
            Self::TSBigIntKeyword(it) => GetAddress::address(it),
            Self::TSBooleanKeyword(it) => GetAddress::address(it),
            Self::TSIntrinsicKeyword(it) => GetAddress::address(it),
            Self::TSNeverKeyword(it) => GetAddress::address(it),
            Self::TSNullKeyword(it) => GetAddress::address(it),
            Self::TSNumberKeyword(it) => GetAddress::address(it),
            Self::TSObjectKeyword(it) => GetAddress::address(it),
            Self::TSStringKeyword(it) => GetAddress::address(it),
            Self::TSSymbolKeyword(it) => GetAddress::address(it),
            Self::TSUndefinedKeyword(it) => GetAddress::address(it),
            Self::TSUnknownKeyword(it) => GetAddress::address(it),
            Self::TSVoidKeyword(it) => GetAddress::address(it),
            Self::TSArrayType(it) => GetAddress::address(it),
            Self::TSConditionalType(it) => GetAddress::address(it),
            Self::TSConstructorType(it) => GetAddress::address(it),
            Self::TSFunctionType(it) => GetAddress::address(it),
            Self::TSImportType(it) => GetAddress::address(it),
            Self::TSIndexedAccessType(it) => GetAddress::address(it),
            Self::TSInferType(it) => GetAddress::address(it),
            Self::TSIntersectionType(it) => GetAddress::address(it),
            Self::TSLiteralType(it) => GetAddress::address(it),
            Self::TSMappedType(it) => GetAddress::address(it),
            Self::TSNamedTupleMember(it) => GetAddress::address(it),
            Self::TSQualifiedName(it) => GetAddress::address(it),
            Self::TSTemplateLiteralType(it) => GetAddress::address(it),
            Self::TSThisType(it) => GetAddress::address(it),
            Self::TSTupleType(it) => GetAddress::address(it),
            Self::TSTypeLiteral(it) => GetAddress::address(it),
            Self::TSTypeOperatorType(it) => GetAddress::address(it),
            Self::TSTypePredicate(it) => GetAddress::address(it),
            Self::TSTypeQuery(it) => GetAddress::address(it),
            Self::TSTypeReference(it) => GetAddress::address(it),
            Self::TSUnionType(it) => GetAddress::address(it),
            Self::TSParenthesizedType(it) => GetAddress::address(it),
            Self::JSDocNullableType(it) => GetAddress::address(it),
            Self::JSDocNonNullableType(it) => GetAddress::address(it),
            Self::JSDocUnknownType(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for TSTupleElement<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::TSOptionalType(it) => GetAddress::address(it),
            Self::TSRestType(it) => GetAddress::address(it),
            Self::TSAnyKeyword(it) => GetAddress::address(it),
            Self::TSBigIntKeyword(it) => GetAddress::address(it),
            Self::TSBooleanKeyword(it) => GetAddress::address(it),
            Self::TSIntrinsicKeyword(it) => GetAddress::address(it),
            Self::TSNeverKeyword(it) => GetAddress::address(it),
            Self::TSNullKeyword(it) => GetAddress::address(it),
            Self::TSNumberKeyword(it) => GetAddress::address(it),
            Self::TSObjectKeyword(it) => GetAddress::address(it),
            Self::TSStringKeyword(it) => GetAddress::address(it),
            Self::TSSymbolKeyword(it) => GetAddress::address(it),
            Self::TSUndefinedKeyword(it) => GetAddress::address(it),
            Self::TSUnknownKeyword(it) => GetAddress::address(it),
            Self::TSVoidKeyword(it) => GetAddress::address(it),
            Self::TSArrayType(it) => GetAddress::address(it),
            Self::TSConditionalType(it) => GetAddress::address(it),
            Self::TSConstructorType(it) => GetAddress::address(it),
            Self::TSFunctionType(it) => GetAddress::address(it),
            Self::TSImportType(it) => GetAddress::address(it),
            Self::TSIndexedAccessType(it) => GetAddress::address(it),
            Self::TSInferType(it) => GetAddress::address(it),
            Self::TSIntersectionType(it) => GetAddress::address(it),
            Self::TSLiteralType(it) => GetAddress::address(it),
            Self::TSMappedType(it) => GetAddress::address(it),
            Self::TSNamedTupleMember(it) => GetAddress::address(it),
            Self::TSQualifiedName(it) => GetAddress::address(it),
            Self::TSTemplateLiteralType(it) => GetAddress::address(it),
            Self::TSThisType(it) => GetAddress::address(it),
            Self::TSTupleType(it) => GetAddress::address(it),
            Self::TSTypeLiteral(it) => GetAddress::address(it),
            Self::TSTypeOperatorType(it) => GetAddress::address(it),
            Self::TSTypePredicate(it) => GetAddress::address(it),
            Self::TSTypeQuery(it) => GetAddress::address(it),
            Self::TSTypeReference(it) => GetAddress::address(it),
            Self::TSUnionType(it) => GetAddress::address(it),
            Self::TSParenthesizedType(it) => GetAddress::address(it),
            Self::JSDocNullableType(it) => GetAddress::address(it),
            Self::JSDocNonNullableType(it) => GetAddress::address(it),
            Self::JSDocUnknownType(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for TSTypeName<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::IdentifierReference(it) => GetAddress::address(it),
            Self::QualifiedName(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for TSSignature<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::TSIndexSignature(it) => GetAddress::address(it),
            Self::TSPropertySignature(it) => GetAddress::address(it),
            Self::TSCallSignatureDeclaration(it) => GetAddress::address(it),
            Self::TSConstructSignatureDeclaration(it) => GetAddress::address(it),
            Self::TSMethodSignature(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for TSModuleDeclarationBody<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::TSModuleDeclaration(it) => GetAddress::address(it),
            Self::TSModuleBlock(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for TSTypeQueryExprName<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::TSImportType(it) => GetAddress::address(it),
            Self::IdentifierReference(it) => GetAddress::address(it),
            Self::QualifiedName(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for TSModuleReference<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::ExternalModuleReference(it) => GetAddress::address(it),
            Self::IdentifierReference(it) => GetAddress::address(it),
            Self::QualifiedName(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for JSXElementName<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::Identifier(it) => GetAddress::address(it),
            Self::IdentifierReference(it) => GetAddress::address(it),
            Self::NamespacedName(it) => GetAddress::address(it),
            Self::MemberExpression(it) => GetAddress::address(it),
            Self::ThisExpression(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for JSXMemberExpressionObject<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::IdentifierReference(it) => GetAddress::address(it),
            Self::MemberExpression(it) => GetAddress::address(it),
            Self::ThisExpression(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for JSXAttributeItem<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::Attribute(it) => GetAddress::address(it),
            Self::SpreadAttribute(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for JSXAttributeName<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::Identifier(it) => GetAddress::address(it),
            Self::NamespacedName(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for JSXAttributeValue<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::StringLiteral(it) => GetAddress::address(it),
            Self::ExpressionContainer(it) => GetAddress::address(it),
            Self::Element(it) => GetAddress::address(it),
            Self::Fragment(it) => GetAddress::address(it),
        }
    }
}

impl GetAddress for JSXChild<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::Text(it) => GetAddress::address(it),
            Self::Element(it) => GetAddress::address(it),
            Self::Fragment(it) => GetAddress::address(it),
            Self::ExpressionContainer(it) => GetAddress::address(it),
            Self::Spread(it) => GetAddress::address(it),
        }
    }
}
