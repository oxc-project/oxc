// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/full_eq.rs`.

#![allow(clippy::match_same_arms, unused_variables)]

use std::num::NonZeroU32;

use oxc::{
    ast::ast::*,
    regular_expression::ast::*,
    span::FileExtension,
    str::JSStr,
    syntax::{
        label::LabelId, node::NodeId, reference::ReferenceId, scope::ScopeId, symbol::SymbolId,
    },
};

use crate::full_eq::FullEq;

impl FullEq for Program<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.source_type, &other.source_type)
            && FullEq::full_eq(&self.source_text, &other.source_text)
            && FullEq::full_eq(&self.comments, &other.comments)
            && FullEq::full_eq(&self.hashbang, &other.hashbang)
            && FullEq::full_eq(&self.directives, &other.directives)
            && FullEq::full_eq(&self.body, &other.body)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for Expression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::BooleanLiteral(a), Self::BooleanLiteral(b)) => FullEq::full_eq(a, b),
            (Self::NullLiteral(a), Self::NullLiteral(b)) => FullEq::full_eq(a, b),
            (Self::NumericLiteral(a), Self::NumericLiteral(b)) => FullEq::full_eq(a, b),
            (Self::BigIntLiteral(a), Self::BigIntLiteral(b)) => FullEq::full_eq(a, b),
            (Self::RegExpLiteral(a), Self::RegExpLiteral(b)) => FullEq::full_eq(a, b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => FullEq::full_eq(a, b),
            (Self::TemplateLiteral(a), Self::TemplateLiteral(b)) => FullEq::full_eq(a, b),
            (Self::Identifier(a), Self::Identifier(b)) => FullEq::full_eq(a, b),
            (Self::Super(a), Self::Super(b)) => FullEq::full_eq(a, b),
            (Self::ArrayExpression(a), Self::ArrayExpression(b)) => FullEq::full_eq(a, b),
            (Self::ArrowFunctionExpression(a), Self::ArrowFunctionExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::AssignmentExpression(a), Self::AssignmentExpression(b)) => FullEq::full_eq(a, b),
            (Self::AwaitExpression(a), Self::AwaitExpression(b)) => FullEq::full_eq(a, b),
            (Self::BinaryExpression(a), Self::BinaryExpression(b)) => FullEq::full_eq(a, b),
            (Self::CallExpression(a), Self::CallExpression(b)) => FullEq::full_eq(a, b),
            (Self::ChainExpression(a), Self::ChainExpression(b)) => FullEq::full_eq(a, b),
            (Self::ClassExpression(a), Self::ClassExpression(b)) => FullEq::full_eq(a, b),
            (Self::ConditionalExpression(a), Self::ConditionalExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::FunctionExpression(a), Self::FunctionExpression(b)) => FullEq::full_eq(a, b),
            (Self::ImportExpression(a), Self::ImportExpression(b)) => FullEq::full_eq(a, b),
            (Self::LogicalExpression(a), Self::LogicalExpression(b)) => FullEq::full_eq(a, b),
            (Self::NewExpression(a), Self::NewExpression(b)) => FullEq::full_eq(a, b),
            (Self::ObjectExpression(a), Self::ObjectExpression(b)) => FullEq::full_eq(a, b),
            (Self::ParenthesizedExpression(a), Self::ParenthesizedExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::SequenceExpression(a), Self::SequenceExpression(b)) => FullEq::full_eq(a, b),
            (Self::TaggedTemplateExpression(a), Self::TaggedTemplateExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ThisExpression(a), Self::ThisExpression(b)) => FullEq::full_eq(a, b),
            (Self::UnaryExpression(a), Self::UnaryExpression(b)) => FullEq::full_eq(a, b),
            (Self::UpdateExpression(a), Self::UpdateExpression(b)) => FullEq::full_eq(a, b),
            (Self::YieldExpression(a), Self::YieldExpression(b)) => FullEq::full_eq(a, b),
            (Self::PrivateInExpression(a), Self::PrivateInExpression(b)) => FullEq::full_eq(a, b),
            (Self::ImportMeta(a), Self::ImportMeta(b)) => FullEq::full_eq(a, b),
            (Self::NewTarget(a), Self::NewTarget(b)) => FullEq::full_eq(a, b),
            (Self::JSXElement(a), Self::JSXElement(b)) => FullEq::full_eq(a, b),
            (Self::JSXFragment(a), Self::JSXFragment(b)) => FullEq::full_eq(a, b),
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => FullEq::full_eq(a, b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::V8IntrinsicExpression(a), Self::V8IntrinsicExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for IdentifierName<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.name, &other.name)
    }
}

impl FullEq for IdentifierReference<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.name, &other.name)
            && FullEq::full_eq(&self.reference_id, &other.reference_id)
    }
}

impl FullEq for BindingIdentifier<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.name, &other.name)
            && FullEq::full_eq(&self.symbol_id, &other.symbol_id)
    }
}

impl FullEq for LabelIdentifier<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.name, &other.name)
            && FullEq::full_eq(&self.label_id, &other.label_id)
    }
}

impl FullEq for ThisExpression {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for ArrayExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.elements, &other.elements)
    }
}

impl FullEq for ArrayExpressionElement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::SpreadElement(a), Self::SpreadElement(b)) => FullEq::full_eq(a, b),
            (Self::Elision(a), Self::Elision(b)) => FullEq::full_eq(a, b),
            (Self::BooleanLiteral(a), Self::BooleanLiteral(b)) => FullEq::full_eq(a, b),
            (Self::NullLiteral(a), Self::NullLiteral(b)) => FullEq::full_eq(a, b),
            (Self::NumericLiteral(a), Self::NumericLiteral(b)) => FullEq::full_eq(a, b),
            (Self::BigIntLiteral(a), Self::BigIntLiteral(b)) => FullEq::full_eq(a, b),
            (Self::RegExpLiteral(a), Self::RegExpLiteral(b)) => FullEq::full_eq(a, b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => FullEq::full_eq(a, b),
            (Self::TemplateLiteral(a), Self::TemplateLiteral(b)) => FullEq::full_eq(a, b),
            (Self::Identifier(a), Self::Identifier(b)) => FullEq::full_eq(a, b),
            (Self::Super(a), Self::Super(b)) => FullEq::full_eq(a, b),
            (Self::ArrayExpression(a), Self::ArrayExpression(b)) => FullEq::full_eq(a, b),
            (Self::ArrowFunctionExpression(a), Self::ArrowFunctionExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::AssignmentExpression(a), Self::AssignmentExpression(b)) => FullEq::full_eq(a, b),
            (Self::AwaitExpression(a), Self::AwaitExpression(b)) => FullEq::full_eq(a, b),
            (Self::BinaryExpression(a), Self::BinaryExpression(b)) => FullEq::full_eq(a, b),
            (Self::CallExpression(a), Self::CallExpression(b)) => FullEq::full_eq(a, b),
            (Self::ChainExpression(a), Self::ChainExpression(b)) => FullEq::full_eq(a, b),
            (Self::ClassExpression(a), Self::ClassExpression(b)) => FullEq::full_eq(a, b),
            (Self::ConditionalExpression(a), Self::ConditionalExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::FunctionExpression(a), Self::FunctionExpression(b)) => FullEq::full_eq(a, b),
            (Self::ImportExpression(a), Self::ImportExpression(b)) => FullEq::full_eq(a, b),
            (Self::LogicalExpression(a), Self::LogicalExpression(b)) => FullEq::full_eq(a, b),
            (Self::NewExpression(a), Self::NewExpression(b)) => FullEq::full_eq(a, b),
            (Self::ObjectExpression(a), Self::ObjectExpression(b)) => FullEq::full_eq(a, b),
            (Self::ParenthesizedExpression(a), Self::ParenthesizedExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::SequenceExpression(a), Self::SequenceExpression(b)) => FullEq::full_eq(a, b),
            (Self::TaggedTemplateExpression(a), Self::TaggedTemplateExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ThisExpression(a), Self::ThisExpression(b)) => FullEq::full_eq(a, b),
            (Self::UnaryExpression(a), Self::UnaryExpression(b)) => FullEq::full_eq(a, b),
            (Self::UpdateExpression(a), Self::UpdateExpression(b)) => FullEq::full_eq(a, b),
            (Self::YieldExpression(a), Self::YieldExpression(b)) => FullEq::full_eq(a, b),
            (Self::PrivateInExpression(a), Self::PrivateInExpression(b)) => FullEq::full_eq(a, b),
            (Self::ImportMeta(a), Self::ImportMeta(b)) => FullEq::full_eq(a, b),
            (Self::NewTarget(a), Self::NewTarget(b)) => FullEq::full_eq(a, b),
            (Self::JSXElement(a), Self::JSXElement(b)) => FullEq::full_eq(a, b),
            (Self::JSXFragment(a), Self::JSXFragment(b)) => FullEq::full_eq(a, b),
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => FullEq::full_eq(a, b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::V8IntrinsicExpression(a), Self::V8IntrinsicExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for Elision {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for ObjectExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.properties, &other.properties)
    }
}

impl FullEq for ObjectPropertyKind<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ObjectProperty(a), Self::ObjectProperty(b)) => FullEq::full_eq(a, b),
            (Self::MethodProperty(a), Self::MethodProperty(b)) => FullEq::full_eq(a, b),
            (Self::SpreadProperty(a), Self::SpreadProperty(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for ObjectProperty<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.key, &other.key)
            && FullEq::full_eq(&self.value, &other.value)
            && FullEq::full_eq(&self.shorthand, &other.shorthand)
    }
}

impl FullEq for ObjectMethodProperty<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.kind, &other.kind)
            && FullEq::full_eq(&self.key, &other.key)
            && FullEq::full_eq(&self.value, &other.value)
    }
}

impl FullEq for ObjectMethodKind {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for PropertyKey<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::StaticIdentifier(a), Self::StaticIdentifier(b)) => FullEq::full_eq(a, b),
            (Self::StaticStringLiteral(a), Self::StaticStringLiteral(b)) => FullEq::full_eq(a, b),
            (Self::StaticNumericLiteral(a), Self::StaticNumericLiteral(b)) => FullEq::full_eq(a, b),
            (Self::StaticBigIntLiteral(a), Self::StaticBigIntLiteral(b)) => FullEq::full_eq(a, b),
            (Self::PrivateIdentifier(a), Self::PrivateIdentifier(b)) => FullEq::full_eq(a, b),
            (Self::BooleanLiteral(a), Self::BooleanLiteral(b)) => FullEq::full_eq(a, b),
            (Self::NullLiteral(a), Self::NullLiteral(b)) => FullEq::full_eq(a, b),
            (Self::NumericLiteral(a), Self::NumericLiteral(b)) => FullEq::full_eq(a, b),
            (Self::BigIntLiteral(a), Self::BigIntLiteral(b)) => FullEq::full_eq(a, b),
            (Self::RegExpLiteral(a), Self::RegExpLiteral(b)) => FullEq::full_eq(a, b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => FullEq::full_eq(a, b),
            (Self::TemplateLiteral(a), Self::TemplateLiteral(b)) => FullEq::full_eq(a, b),
            (Self::Identifier(a), Self::Identifier(b)) => FullEq::full_eq(a, b),
            (Self::Super(a), Self::Super(b)) => FullEq::full_eq(a, b),
            (Self::ArrayExpression(a), Self::ArrayExpression(b)) => FullEq::full_eq(a, b),
            (Self::ArrowFunctionExpression(a), Self::ArrowFunctionExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::AssignmentExpression(a), Self::AssignmentExpression(b)) => FullEq::full_eq(a, b),
            (Self::AwaitExpression(a), Self::AwaitExpression(b)) => FullEq::full_eq(a, b),
            (Self::BinaryExpression(a), Self::BinaryExpression(b)) => FullEq::full_eq(a, b),
            (Self::CallExpression(a), Self::CallExpression(b)) => FullEq::full_eq(a, b),
            (Self::ChainExpression(a), Self::ChainExpression(b)) => FullEq::full_eq(a, b),
            (Self::ClassExpression(a), Self::ClassExpression(b)) => FullEq::full_eq(a, b),
            (Self::ConditionalExpression(a), Self::ConditionalExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::FunctionExpression(a), Self::FunctionExpression(b)) => FullEq::full_eq(a, b),
            (Self::ImportExpression(a), Self::ImportExpression(b)) => FullEq::full_eq(a, b),
            (Self::LogicalExpression(a), Self::LogicalExpression(b)) => FullEq::full_eq(a, b),
            (Self::NewExpression(a), Self::NewExpression(b)) => FullEq::full_eq(a, b),
            (Self::ObjectExpression(a), Self::ObjectExpression(b)) => FullEq::full_eq(a, b),
            (Self::ParenthesizedExpression(a), Self::ParenthesizedExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::SequenceExpression(a), Self::SequenceExpression(b)) => FullEq::full_eq(a, b),
            (Self::TaggedTemplateExpression(a), Self::TaggedTemplateExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ThisExpression(a), Self::ThisExpression(b)) => FullEq::full_eq(a, b),
            (Self::UnaryExpression(a), Self::UnaryExpression(b)) => FullEq::full_eq(a, b),
            (Self::UpdateExpression(a), Self::UpdateExpression(b)) => FullEq::full_eq(a, b),
            (Self::YieldExpression(a), Self::YieldExpression(b)) => FullEq::full_eq(a, b),
            (Self::PrivateInExpression(a), Self::PrivateInExpression(b)) => FullEq::full_eq(a, b),
            (Self::ImportMeta(a), Self::ImportMeta(b)) => FullEq::full_eq(a, b),
            (Self::NewTarget(a), Self::NewTarget(b)) => FullEq::full_eq(a, b),
            (Self::JSXElement(a), Self::JSXElement(b)) => FullEq::full_eq(a, b),
            (Self::JSXFragment(a), Self::JSXFragment(b)) => FullEq::full_eq(a, b),
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => FullEq::full_eq(a, b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::V8IntrinsicExpression(a), Self::V8IntrinsicExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for TemplateLiteral<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.quasis, &other.quasis)
            && FullEq::full_eq(&self.expressions, &other.expressions)
    }
}

impl FullEq for TaggedTemplateExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.tag, &other.tag)
            && FullEq::full_eq(&self.type_arguments, &other.type_arguments)
            && FullEq::full_eq(&self.quasi, &other.quasi)
    }
}

impl FullEq for TemplateElement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.value, &other.value)
            && FullEq::full_eq(&self.tail, &other.tail)
    }
}

impl FullEq for TemplateElementValue<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.raw, &other.raw) && FullEq::full_eq(&self.cooked, &other.cooked)
    }
}

impl FullEq for MemberExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for ComputedMemberExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.object, &other.object)
            && FullEq::full_eq(&self.expression, &other.expression)
            && FullEq::full_eq(&self.optional, &other.optional)
    }
}

impl FullEq for StaticMemberExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.object, &other.object)
            && FullEq::full_eq(&self.property, &other.property)
            && FullEq::full_eq(&self.optional, &other.optional)
    }
}

impl FullEq for PrivateFieldExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.object, &other.object)
            && FullEq::full_eq(&self.field, &other.field)
            && FullEq::full_eq(&self.optional, &other.optional)
    }
}

impl FullEq for CallExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.callee, &other.callee)
            && FullEq::full_eq(&self.type_arguments, &other.type_arguments)
            && FullEq::full_eq(&self.arguments, &other.arguments)
            && FullEq::full_eq(&self.optional, &other.optional)
            && FullEq::full_eq(&self.pure, &other.pure)
    }
}

impl FullEq for NewExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.callee, &other.callee)
            && FullEq::full_eq(&self.type_arguments, &other.type_arguments)
            && FullEq::full_eq(&self.arguments, &other.arguments)
            && FullEq::full_eq(&self.pure, &other.pure)
    }
}

impl FullEq for ImportMeta {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for NewTarget {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for SpreadElement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.argument, &other.argument)
    }
}

impl FullEq for Argument<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::SpreadElement(a), Self::SpreadElement(b)) => FullEq::full_eq(a, b),
            (Self::BooleanLiteral(a), Self::BooleanLiteral(b)) => FullEq::full_eq(a, b),
            (Self::NullLiteral(a), Self::NullLiteral(b)) => FullEq::full_eq(a, b),
            (Self::NumericLiteral(a), Self::NumericLiteral(b)) => FullEq::full_eq(a, b),
            (Self::BigIntLiteral(a), Self::BigIntLiteral(b)) => FullEq::full_eq(a, b),
            (Self::RegExpLiteral(a), Self::RegExpLiteral(b)) => FullEq::full_eq(a, b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => FullEq::full_eq(a, b),
            (Self::TemplateLiteral(a), Self::TemplateLiteral(b)) => FullEq::full_eq(a, b),
            (Self::Identifier(a), Self::Identifier(b)) => FullEq::full_eq(a, b),
            (Self::Super(a), Self::Super(b)) => FullEq::full_eq(a, b),
            (Self::ArrayExpression(a), Self::ArrayExpression(b)) => FullEq::full_eq(a, b),
            (Self::ArrowFunctionExpression(a), Self::ArrowFunctionExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::AssignmentExpression(a), Self::AssignmentExpression(b)) => FullEq::full_eq(a, b),
            (Self::AwaitExpression(a), Self::AwaitExpression(b)) => FullEq::full_eq(a, b),
            (Self::BinaryExpression(a), Self::BinaryExpression(b)) => FullEq::full_eq(a, b),
            (Self::CallExpression(a), Self::CallExpression(b)) => FullEq::full_eq(a, b),
            (Self::ChainExpression(a), Self::ChainExpression(b)) => FullEq::full_eq(a, b),
            (Self::ClassExpression(a), Self::ClassExpression(b)) => FullEq::full_eq(a, b),
            (Self::ConditionalExpression(a), Self::ConditionalExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::FunctionExpression(a), Self::FunctionExpression(b)) => FullEq::full_eq(a, b),
            (Self::ImportExpression(a), Self::ImportExpression(b)) => FullEq::full_eq(a, b),
            (Self::LogicalExpression(a), Self::LogicalExpression(b)) => FullEq::full_eq(a, b),
            (Self::NewExpression(a), Self::NewExpression(b)) => FullEq::full_eq(a, b),
            (Self::ObjectExpression(a), Self::ObjectExpression(b)) => FullEq::full_eq(a, b),
            (Self::ParenthesizedExpression(a), Self::ParenthesizedExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::SequenceExpression(a), Self::SequenceExpression(b)) => FullEq::full_eq(a, b),
            (Self::TaggedTemplateExpression(a), Self::TaggedTemplateExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ThisExpression(a), Self::ThisExpression(b)) => FullEq::full_eq(a, b),
            (Self::UnaryExpression(a), Self::UnaryExpression(b)) => FullEq::full_eq(a, b),
            (Self::UpdateExpression(a), Self::UpdateExpression(b)) => FullEq::full_eq(a, b),
            (Self::YieldExpression(a), Self::YieldExpression(b)) => FullEq::full_eq(a, b),
            (Self::PrivateInExpression(a), Self::PrivateInExpression(b)) => FullEq::full_eq(a, b),
            (Self::ImportMeta(a), Self::ImportMeta(b)) => FullEq::full_eq(a, b),
            (Self::NewTarget(a), Self::NewTarget(b)) => FullEq::full_eq(a, b),
            (Self::JSXElement(a), Self::JSXElement(b)) => FullEq::full_eq(a, b),
            (Self::JSXFragment(a), Self::JSXFragment(b)) => FullEq::full_eq(a, b),
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => FullEq::full_eq(a, b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::V8IntrinsicExpression(a), Self::V8IntrinsicExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for UpdateExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.operator, &other.operator)
            && FullEq::full_eq(&self.prefix, &other.prefix)
            && FullEq::full_eq(&self.argument, &other.argument)
    }
}

impl FullEq for UnaryExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.operator, &other.operator)
            && FullEq::full_eq(&self.argument, &other.argument)
    }
}

impl FullEq for BinaryExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.left, &other.left)
            && FullEq::full_eq(&self.operator, &other.operator)
            && FullEq::full_eq(&self.right, &other.right)
    }
}

impl FullEq for PrivateInExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.left, &other.left)
            && FullEq::full_eq(&self.right, &other.right)
    }
}

impl FullEq for LogicalExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.left, &other.left)
            && FullEq::full_eq(&self.operator, &other.operator)
            && FullEq::full_eq(&self.right, &other.right)
    }
}

impl FullEq for ConditionalExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.test, &other.test)
            && FullEq::full_eq(&self.consequent, &other.consequent)
            && FullEq::full_eq(&self.alternate, &other.alternate)
    }
}

impl FullEq for AssignmentExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.operator, &other.operator)
            && FullEq::full_eq(&self.left, &other.left)
            && FullEq::full_eq(&self.right, &other.right)
    }
}

impl FullEq for AssignmentTarget<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::AssignmentTargetIdentifier(a), Self::AssignmentTargetIdentifier(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => FullEq::full_eq(a, b),
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ArrayAssignmentTarget(a), Self::ArrayAssignmentTarget(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ObjectAssignmentTarget(a), Self::ObjectAssignmentTarget(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for SimpleAssignmentTarget<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::AssignmentTargetIdentifier(a), Self::AssignmentTargetIdentifier(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => FullEq::full_eq(a, b),
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for AssignmentTargetPattern<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ArrayAssignmentTarget(a), Self::ArrayAssignmentTarget(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ObjectAssignmentTarget(a), Self::ObjectAssignmentTarget(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for ArrayAssignmentTarget<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.elements, &other.elements)
            && FullEq::full_eq(&self.rest, &other.rest)
    }
}

impl FullEq for ObjectAssignmentTarget<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.properties, &other.properties)
            && FullEq::full_eq(&self.rest, &other.rest)
    }
}

impl FullEq for AssignmentTargetRest<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.target, &other.target)
    }
}

impl FullEq for AssignmentTargetMaybeDefault<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::AssignmentTargetWithDefault(a), Self::AssignmentTargetWithDefault(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::AssignmentTargetIdentifier(a), Self::AssignmentTargetIdentifier(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => FullEq::full_eq(a, b),
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ArrayAssignmentTarget(a), Self::ArrayAssignmentTarget(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ObjectAssignmentTarget(a), Self::ObjectAssignmentTarget(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for AssignmentTargetWithDefault<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.binding, &other.binding)
            && FullEq::full_eq(&self.init, &other.init)
    }
}

impl FullEq for AssignmentTargetProperty<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::AssignmentTargetPropertyIdentifier(a),
                Self::AssignmentTargetPropertyIdentifier(b),
            ) => FullEq::full_eq(a, b),
            (
                Self::AssignmentTargetPropertyProperty(a),
                Self::AssignmentTargetPropertyProperty(b),
            ) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for AssignmentTargetPropertyIdentifier<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.binding, &other.binding)
            && FullEq::full_eq(&self.init, &other.init)
    }
}

impl FullEq for AssignmentTargetPropertyProperty<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.name, &other.name)
            && FullEq::full_eq(&self.binding, &other.binding)
    }
}

impl FullEq for SequenceExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.expressions, &other.expressions)
    }
}

impl FullEq for Super {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for AwaitExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.argument, &other.argument)
    }
}

impl FullEq for ChainExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.expression, &other.expression)
    }
}

impl FullEq for ChainElement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::CallExpression(a), Self::CallExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => FullEq::full_eq(a, b),
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for ParenthesizedExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.expression, &other.expression)
    }
}

impl FullEq for Statement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::BlockStatement(a), Self::BlockStatement(b)) => FullEq::full_eq(a, b),
            (Self::BreakStatement(a), Self::BreakStatement(b)) => FullEq::full_eq(a, b),
            (Self::ContinueStatement(a), Self::ContinueStatement(b)) => FullEq::full_eq(a, b),
            (Self::DebuggerStatement(a), Self::DebuggerStatement(b)) => FullEq::full_eq(a, b),
            (Self::DoWhileStatement(a), Self::DoWhileStatement(b)) => FullEq::full_eq(a, b),
            (Self::EmptyStatement(a), Self::EmptyStatement(b)) => FullEq::full_eq(a, b),
            (Self::ExpressionStatement(a), Self::ExpressionStatement(b)) => FullEq::full_eq(a, b),
            (Self::ForInStatement(a), Self::ForInStatement(b)) => FullEq::full_eq(a, b),
            (Self::ForOfStatement(a), Self::ForOfStatement(b)) => FullEq::full_eq(a, b),
            (Self::ForStatement(a), Self::ForStatement(b)) => FullEq::full_eq(a, b),
            (Self::IfStatement(a), Self::IfStatement(b)) => FullEq::full_eq(a, b),
            (Self::LabeledStatement(a), Self::LabeledStatement(b)) => FullEq::full_eq(a, b),
            (Self::ReturnStatement(a), Self::ReturnStatement(b)) => FullEq::full_eq(a, b),
            (Self::SwitchStatement(a), Self::SwitchStatement(b)) => FullEq::full_eq(a, b),
            (Self::ThrowStatement(a), Self::ThrowStatement(b)) => FullEq::full_eq(a, b),
            (Self::TryStatement(a), Self::TryStatement(b)) => FullEq::full_eq(a, b),
            (Self::WhileStatement(a), Self::WhileStatement(b)) => FullEq::full_eq(a, b),
            (Self::WithStatement(a), Self::WithStatement(b)) => FullEq::full_eq(a, b),
            (Self::VariableDeclaration(a), Self::VariableDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::FunctionDeclaration(a), Self::FunctionDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::ClassDeclaration(a), Self::ClassDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::TSTypeAliasDeclaration(a), Self::TSTypeAliasDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSInterfaceDeclaration(a), Self::TSInterfaceDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSEnumDeclaration(a), Self::TSEnumDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::TSExternalModuleDeclaration(a), Self::TSExternalModuleDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSNamespaceDeclaration(a), Self::TSNamespaceDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSGlobalDeclaration(a), Self::TSGlobalDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::TSImportEqualsDeclaration(a), Self::TSImportEqualsDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ImportDeclaration(a), Self::ImportDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::ExportAllDeclaration(a), Self::ExportAllDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::ExportDefaultDeclaration(a), Self::ExportDefaultDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ExportDeclaration(a), Self::ExportDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::ExportNamedDeclaration(a), Self::ExportNamedDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ExportFromDeclaration(a), Self::ExportFromDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSExportAssignment(a), Self::TSExportAssignment(b)) => FullEq::full_eq(a, b),
            (Self::TSNamespaceExportDeclaration(a), Self::TSNamespaceExportDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for Directive<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.expression, &other.expression)
            && FullEq::full_eq(&self.directive, &other.directive)
    }
}

impl FullEq for Hashbang<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.value, &other.value)
    }
}

impl FullEq for BlockStatement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.body, &other.body)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for Declaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::VariableDeclaration(a), Self::VariableDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::FunctionDeclaration(a), Self::FunctionDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::ClassDeclaration(a), Self::ClassDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::TSTypeAliasDeclaration(a), Self::TSTypeAliasDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSInterfaceDeclaration(a), Self::TSInterfaceDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSEnumDeclaration(a), Self::TSEnumDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::TSExternalModuleDeclaration(a), Self::TSExternalModuleDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSNamespaceDeclaration(a), Self::TSNamespaceDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSGlobalDeclaration(a), Self::TSGlobalDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::TSImportEqualsDeclaration(a), Self::TSImportEqualsDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for VariableDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.kind, &other.kind)
            && FullEq::full_eq(&self.declarations, &other.declarations)
            && FullEq::full_eq(&self.declare, &other.declare)
    }
}

impl FullEq for VariableDeclarationKind {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for VariableDeclarator<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.id, &other.id)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
            && FullEq::full_eq(&self.init, &other.init)
            && FullEq::full_eq(&self.definite, &other.definite)
    }
}

impl FullEq for EmptyStatement {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for ExpressionStatement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.expression, &other.expression)
    }
}

impl FullEq for IfStatement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.test, &other.test)
            && FullEq::full_eq(&self.consequent, &other.consequent)
            && FullEq::full_eq(&self.alternate, &other.alternate)
    }
}

impl FullEq for DoWhileStatement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.body, &other.body)
            && FullEq::full_eq(&self.test, &other.test)
    }
}

impl FullEq for WhileStatement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.test, &other.test)
            && FullEq::full_eq(&self.body, &other.body)
    }
}

impl FullEq for ForStatement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.init, &other.init)
            && FullEq::full_eq(&self.test, &other.test)
            && FullEq::full_eq(&self.update, &other.update)
            && FullEq::full_eq(&self.body, &other.body)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for ForStatementInit<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::VariableDeclaration(a), Self::VariableDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::BooleanLiteral(a), Self::BooleanLiteral(b)) => FullEq::full_eq(a, b),
            (Self::NullLiteral(a), Self::NullLiteral(b)) => FullEq::full_eq(a, b),
            (Self::NumericLiteral(a), Self::NumericLiteral(b)) => FullEq::full_eq(a, b),
            (Self::BigIntLiteral(a), Self::BigIntLiteral(b)) => FullEq::full_eq(a, b),
            (Self::RegExpLiteral(a), Self::RegExpLiteral(b)) => FullEq::full_eq(a, b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => FullEq::full_eq(a, b),
            (Self::TemplateLiteral(a), Self::TemplateLiteral(b)) => FullEq::full_eq(a, b),
            (Self::Identifier(a), Self::Identifier(b)) => FullEq::full_eq(a, b),
            (Self::Super(a), Self::Super(b)) => FullEq::full_eq(a, b),
            (Self::ArrayExpression(a), Self::ArrayExpression(b)) => FullEq::full_eq(a, b),
            (Self::ArrowFunctionExpression(a), Self::ArrowFunctionExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::AssignmentExpression(a), Self::AssignmentExpression(b)) => FullEq::full_eq(a, b),
            (Self::AwaitExpression(a), Self::AwaitExpression(b)) => FullEq::full_eq(a, b),
            (Self::BinaryExpression(a), Self::BinaryExpression(b)) => FullEq::full_eq(a, b),
            (Self::CallExpression(a), Self::CallExpression(b)) => FullEq::full_eq(a, b),
            (Self::ChainExpression(a), Self::ChainExpression(b)) => FullEq::full_eq(a, b),
            (Self::ClassExpression(a), Self::ClassExpression(b)) => FullEq::full_eq(a, b),
            (Self::ConditionalExpression(a), Self::ConditionalExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::FunctionExpression(a), Self::FunctionExpression(b)) => FullEq::full_eq(a, b),
            (Self::ImportExpression(a), Self::ImportExpression(b)) => FullEq::full_eq(a, b),
            (Self::LogicalExpression(a), Self::LogicalExpression(b)) => FullEq::full_eq(a, b),
            (Self::NewExpression(a), Self::NewExpression(b)) => FullEq::full_eq(a, b),
            (Self::ObjectExpression(a), Self::ObjectExpression(b)) => FullEq::full_eq(a, b),
            (Self::ParenthesizedExpression(a), Self::ParenthesizedExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::SequenceExpression(a), Self::SequenceExpression(b)) => FullEq::full_eq(a, b),
            (Self::TaggedTemplateExpression(a), Self::TaggedTemplateExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ThisExpression(a), Self::ThisExpression(b)) => FullEq::full_eq(a, b),
            (Self::UnaryExpression(a), Self::UnaryExpression(b)) => FullEq::full_eq(a, b),
            (Self::UpdateExpression(a), Self::UpdateExpression(b)) => FullEq::full_eq(a, b),
            (Self::YieldExpression(a), Self::YieldExpression(b)) => FullEq::full_eq(a, b),
            (Self::PrivateInExpression(a), Self::PrivateInExpression(b)) => FullEq::full_eq(a, b),
            (Self::ImportMeta(a), Self::ImportMeta(b)) => FullEq::full_eq(a, b),
            (Self::NewTarget(a), Self::NewTarget(b)) => FullEq::full_eq(a, b),
            (Self::JSXElement(a), Self::JSXElement(b)) => FullEq::full_eq(a, b),
            (Self::JSXFragment(a), Self::JSXFragment(b)) => FullEq::full_eq(a, b),
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => FullEq::full_eq(a, b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::V8IntrinsicExpression(a), Self::V8IntrinsicExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for ForInStatement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.left, &other.left)
            && FullEq::full_eq(&self.right, &other.right)
            && FullEq::full_eq(&self.body, &other.body)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for ForStatementLeft<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::VariableDeclaration(a), Self::VariableDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::AssignmentTargetIdentifier(a), Self::AssignmentTargetIdentifier(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => FullEq::full_eq(a, b),
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ArrayAssignmentTarget(a), Self::ArrayAssignmentTarget(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ObjectAssignmentTarget(a), Self::ObjectAssignmentTarget(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for ForOfStatement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.r#await, &other.r#await)
            && FullEq::full_eq(&self.left, &other.left)
            && FullEq::full_eq(&self.right, &other.right)
            && FullEq::full_eq(&self.body, &other.body)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for ContinueStatement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.label, &other.label)
    }
}

impl FullEq for BreakStatement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.label, &other.label)
    }
}

impl FullEq for ReturnStatement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.argument, &other.argument)
    }
}

impl FullEq for WithStatement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.object, &other.object)
            && FullEq::full_eq(&self.body, &other.body)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for SwitchStatement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.discriminant, &other.discriminant)
            && FullEq::full_eq(&self.cases, &other.cases)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for SwitchCase<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.test, &other.test)
            && FullEq::full_eq(&self.consequent, &other.consequent)
    }
}

impl FullEq for LabeledStatement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.label, &other.label)
            && FullEq::full_eq(&self.body, &other.body)
    }
}

impl FullEq for ThrowStatement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.argument, &other.argument)
    }
}

impl FullEq for TryStatement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.block, &other.block)
            && FullEq::full_eq(&self.handler, &other.handler)
            && FullEq::full_eq(&self.finalizer, &other.finalizer)
    }
}

impl FullEq for CatchClause<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.param, &other.param)
            && FullEq::full_eq(&self.body, &other.body)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for CatchParameter<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.pattern, &other.pattern)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl FullEq for DebuggerStatement {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for BindingPattern<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::BindingIdentifier(a), Self::BindingIdentifier(b)) => FullEq::full_eq(a, b),
            (Self::ObjectPattern(a), Self::ObjectPattern(b)) => FullEq::full_eq(a, b),
            (Self::ArrayPattern(a), Self::ArrayPattern(b)) => FullEq::full_eq(a, b),
            (Self::AssignmentPattern(a), Self::AssignmentPattern(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for AssignmentPattern<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.left, &other.left)
            && FullEq::full_eq(&self.right, &other.right)
    }
}

impl FullEq for ObjectPattern<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.properties, &other.properties)
            && FullEq::full_eq(&self.rest, &other.rest)
    }
}

impl FullEq for BindingProperty<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.key, &other.key)
            && FullEq::full_eq(&self.value, &other.value)
            && FullEq::full_eq(&self.shorthand, &other.shorthand)
    }
}

impl FullEq for ArrayPattern<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.elements, &other.elements)
            && FullEq::full_eq(&self.rest, &other.rest)
    }
}

impl FullEq for BindingRestElement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.argument, &other.argument)
    }
}

impl FullEq for Function<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.r#type, &other.r#type)
            && FullEq::full_eq(&self.id, &other.id)
            && FullEq::full_eq(&self.generator, &other.generator)
            && FullEq::full_eq(&self.r#async, &other.r#async)
            && FullEq::full_eq(&self.declare, &other.declare)
            && FullEq::full_eq(&self.type_parameters, &other.type_parameters)
            && FullEq::full_eq(&self.this_param, &other.this_param)
            && FullEq::full_eq(&self.params, &other.params)
            && FullEq::full_eq(&self.return_type, &other.return_type)
            && FullEq::full_eq(&self.body, &other.body)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
            && FullEq::full_eq(&self.pure, &other.pure)
            && FullEq::full_eq(&self.pife, &other.pife)
    }
}

impl FullEq for FunctionType {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for FormalParameters<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.kind, &other.kind)
            && FullEq::full_eq(&self.items, &other.items)
            && FullEq::full_eq(&self.rest, &other.rest)
    }
}

impl FullEq for FormalParameter<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.decorators, &other.decorators)
            && FullEq::full_eq(&self.pattern, &other.pattern)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
            && FullEq::full_eq(&self.initializer, &other.initializer)
            && FullEq::full_eq(&self.optional, &other.optional)
            && FullEq::full_eq(&self.accessibility, &other.accessibility)
            && FullEq::full_eq(&self.readonly, &other.readonly)
            && FullEq::full_eq(&self.r#override, &other.r#override)
    }
}

impl FullEq for FormalParameterKind {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for FormalParameterRest<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.decorators, &other.decorators)
            && FullEq::full_eq(&self.rest, &other.rest)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl FullEq for FunctionBody<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.directives, &other.directives)
            && FullEq::full_eq(&self.statements, &other.statements)
    }
}

impl FullEq for ArrowFunctionBody<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::FunctionBody(a), Self::FunctionBody(b)) => FullEq::full_eq(a, b),
            (Self::BooleanLiteral(a), Self::BooleanLiteral(b)) => FullEq::full_eq(a, b),
            (Self::NullLiteral(a), Self::NullLiteral(b)) => FullEq::full_eq(a, b),
            (Self::NumericLiteral(a), Self::NumericLiteral(b)) => FullEq::full_eq(a, b),
            (Self::BigIntLiteral(a), Self::BigIntLiteral(b)) => FullEq::full_eq(a, b),
            (Self::RegExpLiteral(a), Self::RegExpLiteral(b)) => FullEq::full_eq(a, b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => FullEq::full_eq(a, b),
            (Self::TemplateLiteral(a), Self::TemplateLiteral(b)) => FullEq::full_eq(a, b),
            (Self::Identifier(a), Self::Identifier(b)) => FullEq::full_eq(a, b),
            (Self::Super(a), Self::Super(b)) => FullEq::full_eq(a, b),
            (Self::ArrayExpression(a), Self::ArrayExpression(b)) => FullEq::full_eq(a, b),
            (Self::ArrowFunctionExpression(a), Self::ArrowFunctionExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::AssignmentExpression(a), Self::AssignmentExpression(b)) => FullEq::full_eq(a, b),
            (Self::AwaitExpression(a), Self::AwaitExpression(b)) => FullEq::full_eq(a, b),
            (Self::BinaryExpression(a), Self::BinaryExpression(b)) => FullEq::full_eq(a, b),
            (Self::CallExpression(a), Self::CallExpression(b)) => FullEq::full_eq(a, b),
            (Self::ChainExpression(a), Self::ChainExpression(b)) => FullEq::full_eq(a, b),
            (Self::ClassExpression(a), Self::ClassExpression(b)) => FullEq::full_eq(a, b),
            (Self::ConditionalExpression(a), Self::ConditionalExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::FunctionExpression(a), Self::FunctionExpression(b)) => FullEq::full_eq(a, b),
            (Self::ImportExpression(a), Self::ImportExpression(b)) => FullEq::full_eq(a, b),
            (Self::LogicalExpression(a), Self::LogicalExpression(b)) => FullEq::full_eq(a, b),
            (Self::NewExpression(a), Self::NewExpression(b)) => FullEq::full_eq(a, b),
            (Self::ObjectExpression(a), Self::ObjectExpression(b)) => FullEq::full_eq(a, b),
            (Self::ParenthesizedExpression(a), Self::ParenthesizedExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::SequenceExpression(a), Self::SequenceExpression(b)) => FullEq::full_eq(a, b),
            (Self::TaggedTemplateExpression(a), Self::TaggedTemplateExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ThisExpression(a), Self::ThisExpression(b)) => FullEq::full_eq(a, b),
            (Self::UnaryExpression(a), Self::UnaryExpression(b)) => FullEq::full_eq(a, b),
            (Self::UpdateExpression(a), Self::UpdateExpression(b)) => FullEq::full_eq(a, b),
            (Self::YieldExpression(a), Self::YieldExpression(b)) => FullEq::full_eq(a, b),
            (Self::PrivateInExpression(a), Self::PrivateInExpression(b)) => FullEq::full_eq(a, b),
            (Self::ImportMeta(a), Self::ImportMeta(b)) => FullEq::full_eq(a, b),
            (Self::NewTarget(a), Self::NewTarget(b)) => FullEq::full_eq(a, b),
            (Self::JSXElement(a), Self::JSXElement(b)) => FullEq::full_eq(a, b),
            (Self::JSXFragment(a), Self::JSXFragment(b)) => FullEq::full_eq(a, b),
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => FullEq::full_eq(a, b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::V8IntrinsicExpression(a), Self::V8IntrinsicExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for ArrowFunctionExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.r#async, &other.r#async)
            && FullEq::full_eq(&self.type_parameters, &other.type_parameters)
            && FullEq::full_eq(&self.params, &other.params)
            && FullEq::full_eq(&self.return_type, &other.return_type)
            && FullEq::full_eq(&self.body, &other.body)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
            && FullEq::full_eq(&self.pure, &other.pure)
            && FullEq::full_eq(&self.pife, &other.pife)
    }
}

impl FullEq for YieldExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.delegate, &other.delegate)
            && FullEq::full_eq(&self.argument, &other.argument)
    }
}

impl FullEq for Class<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.r#type, &other.r#type)
            && FullEq::full_eq(&self.decorators, &other.decorators)
            && FullEq::full_eq(&self.id, &other.id)
            && FullEq::full_eq(&self.type_parameters, &other.type_parameters)
            && FullEq::full_eq(&self.heritage, &other.heritage)
            && FullEq::full_eq(&self.implements, &other.implements)
            && FullEq::full_eq(&self.body, &other.body)
            && FullEq::full_eq(&self.r#abstract, &other.r#abstract)
            && FullEq::full_eq(&self.declare, &other.declare)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for ClassHeritage<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.expression, &other.expression)
            && FullEq::full_eq(&self.type_arguments, &other.type_arguments)
    }
}

impl FullEq for ClassType {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for ClassBody<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.body, &other.body)
    }
}

impl FullEq for ClassElement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::StaticBlock(a), Self::StaticBlock(b)) => FullEq::full_eq(a, b),
            (Self::MethodDefinition(a), Self::MethodDefinition(b)) => FullEq::full_eq(a, b),
            (Self::PropertyDefinition(a), Self::PropertyDefinition(b)) => FullEq::full_eq(a, b),
            (Self::AccessorProperty(a), Self::AccessorProperty(b)) => FullEq::full_eq(a, b),
            (Self::TSIndexSignature(a), Self::TSIndexSignature(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for MethodDefinition<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.r#type, &other.r#type)
            && FullEq::full_eq(&self.decorators, &other.decorators)
            && FullEq::full_eq(&self.key, &other.key)
            && FullEq::full_eq(&self.value, &other.value)
            && FullEq::full_eq(&self.kind, &other.kind)
            && FullEq::full_eq(&self.r#static, &other.r#static)
            && FullEq::full_eq(&self.r#override, &other.r#override)
            && FullEq::full_eq(&self.optional, &other.optional)
            && FullEq::full_eq(&self.accessibility, &other.accessibility)
    }
}

impl FullEq for MethodDefinitionType {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for PropertyDefinition<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.r#type, &other.r#type)
            && FullEq::full_eq(&self.decorators, &other.decorators)
            && FullEq::full_eq(&self.key, &other.key)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
            && FullEq::full_eq(&self.value, &other.value)
            && FullEq::full_eq(&self.r#static, &other.r#static)
            && FullEq::full_eq(&self.declare, &other.declare)
            && FullEq::full_eq(&self.r#override, &other.r#override)
            && FullEq::full_eq(&self.optional, &other.optional)
            && FullEq::full_eq(&self.definite, &other.definite)
            && FullEq::full_eq(&self.readonly, &other.readonly)
            && FullEq::full_eq(&self.accessibility, &other.accessibility)
    }
}

impl FullEq for PropertyDefinitionType {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for MethodDefinitionKind {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for PrivateIdentifier<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.name, &other.name)
    }
}

impl FullEq for StaticBlock<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.body, &other.body)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for ModuleDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ImportDeclaration(a), Self::ImportDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::ExportAllDeclaration(a), Self::ExportAllDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::ExportDefaultDeclaration(a), Self::ExportDefaultDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ExportDeclaration(a), Self::ExportDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::ExportNamedDeclaration(a), Self::ExportNamedDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ExportFromDeclaration(a), Self::ExportFromDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSExportAssignment(a), Self::TSExportAssignment(b)) => FullEq::full_eq(a, b),
            (Self::TSNamespaceExportDeclaration(a), Self::TSNamespaceExportDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for AccessorPropertyType {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for AccessorProperty<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.r#type, &other.r#type)
            && FullEq::full_eq(&self.decorators, &other.decorators)
            && FullEq::full_eq(&self.key, &other.key)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
            && FullEq::full_eq(&self.value, &other.value)
            && FullEq::full_eq(&self.r#static, &other.r#static)
            && FullEq::full_eq(&self.r#override, &other.r#override)
            && FullEq::full_eq(&self.definite, &other.definite)
            && FullEq::full_eq(&self.accessibility, &other.accessibility)
    }
}

impl FullEq for ImportExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.source, &other.source)
            && FullEq::full_eq(&self.options, &other.options)
            && FullEq::full_eq(&self.phase, &other.phase)
    }
}

impl FullEq for ImportDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.specifiers, &other.specifiers)
            && FullEq::full_eq(&self.source, &other.source)
            && FullEq::full_eq(&self.phase, &other.phase)
            && FullEq::full_eq(&self.with_clause, &other.with_clause)
            && FullEq::full_eq(&self.import_kind, &other.import_kind)
    }
}

impl FullEq for ImportPhase {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for ImportDeclarationSpecifier<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ImportSpecifier(a), Self::ImportSpecifier(b)) => FullEq::full_eq(a, b),
            (Self::ImportDefaultSpecifier(a), Self::ImportDefaultSpecifier(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ImportNamespaceSpecifier(a), Self::ImportNamespaceSpecifier(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for ImportSpecifier<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.imported, &other.imported)
            && FullEq::full_eq(&self.local, &other.local)
            && FullEq::full_eq(&self.import_kind, &other.import_kind)
    }
}

impl FullEq for ImportDefaultSpecifier<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.local, &other.local)
    }
}

impl FullEq for ImportNamespaceSpecifier<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.local, &other.local)
    }
}

impl FullEq for WithClause<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.keyword, &other.keyword)
            && FullEq::full_eq(&self.with_entries, &other.with_entries)
    }
}

impl FullEq for WithClauseKeyword {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for ImportAttribute<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.key, &other.key)
            && FullEq::full_eq(&self.value, &other.value)
    }
}

impl FullEq for ImportAttributeKey<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Identifier(a), Self::Identifier(b)) => FullEq::full_eq(a, b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for ExportDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.declaration, &other.declaration)
    }
}

impl FullEq for ExportNamedDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.specifiers, &other.specifiers)
            && FullEq::full_eq(&self.export_kind, &other.export_kind)
    }
}

impl FullEq for ExportFromDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.specifiers, &other.specifiers)
            && FullEq::full_eq(&self.source, &other.source)
            && FullEq::full_eq(&self.export_kind, &other.export_kind)
            && FullEq::full_eq(&self.with_clause, &other.with_clause)
    }
}

impl FullEq for ExportDefaultDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.declaration, &other.declaration)
    }
}

impl FullEq for ExportAllDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.exported, &other.exported)
            && FullEq::full_eq(&self.source, &other.source)
            && FullEq::full_eq(&self.with_clause, &other.with_clause)
            && FullEq::full_eq(&self.export_kind, &other.export_kind)
    }
}

impl FullEq for ExportSpecifier<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.local, &other.local)
            && FullEq::full_eq(&self.exported, &other.exported)
            && FullEq::full_eq(&self.export_kind, &other.export_kind)
    }
}

impl FullEq for ExportDefaultDeclarationKind<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::FunctionDeclaration(a), Self::FunctionDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::ClassDeclaration(a), Self::ClassDeclaration(b)) => FullEq::full_eq(a, b),
            (Self::TSInterfaceDeclaration(a), Self::TSInterfaceDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::BooleanLiteral(a), Self::BooleanLiteral(b)) => FullEq::full_eq(a, b),
            (Self::NullLiteral(a), Self::NullLiteral(b)) => FullEq::full_eq(a, b),
            (Self::NumericLiteral(a), Self::NumericLiteral(b)) => FullEq::full_eq(a, b),
            (Self::BigIntLiteral(a), Self::BigIntLiteral(b)) => FullEq::full_eq(a, b),
            (Self::RegExpLiteral(a), Self::RegExpLiteral(b)) => FullEq::full_eq(a, b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => FullEq::full_eq(a, b),
            (Self::TemplateLiteral(a), Self::TemplateLiteral(b)) => FullEq::full_eq(a, b),
            (Self::Identifier(a), Self::Identifier(b)) => FullEq::full_eq(a, b),
            (Self::Super(a), Self::Super(b)) => FullEq::full_eq(a, b),
            (Self::ArrayExpression(a), Self::ArrayExpression(b)) => FullEq::full_eq(a, b),
            (Self::ArrowFunctionExpression(a), Self::ArrowFunctionExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::AssignmentExpression(a), Self::AssignmentExpression(b)) => FullEq::full_eq(a, b),
            (Self::AwaitExpression(a), Self::AwaitExpression(b)) => FullEq::full_eq(a, b),
            (Self::BinaryExpression(a), Self::BinaryExpression(b)) => FullEq::full_eq(a, b),
            (Self::CallExpression(a), Self::CallExpression(b)) => FullEq::full_eq(a, b),
            (Self::ChainExpression(a), Self::ChainExpression(b)) => FullEq::full_eq(a, b),
            (Self::ClassExpression(a), Self::ClassExpression(b)) => FullEq::full_eq(a, b),
            (Self::ConditionalExpression(a), Self::ConditionalExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::FunctionExpression(a), Self::FunctionExpression(b)) => FullEq::full_eq(a, b),
            (Self::ImportExpression(a), Self::ImportExpression(b)) => FullEq::full_eq(a, b),
            (Self::LogicalExpression(a), Self::LogicalExpression(b)) => FullEq::full_eq(a, b),
            (Self::NewExpression(a), Self::NewExpression(b)) => FullEq::full_eq(a, b),
            (Self::ObjectExpression(a), Self::ObjectExpression(b)) => FullEq::full_eq(a, b),
            (Self::ParenthesizedExpression(a), Self::ParenthesizedExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::SequenceExpression(a), Self::SequenceExpression(b)) => FullEq::full_eq(a, b),
            (Self::TaggedTemplateExpression(a), Self::TaggedTemplateExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ThisExpression(a), Self::ThisExpression(b)) => FullEq::full_eq(a, b),
            (Self::UnaryExpression(a), Self::UnaryExpression(b)) => FullEq::full_eq(a, b),
            (Self::UpdateExpression(a), Self::UpdateExpression(b)) => FullEq::full_eq(a, b),
            (Self::YieldExpression(a), Self::YieldExpression(b)) => FullEq::full_eq(a, b),
            (Self::PrivateInExpression(a), Self::PrivateInExpression(b)) => FullEq::full_eq(a, b),
            (Self::ImportMeta(a), Self::ImportMeta(b)) => FullEq::full_eq(a, b),
            (Self::NewTarget(a), Self::NewTarget(b)) => FullEq::full_eq(a, b),
            (Self::JSXElement(a), Self::JSXElement(b)) => FullEq::full_eq(a, b),
            (Self::JSXFragment(a), Self::JSXFragment(b)) => FullEq::full_eq(a, b),
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => FullEq::full_eq(a, b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::V8IntrinsicExpression(a), Self::V8IntrinsicExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for ModuleExportName<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::IdentifierName(a), Self::IdentifierName(b)) => FullEq::full_eq(a, b),
            (Self::IdentifierReference(a), Self::IdentifierReference(b)) => FullEq::full_eq(a, b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for V8IntrinsicExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.name, &other.name)
            && FullEq::full_eq(&self.arguments, &other.arguments)
    }
}

impl FullEq for BooleanLiteral {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.value, &other.value)
    }
}

impl FullEq for NullLiteral {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for NumericLiteral<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.value, &other.value)
            && FullEq::full_eq(&self.raw, &other.raw)
    }
}

impl FullEq for StringLiteral<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.value, &other.value)
            && FullEq::full_eq(&self.raw, &other.raw)
    }
}

impl FullEq for BigIntLiteral<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.value, &other.value)
            && FullEq::full_eq(&self.raw, &other.raw)
    }
}

impl FullEq for RegExpLiteral<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.regex, &other.regex)
            && FullEq::full_eq(&self.raw, &other.raw)
    }
}

impl FullEq for RegExp<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.pattern, &other.pattern) && FullEq::full_eq(&self.flags, &other.flags)
    }
}

impl FullEq for RegExpPattern<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.text, &other.text) && FullEq::full_eq(&self.pattern, &other.pattern)
    }
}

impl FullEq for RegExpFlags {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for JSXElement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.opening_element, &other.opening_element)
            && FullEq::full_eq(&self.children, &other.children)
            && FullEq::full_eq(&self.closing_element, &other.closing_element)
    }
}

impl FullEq for JSXOpeningElement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.name, &other.name)
            && FullEq::full_eq(&self.type_arguments, &other.type_arguments)
            && FullEq::full_eq(&self.attributes, &other.attributes)
    }
}

impl FullEq for JSXClosingElement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.name, &other.name)
    }
}

impl FullEq for JSXFragment<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.opening_fragment, &other.opening_fragment)
            && FullEq::full_eq(&self.children, &other.children)
            && FullEq::full_eq(&self.closing_fragment, &other.closing_fragment)
    }
}

impl FullEq for JSXOpeningFragment {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for JSXClosingFragment {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for JSXElementName<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Identifier(a), Self::Identifier(b)) => FullEq::full_eq(a, b),
            (Self::IdentifierReference(a), Self::IdentifierReference(b)) => FullEq::full_eq(a, b),
            (Self::NamespacedName(a), Self::NamespacedName(b)) => FullEq::full_eq(a, b),
            (Self::MemberExpression(a), Self::MemberExpression(b)) => FullEq::full_eq(a, b),
            (Self::ThisExpression(a), Self::ThisExpression(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for JSXNamespacedName<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.namespace, &other.namespace)
            && FullEq::full_eq(&self.name, &other.name)
    }
}

impl FullEq for JSXMemberExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.object, &other.object)
            && FullEq::full_eq(&self.property, &other.property)
    }
}

impl FullEq for JSXMemberExpressionObject<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::IdentifierReference(a), Self::IdentifierReference(b)) => FullEq::full_eq(a, b),
            (Self::MemberExpression(a), Self::MemberExpression(b)) => FullEq::full_eq(a, b),
            (Self::ThisExpression(a), Self::ThisExpression(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for JSXExpressionContainer<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.expression, &other.expression)
    }
}

impl FullEq for JSXExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::EmptyExpression(a), Self::EmptyExpression(b)) => FullEq::full_eq(a, b),
            (Self::BooleanLiteral(a), Self::BooleanLiteral(b)) => FullEq::full_eq(a, b),
            (Self::NullLiteral(a), Self::NullLiteral(b)) => FullEq::full_eq(a, b),
            (Self::NumericLiteral(a), Self::NumericLiteral(b)) => FullEq::full_eq(a, b),
            (Self::BigIntLiteral(a), Self::BigIntLiteral(b)) => FullEq::full_eq(a, b),
            (Self::RegExpLiteral(a), Self::RegExpLiteral(b)) => FullEq::full_eq(a, b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => FullEq::full_eq(a, b),
            (Self::TemplateLiteral(a), Self::TemplateLiteral(b)) => FullEq::full_eq(a, b),
            (Self::Identifier(a), Self::Identifier(b)) => FullEq::full_eq(a, b),
            (Self::Super(a), Self::Super(b)) => FullEq::full_eq(a, b),
            (Self::ArrayExpression(a), Self::ArrayExpression(b)) => FullEq::full_eq(a, b),
            (Self::ArrowFunctionExpression(a), Self::ArrowFunctionExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::AssignmentExpression(a), Self::AssignmentExpression(b)) => FullEq::full_eq(a, b),
            (Self::AwaitExpression(a), Self::AwaitExpression(b)) => FullEq::full_eq(a, b),
            (Self::BinaryExpression(a), Self::BinaryExpression(b)) => FullEq::full_eq(a, b),
            (Self::CallExpression(a), Self::CallExpression(b)) => FullEq::full_eq(a, b),
            (Self::ChainExpression(a), Self::ChainExpression(b)) => FullEq::full_eq(a, b),
            (Self::ClassExpression(a), Self::ClassExpression(b)) => FullEq::full_eq(a, b),
            (Self::ConditionalExpression(a), Self::ConditionalExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::FunctionExpression(a), Self::FunctionExpression(b)) => FullEq::full_eq(a, b),
            (Self::ImportExpression(a), Self::ImportExpression(b)) => FullEq::full_eq(a, b),
            (Self::LogicalExpression(a), Self::LogicalExpression(b)) => FullEq::full_eq(a, b),
            (Self::NewExpression(a), Self::NewExpression(b)) => FullEq::full_eq(a, b),
            (Self::ObjectExpression(a), Self::ObjectExpression(b)) => FullEq::full_eq(a, b),
            (Self::ParenthesizedExpression(a), Self::ParenthesizedExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::SequenceExpression(a), Self::SequenceExpression(b)) => FullEq::full_eq(a, b),
            (Self::TaggedTemplateExpression(a), Self::TaggedTemplateExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ThisExpression(a), Self::ThisExpression(b)) => FullEq::full_eq(a, b),
            (Self::UnaryExpression(a), Self::UnaryExpression(b)) => FullEq::full_eq(a, b),
            (Self::UpdateExpression(a), Self::UpdateExpression(b)) => FullEq::full_eq(a, b),
            (Self::YieldExpression(a), Self::YieldExpression(b)) => FullEq::full_eq(a, b),
            (Self::PrivateInExpression(a), Self::PrivateInExpression(b)) => FullEq::full_eq(a, b),
            (Self::ImportMeta(a), Self::ImportMeta(b)) => FullEq::full_eq(a, b),
            (Self::NewTarget(a), Self::NewTarget(b)) => FullEq::full_eq(a, b),
            (Self::JSXElement(a), Self::JSXElement(b)) => FullEq::full_eq(a, b),
            (Self::JSXFragment(a), Self::JSXFragment(b)) => FullEq::full_eq(a, b),
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => FullEq::full_eq(a, b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => FullEq::full_eq(a, b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::V8IntrinsicExpression(a), Self::V8IntrinsicExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for JSXEmptyExpression {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for JSXAttributeItem<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Attribute(a), Self::Attribute(b)) => FullEq::full_eq(a, b),
            (Self::SpreadAttribute(a), Self::SpreadAttribute(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for JSXAttribute<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.name, &other.name)
            && FullEq::full_eq(&self.value, &other.value)
    }
}

impl FullEq for JSXSpreadAttribute<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.argument, &other.argument)
    }
}

impl FullEq for JSXAttributeName<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Identifier(a), Self::Identifier(b)) => FullEq::full_eq(a, b),
            (Self::NamespacedName(a), Self::NamespacedName(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for JSXAttributeValue<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::StringLiteral(a), Self::StringLiteral(b)) => FullEq::full_eq(a, b),
            (Self::ExpressionContainer(a), Self::ExpressionContainer(b)) => FullEq::full_eq(a, b),
            (Self::Element(a), Self::Element(b)) => FullEq::full_eq(a, b),
            (Self::Fragment(a), Self::Fragment(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for JSXIdentifier<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.name, &other.name)
    }
}

impl FullEq for JSXChild<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Text(a), Self::Text(b)) => FullEq::full_eq(a, b),
            (Self::Element(a), Self::Element(b)) => FullEq::full_eq(a, b),
            (Self::Fragment(a), Self::Fragment(b)) => FullEq::full_eq(a, b),
            (Self::ExpressionContainer(a), Self::ExpressionContainer(b)) => FullEq::full_eq(a, b),
            (Self::Spread(a), Self::Spread(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for JSXSpreadChild<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.expression, &other.expression)
    }
}

impl FullEq for JSXText<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.value, &other.value)
            && FullEq::full_eq(&self.raw, &other.raw)
    }
}

impl FullEq for TSThisParameter<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.this_span, &other.this_span)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl FullEq for TSEnumDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.id, &other.id)
            && FullEq::full_eq(&self.body, &other.body)
            && FullEq::full_eq(&self.r#const, &other.r#const)
            && FullEq::full_eq(&self.declare, &other.declare)
    }
}

impl FullEq for TSEnumBody<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.members, &other.members)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for TSEnumMember<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.id, &other.id)
            && FullEq::full_eq(&self.initializer, &other.initializer)
    }
}

impl FullEq for TSEnumMemberName<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Identifier(a), Self::Identifier(b)) => FullEq::full_eq(a, b),
            (Self::String(a), Self::String(b)) => FullEq::full_eq(a, b),
            (Self::ComputedString(a), Self::ComputedString(b)) => FullEq::full_eq(a, b),
            (Self::ComputedTemplateString(a), Self::ComputedTemplateString(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for TSTypeAnnotation<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl FullEq for TSLiteralType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.literal, &other.literal)
    }
}

impl FullEq for TSLiteral<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::BooleanLiteral(a), Self::BooleanLiteral(b)) => FullEq::full_eq(a, b),
            (Self::NumericLiteral(a), Self::NumericLiteral(b)) => FullEq::full_eq(a, b),
            (Self::BigIntLiteral(a), Self::BigIntLiteral(b)) => FullEq::full_eq(a, b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => FullEq::full_eq(a, b),
            (Self::TemplateLiteral(a), Self::TemplateLiteral(b)) => FullEq::full_eq(a, b),
            (Self::UnaryExpression(a), Self::UnaryExpression(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for TSType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::TSAnyKeyword(a), Self::TSAnyKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSBigIntKeyword(a), Self::TSBigIntKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSBooleanKeyword(a), Self::TSBooleanKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSIntrinsicKeyword(a), Self::TSIntrinsicKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSNeverKeyword(a), Self::TSNeverKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSNullKeyword(a), Self::TSNullKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSNumberKeyword(a), Self::TSNumberKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSObjectKeyword(a), Self::TSObjectKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSStringKeyword(a), Self::TSStringKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSSymbolKeyword(a), Self::TSSymbolKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSUndefinedKeyword(a), Self::TSUndefinedKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSUnknownKeyword(a), Self::TSUnknownKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSVoidKeyword(a), Self::TSVoidKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSArrayType(a), Self::TSArrayType(b)) => FullEq::full_eq(a, b),
            (Self::TSConditionalType(a), Self::TSConditionalType(b)) => FullEq::full_eq(a, b),
            (Self::TSConstructorType(a), Self::TSConstructorType(b)) => FullEq::full_eq(a, b),
            (Self::TSFunctionType(a), Self::TSFunctionType(b)) => FullEq::full_eq(a, b),
            (Self::TSImportType(a), Self::TSImportType(b)) => FullEq::full_eq(a, b),
            (Self::TSIndexedAccessType(a), Self::TSIndexedAccessType(b)) => FullEq::full_eq(a, b),
            (Self::TSInferType(a), Self::TSInferType(b)) => FullEq::full_eq(a, b),
            (Self::TSIntersectionType(a), Self::TSIntersectionType(b)) => FullEq::full_eq(a, b),
            (Self::TSLiteralType(a), Self::TSLiteralType(b)) => FullEq::full_eq(a, b),
            (Self::TSMappedType(a), Self::TSMappedType(b)) => FullEq::full_eq(a, b),
            (Self::TSNamedTupleMember(a), Self::TSNamedTupleMember(b)) => FullEq::full_eq(a, b),
            (Self::TSTemplateLiteralType(a), Self::TSTemplateLiteralType(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSThisType(a), Self::TSThisType(b)) => FullEq::full_eq(a, b),
            (Self::TSTupleType(a), Self::TSTupleType(b)) => FullEq::full_eq(a, b),
            (Self::TSTypeLiteral(a), Self::TSTypeLiteral(b)) => FullEq::full_eq(a, b),
            (Self::TSTypeOperatorType(a), Self::TSTypeOperatorType(b)) => FullEq::full_eq(a, b),
            (Self::TSTypePredicate(a), Self::TSTypePredicate(b)) => FullEq::full_eq(a, b),
            (Self::TSTypeQuery(a), Self::TSTypeQuery(b)) => FullEq::full_eq(a, b),
            (Self::TSTypeReference(a), Self::TSTypeReference(b)) => FullEq::full_eq(a, b),
            (Self::TSUnionType(a), Self::TSUnionType(b)) => FullEq::full_eq(a, b),
            (Self::TSParenthesizedType(a), Self::TSParenthesizedType(b)) => FullEq::full_eq(a, b),
            (Self::JSDocNullableType(a), Self::JSDocNullableType(b)) => FullEq::full_eq(a, b),
            (Self::JSDocNonNullableType(a), Self::JSDocNonNullableType(b)) => FullEq::full_eq(a, b),
            (Self::JSDocUnknownType(a), Self::JSDocUnknownType(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for TSConditionalType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.check_type, &other.check_type)
            && FullEq::full_eq(&self.extends_type, &other.extends_type)
            && FullEq::full_eq(&self.true_type, &other.true_type)
            && FullEq::full_eq(&self.false_type, &other.false_type)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for TSUnionType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.types, &other.types)
    }
}

impl FullEq for TSIntersectionType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.types, &other.types)
    }
}

impl FullEq for TSParenthesizedType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl FullEq for TSTypeOperator<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.operator, &other.operator)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl FullEq for TSTypeOperatorOperator {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for TSArrayType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.element_type, &other.element_type)
    }
}

impl FullEq for TSIndexedAccessType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.object_type, &other.object_type)
            && FullEq::full_eq(&self.index_type, &other.index_type)
    }
}

impl FullEq for TSTupleType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.element_types, &other.element_types)
    }
}

impl FullEq for TSNamedTupleMember<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.label, &other.label)
            && FullEq::full_eq(&self.element_type, &other.element_type)
            && FullEq::full_eq(&self.optional, &other.optional)
    }
}

impl FullEq for TSOptionalType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl FullEq for TSRestType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl FullEq for TSTupleElement<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::TSOptionalType(a), Self::TSOptionalType(b)) => FullEq::full_eq(a, b),
            (Self::TSRestType(a), Self::TSRestType(b)) => FullEq::full_eq(a, b),
            (Self::TSAnyKeyword(a), Self::TSAnyKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSBigIntKeyword(a), Self::TSBigIntKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSBooleanKeyword(a), Self::TSBooleanKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSIntrinsicKeyword(a), Self::TSIntrinsicKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSNeverKeyword(a), Self::TSNeverKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSNullKeyword(a), Self::TSNullKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSNumberKeyword(a), Self::TSNumberKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSObjectKeyword(a), Self::TSObjectKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSStringKeyword(a), Self::TSStringKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSSymbolKeyword(a), Self::TSSymbolKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSUndefinedKeyword(a), Self::TSUndefinedKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSUnknownKeyword(a), Self::TSUnknownKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSVoidKeyword(a), Self::TSVoidKeyword(b)) => FullEq::full_eq(a, b),
            (Self::TSArrayType(a), Self::TSArrayType(b)) => FullEq::full_eq(a, b),
            (Self::TSConditionalType(a), Self::TSConditionalType(b)) => FullEq::full_eq(a, b),
            (Self::TSConstructorType(a), Self::TSConstructorType(b)) => FullEq::full_eq(a, b),
            (Self::TSFunctionType(a), Self::TSFunctionType(b)) => FullEq::full_eq(a, b),
            (Self::TSImportType(a), Self::TSImportType(b)) => FullEq::full_eq(a, b),
            (Self::TSIndexedAccessType(a), Self::TSIndexedAccessType(b)) => FullEq::full_eq(a, b),
            (Self::TSInferType(a), Self::TSInferType(b)) => FullEq::full_eq(a, b),
            (Self::TSIntersectionType(a), Self::TSIntersectionType(b)) => FullEq::full_eq(a, b),
            (Self::TSLiteralType(a), Self::TSLiteralType(b)) => FullEq::full_eq(a, b),
            (Self::TSMappedType(a), Self::TSMappedType(b)) => FullEq::full_eq(a, b),
            (Self::TSNamedTupleMember(a), Self::TSNamedTupleMember(b)) => FullEq::full_eq(a, b),
            (Self::TSTemplateLiteralType(a), Self::TSTemplateLiteralType(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSThisType(a), Self::TSThisType(b)) => FullEq::full_eq(a, b),
            (Self::TSTupleType(a), Self::TSTupleType(b)) => FullEq::full_eq(a, b),
            (Self::TSTypeLiteral(a), Self::TSTypeLiteral(b)) => FullEq::full_eq(a, b),
            (Self::TSTypeOperatorType(a), Self::TSTypeOperatorType(b)) => FullEq::full_eq(a, b),
            (Self::TSTypePredicate(a), Self::TSTypePredicate(b)) => FullEq::full_eq(a, b),
            (Self::TSTypeQuery(a), Self::TSTypeQuery(b)) => FullEq::full_eq(a, b),
            (Self::TSTypeReference(a), Self::TSTypeReference(b)) => FullEq::full_eq(a, b),
            (Self::TSUnionType(a), Self::TSUnionType(b)) => FullEq::full_eq(a, b),
            (Self::TSParenthesizedType(a), Self::TSParenthesizedType(b)) => FullEq::full_eq(a, b),
            (Self::JSDocNullableType(a), Self::JSDocNullableType(b)) => FullEq::full_eq(a, b),
            (Self::JSDocNonNullableType(a), Self::JSDocNonNullableType(b)) => FullEq::full_eq(a, b),
            (Self::JSDocUnknownType(a), Self::JSDocUnknownType(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for TSAnyKeyword {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for TSStringKeyword {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for TSBooleanKeyword {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for TSNumberKeyword {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for TSNeverKeyword {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for TSIntrinsicKeyword {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for TSUnknownKeyword {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for TSNullKeyword {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for TSUndefinedKeyword {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for TSVoidKeyword {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for TSSymbolKeyword {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for TSThisType {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for TSObjectKeyword {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for TSBigIntKeyword {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for TSTypeReference<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.type_name, &other.type_name)
            && FullEq::full_eq(&self.type_arguments, &other.type_arguments)
    }
}

impl FullEq for TSTypeName<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::IdentifierReference(a), Self::IdentifierReference(b)) => FullEq::full_eq(a, b),
            (Self::QualifiedName(a), Self::QualifiedName(b)) => FullEq::full_eq(a, b),
            (Self::ThisExpression(a), Self::ThisExpression(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for TSQualifiedName<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.left, &other.left)
            && FullEq::full_eq(&self.right, &other.right)
    }
}

impl FullEq for TSTypeParameterInstantiation<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.params, &other.params)
    }
}

impl FullEq for TSTypeParameter<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.name, &other.name)
            && FullEq::full_eq(&self.constraint, &other.constraint)
            && FullEq::full_eq(&self.default, &other.default)
            && FullEq::full_eq(&self.r#in, &other.r#in)
            && FullEq::full_eq(&self.out, &other.out)
            && FullEq::full_eq(&self.r#const, &other.r#const)
    }
}

impl FullEq for TSTypeParameterDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.params, &other.params)
    }
}

impl FullEq for TSTypeAliasDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.id, &other.id)
            && FullEq::full_eq(&self.type_parameters, &other.type_parameters)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
            && FullEq::full_eq(&self.declare, &other.declare)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for TSAccessibility {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for TSClassImplements<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.expression, &other.expression)
            && FullEq::full_eq(&self.type_arguments, &other.type_arguments)
    }
}

impl FullEq for TSInterfaceDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.id, &other.id)
            && FullEq::full_eq(&self.type_parameters, &other.type_parameters)
            && FullEq::full_eq(&self.extends, &other.extends)
            && FullEq::full_eq(&self.body, &other.body)
            && FullEq::full_eq(&self.declare, &other.declare)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for TSInterfaceBody<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.body, &other.body)
    }
}

impl FullEq for TSPropertySignature<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.optional, &other.optional)
            && FullEq::full_eq(&self.readonly, &other.readonly)
            && FullEq::full_eq(&self.key, &other.key)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl FullEq for TSSignature<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::TSIndexSignature(a), Self::TSIndexSignature(b)) => FullEq::full_eq(a, b),
            (Self::TSPropertySignature(a), Self::TSPropertySignature(b)) => FullEq::full_eq(a, b),
            (Self::TSCallSignatureDeclaration(a), Self::TSCallSignatureDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            (
                Self::TSConstructSignatureDeclaration(a),
                Self::TSConstructSignatureDeclaration(b),
            ) => FullEq::full_eq(a, b),
            (Self::TSMethodSignature(a), Self::TSMethodSignature(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for TSIndexSignature<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.parameter, &other.parameter)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
            && FullEq::full_eq(&self.readonly, &other.readonly)
            && FullEq::full_eq(&self.r#static, &other.r#static)
    }
}

impl FullEq for TSCallSignatureDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.type_parameters, &other.type_parameters)
            && FullEq::full_eq(&self.this_param, &other.this_param)
            && FullEq::full_eq(&self.params, &other.params)
            && FullEq::full_eq(&self.return_type, &other.return_type)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for TSMethodSignatureKind {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for TSMethodSignature<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.key, &other.key)
            && FullEq::full_eq(&self.optional, &other.optional)
            && FullEq::full_eq(&self.kind, &other.kind)
            && FullEq::full_eq(&self.type_parameters, &other.type_parameters)
            && FullEq::full_eq(&self.this_param, &other.this_param)
            && FullEq::full_eq(&self.params, &other.params)
            && FullEq::full_eq(&self.return_type, &other.return_type)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for TSConstructSignatureDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.type_parameters, &other.type_parameters)
            && FullEq::full_eq(&self.params, &other.params)
            && FullEq::full_eq(&self.return_type, &other.return_type)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for TSIndexSignatureName<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.name, &other.name)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl FullEq for TSInterfaceHeritage<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.type_name, &other.type_name)
            && FullEq::full_eq(&self.type_arguments, &other.type_arguments)
    }
}

impl FullEq for TSTypePredicate<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.parameter_name, &other.parameter_name)
            && FullEq::full_eq(&self.asserts, &other.asserts)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl FullEq for TSTypePredicateName<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Identifier(a), Self::Identifier(b)) => FullEq::full_eq(a, b),
            (Self::This(a), Self::This(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for TSExternalModuleDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.id, &other.id)
            && FullEq::full_eq(&self.body, &other.body)
            && FullEq::full_eq(&self.declare, &other.declare)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for TSNamespaceDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.id, &other.id)
            && FullEq::full_eq(&self.body, &other.body)
            && FullEq::full_eq(&self.kind, &other.kind)
            && FullEq::full_eq(&self.declare, &other.declare)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for TSNamespaceDeclarationKind {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for TSNamespaceDeclarationBody<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::TSNamespaceDeclaration(a), Self::TSNamespaceDeclaration(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::TSModuleBlock(a), Self::TSModuleBlock(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for TSGlobalDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.global_span, &other.global_span)
            && FullEq::full_eq(&self.body, &other.body)
            && FullEq::full_eq(&self.declare, &other.declare)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for TSModuleBlock<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.directives, &other.directives)
            && FullEq::full_eq(&self.body, &other.body)
    }
}

impl FullEq for TSTypeLiteral<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.members, &other.members)
    }
}

impl FullEq for TSInferType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.type_parameter, &other.type_parameter)
    }
}

impl FullEq for TSTypeQuery<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.expr_name, &other.expr_name)
            && FullEq::full_eq(&self.type_arguments, &other.type_arguments)
    }
}

impl FullEq for TSTypeQueryExprName<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::TSImportType(a), Self::TSImportType(b)) => FullEq::full_eq(a, b),
            (Self::IdentifierReference(a), Self::IdentifierReference(b)) => FullEq::full_eq(a, b),
            (Self::QualifiedName(a), Self::QualifiedName(b)) => FullEq::full_eq(a, b),
            (Self::ThisExpression(a), Self::ThisExpression(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for TSImportType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.source, &other.source)
            && FullEq::full_eq(&self.options, &other.options)
            && FullEq::full_eq(&self.qualifier, &other.qualifier)
            && FullEq::full_eq(&self.type_arguments, &other.type_arguments)
    }
}

impl FullEq for TSImportTypeQualifier<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Identifier(a), Self::Identifier(b)) => FullEq::full_eq(a, b),
            (Self::QualifiedName(a), Self::QualifiedName(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for TSImportTypeQualifiedName<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.left, &other.left)
            && FullEq::full_eq(&self.right, &other.right)
    }
}

impl FullEq for TSFunctionType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.type_parameters, &other.type_parameters)
            && FullEq::full_eq(&self.this_param, &other.this_param)
            && FullEq::full_eq(&self.params, &other.params)
            && FullEq::full_eq(&self.return_type, &other.return_type)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for TSConstructorType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.r#abstract, &other.r#abstract)
            && FullEq::full_eq(&self.type_parameters, &other.type_parameters)
            && FullEq::full_eq(&self.params, &other.params)
            && FullEq::full_eq(&self.return_type, &other.return_type)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for TSMappedType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.key, &other.key)
            && FullEq::full_eq(&self.constraint, &other.constraint)
            && FullEq::full_eq(&self.name_type, &other.name_type)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
            && FullEq::full_eq(&self.optional, &other.optional)
            && FullEq::full_eq(&self.readonly, &other.readonly)
            && FullEq::full_eq(&self.scope_id, &other.scope_id)
    }
}

impl FullEq for TSMappedTypeModifierOperator {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for TSTemplateLiteralType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.quasis, &other.quasis)
            && FullEq::full_eq(&self.types, &other.types)
    }
}

impl FullEq for TSAsExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.expression, &other.expression)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl FullEq for TSSatisfiesExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.expression, &other.expression)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl FullEq for TSTypeAssertion<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
            && FullEq::full_eq(&self.expression, &other.expression)
    }
}

impl FullEq for TSImportEqualsDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.id, &other.id)
            && FullEq::full_eq(&self.module_reference, &other.module_reference)
            && FullEq::full_eq(&self.import_kind, &other.import_kind)
    }
}

impl FullEq for TSModuleReference<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ExternalModuleReference(a), Self::ExternalModuleReference(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::IdentifierReference(a), Self::IdentifierReference(b)) => FullEq::full_eq(a, b),
            (Self::QualifiedName(a), Self::QualifiedName(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for TSExternalModuleReference<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.expression, &other.expression)
    }
}

impl FullEq for TSNonNullExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.expression, &other.expression)
    }
}

impl FullEq for Decorator<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.expression, &other.expression)
    }
}

impl FullEq for TSExportAssignment<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.expression, &other.expression)
    }
}

impl FullEq for TSNamespaceExportDeclaration<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.id, &other.id)
    }
}

impl FullEq for TSInstantiationExpression<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.expression, &other.expression)
            && FullEq::full_eq(&self.type_arguments, &other.type_arguments)
    }
}

impl FullEq for ImportOrExportKind {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for JSDocNullableType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
            && FullEq::full_eq(&self.postfix, &other.postfix)
    }
}

impl FullEq for JSDocNonNullableType<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.type_annotation, &other.type_annotation)
            && FullEq::full_eq(&self.postfix, &other.postfix)
    }
}

impl FullEq for JSDocUnknownType {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.node_id, &other.node_id) && FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for CommentKind {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for CommentPosition {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for CommentContent {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for CommentNewlines {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for CommentPlacement {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for Comment {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.attached_to, &other.attached_to)
            && FullEq::full_eq(&self.node_id, &other.node_id)
            && FullEq::full_eq(&self.placement, &other.placement)
            && FullEq::full_eq(&self.kind, &other.kind)
            && FullEq::full_eq(&self.position, &other.position)
            && FullEq::full_eq(&self.newlines, &other.newlines)
            && FullEq::full_eq(&self.content, &other.content)
    }
}

impl FullEq for Pattern<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span) && FullEq::full_eq(&self.body, &other.body)
    }
}

impl FullEq for Disjunction<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span) && FullEq::full_eq(&self.body, &other.body)
    }
}

impl FullEq for Alternative<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span) && FullEq::full_eq(&self.body, &other.body)
    }
}

impl FullEq for Term<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::BoundaryAssertion(a), Self::BoundaryAssertion(b)) => FullEq::full_eq(a, b),
            (Self::LookAroundAssertion(a), Self::LookAroundAssertion(b)) => FullEq::full_eq(a, b),
            (Self::Quantifier(a), Self::Quantifier(b)) => FullEq::full_eq(a, b),
            (Self::Character(a), Self::Character(b)) => FullEq::full_eq(a, b),
            (Self::Dot(a), Self::Dot(b)) => FullEq::full_eq(a, b),
            (Self::CharacterClassEscape(a), Self::CharacterClassEscape(b)) => FullEq::full_eq(a, b),
            (Self::UnicodePropertyEscape(a), Self::UnicodePropertyEscape(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::CharacterClass(a), Self::CharacterClass(b)) => FullEq::full_eq(a, b),
            (Self::CapturingGroup(a), Self::CapturingGroup(b)) => FullEq::full_eq(a, b),
            (Self::IgnoreGroup(a), Self::IgnoreGroup(b)) => FullEq::full_eq(a, b),
            (Self::IndexedReference(a), Self::IndexedReference(b)) => FullEq::full_eq(a, b),
            (Self::NamedReference(a), Self::NamedReference(b)) => FullEq::full_eq(a, b),
            _ => false,
        }
    }
}

impl FullEq for BoundaryAssertion {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span) && FullEq::full_eq(&self.kind, &other.kind)
    }
}

impl FullEq for BoundaryAssertionKind {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for LookAroundAssertion<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.kind, &other.kind)
            && FullEq::full_eq(&self.body, &other.body)
    }
}

impl FullEq for LookAroundAssertionKind {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for Quantifier<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.min, &other.min)
            && FullEq::full_eq(&self.max, &other.max)
            && FullEq::full_eq(&self.greedy, &other.greedy)
            && FullEq::full_eq(&self.body, &other.body)
    }
}

impl FullEq for Character {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.kind, &other.kind)
            && FullEq::full_eq(&self.value, &other.value)
    }
}

impl FullEq for CharacterKind {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for CharacterClassEscape {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span) && FullEq::full_eq(&self.kind, &other.kind)
    }
}

impl FullEq for CharacterClassEscapeKind {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for UnicodePropertyEscape<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.negative, &other.negative)
            && FullEq::full_eq(&self.strings, &other.strings)
            && FullEq::full_eq(&self.name, &other.name)
            && FullEq::full_eq(&self.value, &other.value)
    }
}

impl FullEq for Dot {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span)
    }
}

impl FullEq for CharacterClass<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.negative, &other.negative)
            && FullEq::full_eq(&self.strings, &other.strings)
            && FullEq::full_eq(&self.kind, &other.kind)
            && FullEq::full_eq(&self.body, &other.body)
    }
}

impl FullEq for CharacterClassContentsKind {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for CharacterClassContents<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::CharacterClassRange(a), Self::CharacterClassRange(b)) => FullEq::full_eq(a, b),
            (Self::CharacterClassEscape(a), Self::CharacterClassEscape(b)) => FullEq::full_eq(a, b),
            (Self::UnicodePropertyEscape(a), Self::UnicodePropertyEscape(b)) => {
                FullEq::full_eq(a, b)
            }
            (Self::Character(a), Self::Character(b)) => FullEq::full_eq(a, b),
            (Self::NestedCharacterClass(a), Self::NestedCharacterClass(b)) => FullEq::full_eq(a, b),
            (Self::ClassStringDisjunction(a), Self::ClassStringDisjunction(b)) => {
                FullEq::full_eq(a, b)
            }
            _ => false,
        }
    }
}

impl FullEq for CharacterClassRange {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.min, &other.min)
            && FullEq::full_eq(&self.max, &other.max)
    }
}

impl FullEq for ClassStringDisjunction<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.strings, &other.strings)
            && FullEq::full_eq(&self.body, &other.body)
    }
}

impl FullEq for ClassString<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.strings, &other.strings)
            && FullEq::full_eq(&self.body, &other.body)
    }
}

impl FullEq for CapturingGroup<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.name, &other.name)
            && FullEq::full_eq(&self.body, &other.body)
    }
}

impl FullEq for IgnoreGroup<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.modifiers, &other.modifiers)
            && FullEq::full_eq(&self.body, &other.body)
    }
}

impl FullEq for Modifiers {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span)
            && FullEq::full_eq(&self.enabling, &other.enabling)
            && FullEq::full_eq(&self.disabling, &other.disabling)
    }
}

impl FullEq for Modifier {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for IndexedReference {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span) && FullEq::full_eq(&self.index, &other.index)
    }
}

impl FullEq for NamedReference<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        FullEq::full_eq(&self.span, &other.span) && FullEq::full_eq(&self.name, &other.name)
    }
}

impl FullEq for SourceType {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for Language {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for ModuleKind {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for LanguageVariant {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for FileExtension {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for Span {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for LabelId {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for NodeId {
    fn full_eq(&self, other: &Self) -> bool {
        true
    }
}

impl FullEq for AssignmentOperator {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for BinaryOperator {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for LogicalOperator {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for UnaryOperator {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for UpdateOperator {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for ReferenceId {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for ScopeId {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for SymbolId {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for &str {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for Ident<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for bool {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for Str<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for JSStr<'_> {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for f64 {
    fn full_eq(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits()
    }
}

impl FullEq for u8 {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for u32 {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for u64 {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for i32 {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FullEq for NonZeroU32 {
    fn full_eq(&self, other: &Self) -> bool {
        self == other
    }
}
