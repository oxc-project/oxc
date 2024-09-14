// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/get_node_id.rs`

#![allow(clippy::match_same_arms)]

use oxc_syntax::node::{GetNodeId, NodeId};

#[allow(clippy::wildcard_imports)]
use crate::ast::js::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::jsx::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::literal::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::ts::*;

impl GetNodeId for BooleanLiteral {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for NullLiteral {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for NumericLiteral<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for BigIntLiteral<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for RegExpLiteral<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for StringLiteral<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for Program<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for Expression<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::BooleanLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::NullLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::NumericLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::BigIntLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::RegExpLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::StringLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::TemplateLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::Identifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::MetaProperty(it) => GetNodeId::node_id(it.as_ref()),
            Self::Super(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrayExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrowFunctionExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::AssignmentExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::AwaitExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::BinaryExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::CallExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ChainExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ClassExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ConditionalExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::FunctionExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ImportExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::LogicalExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::NewExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ObjectExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ParenthesizedExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::SequenceExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TaggedTemplateExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ThisExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::UnaryExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::UpdateExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::YieldExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateInExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSXElement(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSXFragment(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSAsExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeAssertion(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNonNullExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::StaticMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::BooleanLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NullLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NumericLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BigIntLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::RegExpLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StringLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TemplateLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Identifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::MetaProperty(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Super(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrayExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrowFunctionExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AssignmentExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AwaitExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BinaryExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::CallExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ChainExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ClassExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ConditionalExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::FunctionExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ImportExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::LogicalExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NewExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ObjectExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ParenthesizedExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::SequenceExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TaggedTemplateExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ThisExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::UnaryExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::UpdateExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::YieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateInExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSXElement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSXFragment(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSAsExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for IdentifierName<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for IdentifierReference<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for BindingIdentifier<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for LabelIdentifier<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for ThisExpression {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ArrayExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ArrayExpressionElement<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::SpreadElement(it) => GetNodeId::node_id(it.as_ref()),
            Self::Elision(it) => GetNodeId::node_id(it),
            Self::BooleanLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::NullLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::NumericLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::BigIntLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::RegExpLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::StringLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::TemplateLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::Identifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::MetaProperty(it) => GetNodeId::node_id(it.as_ref()),
            Self::Super(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrayExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrowFunctionExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::AssignmentExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::AwaitExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::BinaryExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::CallExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ChainExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ClassExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ConditionalExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::FunctionExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ImportExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::LogicalExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::NewExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ObjectExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ParenthesizedExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::SequenceExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TaggedTemplateExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ThisExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::UnaryExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::UpdateExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::YieldExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateInExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSXElement(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSXFragment(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSAsExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeAssertion(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNonNullExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::StaticMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::SpreadElement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Elision(it) => GetNodeId::node_id_mut(it),
            Self::BooleanLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NullLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NumericLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BigIntLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::RegExpLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StringLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TemplateLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Identifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::MetaProperty(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Super(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrayExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrowFunctionExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AssignmentExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AwaitExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BinaryExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::CallExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ChainExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ClassExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ConditionalExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::FunctionExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ImportExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::LogicalExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NewExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ObjectExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ParenthesizedExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::SequenceExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TaggedTemplateExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ThisExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::UnaryExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::UpdateExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::YieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateInExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSXElement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSXFragment(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSAsExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl GetNodeId for Elision {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ObjectExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ObjectPropertyKind<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::ObjectProperty(it) => GetNodeId::node_id(it.as_ref()),
            Self::SpreadProperty(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::ObjectProperty(it) => GetNodeId::node_id_mut(&mut **it),
            Self::SpreadProperty(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for ObjectProperty<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for PropertyKey<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::StaticIdentifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateIdentifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::BooleanLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::NullLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::NumericLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::BigIntLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::RegExpLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::StringLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::TemplateLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::Identifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::MetaProperty(it) => GetNodeId::node_id(it.as_ref()),
            Self::Super(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrayExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrowFunctionExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::AssignmentExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::AwaitExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::BinaryExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::CallExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ChainExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ClassExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ConditionalExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::FunctionExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ImportExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::LogicalExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::NewExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ObjectExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ParenthesizedExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::SequenceExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TaggedTemplateExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ThisExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::UnaryExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::UpdateExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::YieldExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateInExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSXElement(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSXFragment(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSAsExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeAssertion(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNonNullExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::StaticMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::StaticIdentifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateIdentifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BooleanLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NullLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NumericLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BigIntLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::RegExpLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StringLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TemplateLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Identifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::MetaProperty(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Super(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrayExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrowFunctionExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AssignmentExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AwaitExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BinaryExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::CallExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ChainExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ClassExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ConditionalExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::FunctionExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ImportExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::LogicalExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NewExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ObjectExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ParenthesizedExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::SequenceExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TaggedTemplateExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ThisExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::UnaryExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::UpdateExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::YieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateInExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSXElement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSXFragment(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSAsExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for TemplateLiteral<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TaggedTemplateExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TemplateElement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for MemberExpression<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::ComputedMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::StaticMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::ComputedMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for ComputedMemberExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for StaticMemberExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for PrivateFieldExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for CallExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for NewExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for MetaProperty<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for SpreadElement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for Argument<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::SpreadElement(it) => GetNodeId::node_id(it.as_ref()),
            Self::BooleanLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::NullLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::NumericLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::BigIntLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::RegExpLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::StringLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::TemplateLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::Identifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::MetaProperty(it) => GetNodeId::node_id(it.as_ref()),
            Self::Super(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrayExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrowFunctionExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::AssignmentExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::AwaitExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::BinaryExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::CallExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ChainExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ClassExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ConditionalExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::FunctionExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ImportExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::LogicalExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::NewExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ObjectExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ParenthesizedExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::SequenceExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TaggedTemplateExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ThisExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::UnaryExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::UpdateExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::YieldExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateInExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSXElement(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSXFragment(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSAsExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeAssertion(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNonNullExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::StaticMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::SpreadElement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BooleanLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NullLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NumericLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BigIntLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::RegExpLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StringLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TemplateLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Identifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::MetaProperty(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Super(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrayExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrowFunctionExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AssignmentExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AwaitExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BinaryExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::CallExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ChainExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ClassExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ConditionalExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::FunctionExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ImportExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::LogicalExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NewExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ObjectExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ParenthesizedExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::SequenceExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TaggedTemplateExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ThisExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::UnaryExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::UpdateExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::YieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateInExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSXElement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSXFragment(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSAsExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for UpdateExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for UnaryExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for BinaryExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for PrivateInExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for LogicalExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ConditionalExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for AssignmentExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for AssignmentTarget<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::AssignmentTargetIdentifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSAsExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNonNullExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeAssertion(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::StaticMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrayAssignmentTarget(it) => GetNodeId::node_id(it.as_ref()),
            Self::ObjectAssignmentTarget(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::AssignmentTargetIdentifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSAsExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrayAssignmentTarget(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ObjectAssignmentTarget(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for SimpleAssignmentTarget<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::AssignmentTargetIdentifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSAsExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNonNullExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeAssertion(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::StaticMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::AssignmentTargetIdentifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSAsExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for AssignmentTargetPattern<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::ArrayAssignmentTarget(it) => GetNodeId::node_id(it.as_ref()),
            Self::ObjectAssignmentTarget(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::ArrayAssignmentTarget(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ObjectAssignmentTarget(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for ArrayAssignmentTarget<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ObjectAssignmentTarget<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for AssignmentTargetRest<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for AssignmentTargetMaybeDefault<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::AssignmentTargetWithDefault(it) => GetNodeId::node_id(it.as_ref()),
            Self::AssignmentTargetIdentifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSAsExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNonNullExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeAssertion(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::StaticMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrayAssignmentTarget(it) => GetNodeId::node_id(it.as_ref()),
            Self::ObjectAssignmentTarget(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::AssignmentTargetWithDefault(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AssignmentTargetIdentifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSAsExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrayAssignmentTarget(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ObjectAssignmentTarget(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for AssignmentTargetWithDefault<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for AssignmentTargetProperty<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::AssignmentTargetPropertyProperty(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AssignmentTargetPropertyProperty(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for AssignmentTargetPropertyIdentifier<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for AssignmentTargetPropertyProperty<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for SequenceExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for Super {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for AwaitExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ChainExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ChainElement<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::CallExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::StaticMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::CallExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for ParenthesizedExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for Statement<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::BlockStatement(it) => GetNodeId::node_id(it.as_ref()),
            Self::BreakStatement(it) => GetNodeId::node_id(it.as_ref()),
            Self::ContinueStatement(it) => GetNodeId::node_id(it.as_ref()),
            Self::DebuggerStatement(it) => GetNodeId::node_id(it.as_ref()),
            Self::DoWhileStatement(it) => GetNodeId::node_id(it.as_ref()),
            Self::EmptyStatement(it) => GetNodeId::node_id(it.as_ref()),
            Self::ExpressionStatement(it) => GetNodeId::node_id(it.as_ref()),
            Self::ForInStatement(it) => GetNodeId::node_id(it.as_ref()),
            Self::ForOfStatement(it) => GetNodeId::node_id(it.as_ref()),
            Self::ForStatement(it) => GetNodeId::node_id(it.as_ref()),
            Self::IfStatement(it) => GetNodeId::node_id(it.as_ref()),
            Self::LabeledStatement(it) => GetNodeId::node_id(it.as_ref()),
            Self::ReturnStatement(it) => GetNodeId::node_id(it.as_ref()),
            Self::SwitchStatement(it) => GetNodeId::node_id(it.as_ref()),
            Self::ThrowStatement(it) => GetNodeId::node_id(it.as_ref()),
            Self::TryStatement(it) => GetNodeId::node_id(it.as_ref()),
            Self::WhileStatement(it) => GetNodeId::node_id(it.as_ref()),
            Self::WithStatement(it) => GetNodeId::node_id(it.as_ref()),
            Self::VariableDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::FunctionDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::ClassDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeAliasDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSInterfaceDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSEnumDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSModuleDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSImportEqualsDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::ImportDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::ExportAllDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::ExportDefaultDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::ExportNamedDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSExportAssignment(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNamespaceExportDeclaration(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::BlockStatement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BreakStatement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ContinueStatement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::DebuggerStatement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::DoWhileStatement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::EmptyStatement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ExpressionStatement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ForInStatement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ForOfStatement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ForStatement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::IfStatement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::LabeledStatement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ReturnStatement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::SwitchStatement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ThrowStatement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TryStatement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::WhileStatement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::WithStatement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::VariableDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::FunctionDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ClassDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeAliasDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSInterfaceDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSEnumDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSModuleDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSImportEqualsDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ImportDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ExportAllDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ExportDefaultDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ExportNamedDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSExportAssignment(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNamespaceExportDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for Directive<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for Hashbang<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for BlockStatement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for Declaration<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::VariableDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::FunctionDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::ClassDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeAliasDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSInterfaceDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSEnumDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSModuleDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSImportEqualsDeclaration(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::VariableDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::FunctionDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ClassDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeAliasDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSInterfaceDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSEnumDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSModuleDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSImportEqualsDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for VariableDeclaration<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for VariableDeclarator<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for EmptyStatement {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ExpressionStatement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for IfStatement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for DoWhileStatement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for WhileStatement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ForStatement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ForStatementInit<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::VariableDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::BooleanLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::NullLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::NumericLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::BigIntLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::RegExpLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::StringLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::TemplateLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::Identifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::MetaProperty(it) => GetNodeId::node_id(it.as_ref()),
            Self::Super(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrayExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrowFunctionExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::AssignmentExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::AwaitExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::BinaryExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::CallExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ChainExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ClassExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ConditionalExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::FunctionExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ImportExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::LogicalExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::NewExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ObjectExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ParenthesizedExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::SequenceExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TaggedTemplateExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ThisExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::UnaryExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::UpdateExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::YieldExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateInExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSXElement(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSXFragment(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSAsExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeAssertion(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNonNullExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::StaticMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::VariableDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BooleanLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NullLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NumericLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BigIntLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::RegExpLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StringLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TemplateLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Identifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::MetaProperty(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Super(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrayExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrowFunctionExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AssignmentExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AwaitExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BinaryExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::CallExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ChainExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ClassExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ConditionalExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::FunctionExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ImportExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::LogicalExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NewExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ObjectExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ParenthesizedExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::SequenceExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TaggedTemplateExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ThisExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::UnaryExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::UpdateExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::YieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateInExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSXElement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSXFragment(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSAsExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for ForInStatement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ForStatementLeft<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::VariableDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::AssignmentTargetIdentifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSAsExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNonNullExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeAssertion(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::StaticMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrayAssignmentTarget(it) => GetNodeId::node_id(it.as_ref()),
            Self::ObjectAssignmentTarget(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::VariableDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AssignmentTargetIdentifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSAsExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrayAssignmentTarget(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ObjectAssignmentTarget(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for ForOfStatement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ContinueStatement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for BreakStatement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ReturnStatement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for WithStatement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for SwitchStatement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for SwitchCase<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for LabeledStatement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ThrowStatement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TryStatement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for CatchClause<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for CatchParameter<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for DebuggerStatement {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for BindingPattern<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for BindingPatternKind<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::BindingIdentifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::ObjectPattern(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrayPattern(it) => GetNodeId::node_id(it.as_ref()),
            Self::AssignmentPattern(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::BindingIdentifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ObjectPattern(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrayPattern(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AssignmentPattern(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for AssignmentPattern<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ObjectPattern<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for BindingProperty<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ArrayPattern<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for BindingRestElement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for Function<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for FormalParameters<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for FormalParameter<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for FunctionBody<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ArrowFunctionExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for YieldExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for Class<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ClassBody<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ClassElement<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::StaticBlock(it) => GetNodeId::node_id(it.as_ref()),
            Self::MethodDefinition(it) => GetNodeId::node_id(it.as_ref()),
            Self::PropertyDefinition(it) => GetNodeId::node_id(it.as_ref()),
            Self::AccessorProperty(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSIndexSignature(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::StaticBlock(it) => GetNodeId::node_id_mut(&mut **it),
            Self::MethodDefinition(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PropertyDefinition(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AccessorProperty(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSIndexSignature(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for MethodDefinition<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for PropertyDefinition<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for PrivateIdentifier<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for StaticBlock<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ModuleDeclaration<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::ImportDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::ExportAllDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::ExportDefaultDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::ExportNamedDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSExportAssignment(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNamespaceExportDeclaration(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::ImportDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ExportAllDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ExportDefaultDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ExportNamedDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSExportAssignment(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNamespaceExportDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for AccessorProperty<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ImportExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ImportDeclaration<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ImportDeclarationSpecifier<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::ImportSpecifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::ImportDefaultSpecifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::ImportNamespaceSpecifier(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::ImportSpecifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ImportDefaultSpecifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ImportNamespaceSpecifier(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for ImportSpecifier<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ImportDefaultSpecifier<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ImportNamespaceSpecifier<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for WithClause<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ImportAttribute<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ImportAttributeKey<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::Identifier(it) => GetNodeId::node_id(it),
            Self::StringLiteral(it) => GetNodeId::node_id(it),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::Identifier(it) => GetNodeId::node_id_mut(it),
            Self::StringLiteral(it) => GetNodeId::node_id_mut(it),
        }
    }
}

impl<'a> GetNodeId for ExportNamedDeclaration<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ExportDefaultDeclaration<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ExportAllDeclaration<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ExportSpecifier<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for ExportDefaultDeclarationKind<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::FunctionDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::ClassDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSInterfaceDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::BooleanLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::NullLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::NumericLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::BigIntLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::RegExpLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::StringLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::TemplateLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::Identifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::MetaProperty(it) => GetNodeId::node_id(it.as_ref()),
            Self::Super(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrayExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrowFunctionExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::AssignmentExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::AwaitExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::BinaryExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::CallExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ChainExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ClassExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ConditionalExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::FunctionExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ImportExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::LogicalExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::NewExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ObjectExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ParenthesizedExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::SequenceExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TaggedTemplateExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ThisExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::UnaryExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::UpdateExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::YieldExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateInExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSXElement(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSXFragment(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSAsExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeAssertion(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNonNullExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::StaticMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::FunctionDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ClassDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSInterfaceDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BooleanLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NullLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NumericLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BigIntLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::RegExpLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StringLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TemplateLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Identifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::MetaProperty(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Super(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrayExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrowFunctionExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AssignmentExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AwaitExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BinaryExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::CallExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ChainExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ClassExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ConditionalExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::FunctionExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ImportExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::LogicalExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NewExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ObjectExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ParenthesizedExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::SequenceExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TaggedTemplateExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ThisExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::UnaryExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::UpdateExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::YieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateInExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSXElement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSXFragment(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSAsExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for ModuleExportName<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::IdentifierName(it) => GetNodeId::node_id(it),
            Self::IdentifierReference(it) => GetNodeId::node_id(it),
            Self::StringLiteral(it) => GetNodeId::node_id(it),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::IdentifierName(it) => GetNodeId::node_id_mut(it),
            Self::IdentifierReference(it) => GetNodeId::node_id_mut(it),
            Self::StringLiteral(it) => GetNodeId::node_id_mut(it),
        }
    }
}

impl<'a> GetNodeId for TSThisParameter<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSEnumDeclaration<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSEnumMember<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSEnumMemberName<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::StaticIdentifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::StaticStringLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::StaticTemplateLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::StaticNumericLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::BooleanLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::NullLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::NumericLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::BigIntLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::RegExpLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::StringLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::TemplateLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::Identifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::MetaProperty(it) => GetNodeId::node_id(it.as_ref()),
            Self::Super(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrayExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrowFunctionExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::AssignmentExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::AwaitExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::BinaryExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::CallExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ChainExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ClassExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ConditionalExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::FunctionExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ImportExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::LogicalExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::NewExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ObjectExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ParenthesizedExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::SequenceExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TaggedTemplateExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ThisExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::UnaryExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::UpdateExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::YieldExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateInExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSXElement(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSXFragment(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSAsExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeAssertion(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNonNullExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::StaticMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::StaticIdentifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StaticStringLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StaticTemplateLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StaticNumericLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BooleanLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NullLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NumericLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BigIntLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::RegExpLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StringLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TemplateLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Identifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::MetaProperty(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Super(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrayExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrowFunctionExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AssignmentExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AwaitExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BinaryExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::CallExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ChainExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ClassExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ConditionalExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::FunctionExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ImportExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::LogicalExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NewExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ObjectExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ParenthesizedExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::SequenceExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TaggedTemplateExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ThisExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::UnaryExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::UpdateExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::YieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateInExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSXElement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSXFragment(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSAsExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for TSTypeAnnotation<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSLiteralType<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSLiteral<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::BooleanLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::NullLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::NumericLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::BigIntLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::RegExpLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::StringLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::TemplateLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::UnaryExpression(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::BooleanLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NullLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NumericLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BigIntLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::RegExpLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StringLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TemplateLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::UnaryExpression(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for TSType<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::TSAnyKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSBigIntKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSBooleanKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSIntrinsicKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNeverKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNullKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNumberKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSObjectKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSStringKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSSymbolKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSUndefinedKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSUnknownKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSVoidKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSArrayType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSConditionalType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSConstructorType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSFunctionType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSImportType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSIndexedAccessType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSInferType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSIntersectionType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSLiteralType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSMappedType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNamedTupleMember(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSQualifiedName(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTemplateLiteralType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSThisType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTupleType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeOperatorType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypePredicate(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeQuery(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeReference(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSUnionType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSParenthesizedType(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSDocNullableType(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSDocNonNullableType(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSDocUnknownType(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::TSAnyKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSBigIntKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSBooleanKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSIntrinsicKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNeverKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNullKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNumberKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSObjectKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSStringKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSSymbolKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSUndefinedKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSUnknownKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSVoidKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSArrayType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSConditionalType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSConstructorType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSFunctionType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSImportType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSIndexedAccessType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSInferType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSIntersectionType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSLiteralType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSMappedType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNamedTupleMember(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSQualifiedName(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTemplateLiteralType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSThisType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTupleType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeOperatorType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypePredicate(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeQuery(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeReference(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSUnionType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSParenthesizedType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSDocNullableType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSDocNonNullableType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSDocUnknownType(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for TSConditionalType<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSUnionType<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSIntersectionType<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSParenthesizedType<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSTypeOperator<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSArrayType<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSIndexedAccessType<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSTupleType<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSNamedTupleMember<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSOptionalType<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSRestType<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSTupleElement<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::TSOptionalType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSRestType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSAnyKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSBigIntKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSBooleanKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSIntrinsicKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNeverKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNullKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNumberKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSObjectKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSStringKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSSymbolKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSUndefinedKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSUnknownKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSVoidKeyword(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSArrayType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSConditionalType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSConstructorType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSFunctionType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSImportType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSIndexedAccessType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSInferType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSIntersectionType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSLiteralType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSMappedType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNamedTupleMember(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSQualifiedName(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTemplateLiteralType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSThisType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTupleType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeOperatorType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypePredicate(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeQuery(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeReference(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSUnionType(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSParenthesizedType(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSDocNullableType(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSDocNonNullableType(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSDocUnknownType(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::TSOptionalType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSRestType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSAnyKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSBigIntKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSBooleanKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSIntrinsicKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNeverKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNullKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNumberKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSObjectKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSStringKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSSymbolKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSUndefinedKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSUnknownKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSVoidKeyword(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSArrayType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSConditionalType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSConstructorType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSFunctionType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSImportType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSIndexedAccessType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSInferType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSIntersectionType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSLiteralType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSMappedType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNamedTupleMember(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSQualifiedName(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTemplateLiteralType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSThisType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTupleType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeOperatorType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypePredicate(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeQuery(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeReference(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSUnionType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSParenthesizedType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSDocNullableType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSDocNonNullableType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSDocUnknownType(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl GetNodeId for TSAnyKeyword {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for TSStringKeyword {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for TSBooleanKeyword {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for TSNumberKeyword {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for TSNeverKeyword {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for TSIntrinsicKeyword {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for TSUnknownKeyword {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for TSNullKeyword {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for TSUndefinedKeyword {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for TSVoidKeyword {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for TSSymbolKeyword {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for TSThisType {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for TSObjectKeyword {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for TSBigIntKeyword {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSTypeReference<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSTypeName<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::IdentifierReference(it) => GetNodeId::node_id(it.as_ref()),
            Self::QualifiedName(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::IdentifierReference(it) => GetNodeId::node_id_mut(&mut **it),
            Self::QualifiedName(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for TSQualifiedName<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSTypeParameterInstantiation<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSTypeParameter<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSTypeParameterDeclaration<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSTypeAliasDeclaration<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSClassImplements<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSInterfaceDeclaration<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSInterfaceBody<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSPropertySignature<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSSignature<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::TSIndexSignature(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSPropertySignature(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSCallSignatureDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSConstructSignatureDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSMethodSignature(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::TSIndexSignature(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSPropertySignature(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSCallSignatureDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSConstructSignatureDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSMethodSignature(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for TSIndexSignature<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSCallSignatureDeclaration<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSMethodSignature<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSConstructSignatureDeclaration<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSIndexSignatureName<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSInterfaceHeritage<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSTypePredicate<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSTypePredicateName<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::Identifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::This(it) => GetNodeId::node_id(it),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::Identifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::This(it) => GetNodeId::node_id_mut(it),
        }
    }
}

impl<'a> GetNodeId for TSModuleDeclaration<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSModuleDeclarationName<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::Identifier(it) => GetNodeId::node_id(it),
            Self::StringLiteral(it) => GetNodeId::node_id(it),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::Identifier(it) => GetNodeId::node_id_mut(it),
            Self::StringLiteral(it) => GetNodeId::node_id_mut(it),
        }
    }
}

impl<'a> GetNodeId for TSModuleDeclarationBody<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::TSModuleDeclaration(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSModuleBlock(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::TSModuleDeclaration(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSModuleBlock(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for TSModuleBlock<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSTypeLiteral<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSInferType<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSTypeQuery<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSTypeQueryExprName<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::TSImportType(it) => GetNodeId::node_id(it.as_ref()),
            Self::IdentifierReference(it) => GetNodeId::node_id(it.as_ref()),
            Self::QualifiedName(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::TSImportType(it) => GetNodeId::node_id_mut(&mut **it),
            Self::IdentifierReference(it) => GetNodeId::node_id_mut(&mut **it),
            Self::QualifiedName(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for TSImportType<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSImportAttributes<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSImportAttribute<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSImportAttributeName<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::Identifier(it) => GetNodeId::node_id(it),
            Self::StringLiteral(it) => GetNodeId::node_id(it),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::Identifier(it) => GetNodeId::node_id_mut(it),
            Self::StringLiteral(it) => GetNodeId::node_id_mut(it),
        }
    }
}

impl<'a> GetNodeId for TSFunctionType<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSConstructorType<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSMappedType<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSTemplateLiteralType<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSAsExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSSatisfiesExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSTypeAssertion<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSImportEqualsDeclaration<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSModuleReference<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::ExternalModuleReference(it) => GetNodeId::node_id(it.as_ref()),
            Self::IdentifierReference(it) => GetNodeId::node_id(it.as_ref()),
            Self::QualifiedName(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::ExternalModuleReference(it) => GetNodeId::node_id_mut(&mut **it),
            Self::IdentifierReference(it) => GetNodeId::node_id_mut(&mut **it),
            Self::QualifiedName(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for TSExternalModuleReference<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSNonNullExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for Decorator<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSExportAssignment<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSNamespaceExportDeclaration<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for TSInstantiationExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for JSDocNullableType<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for JSDocNonNullableType<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl GetNodeId for JSDocUnknownType {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for JSXElement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for JSXOpeningElement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for JSXClosingElement<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for JSXFragment<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for JSXElementName<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::Identifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::IdentifierReference(it) => GetNodeId::node_id(it.as_ref()),
            Self::NamespacedName(it) => GetNodeId::node_id(it.as_ref()),
            Self::MemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ThisExpression(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::Identifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::IdentifierReference(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NamespacedName(it) => GetNodeId::node_id_mut(&mut **it),
            Self::MemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ThisExpression(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for JSXNamespacedName<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for JSXMemberExpression<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for JSXMemberExpressionObject<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::IdentifierReference(it) => GetNodeId::node_id(it.as_ref()),
            Self::MemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ThisExpression(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::IdentifierReference(it) => GetNodeId::node_id_mut(&mut **it),
            Self::MemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ThisExpression(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for JSXExpressionContainer<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for JSXExpression<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::EmptyExpression(it) => GetNodeId::node_id(it),
            Self::BooleanLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::NullLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::NumericLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::BigIntLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::RegExpLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::StringLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::TemplateLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::Identifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::MetaProperty(it) => GetNodeId::node_id(it.as_ref()),
            Self::Super(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrayExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ArrowFunctionExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::AssignmentExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::AwaitExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::BinaryExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::CallExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ChainExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ClassExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ConditionalExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::FunctionExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ImportExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::LogicalExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::NewExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ObjectExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ParenthesizedExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::SequenceExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TaggedTemplateExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ThisExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::UnaryExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::UpdateExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::YieldExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateInExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSXElement(it) => GetNodeId::node_id(it.as_ref()),
            Self::JSXFragment(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSAsExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSTypeAssertion(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSNonNullExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::StaticMemberExpression(it) => GetNodeId::node_id(it.as_ref()),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::EmptyExpression(it) => GetNodeId::node_id_mut(it),
            Self::BooleanLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NullLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NumericLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BigIntLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::RegExpLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StringLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TemplateLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Identifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::MetaProperty(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Super(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrayExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ArrowFunctionExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AssignmentExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::AwaitExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::BinaryExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::CallExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ChainExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ClassExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ConditionalExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::FunctionExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ImportExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::LogicalExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NewExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ObjectExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ParenthesizedExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::SequenceExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TaggedTemplateExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ThisExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::UnaryExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::UpdateExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::YieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateInExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSXElement(it) => GetNodeId::node_id_mut(&mut **it),
            Self::JSXFragment(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSAsExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSSatisfiesExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSTypeAssertion(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSNonNullExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::TSInstantiationExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ComputedMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::StaticMemberExpression(it) => GetNodeId::node_id_mut(&mut **it),
            Self::PrivateFieldExpression(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl GetNodeId for JSXEmptyExpression {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for JSXAttributeItem<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::Attribute(it) => GetNodeId::node_id(it.as_ref()),
            Self::SpreadAttribute(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::Attribute(it) => GetNodeId::node_id_mut(&mut **it),
            Self::SpreadAttribute(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for JSXAttribute<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for JSXSpreadAttribute<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for JSXAttributeName<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::Identifier(it) => GetNodeId::node_id(it.as_ref()),
            Self::NamespacedName(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::Identifier(it) => GetNodeId::node_id_mut(&mut **it),
            Self::NamespacedName(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for JSXAttributeValue<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::StringLiteral(it) => GetNodeId::node_id(it.as_ref()),
            Self::ExpressionContainer(it) => GetNodeId::node_id(it.as_ref()),
            Self::Element(it) => GetNodeId::node_id(it.as_ref()),
            Self::Fragment(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::StringLiteral(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ExpressionContainer(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Element(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Fragment(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for JSXIdentifier<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for JSXChild<'a> {
    fn node_id(&self) -> NodeId {
        match self {
            Self::Text(it) => GetNodeId::node_id(it.as_ref()),
            Self::Element(it) => GetNodeId::node_id(it.as_ref()),
            Self::Fragment(it) => GetNodeId::node_id(it.as_ref()),
            Self::ExpressionContainer(it) => GetNodeId::node_id(it.as_ref()),
            Self::Spread(it) => GetNodeId::node_id(it.as_ref()),
        }
    }
    fn node_id_mut(&mut self) -> &mut NodeId {
        match self {
            Self::Text(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Element(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Fragment(it) => GetNodeId::node_id_mut(&mut **it),
            Self::ExpressionContainer(it) => GetNodeId::node_id_mut(&mut **it),
            Self::Spread(it) => GetNodeId::node_id_mut(&mut **it),
        }
    }
}

impl<'a> GetNodeId for JSXSpreadChild<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}

impl<'a> GetNodeId for JSXText<'a> {
    #[inline]
    fn node_id(&self) -> NodeId {
        self.node_id
    }
    #[inline]
    fn node_id_mut(&mut self) -> &mut NodeId {
        &mut self.node_id
    }
}
