// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/content_eq.rs`

use oxc_span::cmp::ContentEq;

#[allow(clippy::wildcard_imports)]
use crate::ast::js::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::jsx::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::literal::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::ts::*;

impl ContentEq for BooleanLiteral {
    fn content_eq(&self, other: &Self) -> bool {
        self.value.content_eq(&other.value)
    }
}

impl ContentEq for NullLiteral {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl<'a> ContentEq for NumericLiteral<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.value.content_eq(&other.value)
            && self.raw.content_eq(&other.raw)
            && self.base.content_eq(&other.base)
    }
}

impl<'a> ContentEq for BigIntLiteral<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.raw.content_eq(&other.raw) && self.base.content_eq(&other.base)
    }
}

impl<'a> ContentEq for RegExpLiteral<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.value.content_eq(&other.value) && self.regex.content_eq(&other.regex)
    }
}

impl<'a> ContentEq for RegExp<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.pattern.content_eq(&other.pattern) && self.flags.content_eq(&other.flags)
    }
}

impl<'a> ContentEq for RegExpPattern<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::Raw(it) => matches!(other, Self::Raw(other) if it.content_eq(other)),
            Self::Invalid(it) => {
                matches!(other, Self::Invalid(other) if it.content_eq(other))
            }
            Self::Pattern(it) => {
                matches!(other, Self::Pattern(other) if it.content_eq(other))
            }
        }
    }
}

impl ContentEq for EmptyObject {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl<'a> ContentEq for StringLiteral<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.value.content_eq(&other.value)
    }
}

impl<'a> ContentEq for Program<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.source_type.content_eq(&other.source_type)
            && self.hashbang.content_eq(&other.hashbang)
            && self.directives.content_eq(&other.directives)
            && self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for Expression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::BooleanLiteral(it) => {
                matches!(other, Self::BooleanLiteral(other) if it.content_eq(other))
            }
            Self::NullLiteral(it) => {
                matches!(other, Self::NullLiteral(other) if it.content_eq(other))
            }
            Self::NumericLiteral(it) => {
                matches!(other, Self::NumericLiteral(other) if it.content_eq(other))
            }
            Self::BigIntLiteral(it) => {
                matches!(other, Self::BigIntLiteral(other) if it.content_eq(other))
            }
            Self::RegExpLiteral(it) => {
                matches!(other, Self::RegExpLiteral(other) if it.content_eq(other))
            }
            Self::StringLiteral(it) => {
                matches!(other, Self::StringLiteral(other) if it.content_eq(other))
            }
            Self::TemplateLiteral(it) => {
                matches!(other, Self::TemplateLiteral(other) if it.content_eq(other))
            }
            Self::Identifier(it) => {
                matches!(other, Self::Identifier(other) if it.content_eq(other))
            }
            Self::MetaProperty(it) => {
                matches!(other, Self::MetaProperty(other) if it.content_eq(other))
            }
            Self::Super(it) => {
                matches!(other, Self::Super(other) if it.content_eq(other))
            }
            Self::ArrayExpression(it) => {
                matches!(other, Self::ArrayExpression(other) if it.content_eq(other))
            }
            Self::ArrowFunctionExpression(it) => {
                matches!(
                    other, Self::ArrowFunctionExpression(other) if it.content_eq(other)
                )
            }
            Self::AssignmentExpression(it) => {
                matches!(
                    other, Self::AssignmentExpression(other) if it.content_eq(other)
                )
            }
            Self::AwaitExpression(it) => {
                matches!(other, Self::AwaitExpression(other) if it.content_eq(other))
            }
            Self::BinaryExpression(it) => {
                matches!(other, Self::BinaryExpression(other) if it.content_eq(other))
            }
            Self::CallExpression(it) => {
                matches!(other, Self::CallExpression(other) if it.content_eq(other))
            }
            Self::ChainExpression(it) => {
                matches!(other, Self::ChainExpression(other) if it.content_eq(other))
            }
            Self::ClassExpression(it) => {
                matches!(other, Self::ClassExpression(other) if it.content_eq(other))
            }
            Self::ConditionalExpression(it) => {
                matches!(
                    other, Self::ConditionalExpression(other) if it.content_eq(other)
                )
            }
            Self::FunctionExpression(it) => {
                matches!(other, Self::FunctionExpression(other) if it.content_eq(other))
            }
            Self::ImportExpression(it) => {
                matches!(other, Self::ImportExpression(other) if it.content_eq(other))
            }
            Self::LogicalExpression(it) => {
                matches!(other, Self::LogicalExpression(other) if it.content_eq(other))
            }
            Self::NewExpression(it) => {
                matches!(other, Self::NewExpression(other) if it.content_eq(other))
            }
            Self::ObjectExpression(it) => {
                matches!(other, Self::ObjectExpression(other) if it.content_eq(other))
            }
            Self::ParenthesizedExpression(it) => {
                matches!(
                    other, Self::ParenthesizedExpression(other) if it.content_eq(other)
                )
            }
            Self::SequenceExpression(it) => {
                matches!(other, Self::SequenceExpression(other) if it.content_eq(other))
            }
            Self::TaggedTemplateExpression(it) => {
                matches!(
                    other, Self::TaggedTemplateExpression(other) if it.content_eq(other)
                )
            }
            Self::ThisExpression(it) => {
                matches!(other, Self::ThisExpression(other) if it.content_eq(other))
            }
            Self::UnaryExpression(it) => {
                matches!(other, Self::UnaryExpression(other) if it.content_eq(other))
            }
            Self::UpdateExpression(it) => {
                matches!(other, Self::UpdateExpression(other) if it.content_eq(other))
            }
            Self::YieldExpression(it) => {
                matches!(other, Self::YieldExpression(other) if it.content_eq(other))
            }
            Self::PrivateInExpression(it) => {
                matches!(other, Self::PrivateInExpression(other) if it.content_eq(other))
            }
            Self::JSXElement(it) => {
                matches!(other, Self::JSXElement(other) if it.content_eq(other))
            }
            Self::JSXFragment(it) => {
                matches!(other, Self::JSXFragment(other) if it.content_eq(other))
            }
            Self::TSAsExpression(it) => {
                matches!(other, Self::TSAsExpression(other) if it.content_eq(other))
            }
            Self::TSSatisfiesExpression(it) => {
                matches!(
                    other, Self::TSSatisfiesExpression(other) if it.content_eq(other)
                )
            }
            Self::TSTypeAssertion(it) => {
                matches!(other, Self::TSTypeAssertion(other) if it.content_eq(other))
            }
            Self::TSNonNullExpression(it) => {
                matches!(other, Self::TSNonNullExpression(other) if it.content_eq(other))
            }
            Self::TSInstantiationExpression(it) => {
                matches!(
                    other, Self::TSInstantiationExpression(other) if it.content_eq(other)
                )
            }
            Self::ComputedMemberExpression(it) => {
                matches!(
                    other, Self::ComputedMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::StaticMemberExpression(it) => {
                matches!(
                    other, Self::StaticMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::PrivateFieldExpression(it) => {
                matches!(
                    other, Self::PrivateFieldExpression(other) if it.content_eq(other)
                )
            }
        }
    }
}

impl<'a> ContentEq for IdentifierName<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.name.content_eq(&other.name)
    }
}

impl<'a> ContentEq for IdentifierReference<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.name.content_eq(&other.name)
    }
}

impl<'a> ContentEq for BindingIdentifier<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.name.content_eq(&other.name)
    }
}

impl<'a> ContentEq for LabelIdentifier<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.name.content_eq(&other.name)
    }
}

impl ContentEq for ThisExpression {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl<'a> ContentEq for ArrayExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.elements.content_eq(&other.elements)
    }
}

impl<'a> ContentEq for ArrayExpressionElement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::SpreadElement(it) => {
                matches!(other, Self::SpreadElement(other) if it.content_eq(other))
            }
            Self::Elision(it) => {
                matches!(other, Self::Elision(other) if it.content_eq(other))
            }
            Self::BooleanLiteral(it) => {
                matches!(other, Self::BooleanLiteral(other) if it.content_eq(other))
            }
            Self::NullLiteral(it) => {
                matches!(other, Self::NullLiteral(other) if it.content_eq(other))
            }
            Self::NumericLiteral(it) => {
                matches!(other, Self::NumericLiteral(other) if it.content_eq(other))
            }
            Self::BigIntLiteral(it) => {
                matches!(other, Self::BigIntLiteral(other) if it.content_eq(other))
            }
            Self::RegExpLiteral(it) => {
                matches!(other, Self::RegExpLiteral(other) if it.content_eq(other))
            }
            Self::StringLiteral(it) => {
                matches!(other, Self::StringLiteral(other) if it.content_eq(other))
            }
            Self::TemplateLiteral(it) => {
                matches!(other, Self::TemplateLiteral(other) if it.content_eq(other))
            }
            Self::Identifier(it) => {
                matches!(other, Self::Identifier(other) if it.content_eq(other))
            }
            Self::MetaProperty(it) => {
                matches!(other, Self::MetaProperty(other) if it.content_eq(other))
            }
            Self::Super(it) => {
                matches!(other, Self::Super(other) if it.content_eq(other))
            }
            Self::ArrayExpression(it) => {
                matches!(other, Self::ArrayExpression(other) if it.content_eq(other))
            }
            Self::ArrowFunctionExpression(it) => {
                matches!(
                    other, Self::ArrowFunctionExpression(other) if it.content_eq(other)
                )
            }
            Self::AssignmentExpression(it) => {
                matches!(
                    other, Self::AssignmentExpression(other) if it.content_eq(other)
                )
            }
            Self::AwaitExpression(it) => {
                matches!(other, Self::AwaitExpression(other) if it.content_eq(other))
            }
            Self::BinaryExpression(it) => {
                matches!(other, Self::BinaryExpression(other) if it.content_eq(other))
            }
            Self::CallExpression(it) => {
                matches!(other, Self::CallExpression(other) if it.content_eq(other))
            }
            Self::ChainExpression(it) => {
                matches!(other, Self::ChainExpression(other) if it.content_eq(other))
            }
            Self::ClassExpression(it) => {
                matches!(other, Self::ClassExpression(other) if it.content_eq(other))
            }
            Self::ConditionalExpression(it) => {
                matches!(
                    other, Self::ConditionalExpression(other) if it.content_eq(other)
                )
            }
            Self::FunctionExpression(it) => {
                matches!(other, Self::FunctionExpression(other) if it.content_eq(other))
            }
            Self::ImportExpression(it) => {
                matches!(other, Self::ImportExpression(other) if it.content_eq(other))
            }
            Self::LogicalExpression(it) => {
                matches!(other, Self::LogicalExpression(other) if it.content_eq(other))
            }
            Self::NewExpression(it) => {
                matches!(other, Self::NewExpression(other) if it.content_eq(other))
            }
            Self::ObjectExpression(it) => {
                matches!(other, Self::ObjectExpression(other) if it.content_eq(other))
            }
            Self::ParenthesizedExpression(it) => {
                matches!(
                    other, Self::ParenthesizedExpression(other) if it.content_eq(other)
                )
            }
            Self::SequenceExpression(it) => {
                matches!(other, Self::SequenceExpression(other) if it.content_eq(other))
            }
            Self::TaggedTemplateExpression(it) => {
                matches!(
                    other, Self::TaggedTemplateExpression(other) if it.content_eq(other)
                )
            }
            Self::ThisExpression(it) => {
                matches!(other, Self::ThisExpression(other) if it.content_eq(other))
            }
            Self::UnaryExpression(it) => {
                matches!(other, Self::UnaryExpression(other) if it.content_eq(other))
            }
            Self::UpdateExpression(it) => {
                matches!(other, Self::UpdateExpression(other) if it.content_eq(other))
            }
            Self::YieldExpression(it) => {
                matches!(other, Self::YieldExpression(other) if it.content_eq(other))
            }
            Self::PrivateInExpression(it) => {
                matches!(other, Self::PrivateInExpression(other) if it.content_eq(other))
            }
            Self::JSXElement(it) => {
                matches!(other, Self::JSXElement(other) if it.content_eq(other))
            }
            Self::JSXFragment(it) => {
                matches!(other, Self::JSXFragment(other) if it.content_eq(other))
            }
            Self::TSAsExpression(it) => {
                matches!(other, Self::TSAsExpression(other) if it.content_eq(other))
            }
            Self::TSSatisfiesExpression(it) => {
                matches!(
                    other, Self::TSSatisfiesExpression(other) if it.content_eq(other)
                )
            }
            Self::TSTypeAssertion(it) => {
                matches!(other, Self::TSTypeAssertion(other) if it.content_eq(other))
            }
            Self::TSNonNullExpression(it) => {
                matches!(other, Self::TSNonNullExpression(other) if it.content_eq(other))
            }
            Self::TSInstantiationExpression(it) => {
                matches!(
                    other, Self::TSInstantiationExpression(other) if it.content_eq(other)
                )
            }
            Self::ComputedMemberExpression(it) => {
                matches!(
                    other, Self::ComputedMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::StaticMemberExpression(it) => {
                matches!(
                    other, Self::StaticMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::PrivateFieldExpression(it) => {
                matches!(
                    other, Self::PrivateFieldExpression(other) if it.content_eq(other)
                )
            }
        }
    }
}

impl ContentEq for Elision {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl<'a> ContentEq for ObjectExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.properties.content_eq(&other.properties)
    }
}

impl<'a> ContentEq for ObjectPropertyKind<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::ObjectProperty(it) => {
                matches!(other, Self::ObjectProperty(other) if it.content_eq(other))
            }
            Self::SpreadProperty(it) => {
                matches!(other, Self::SpreadProperty(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for ObjectProperty<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.kind.content_eq(&other.kind)
            && self.key.content_eq(&other.key)
            && self.value.content_eq(&other.value)
            && self.init.content_eq(&other.init)
            && self.method.content_eq(&other.method)
            && self.shorthand.content_eq(&other.shorthand)
            && self.computed.content_eq(&other.computed)
    }
}

impl<'a> ContentEq for PropertyKey<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::StaticIdentifier(it) => {
                matches!(other, Self::StaticIdentifier(other) if it.content_eq(other))
            }
            Self::PrivateIdentifier(it) => {
                matches!(other, Self::PrivateIdentifier(other) if it.content_eq(other))
            }
            Self::BooleanLiteral(it) => {
                matches!(other, Self::BooleanLiteral(other) if it.content_eq(other))
            }
            Self::NullLiteral(it) => {
                matches!(other, Self::NullLiteral(other) if it.content_eq(other))
            }
            Self::NumericLiteral(it) => {
                matches!(other, Self::NumericLiteral(other) if it.content_eq(other))
            }
            Self::BigIntLiteral(it) => {
                matches!(other, Self::BigIntLiteral(other) if it.content_eq(other))
            }
            Self::RegExpLiteral(it) => {
                matches!(other, Self::RegExpLiteral(other) if it.content_eq(other))
            }
            Self::StringLiteral(it) => {
                matches!(other, Self::StringLiteral(other) if it.content_eq(other))
            }
            Self::TemplateLiteral(it) => {
                matches!(other, Self::TemplateLiteral(other) if it.content_eq(other))
            }
            Self::Identifier(it) => {
                matches!(other, Self::Identifier(other) if it.content_eq(other))
            }
            Self::MetaProperty(it) => {
                matches!(other, Self::MetaProperty(other) if it.content_eq(other))
            }
            Self::Super(it) => {
                matches!(other, Self::Super(other) if it.content_eq(other))
            }
            Self::ArrayExpression(it) => {
                matches!(other, Self::ArrayExpression(other) if it.content_eq(other))
            }
            Self::ArrowFunctionExpression(it) => {
                matches!(
                    other, Self::ArrowFunctionExpression(other) if it.content_eq(other)
                )
            }
            Self::AssignmentExpression(it) => {
                matches!(
                    other, Self::AssignmentExpression(other) if it.content_eq(other)
                )
            }
            Self::AwaitExpression(it) => {
                matches!(other, Self::AwaitExpression(other) if it.content_eq(other))
            }
            Self::BinaryExpression(it) => {
                matches!(other, Self::BinaryExpression(other) if it.content_eq(other))
            }
            Self::CallExpression(it) => {
                matches!(other, Self::CallExpression(other) if it.content_eq(other))
            }
            Self::ChainExpression(it) => {
                matches!(other, Self::ChainExpression(other) if it.content_eq(other))
            }
            Self::ClassExpression(it) => {
                matches!(other, Self::ClassExpression(other) if it.content_eq(other))
            }
            Self::ConditionalExpression(it) => {
                matches!(
                    other, Self::ConditionalExpression(other) if it.content_eq(other)
                )
            }
            Self::FunctionExpression(it) => {
                matches!(other, Self::FunctionExpression(other) if it.content_eq(other))
            }
            Self::ImportExpression(it) => {
                matches!(other, Self::ImportExpression(other) if it.content_eq(other))
            }
            Self::LogicalExpression(it) => {
                matches!(other, Self::LogicalExpression(other) if it.content_eq(other))
            }
            Self::NewExpression(it) => {
                matches!(other, Self::NewExpression(other) if it.content_eq(other))
            }
            Self::ObjectExpression(it) => {
                matches!(other, Self::ObjectExpression(other) if it.content_eq(other))
            }
            Self::ParenthesizedExpression(it) => {
                matches!(
                    other, Self::ParenthesizedExpression(other) if it.content_eq(other)
                )
            }
            Self::SequenceExpression(it) => {
                matches!(other, Self::SequenceExpression(other) if it.content_eq(other))
            }
            Self::TaggedTemplateExpression(it) => {
                matches!(
                    other, Self::TaggedTemplateExpression(other) if it.content_eq(other)
                )
            }
            Self::ThisExpression(it) => {
                matches!(other, Self::ThisExpression(other) if it.content_eq(other))
            }
            Self::UnaryExpression(it) => {
                matches!(other, Self::UnaryExpression(other) if it.content_eq(other))
            }
            Self::UpdateExpression(it) => {
                matches!(other, Self::UpdateExpression(other) if it.content_eq(other))
            }
            Self::YieldExpression(it) => {
                matches!(other, Self::YieldExpression(other) if it.content_eq(other))
            }
            Self::PrivateInExpression(it) => {
                matches!(other, Self::PrivateInExpression(other) if it.content_eq(other))
            }
            Self::JSXElement(it) => {
                matches!(other, Self::JSXElement(other) if it.content_eq(other))
            }
            Self::JSXFragment(it) => {
                matches!(other, Self::JSXFragment(other) if it.content_eq(other))
            }
            Self::TSAsExpression(it) => {
                matches!(other, Self::TSAsExpression(other) if it.content_eq(other))
            }
            Self::TSSatisfiesExpression(it) => {
                matches!(
                    other, Self::TSSatisfiesExpression(other) if it.content_eq(other)
                )
            }
            Self::TSTypeAssertion(it) => {
                matches!(other, Self::TSTypeAssertion(other) if it.content_eq(other))
            }
            Self::TSNonNullExpression(it) => {
                matches!(other, Self::TSNonNullExpression(other) if it.content_eq(other))
            }
            Self::TSInstantiationExpression(it) => {
                matches!(
                    other, Self::TSInstantiationExpression(other) if it.content_eq(other)
                )
            }
            Self::ComputedMemberExpression(it) => {
                matches!(
                    other, Self::ComputedMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::StaticMemberExpression(it) => {
                matches!(
                    other, Self::StaticMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::PrivateFieldExpression(it) => {
                matches!(
                    other, Self::PrivateFieldExpression(other) if it.content_eq(other)
                )
            }
        }
    }
}

impl ContentEq for PropertyKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentEq for TemplateLiteral<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.quasis.content_eq(&other.quasis) && self.expressions.content_eq(&other.expressions)
    }
}

impl<'a> ContentEq for TaggedTemplateExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.tag.content_eq(&other.tag)
            && self.quasi.content_eq(&other.quasi)
            && self.type_parameters.content_eq(&other.type_parameters)
    }
}

impl<'a> ContentEq for TemplateElement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.tail.content_eq(&other.tail) && self.value.content_eq(&other.value)
    }
}

impl<'a> ContentEq for TemplateElementValue<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.raw.content_eq(&other.raw) && self.cooked.content_eq(&other.cooked)
    }
}

impl<'a> ContentEq for MemberExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::ComputedMemberExpression(it) => {
                matches!(
                    other, Self::ComputedMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::StaticMemberExpression(it) => {
                matches!(
                    other, Self::StaticMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::PrivateFieldExpression(it) => {
                matches!(
                    other, Self::PrivateFieldExpression(other) if it.content_eq(other)
                )
            }
        }
    }
}

impl<'a> ContentEq for ComputedMemberExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.object.content_eq(&other.object)
            && self.expression.content_eq(&other.expression)
            && self.optional.content_eq(&other.optional)
    }
}

impl<'a> ContentEq for StaticMemberExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.object.content_eq(&other.object)
            && self.property.content_eq(&other.property)
            && self.optional.content_eq(&other.optional)
    }
}

impl<'a> ContentEq for PrivateFieldExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.object.content_eq(&other.object)
            && self.field.content_eq(&other.field)
            && self.optional.content_eq(&other.optional)
    }
}

impl<'a> ContentEq for CallExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.callee.content_eq(&other.callee)
            && self.type_parameters.content_eq(&other.type_parameters)
            && self.arguments.content_eq(&other.arguments)
            && self.optional.content_eq(&other.optional)
    }
}

impl<'a> ContentEq for NewExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.callee.content_eq(&other.callee)
            && self.arguments.content_eq(&other.arguments)
            && self.type_parameters.content_eq(&other.type_parameters)
    }
}

impl<'a> ContentEq for MetaProperty<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.meta.content_eq(&other.meta) && self.property.content_eq(&other.property)
    }
}

impl<'a> ContentEq for SpreadElement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.argument.content_eq(&other.argument)
    }
}

impl<'a> ContentEq for Argument<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::SpreadElement(it) => {
                matches!(other, Self::SpreadElement(other) if it.content_eq(other))
            }
            Self::BooleanLiteral(it) => {
                matches!(other, Self::BooleanLiteral(other) if it.content_eq(other))
            }
            Self::NullLiteral(it) => {
                matches!(other, Self::NullLiteral(other) if it.content_eq(other))
            }
            Self::NumericLiteral(it) => {
                matches!(other, Self::NumericLiteral(other) if it.content_eq(other))
            }
            Self::BigIntLiteral(it) => {
                matches!(other, Self::BigIntLiteral(other) if it.content_eq(other))
            }
            Self::RegExpLiteral(it) => {
                matches!(other, Self::RegExpLiteral(other) if it.content_eq(other))
            }
            Self::StringLiteral(it) => {
                matches!(other, Self::StringLiteral(other) if it.content_eq(other))
            }
            Self::TemplateLiteral(it) => {
                matches!(other, Self::TemplateLiteral(other) if it.content_eq(other))
            }
            Self::Identifier(it) => {
                matches!(other, Self::Identifier(other) if it.content_eq(other))
            }
            Self::MetaProperty(it) => {
                matches!(other, Self::MetaProperty(other) if it.content_eq(other))
            }
            Self::Super(it) => {
                matches!(other, Self::Super(other) if it.content_eq(other))
            }
            Self::ArrayExpression(it) => {
                matches!(other, Self::ArrayExpression(other) if it.content_eq(other))
            }
            Self::ArrowFunctionExpression(it) => {
                matches!(
                    other, Self::ArrowFunctionExpression(other) if it.content_eq(other)
                )
            }
            Self::AssignmentExpression(it) => {
                matches!(
                    other, Self::AssignmentExpression(other) if it.content_eq(other)
                )
            }
            Self::AwaitExpression(it) => {
                matches!(other, Self::AwaitExpression(other) if it.content_eq(other))
            }
            Self::BinaryExpression(it) => {
                matches!(other, Self::BinaryExpression(other) if it.content_eq(other))
            }
            Self::CallExpression(it) => {
                matches!(other, Self::CallExpression(other) if it.content_eq(other))
            }
            Self::ChainExpression(it) => {
                matches!(other, Self::ChainExpression(other) if it.content_eq(other))
            }
            Self::ClassExpression(it) => {
                matches!(other, Self::ClassExpression(other) if it.content_eq(other))
            }
            Self::ConditionalExpression(it) => {
                matches!(
                    other, Self::ConditionalExpression(other) if it.content_eq(other)
                )
            }
            Self::FunctionExpression(it) => {
                matches!(other, Self::FunctionExpression(other) if it.content_eq(other))
            }
            Self::ImportExpression(it) => {
                matches!(other, Self::ImportExpression(other) if it.content_eq(other))
            }
            Self::LogicalExpression(it) => {
                matches!(other, Self::LogicalExpression(other) if it.content_eq(other))
            }
            Self::NewExpression(it) => {
                matches!(other, Self::NewExpression(other) if it.content_eq(other))
            }
            Self::ObjectExpression(it) => {
                matches!(other, Self::ObjectExpression(other) if it.content_eq(other))
            }
            Self::ParenthesizedExpression(it) => {
                matches!(
                    other, Self::ParenthesizedExpression(other) if it.content_eq(other)
                )
            }
            Self::SequenceExpression(it) => {
                matches!(other, Self::SequenceExpression(other) if it.content_eq(other))
            }
            Self::TaggedTemplateExpression(it) => {
                matches!(
                    other, Self::TaggedTemplateExpression(other) if it.content_eq(other)
                )
            }
            Self::ThisExpression(it) => {
                matches!(other, Self::ThisExpression(other) if it.content_eq(other))
            }
            Self::UnaryExpression(it) => {
                matches!(other, Self::UnaryExpression(other) if it.content_eq(other))
            }
            Self::UpdateExpression(it) => {
                matches!(other, Self::UpdateExpression(other) if it.content_eq(other))
            }
            Self::YieldExpression(it) => {
                matches!(other, Self::YieldExpression(other) if it.content_eq(other))
            }
            Self::PrivateInExpression(it) => {
                matches!(other, Self::PrivateInExpression(other) if it.content_eq(other))
            }
            Self::JSXElement(it) => {
                matches!(other, Self::JSXElement(other) if it.content_eq(other))
            }
            Self::JSXFragment(it) => {
                matches!(other, Self::JSXFragment(other) if it.content_eq(other))
            }
            Self::TSAsExpression(it) => {
                matches!(other, Self::TSAsExpression(other) if it.content_eq(other))
            }
            Self::TSSatisfiesExpression(it) => {
                matches!(
                    other, Self::TSSatisfiesExpression(other) if it.content_eq(other)
                )
            }
            Self::TSTypeAssertion(it) => {
                matches!(other, Self::TSTypeAssertion(other) if it.content_eq(other))
            }
            Self::TSNonNullExpression(it) => {
                matches!(other, Self::TSNonNullExpression(other) if it.content_eq(other))
            }
            Self::TSInstantiationExpression(it) => {
                matches!(
                    other, Self::TSInstantiationExpression(other) if it.content_eq(other)
                )
            }
            Self::ComputedMemberExpression(it) => {
                matches!(
                    other, Self::ComputedMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::StaticMemberExpression(it) => {
                matches!(
                    other, Self::StaticMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::PrivateFieldExpression(it) => {
                matches!(
                    other, Self::PrivateFieldExpression(other) if it.content_eq(other)
                )
            }
        }
    }
}

impl<'a> ContentEq for UpdateExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.operator.content_eq(&other.operator)
            && self.prefix.content_eq(&other.prefix)
            && self.argument.content_eq(&other.argument)
    }
}

impl<'a> ContentEq for UnaryExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.operator.content_eq(&other.operator) && self.argument.content_eq(&other.argument)
    }
}

impl<'a> ContentEq for BinaryExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.left.content_eq(&other.left)
            && self.operator.content_eq(&other.operator)
            && self.right.content_eq(&other.right)
    }
}

impl<'a> ContentEq for PrivateInExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.left.content_eq(&other.left)
            && self.operator.content_eq(&other.operator)
            && self.right.content_eq(&other.right)
    }
}

impl<'a> ContentEq for LogicalExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.left.content_eq(&other.left)
            && self.operator.content_eq(&other.operator)
            && self.right.content_eq(&other.right)
    }
}

impl<'a> ContentEq for ConditionalExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.test.content_eq(&other.test)
            && self.consequent.content_eq(&other.consequent)
            && self.alternate.content_eq(&other.alternate)
    }
}

impl<'a> ContentEq for AssignmentExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.operator.content_eq(&other.operator)
            && self.left.content_eq(&other.left)
            && self.right.content_eq(&other.right)
    }
}

impl<'a> ContentEq for AssignmentTarget<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::AssignmentTargetIdentifier(it) => {
                matches!(
                    other, Self::AssignmentTargetIdentifier(other) if it
                    .content_eq(other)
                )
            }
            Self::TSAsExpression(it) => {
                matches!(other, Self::TSAsExpression(other) if it.content_eq(other))
            }
            Self::TSSatisfiesExpression(it) => {
                matches!(
                    other, Self::TSSatisfiesExpression(other) if it.content_eq(other)
                )
            }
            Self::TSNonNullExpression(it) => {
                matches!(other, Self::TSNonNullExpression(other) if it.content_eq(other))
            }
            Self::TSTypeAssertion(it) => {
                matches!(other, Self::TSTypeAssertion(other) if it.content_eq(other))
            }
            Self::TSInstantiationExpression(it) => {
                matches!(
                    other, Self::TSInstantiationExpression(other) if it.content_eq(other)
                )
            }
            Self::ComputedMemberExpression(it) => {
                matches!(
                    other, Self::ComputedMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::StaticMemberExpression(it) => {
                matches!(
                    other, Self::StaticMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::PrivateFieldExpression(it) => {
                matches!(
                    other, Self::PrivateFieldExpression(other) if it.content_eq(other)
                )
            }
            Self::ArrayAssignmentTarget(it) => {
                matches!(
                    other, Self::ArrayAssignmentTarget(other) if it.content_eq(other)
                )
            }
            Self::ObjectAssignmentTarget(it) => {
                matches!(
                    other, Self::ObjectAssignmentTarget(other) if it.content_eq(other)
                )
            }
        }
    }
}

impl<'a> ContentEq for SimpleAssignmentTarget<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::AssignmentTargetIdentifier(it) => {
                matches!(
                    other, Self::AssignmentTargetIdentifier(other) if it
                    .content_eq(other)
                )
            }
            Self::TSAsExpression(it) => {
                matches!(other, Self::TSAsExpression(other) if it.content_eq(other))
            }
            Self::TSSatisfiesExpression(it) => {
                matches!(
                    other, Self::TSSatisfiesExpression(other) if it.content_eq(other)
                )
            }
            Self::TSNonNullExpression(it) => {
                matches!(other, Self::TSNonNullExpression(other) if it.content_eq(other))
            }
            Self::TSTypeAssertion(it) => {
                matches!(other, Self::TSTypeAssertion(other) if it.content_eq(other))
            }
            Self::TSInstantiationExpression(it) => {
                matches!(
                    other, Self::TSInstantiationExpression(other) if it.content_eq(other)
                )
            }
            Self::ComputedMemberExpression(it) => {
                matches!(
                    other, Self::ComputedMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::StaticMemberExpression(it) => {
                matches!(
                    other, Self::StaticMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::PrivateFieldExpression(it) => {
                matches!(
                    other, Self::PrivateFieldExpression(other) if it.content_eq(other)
                )
            }
        }
    }
}

impl<'a> ContentEq for AssignmentTargetPattern<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::ArrayAssignmentTarget(it) => {
                matches!(
                    other, Self::ArrayAssignmentTarget(other) if it.content_eq(other)
                )
            }
            Self::ObjectAssignmentTarget(it) => {
                matches!(
                    other, Self::ObjectAssignmentTarget(other) if it.content_eq(other)
                )
            }
        }
    }
}

impl<'a> ContentEq for ArrayAssignmentTarget<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.elements.content_eq(&other.elements) && self.rest.content_eq(&other.rest)
    }
}

impl<'a> ContentEq for ObjectAssignmentTarget<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.properties.content_eq(&other.properties) && self.rest.content_eq(&other.rest)
    }
}

impl<'a> ContentEq for AssignmentTargetRest<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.target.content_eq(&other.target)
    }
}

impl<'a> ContentEq for AssignmentTargetMaybeDefault<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::AssignmentTargetWithDefault(it) => {
                matches!(
                    other, Self::AssignmentTargetWithDefault(other) if it
                    .content_eq(other)
                )
            }
            Self::AssignmentTargetIdentifier(it) => {
                matches!(
                    other, Self::AssignmentTargetIdentifier(other) if it
                    .content_eq(other)
                )
            }
            Self::TSAsExpression(it) => {
                matches!(other, Self::TSAsExpression(other) if it.content_eq(other))
            }
            Self::TSSatisfiesExpression(it) => {
                matches!(
                    other, Self::TSSatisfiesExpression(other) if it.content_eq(other)
                )
            }
            Self::TSNonNullExpression(it) => {
                matches!(other, Self::TSNonNullExpression(other) if it.content_eq(other))
            }
            Self::TSTypeAssertion(it) => {
                matches!(other, Self::TSTypeAssertion(other) if it.content_eq(other))
            }
            Self::TSInstantiationExpression(it) => {
                matches!(
                    other, Self::TSInstantiationExpression(other) if it.content_eq(other)
                )
            }
            Self::ComputedMemberExpression(it) => {
                matches!(
                    other, Self::ComputedMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::StaticMemberExpression(it) => {
                matches!(
                    other, Self::StaticMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::PrivateFieldExpression(it) => {
                matches!(
                    other, Self::PrivateFieldExpression(other) if it.content_eq(other)
                )
            }
            Self::ArrayAssignmentTarget(it) => {
                matches!(
                    other, Self::ArrayAssignmentTarget(other) if it.content_eq(other)
                )
            }
            Self::ObjectAssignmentTarget(it) => {
                matches!(
                    other, Self::ObjectAssignmentTarget(other) if it.content_eq(other)
                )
            }
        }
    }
}

impl<'a> ContentEq for AssignmentTargetWithDefault<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.binding.content_eq(&other.binding) && self.init.content_eq(&other.init)
    }
}

impl<'a> ContentEq for AssignmentTargetProperty<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => {
                matches!(
                    other, Self::AssignmentTargetPropertyIdentifier(other) if it
                    .content_eq(other)
                )
            }
            Self::AssignmentTargetPropertyProperty(it) => {
                matches!(
                    other, Self::AssignmentTargetPropertyProperty(other) if it
                    .content_eq(other)
                )
            }
        }
    }
}

impl<'a> ContentEq for AssignmentTargetPropertyIdentifier<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.binding.content_eq(&other.binding) && self.init.content_eq(&other.init)
    }
}

impl<'a> ContentEq for AssignmentTargetPropertyProperty<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.name.content_eq(&other.name) && self.binding.content_eq(&other.binding)
    }
}

impl<'a> ContentEq for SequenceExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expressions.content_eq(&other.expressions)
    }
}

impl ContentEq for Super {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl<'a> ContentEq for AwaitExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.argument.content_eq(&other.argument)
    }
}

impl<'a> ContentEq for ChainExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expression.content_eq(&other.expression)
    }
}

impl<'a> ContentEq for ChainElement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::CallExpression(it) => {
                matches!(other, Self::CallExpression(other) if it.content_eq(other))
            }
            Self::ComputedMemberExpression(it) => {
                matches!(
                    other, Self::ComputedMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::StaticMemberExpression(it) => {
                matches!(
                    other, Self::StaticMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::PrivateFieldExpression(it) => {
                matches!(
                    other, Self::PrivateFieldExpression(other) if it.content_eq(other)
                )
            }
        }
    }
}

impl<'a> ContentEq for ParenthesizedExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expression.content_eq(&other.expression)
    }
}

impl<'a> ContentEq for Statement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::BlockStatement(it) => {
                matches!(other, Self::BlockStatement(other) if it.content_eq(other))
            }
            Self::BreakStatement(it) => {
                matches!(other, Self::BreakStatement(other) if it.content_eq(other))
            }
            Self::ContinueStatement(it) => {
                matches!(other, Self::ContinueStatement(other) if it.content_eq(other))
            }
            Self::DebuggerStatement(it) => {
                matches!(other, Self::DebuggerStatement(other) if it.content_eq(other))
            }
            Self::DoWhileStatement(it) => {
                matches!(other, Self::DoWhileStatement(other) if it.content_eq(other))
            }
            Self::EmptyStatement(it) => {
                matches!(other, Self::EmptyStatement(other) if it.content_eq(other))
            }
            Self::ExpressionStatement(it) => {
                matches!(other, Self::ExpressionStatement(other) if it.content_eq(other))
            }
            Self::ForInStatement(it) => {
                matches!(other, Self::ForInStatement(other) if it.content_eq(other))
            }
            Self::ForOfStatement(it) => {
                matches!(other, Self::ForOfStatement(other) if it.content_eq(other))
            }
            Self::ForStatement(it) => {
                matches!(other, Self::ForStatement(other) if it.content_eq(other))
            }
            Self::IfStatement(it) => {
                matches!(other, Self::IfStatement(other) if it.content_eq(other))
            }
            Self::LabeledStatement(it) => {
                matches!(other, Self::LabeledStatement(other) if it.content_eq(other))
            }
            Self::ReturnStatement(it) => {
                matches!(other, Self::ReturnStatement(other) if it.content_eq(other))
            }
            Self::SwitchStatement(it) => {
                matches!(other, Self::SwitchStatement(other) if it.content_eq(other))
            }
            Self::ThrowStatement(it) => {
                matches!(other, Self::ThrowStatement(other) if it.content_eq(other))
            }
            Self::TryStatement(it) => {
                matches!(other, Self::TryStatement(other) if it.content_eq(other))
            }
            Self::WhileStatement(it) => {
                matches!(other, Self::WhileStatement(other) if it.content_eq(other))
            }
            Self::WithStatement(it) => {
                matches!(other, Self::WithStatement(other) if it.content_eq(other))
            }
            Self::VariableDeclaration(it) => {
                matches!(other, Self::VariableDeclaration(other) if it.content_eq(other))
            }
            Self::FunctionDeclaration(it) => {
                matches!(other, Self::FunctionDeclaration(other) if it.content_eq(other))
            }
            Self::ClassDeclaration(it) => {
                matches!(other, Self::ClassDeclaration(other) if it.content_eq(other))
            }
            Self::TSTypeAliasDeclaration(it) => {
                matches!(
                    other, Self::TSTypeAliasDeclaration(other) if it.content_eq(other)
                )
            }
            Self::TSInterfaceDeclaration(it) => {
                matches!(
                    other, Self::TSInterfaceDeclaration(other) if it.content_eq(other)
                )
            }
            Self::TSEnumDeclaration(it) => {
                matches!(other, Self::TSEnumDeclaration(other) if it.content_eq(other))
            }
            Self::TSModuleDeclaration(it) => {
                matches!(other, Self::TSModuleDeclaration(other) if it.content_eq(other))
            }
            Self::TSImportEqualsDeclaration(it) => {
                matches!(
                    other, Self::TSImportEqualsDeclaration(other) if it.content_eq(other)
                )
            }
            Self::ImportDeclaration(it) => {
                matches!(other, Self::ImportDeclaration(other) if it.content_eq(other))
            }
            Self::ExportAllDeclaration(it) => {
                matches!(
                    other, Self::ExportAllDeclaration(other) if it.content_eq(other)
                )
            }
            Self::ExportDefaultDeclaration(it) => {
                matches!(
                    other, Self::ExportDefaultDeclaration(other) if it.content_eq(other)
                )
            }
            Self::ExportNamedDeclaration(it) => {
                matches!(
                    other, Self::ExportNamedDeclaration(other) if it.content_eq(other)
                )
            }
            Self::TSExportAssignment(it) => {
                matches!(other, Self::TSExportAssignment(other) if it.content_eq(other))
            }
            Self::TSNamespaceExportDeclaration(it) => {
                matches!(
                    other, Self::TSNamespaceExportDeclaration(other) if it
                    .content_eq(other)
                )
            }
        }
    }
}

impl<'a> ContentEq for Directive<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expression.content_eq(&other.expression) && self.directive.content_eq(&other.directive)
    }
}

impl<'a> ContentEq for Hashbang<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.value.content_eq(&other.value)
    }
}

impl<'a> ContentEq for BlockStatement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for Declaration<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::VariableDeclaration(it) => {
                matches!(other, Self::VariableDeclaration(other) if it.content_eq(other))
            }
            Self::FunctionDeclaration(it) => {
                matches!(other, Self::FunctionDeclaration(other) if it.content_eq(other))
            }
            Self::ClassDeclaration(it) => {
                matches!(other, Self::ClassDeclaration(other) if it.content_eq(other))
            }
            Self::TSTypeAliasDeclaration(it) => {
                matches!(
                    other, Self::TSTypeAliasDeclaration(other) if it.content_eq(other)
                )
            }
            Self::TSInterfaceDeclaration(it) => {
                matches!(
                    other, Self::TSInterfaceDeclaration(other) if it.content_eq(other)
                )
            }
            Self::TSEnumDeclaration(it) => {
                matches!(other, Self::TSEnumDeclaration(other) if it.content_eq(other))
            }
            Self::TSModuleDeclaration(it) => {
                matches!(other, Self::TSModuleDeclaration(other) if it.content_eq(other))
            }
            Self::TSImportEqualsDeclaration(it) => {
                matches!(
                    other, Self::TSImportEqualsDeclaration(other) if it.content_eq(other)
                )
            }
        }
    }
}

impl<'a> ContentEq for VariableDeclaration<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.kind.content_eq(&other.kind)
            && self.declarations.content_eq(&other.declarations)
            && self.declare.content_eq(&other.declare)
    }
}

impl ContentEq for VariableDeclarationKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentEq for VariableDeclarator<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.kind.content_eq(&other.kind)
            && self.id.content_eq(&other.id)
            && self.init.content_eq(&other.init)
            && self.definite.content_eq(&other.definite)
    }
}

impl ContentEq for EmptyStatement {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl<'a> ContentEq for ExpressionStatement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expression.content_eq(&other.expression)
    }
}

impl<'a> ContentEq for IfStatement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.test.content_eq(&other.test)
            && self.consequent.content_eq(&other.consequent)
            && self.alternate.content_eq(&other.alternate)
    }
}

impl<'a> ContentEq for DoWhileStatement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.body.content_eq(&other.body) && self.test.content_eq(&other.test)
    }
}

impl<'a> ContentEq for WhileStatement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.test.content_eq(&other.test) && self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for ForStatement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.init.content_eq(&other.init)
            && self.test.content_eq(&other.test)
            && self.update.content_eq(&other.update)
            && self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for ForStatementInit<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::VariableDeclaration(it) => {
                matches!(other, Self::VariableDeclaration(other) if it.content_eq(other))
            }
            Self::BooleanLiteral(it) => {
                matches!(other, Self::BooleanLiteral(other) if it.content_eq(other))
            }
            Self::NullLiteral(it) => {
                matches!(other, Self::NullLiteral(other) if it.content_eq(other))
            }
            Self::NumericLiteral(it) => {
                matches!(other, Self::NumericLiteral(other) if it.content_eq(other))
            }
            Self::BigIntLiteral(it) => {
                matches!(other, Self::BigIntLiteral(other) if it.content_eq(other))
            }
            Self::RegExpLiteral(it) => {
                matches!(other, Self::RegExpLiteral(other) if it.content_eq(other))
            }
            Self::StringLiteral(it) => {
                matches!(other, Self::StringLiteral(other) if it.content_eq(other))
            }
            Self::TemplateLiteral(it) => {
                matches!(other, Self::TemplateLiteral(other) if it.content_eq(other))
            }
            Self::Identifier(it) => {
                matches!(other, Self::Identifier(other) if it.content_eq(other))
            }
            Self::MetaProperty(it) => {
                matches!(other, Self::MetaProperty(other) if it.content_eq(other))
            }
            Self::Super(it) => {
                matches!(other, Self::Super(other) if it.content_eq(other))
            }
            Self::ArrayExpression(it) => {
                matches!(other, Self::ArrayExpression(other) if it.content_eq(other))
            }
            Self::ArrowFunctionExpression(it) => {
                matches!(
                    other, Self::ArrowFunctionExpression(other) if it.content_eq(other)
                )
            }
            Self::AssignmentExpression(it) => {
                matches!(
                    other, Self::AssignmentExpression(other) if it.content_eq(other)
                )
            }
            Self::AwaitExpression(it) => {
                matches!(other, Self::AwaitExpression(other) if it.content_eq(other))
            }
            Self::BinaryExpression(it) => {
                matches!(other, Self::BinaryExpression(other) if it.content_eq(other))
            }
            Self::CallExpression(it) => {
                matches!(other, Self::CallExpression(other) if it.content_eq(other))
            }
            Self::ChainExpression(it) => {
                matches!(other, Self::ChainExpression(other) if it.content_eq(other))
            }
            Self::ClassExpression(it) => {
                matches!(other, Self::ClassExpression(other) if it.content_eq(other))
            }
            Self::ConditionalExpression(it) => {
                matches!(
                    other, Self::ConditionalExpression(other) if it.content_eq(other)
                )
            }
            Self::FunctionExpression(it) => {
                matches!(other, Self::FunctionExpression(other) if it.content_eq(other))
            }
            Self::ImportExpression(it) => {
                matches!(other, Self::ImportExpression(other) if it.content_eq(other))
            }
            Self::LogicalExpression(it) => {
                matches!(other, Self::LogicalExpression(other) if it.content_eq(other))
            }
            Self::NewExpression(it) => {
                matches!(other, Self::NewExpression(other) if it.content_eq(other))
            }
            Self::ObjectExpression(it) => {
                matches!(other, Self::ObjectExpression(other) if it.content_eq(other))
            }
            Self::ParenthesizedExpression(it) => {
                matches!(
                    other, Self::ParenthesizedExpression(other) if it.content_eq(other)
                )
            }
            Self::SequenceExpression(it) => {
                matches!(other, Self::SequenceExpression(other) if it.content_eq(other))
            }
            Self::TaggedTemplateExpression(it) => {
                matches!(
                    other, Self::TaggedTemplateExpression(other) if it.content_eq(other)
                )
            }
            Self::ThisExpression(it) => {
                matches!(other, Self::ThisExpression(other) if it.content_eq(other))
            }
            Self::UnaryExpression(it) => {
                matches!(other, Self::UnaryExpression(other) if it.content_eq(other))
            }
            Self::UpdateExpression(it) => {
                matches!(other, Self::UpdateExpression(other) if it.content_eq(other))
            }
            Self::YieldExpression(it) => {
                matches!(other, Self::YieldExpression(other) if it.content_eq(other))
            }
            Self::PrivateInExpression(it) => {
                matches!(other, Self::PrivateInExpression(other) if it.content_eq(other))
            }
            Self::JSXElement(it) => {
                matches!(other, Self::JSXElement(other) if it.content_eq(other))
            }
            Self::JSXFragment(it) => {
                matches!(other, Self::JSXFragment(other) if it.content_eq(other))
            }
            Self::TSAsExpression(it) => {
                matches!(other, Self::TSAsExpression(other) if it.content_eq(other))
            }
            Self::TSSatisfiesExpression(it) => {
                matches!(
                    other, Self::TSSatisfiesExpression(other) if it.content_eq(other)
                )
            }
            Self::TSTypeAssertion(it) => {
                matches!(other, Self::TSTypeAssertion(other) if it.content_eq(other))
            }
            Self::TSNonNullExpression(it) => {
                matches!(other, Self::TSNonNullExpression(other) if it.content_eq(other))
            }
            Self::TSInstantiationExpression(it) => {
                matches!(
                    other, Self::TSInstantiationExpression(other) if it.content_eq(other)
                )
            }
            Self::ComputedMemberExpression(it) => {
                matches!(
                    other, Self::ComputedMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::StaticMemberExpression(it) => {
                matches!(
                    other, Self::StaticMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::PrivateFieldExpression(it) => {
                matches!(
                    other, Self::PrivateFieldExpression(other) if it.content_eq(other)
                )
            }
        }
    }
}

impl<'a> ContentEq for ForInStatement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.left.content_eq(&other.left)
            && self.right.content_eq(&other.right)
            && self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for ForStatementLeft<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::VariableDeclaration(it) => {
                matches!(other, Self::VariableDeclaration(other) if it.content_eq(other))
            }
            Self::AssignmentTargetIdentifier(it) => {
                matches!(
                    other, Self::AssignmentTargetIdentifier(other) if it
                    .content_eq(other)
                )
            }
            Self::TSAsExpression(it) => {
                matches!(other, Self::TSAsExpression(other) if it.content_eq(other))
            }
            Self::TSSatisfiesExpression(it) => {
                matches!(
                    other, Self::TSSatisfiesExpression(other) if it.content_eq(other)
                )
            }
            Self::TSNonNullExpression(it) => {
                matches!(other, Self::TSNonNullExpression(other) if it.content_eq(other))
            }
            Self::TSTypeAssertion(it) => {
                matches!(other, Self::TSTypeAssertion(other) if it.content_eq(other))
            }
            Self::TSInstantiationExpression(it) => {
                matches!(
                    other, Self::TSInstantiationExpression(other) if it.content_eq(other)
                )
            }
            Self::ComputedMemberExpression(it) => {
                matches!(
                    other, Self::ComputedMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::StaticMemberExpression(it) => {
                matches!(
                    other, Self::StaticMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::PrivateFieldExpression(it) => {
                matches!(
                    other, Self::PrivateFieldExpression(other) if it.content_eq(other)
                )
            }
            Self::ArrayAssignmentTarget(it) => {
                matches!(
                    other, Self::ArrayAssignmentTarget(other) if it.content_eq(other)
                )
            }
            Self::ObjectAssignmentTarget(it) => {
                matches!(
                    other, Self::ObjectAssignmentTarget(other) if it.content_eq(other)
                )
            }
        }
    }
}

impl<'a> ContentEq for ForOfStatement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.r#await.content_eq(&other.r#await)
            && self.left.content_eq(&other.left)
            && self.right.content_eq(&other.right)
            && self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for ContinueStatement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.label.content_eq(&other.label)
    }
}

impl<'a> ContentEq for BreakStatement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.label.content_eq(&other.label)
    }
}

impl<'a> ContentEq for ReturnStatement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.argument.content_eq(&other.argument)
    }
}

impl<'a> ContentEq for WithStatement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.object.content_eq(&other.object) && self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for SwitchStatement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.discriminant.content_eq(&other.discriminant) && self.cases.content_eq(&other.cases)
    }
}

impl<'a> ContentEq for SwitchCase<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.test.content_eq(&other.test) && self.consequent.content_eq(&other.consequent)
    }
}

impl<'a> ContentEq for LabeledStatement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.label.content_eq(&other.label) && self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for ThrowStatement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.argument.content_eq(&other.argument)
    }
}

impl<'a> ContentEq for TryStatement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.block.content_eq(&other.block)
            && self.handler.content_eq(&other.handler)
            && self.finalizer.content_eq(&other.finalizer)
    }
}

impl<'a> ContentEq for CatchClause<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.param.content_eq(&other.param) && self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for CatchParameter<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.pattern.content_eq(&other.pattern)
    }
}

impl ContentEq for DebuggerStatement {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl<'a> ContentEq for BindingPattern<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.kind.content_eq(&other.kind)
            && self.type_annotation.content_eq(&other.type_annotation)
            && self.optional.content_eq(&other.optional)
    }
}

impl<'a> ContentEq for BindingPatternKind<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::BindingIdentifier(it) => {
                matches!(other, Self::BindingIdentifier(other) if it.content_eq(other))
            }
            Self::ObjectPattern(it) => {
                matches!(other, Self::ObjectPattern(other) if it.content_eq(other))
            }
            Self::ArrayPattern(it) => {
                matches!(other, Self::ArrayPattern(other) if it.content_eq(other))
            }
            Self::AssignmentPattern(it) => {
                matches!(other, Self::AssignmentPattern(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for AssignmentPattern<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.left.content_eq(&other.left) && self.right.content_eq(&other.right)
    }
}

impl<'a> ContentEq for ObjectPattern<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.properties.content_eq(&other.properties) && self.rest.content_eq(&other.rest)
    }
}

impl<'a> ContentEq for BindingProperty<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.key.content_eq(&other.key)
            && self.value.content_eq(&other.value)
            && self.shorthand.content_eq(&other.shorthand)
            && self.computed.content_eq(&other.computed)
    }
}

impl<'a> ContentEq for ArrayPattern<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.elements.content_eq(&other.elements) && self.rest.content_eq(&other.rest)
    }
}

impl<'a> ContentEq for BindingRestElement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.argument.content_eq(&other.argument)
    }
}

impl<'a> ContentEq for Function<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.r#type.content_eq(&other.r#type)
            && self.id.content_eq(&other.id)
            && self.generator.content_eq(&other.generator)
            && self.r#async.content_eq(&other.r#async)
            && self.declare.content_eq(&other.declare)
            && self.type_parameters.content_eq(&other.type_parameters)
            && self.this_param.content_eq(&other.this_param)
            && self.params.content_eq(&other.params)
            && self.return_type.content_eq(&other.return_type)
            && self.body.content_eq(&other.body)
    }
}

impl ContentEq for FunctionType {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentEq for FormalParameters<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.kind.content_eq(&other.kind)
            && self.items.content_eq(&other.items)
            && self.rest.content_eq(&other.rest)
    }
}

impl<'a> ContentEq for FormalParameter<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.decorators.content_eq(&other.decorators)
            && self.pattern.content_eq(&other.pattern)
            && self.accessibility.content_eq(&other.accessibility)
            && self.readonly.content_eq(&other.readonly)
            && self.r#override.content_eq(&other.r#override)
    }
}

impl ContentEq for FormalParameterKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentEq for FunctionBody<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.directives.content_eq(&other.directives)
            && self.statements.content_eq(&other.statements)
    }
}

impl<'a> ContentEq for ArrowFunctionExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expression.content_eq(&other.expression)
            && self.r#async.content_eq(&other.r#async)
            && self.type_parameters.content_eq(&other.type_parameters)
            && self.params.content_eq(&other.params)
            && self.return_type.content_eq(&other.return_type)
            && self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for YieldExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.delegate.content_eq(&other.delegate) && self.argument.content_eq(&other.argument)
    }
}

impl<'a> ContentEq for Class<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.r#type.content_eq(&other.r#type)
            && self.decorators.content_eq(&other.decorators)
            && self.id.content_eq(&other.id)
            && self.type_parameters.content_eq(&other.type_parameters)
            && self.super_class.content_eq(&other.super_class)
            && self.super_type_parameters.content_eq(&other.super_type_parameters)
            && self.implements.content_eq(&other.implements)
            && self.body.content_eq(&other.body)
            && self.r#abstract.content_eq(&other.r#abstract)
            && self.declare.content_eq(&other.declare)
    }
}

impl ContentEq for ClassType {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentEq for ClassBody<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for ClassElement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::StaticBlock(it) => {
                matches!(other, Self::StaticBlock(other) if it.content_eq(other))
            }
            Self::MethodDefinition(it) => {
                matches!(other, Self::MethodDefinition(other) if it.content_eq(other))
            }
            Self::PropertyDefinition(it) => {
                matches!(other, Self::PropertyDefinition(other) if it.content_eq(other))
            }
            Self::AccessorProperty(it) => {
                matches!(other, Self::AccessorProperty(other) if it.content_eq(other))
            }
            Self::TSIndexSignature(it) => {
                matches!(other, Self::TSIndexSignature(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for MethodDefinition<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.r#type.content_eq(&other.r#type)
            && self.decorators.content_eq(&other.decorators)
            && self.key.content_eq(&other.key)
            && self.value.content_eq(&other.value)
            && self.kind.content_eq(&other.kind)
            && self.computed.content_eq(&other.computed)
            && self.r#static.content_eq(&other.r#static)
            && self.r#override.content_eq(&other.r#override)
            && self.optional.content_eq(&other.optional)
            && self.accessibility.content_eq(&other.accessibility)
    }
}

impl ContentEq for MethodDefinitionType {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentEq for PropertyDefinition<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.r#type.content_eq(&other.r#type)
            && self.decorators.content_eq(&other.decorators)
            && self.key.content_eq(&other.key)
            && self.value.content_eq(&other.value)
            && self.computed.content_eq(&other.computed)
            && self.r#static.content_eq(&other.r#static)
            && self.declare.content_eq(&other.declare)
            && self.r#override.content_eq(&other.r#override)
            && self.optional.content_eq(&other.optional)
            && self.definite.content_eq(&other.definite)
            && self.readonly.content_eq(&other.readonly)
            && self.type_annotation.content_eq(&other.type_annotation)
            && self.accessibility.content_eq(&other.accessibility)
    }
}

impl ContentEq for PropertyDefinitionType {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for MethodDefinitionKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentEq for PrivateIdentifier<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.name.content_eq(&other.name)
    }
}

impl<'a> ContentEq for StaticBlock<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for ModuleDeclaration<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::ImportDeclaration(it) => {
                matches!(other, Self::ImportDeclaration(other) if it.content_eq(other))
            }
            Self::ExportAllDeclaration(it) => {
                matches!(
                    other, Self::ExportAllDeclaration(other) if it.content_eq(other)
                )
            }
            Self::ExportDefaultDeclaration(it) => {
                matches!(
                    other, Self::ExportDefaultDeclaration(other) if it.content_eq(other)
                )
            }
            Self::ExportNamedDeclaration(it) => {
                matches!(
                    other, Self::ExportNamedDeclaration(other) if it.content_eq(other)
                )
            }
            Self::TSExportAssignment(it) => {
                matches!(other, Self::TSExportAssignment(other) if it.content_eq(other))
            }
            Self::TSNamespaceExportDeclaration(it) => {
                matches!(
                    other, Self::TSNamespaceExportDeclaration(other) if it
                    .content_eq(other)
                )
            }
        }
    }
}

impl ContentEq for AccessorPropertyType {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentEq for AccessorProperty<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.r#type.content_eq(&other.r#type)
            && self.decorators.content_eq(&other.decorators)
            && self.key.content_eq(&other.key)
            && self.value.content_eq(&other.value)
            && self.computed.content_eq(&other.computed)
            && self.r#static.content_eq(&other.r#static)
            && self.definite.content_eq(&other.definite)
            && self.type_annotation.content_eq(&other.type_annotation)
            && self.accessibility.content_eq(&other.accessibility)
    }
}

impl<'a> ContentEq for ImportExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.source.content_eq(&other.source) && self.arguments.content_eq(&other.arguments)
    }
}

impl<'a> ContentEq for ImportDeclaration<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.specifiers.content_eq(&other.specifiers)
            && self.source.content_eq(&other.source)
            && self.with_clause.content_eq(&other.with_clause)
            && self.import_kind.content_eq(&other.import_kind)
    }
}

impl<'a> ContentEq for ImportDeclarationSpecifier<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::ImportSpecifier(it) => {
                matches!(other, Self::ImportSpecifier(other) if it.content_eq(other))
            }
            Self::ImportDefaultSpecifier(it) => {
                matches!(
                    other, Self::ImportDefaultSpecifier(other) if it.content_eq(other)
                )
            }
            Self::ImportNamespaceSpecifier(it) => {
                matches!(
                    other, Self::ImportNamespaceSpecifier(other) if it.content_eq(other)
                )
            }
        }
    }
}

impl<'a> ContentEq for ImportSpecifier<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.imported.content_eq(&other.imported)
            && self.local.content_eq(&other.local)
            && self.import_kind.content_eq(&other.import_kind)
    }
}

impl<'a> ContentEq for ImportDefaultSpecifier<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.local.content_eq(&other.local)
    }
}

impl<'a> ContentEq for ImportNamespaceSpecifier<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.local.content_eq(&other.local)
    }
}

impl<'a> ContentEq for WithClause<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.attributes_keyword.content_eq(&other.attributes_keyword)
            && self.with_entries.content_eq(&other.with_entries)
    }
}

impl<'a> ContentEq for ImportAttribute<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.key.content_eq(&other.key) && self.value.content_eq(&other.value)
    }
}

impl<'a> ContentEq for ImportAttributeKey<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::Identifier(it) => {
                matches!(other, Self::Identifier(other) if it.content_eq(other))
            }
            Self::StringLiteral(it) => {
                matches!(other, Self::StringLiteral(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for ExportNamedDeclaration<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.declaration.content_eq(&other.declaration)
            && self.specifiers.content_eq(&other.specifiers)
            && self.source.content_eq(&other.source)
            && self.export_kind.content_eq(&other.export_kind)
            && self.with_clause.content_eq(&other.with_clause)
    }
}

impl<'a> ContentEq for ExportDefaultDeclaration<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.declaration.content_eq(&other.declaration) && self.exported.content_eq(&other.exported)
    }
}

impl<'a> ContentEq for ExportAllDeclaration<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.exported.content_eq(&other.exported)
            && self.source.content_eq(&other.source)
            && self.with_clause.content_eq(&other.with_clause)
            && self.export_kind.content_eq(&other.export_kind)
    }
}

impl<'a> ContentEq for ExportSpecifier<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.local.content_eq(&other.local)
            && self.exported.content_eq(&other.exported)
            && self.export_kind.content_eq(&other.export_kind)
    }
}

impl<'a> ContentEq for ExportDefaultDeclarationKind<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::FunctionDeclaration(it) => {
                matches!(other, Self::FunctionDeclaration(other) if it.content_eq(other))
            }
            Self::ClassDeclaration(it) => {
                matches!(other, Self::ClassDeclaration(other) if it.content_eq(other))
            }
            Self::TSInterfaceDeclaration(it) => {
                matches!(
                    other, Self::TSInterfaceDeclaration(other) if it.content_eq(other)
                )
            }
            Self::BooleanLiteral(it) => {
                matches!(other, Self::BooleanLiteral(other) if it.content_eq(other))
            }
            Self::NullLiteral(it) => {
                matches!(other, Self::NullLiteral(other) if it.content_eq(other))
            }
            Self::NumericLiteral(it) => {
                matches!(other, Self::NumericLiteral(other) if it.content_eq(other))
            }
            Self::BigIntLiteral(it) => {
                matches!(other, Self::BigIntLiteral(other) if it.content_eq(other))
            }
            Self::RegExpLiteral(it) => {
                matches!(other, Self::RegExpLiteral(other) if it.content_eq(other))
            }
            Self::StringLiteral(it) => {
                matches!(other, Self::StringLiteral(other) if it.content_eq(other))
            }
            Self::TemplateLiteral(it) => {
                matches!(other, Self::TemplateLiteral(other) if it.content_eq(other))
            }
            Self::Identifier(it) => {
                matches!(other, Self::Identifier(other) if it.content_eq(other))
            }
            Self::MetaProperty(it) => {
                matches!(other, Self::MetaProperty(other) if it.content_eq(other))
            }
            Self::Super(it) => {
                matches!(other, Self::Super(other) if it.content_eq(other))
            }
            Self::ArrayExpression(it) => {
                matches!(other, Self::ArrayExpression(other) if it.content_eq(other))
            }
            Self::ArrowFunctionExpression(it) => {
                matches!(
                    other, Self::ArrowFunctionExpression(other) if it.content_eq(other)
                )
            }
            Self::AssignmentExpression(it) => {
                matches!(
                    other, Self::AssignmentExpression(other) if it.content_eq(other)
                )
            }
            Self::AwaitExpression(it) => {
                matches!(other, Self::AwaitExpression(other) if it.content_eq(other))
            }
            Self::BinaryExpression(it) => {
                matches!(other, Self::BinaryExpression(other) if it.content_eq(other))
            }
            Self::CallExpression(it) => {
                matches!(other, Self::CallExpression(other) if it.content_eq(other))
            }
            Self::ChainExpression(it) => {
                matches!(other, Self::ChainExpression(other) if it.content_eq(other))
            }
            Self::ClassExpression(it) => {
                matches!(other, Self::ClassExpression(other) if it.content_eq(other))
            }
            Self::ConditionalExpression(it) => {
                matches!(
                    other, Self::ConditionalExpression(other) if it.content_eq(other)
                )
            }
            Self::FunctionExpression(it) => {
                matches!(other, Self::FunctionExpression(other) if it.content_eq(other))
            }
            Self::ImportExpression(it) => {
                matches!(other, Self::ImportExpression(other) if it.content_eq(other))
            }
            Self::LogicalExpression(it) => {
                matches!(other, Self::LogicalExpression(other) if it.content_eq(other))
            }
            Self::NewExpression(it) => {
                matches!(other, Self::NewExpression(other) if it.content_eq(other))
            }
            Self::ObjectExpression(it) => {
                matches!(other, Self::ObjectExpression(other) if it.content_eq(other))
            }
            Self::ParenthesizedExpression(it) => {
                matches!(
                    other, Self::ParenthesizedExpression(other) if it.content_eq(other)
                )
            }
            Self::SequenceExpression(it) => {
                matches!(other, Self::SequenceExpression(other) if it.content_eq(other))
            }
            Self::TaggedTemplateExpression(it) => {
                matches!(
                    other, Self::TaggedTemplateExpression(other) if it.content_eq(other)
                )
            }
            Self::ThisExpression(it) => {
                matches!(other, Self::ThisExpression(other) if it.content_eq(other))
            }
            Self::UnaryExpression(it) => {
                matches!(other, Self::UnaryExpression(other) if it.content_eq(other))
            }
            Self::UpdateExpression(it) => {
                matches!(other, Self::UpdateExpression(other) if it.content_eq(other))
            }
            Self::YieldExpression(it) => {
                matches!(other, Self::YieldExpression(other) if it.content_eq(other))
            }
            Self::PrivateInExpression(it) => {
                matches!(other, Self::PrivateInExpression(other) if it.content_eq(other))
            }
            Self::JSXElement(it) => {
                matches!(other, Self::JSXElement(other) if it.content_eq(other))
            }
            Self::JSXFragment(it) => {
                matches!(other, Self::JSXFragment(other) if it.content_eq(other))
            }
            Self::TSAsExpression(it) => {
                matches!(other, Self::TSAsExpression(other) if it.content_eq(other))
            }
            Self::TSSatisfiesExpression(it) => {
                matches!(
                    other, Self::TSSatisfiesExpression(other) if it.content_eq(other)
                )
            }
            Self::TSTypeAssertion(it) => {
                matches!(other, Self::TSTypeAssertion(other) if it.content_eq(other))
            }
            Self::TSNonNullExpression(it) => {
                matches!(other, Self::TSNonNullExpression(other) if it.content_eq(other))
            }
            Self::TSInstantiationExpression(it) => {
                matches!(
                    other, Self::TSInstantiationExpression(other) if it.content_eq(other)
                )
            }
            Self::ComputedMemberExpression(it) => {
                matches!(
                    other, Self::ComputedMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::StaticMemberExpression(it) => {
                matches!(
                    other, Self::StaticMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::PrivateFieldExpression(it) => {
                matches!(
                    other, Self::PrivateFieldExpression(other) if it.content_eq(other)
                )
            }
        }
    }
}

impl<'a> ContentEq for ModuleExportName<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::IdentifierName(it) => {
                matches!(other, Self::IdentifierName(other) if it.content_eq(other))
            }
            Self::IdentifierReference(it) => {
                matches!(other, Self::IdentifierReference(other) if it.content_eq(other))
            }
            Self::StringLiteral(it) => {
                matches!(other, Self::StringLiteral(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for TSThisParameter<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.this.content_eq(&other.this) && self.type_annotation.content_eq(&other.type_annotation)
    }
}

impl<'a> ContentEq for TSEnumDeclaration<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.id.content_eq(&other.id)
            && self.members.content_eq(&other.members)
            && self.r#const.content_eq(&other.r#const)
            && self.declare.content_eq(&other.declare)
    }
}

impl<'a> ContentEq for TSEnumMember<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.id.content_eq(&other.id) && self.initializer.content_eq(&other.initializer)
    }
}

impl<'a> ContentEq for TSEnumMemberName<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::StaticIdentifier(it) => {
                matches!(other, Self::StaticIdentifier(other) if it.content_eq(other))
            }
            Self::StaticStringLiteral(it) => {
                matches!(other, Self::StaticStringLiteral(other) if it.content_eq(other))
            }
            Self::StaticTemplateLiteral(it) => {
                matches!(
                    other, Self::StaticTemplateLiteral(other) if it.content_eq(other)
                )
            }
            Self::StaticNumericLiteral(it) => {
                matches!(
                    other, Self::StaticNumericLiteral(other) if it.content_eq(other)
                )
            }
            Self::BooleanLiteral(it) => {
                matches!(other, Self::BooleanLiteral(other) if it.content_eq(other))
            }
            Self::NullLiteral(it) => {
                matches!(other, Self::NullLiteral(other) if it.content_eq(other))
            }
            Self::NumericLiteral(it) => {
                matches!(other, Self::NumericLiteral(other) if it.content_eq(other))
            }
            Self::BigIntLiteral(it) => {
                matches!(other, Self::BigIntLiteral(other) if it.content_eq(other))
            }
            Self::RegExpLiteral(it) => {
                matches!(other, Self::RegExpLiteral(other) if it.content_eq(other))
            }
            Self::StringLiteral(it) => {
                matches!(other, Self::StringLiteral(other) if it.content_eq(other))
            }
            Self::TemplateLiteral(it) => {
                matches!(other, Self::TemplateLiteral(other) if it.content_eq(other))
            }
            Self::Identifier(it) => {
                matches!(other, Self::Identifier(other) if it.content_eq(other))
            }
            Self::MetaProperty(it) => {
                matches!(other, Self::MetaProperty(other) if it.content_eq(other))
            }
            Self::Super(it) => {
                matches!(other, Self::Super(other) if it.content_eq(other))
            }
            Self::ArrayExpression(it) => {
                matches!(other, Self::ArrayExpression(other) if it.content_eq(other))
            }
            Self::ArrowFunctionExpression(it) => {
                matches!(
                    other, Self::ArrowFunctionExpression(other) if it.content_eq(other)
                )
            }
            Self::AssignmentExpression(it) => {
                matches!(
                    other, Self::AssignmentExpression(other) if it.content_eq(other)
                )
            }
            Self::AwaitExpression(it) => {
                matches!(other, Self::AwaitExpression(other) if it.content_eq(other))
            }
            Self::BinaryExpression(it) => {
                matches!(other, Self::BinaryExpression(other) if it.content_eq(other))
            }
            Self::CallExpression(it) => {
                matches!(other, Self::CallExpression(other) if it.content_eq(other))
            }
            Self::ChainExpression(it) => {
                matches!(other, Self::ChainExpression(other) if it.content_eq(other))
            }
            Self::ClassExpression(it) => {
                matches!(other, Self::ClassExpression(other) if it.content_eq(other))
            }
            Self::ConditionalExpression(it) => {
                matches!(
                    other, Self::ConditionalExpression(other) if it.content_eq(other)
                )
            }
            Self::FunctionExpression(it) => {
                matches!(other, Self::FunctionExpression(other) if it.content_eq(other))
            }
            Self::ImportExpression(it) => {
                matches!(other, Self::ImportExpression(other) if it.content_eq(other))
            }
            Self::LogicalExpression(it) => {
                matches!(other, Self::LogicalExpression(other) if it.content_eq(other))
            }
            Self::NewExpression(it) => {
                matches!(other, Self::NewExpression(other) if it.content_eq(other))
            }
            Self::ObjectExpression(it) => {
                matches!(other, Self::ObjectExpression(other) if it.content_eq(other))
            }
            Self::ParenthesizedExpression(it) => {
                matches!(
                    other, Self::ParenthesizedExpression(other) if it.content_eq(other)
                )
            }
            Self::SequenceExpression(it) => {
                matches!(other, Self::SequenceExpression(other) if it.content_eq(other))
            }
            Self::TaggedTemplateExpression(it) => {
                matches!(
                    other, Self::TaggedTemplateExpression(other) if it.content_eq(other)
                )
            }
            Self::ThisExpression(it) => {
                matches!(other, Self::ThisExpression(other) if it.content_eq(other))
            }
            Self::UnaryExpression(it) => {
                matches!(other, Self::UnaryExpression(other) if it.content_eq(other))
            }
            Self::UpdateExpression(it) => {
                matches!(other, Self::UpdateExpression(other) if it.content_eq(other))
            }
            Self::YieldExpression(it) => {
                matches!(other, Self::YieldExpression(other) if it.content_eq(other))
            }
            Self::PrivateInExpression(it) => {
                matches!(other, Self::PrivateInExpression(other) if it.content_eq(other))
            }
            Self::JSXElement(it) => {
                matches!(other, Self::JSXElement(other) if it.content_eq(other))
            }
            Self::JSXFragment(it) => {
                matches!(other, Self::JSXFragment(other) if it.content_eq(other))
            }
            Self::TSAsExpression(it) => {
                matches!(other, Self::TSAsExpression(other) if it.content_eq(other))
            }
            Self::TSSatisfiesExpression(it) => {
                matches!(
                    other, Self::TSSatisfiesExpression(other) if it.content_eq(other)
                )
            }
            Self::TSTypeAssertion(it) => {
                matches!(other, Self::TSTypeAssertion(other) if it.content_eq(other))
            }
            Self::TSNonNullExpression(it) => {
                matches!(other, Self::TSNonNullExpression(other) if it.content_eq(other))
            }
            Self::TSInstantiationExpression(it) => {
                matches!(
                    other, Self::TSInstantiationExpression(other) if it.content_eq(other)
                )
            }
            Self::ComputedMemberExpression(it) => {
                matches!(
                    other, Self::ComputedMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::StaticMemberExpression(it) => {
                matches!(
                    other, Self::StaticMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::PrivateFieldExpression(it) => {
                matches!(
                    other, Self::PrivateFieldExpression(other) if it.content_eq(other)
                )
            }
        }
    }
}

impl<'a> ContentEq for TSTypeAnnotation<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.type_annotation.content_eq(&other.type_annotation)
    }
}

impl<'a> ContentEq for TSLiteralType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.literal.content_eq(&other.literal)
    }
}

impl<'a> ContentEq for TSLiteral<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::BooleanLiteral(it) => {
                matches!(other, Self::BooleanLiteral(other) if it.content_eq(other))
            }
            Self::NullLiteral(it) => {
                matches!(other, Self::NullLiteral(other) if it.content_eq(other))
            }
            Self::NumericLiteral(it) => {
                matches!(other, Self::NumericLiteral(other) if it.content_eq(other))
            }
            Self::BigIntLiteral(it) => {
                matches!(other, Self::BigIntLiteral(other) if it.content_eq(other))
            }
            Self::RegExpLiteral(it) => {
                matches!(other, Self::RegExpLiteral(other) if it.content_eq(other))
            }
            Self::StringLiteral(it) => {
                matches!(other, Self::StringLiteral(other) if it.content_eq(other))
            }
            Self::TemplateLiteral(it) => {
                matches!(other, Self::TemplateLiteral(other) if it.content_eq(other))
            }
            Self::UnaryExpression(it) => {
                matches!(other, Self::UnaryExpression(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for TSType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::TSAnyKeyword(it) => {
                matches!(other, Self::TSAnyKeyword(other) if it.content_eq(other))
            }
            Self::TSBigIntKeyword(it) => {
                matches!(other, Self::TSBigIntKeyword(other) if it.content_eq(other))
            }
            Self::TSBooleanKeyword(it) => {
                matches!(other, Self::TSBooleanKeyword(other) if it.content_eq(other))
            }
            Self::TSIntrinsicKeyword(it) => {
                matches!(other, Self::TSIntrinsicKeyword(other) if it.content_eq(other))
            }
            Self::TSNeverKeyword(it) => {
                matches!(other, Self::TSNeverKeyword(other) if it.content_eq(other))
            }
            Self::TSNullKeyword(it) => {
                matches!(other, Self::TSNullKeyword(other) if it.content_eq(other))
            }
            Self::TSNumberKeyword(it) => {
                matches!(other, Self::TSNumberKeyword(other) if it.content_eq(other))
            }
            Self::TSObjectKeyword(it) => {
                matches!(other, Self::TSObjectKeyword(other) if it.content_eq(other))
            }
            Self::TSStringKeyword(it) => {
                matches!(other, Self::TSStringKeyword(other) if it.content_eq(other))
            }
            Self::TSSymbolKeyword(it) => {
                matches!(other, Self::TSSymbolKeyword(other) if it.content_eq(other))
            }
            Self::TSUndefinedKeyword(it) => {
                matches!(other, Self::TSUndefinedKeyword(other) if it.content_eq(other))
            }
            Self::TSUnknownKeyword(it) => {
                matches!(other, Self::TSUnknownKeyword(other) if it.content_eq(other))
            }
            Self::TSVoidKeyword(it) => {
                matches!(other, Self::TSVoidKeyword(other) if it.content_eq(other))
            }
            Self::TSArrayType(it) => {
                matches!(other, Self::TSArrayType(other) if it.content_eq(other))
            }
            Self::TSConditionalType(it) => {
                matches!(other, Self::TSConditionalType(other) if it.content_eq(other))
            }
            Self::TSConstructorType(it) => {
                matches!(other, Self::TSConstructorType(other) if it.content_eq(other))
            }
            Self::TSFunctionType(it) => {
                matches!(other, Self::TSFunctionType(other) if it.content_eq(other))
            }
            Self::TSImportType(it) => {
                matches!(other, Self::TSImportType(other) if it.content_eq(other))
            }
            Self::TSIndexedAccessType(it) => {
                matches!(other, Self::TSIndexedAccessType(other) if it.content_eq(other))
            }
            Self::TSInferType(it) => {
                matches!(other, Self::TSInferType(other) if it.content_eq(other))
            }
            Self::TSIntersectionType(it) => {
                matches!(other, Self::TSIntersectionType(other) if it.content_eq(other))
            }
            Self::TSLiteralType(it) => {
                matches!(other, Self::TSLiteralType(other) if it.content_eq(other))
            }
            Self::TSMappedType(it) => {
                matches!(other, Self::TSMappedType(other) if it.content_eq(other))
            }
            Self::TSNamedTupleMember(it) => {
                matches!(other, Self::TSNamedTupleMember(other) if it.content_eq(other))
            }
            Self::TSQualifiedName(it) => {
                matches!(other, Self::TSQualifiedName(other) if it.content_eq(other))
            }
            Self::TSTemplateLiteralType(it) => {
                matches!(
                    other, Self::TSTemplateLiteralType(other) if it.content_eq(other)
                )
            }
            Self::TSThisType(it) => {
                matches!(other, Self::TSThisType(other) if it.content_eq(other))
            }
            Self::TSTupleType(it) => {
                matches!(other, Self::TSTupleType(other) if it.content_eq(other))
            }
            Self::TSTypeLiteral(it) => {
                matches!(other, Self::TSTypeLiteral(other) if it.content_eq(other))
            }
            Self::TSTypeOperatorType(it) => {
                matches!(other, Self::TSTypeOperatorType(other) if it.content_eq(other))
            }
            Self::TSTypePredicate(it) => {
                matches!(other, Self::TSTypePredicate(other) if it.content_eq(other))
            }
            Self::TSTypeQuery(it) => {
                matches!(other, Self::TSTypeQuery(other) if it.content_eq(other))
            }
            Self::TSTypeReference(it) => {
                matches!(other, Self::TSTypeReference(other) if it.content_eq(other))
            }
            Self::TSUnionType(it) => {
                matches!(other, Self::TSUnionType(other) if it.content_eq(other))
            }
            Self::TSParenthesizedType(it) => {
                matches!(other, Self::TSParenthesizedType(other) if it.content_eq(other))
            }
            Self::JSDocNullableType(it) => {
                matches!(other, Self::JSDocNullableType(other) if it.content_eq(other))
            }
            Self::JSDocNonNullableType(it) => {
                matches!(
                    other, Self::JSDocNonNullableType(other) if it.content_eq(other)
                )
            }
            Self::JSDocUnknownType(it) => {
                matches!(other, Self::JSDocUnknownType(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for TSConditionalType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.check_type.content_eq(&other.check_type)
            && self.extends_type.content_eq(&other.extends_type)
            && self.true_type.content_eq(&other.true_type)
            && self.false_type.content_eq(&other.false_type)
    }
}

impl<'a> ContentEq for TSUnionType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.types.content_eq(&other.types)
    }
}

impl<'a> ContentEq for TSIntersectionType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.types.content_eq(&other.types)
    }
}

impl<'a> ContentEq for TSParenthesizedType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.type_annotation.content_eq(&other.type_annotation)
    }
}

impl<'a> ContentEq for TSTypeOperator<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.operator.content_eq(&other.operator)
            && self.type_annotation.content_eq(&other.type_annotation)
    }
}

impl ContentEq for TSTypeOperatorOperator {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentEq for TSArrayType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.element_type.content_eq(&other.element_type)
    }
}

impl<'a> ContentEq for TSIndexedAccessType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.object_type.content_eq(&other.object_type)
            && self.index_type.content_eq(&other.index_type)
    }
}

impl<'a> ContentEq for TSTupleType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.element_types.content_eq(&other.element_types)
    }
}

impl<'a> ContentEq for TSNamedTupleMember<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.element_type.content_eq(&other.element_type)
            && self.label.content_eq(&other.label)
            && self.optional.content_eq(&other.optional)
    }
}

impl<'a> ContentEq for TSOptionalType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.type_annotation.content_eq(&other.type_annotation)
    }
}

impl<'a> ContentEq for TSRestType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.type_annotation.content_eq(&other.type_annotation)
    }
}

impl<'a> ContentEq for TSTupleElement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::TSOptionalType(it) => {
                matches!(other, Self::TSOptionalType(other) if it.content_eq(other))
            }
            Self::TSRestType(it) => {
                matches!(other, Self::TSRestType(other) if it.content_eq(other))
            }
            Self::TSAnyKeyword(it) => {
                matches!(other, Self::TSAnyKeyword(other) if it.content_eq(other))
            }
            Self::TSBigIntKeyword(it) => {
                matches!(other, Self::TSBigIntKeyword(other) if it.content_eq(other))
            }
            Self::TSBooleanKeyword(it) => {
                matches!(other, Self::TSBooleanKeyword(other) if it.content_eq(other))
            }
            Self::TSIntrinsicKeyword(it) => {
                matches!(other, Self::TSIntrinsicKeyword(other) if it.content_eq(other))
            }
            Self::TSNeverKeyword(it) => {
                matches!(other, Self::TSNeverKeyword(other) if it.content_eq(other))
            }
            Self::TSNullKeyword(it) => {
                matches!(other, Self::TSNullKeyword(other) if it.content_eq(other))
            }
            Self::TSNumberKeyword(it) => {
                matches!(other, Self::TSNumberKeyword(other) if it.content_eq(other))
            }
            Self::TSObjectKeyword(it) => {
                matches!(other, Self::TSObjectKeyword(other) if it.content_eq(other))
            }
            Self::TSStringKeyword(it) => {
                matches!(other, Self::TSStringKeyword(other) if it.content_eq(other))
            }
            Self::TSSymbolKeyword(it) => {
                matches!(other, Self::TSSymbolKeyword(other) if it.content_eq(other))
            }
            Self::TSUndefinedKeyword(it) => {
                matches!(other, Self::TSUndefinedKeyword(other) if it.content_eq(other))
            }
            Self::TSUnknownKeyword(it) => {
                matches!(other, Self::TSUnknownKeyword(other) if it.content_eq(other))
            }
            Self::TSVoidKeyword(it) => {
                matches!(other, Self::TSVoidKeyword(other) if it.content_eq(other))
            }
            Self::TSArrayType(it) => {
                matches!(other, Self::TSArrayType(other) if it.content_eq(other))
            }
            Self::TSConditionalType(it) => {
                matches!(other, Self::TSConditionalType(other) if it.content_eq(other))
            }
            Self::TSConstructorType(it) => {
                matches!(other, Self::TSConstructorType(other) if it.content_eq(other))
            }
            Self::TSFunctionType(it) => {
                matches!(other, Self::TSFunctionType(other) if it.content_eq(other))
            }
            Self::TSImportType(it) => {
                matches!(other, Self::TSImportType(other) if it.content_eq(other))
            }
            Self::TSIndexedAccessType(it) => {
                matches!(other, Self::TSIndexedAccessType(other) if it.content_eq(other))
            }
            Self::TSInferType(it) => {
                matches!(other, Self::TSInferType(other) if it.content_eq(other))
            }
            Self::TSIntersectionType(it) => {
                matches!(other, Self::TSIntersectionType(other) if it.content_eq(other))
            }
            Self::TSLiteralType(it) => {
                matches!(other, Self::TSLiteralType(other) if it.content_eq(other))
            }
            Self::TSMappedType(it) => {
                matches!(other, Self::TSMappedType(other) if it.content_eq(other))
            }
            Self::TSNamedTupleMember(it) => {
                matches!(other, Self::TSNamedTupleMember(other) if it.content_eq(other))
            }
            Self::TSQualifiedName(it) => {
                matches!(other, Self::TSQualifiedName(other) if it.content_eq(other))
            }
            Self::TSTemplateLiteralType(it) => {
                matches!(
                    other, Self::TSTemplateLiteralType(other) if it.content_eq(other)
                )
            }
            Self::TSThisType(it) => {
                matches!(other, Self::TSThisType(other) if it.content_eq(other))
            }
            Self::TSTupleType(it) => {
                matches!(other, Self::TSTupleType(other) if it.content_eq(other))
            }
            Self::TSTypeLiteral(it) => {
                matches!(other, Self::TSTypeLiteral(other) if it.content_eq(other))
            }
            Self::TSTypeOperatorType(it) => {
                matches!(other, Self::TSTypeOperatorType(other) if it.content_eq(other))
            }
            Self::TSTypePredicate(it) => {
                matches!(other, Self::TSTypePredicate(other) if it.content_eq(other))
            }
            Self::TSTypeQuery(it) => {
                matches!(other, Self::TSTypeQuery(other) if it.content_eq(other))
            }
            Self::TSTypeReference(it) => {
                matches!(other, Self::TSTypeReference(other) if it.content_eq(other))
            }
            Self::TSUnionType(it) => {
                matches!(other, Self::TSUnionType(other) if it.content_eq(other))
            }
            Self::TSParenthesizedType(it) => {
                matches!(other, Self::TSParenthesizedType(other) if it.content_eq(other))
            }
            Self::JSDocNullableType(it) => {
                matches!(other, Self::JSDocNullableType(other) if it.content_eq(other))
            }
            Self::JSDocNonNullableType(it) => {
                matches!(
                    other, Self::JSDocNonNullableType(other) if it.content_eq(other)
                )
            }
            Self::JSDocUnknownType(it) => {
                matches!(other, Self::JSDocUnknownType(other) if it.content_eq(other))
            }
        }
    }
}

impl ContentEq for TSAnyKeyword {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for TSStringKeyword {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for TSBooleanKeyword {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for TSNumberKeyword {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for TSNeverKeyword {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for TSIntrinsicKeyword {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for TSUnknownKeyword {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for TSNullKeyword {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for TSUndefinedKeyword {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for TSVoidKeyword {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for TSSymbolKeyword {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for TSThisType {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for TSObjectKeyword {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for TSBigIntKeyword {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl<'a> ContentEq for TSTypeReference<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.type_name.content_eq(&other.type_name)
            && self.type_parameters.content_eq(&other.type_parameters)
    }
}

impl<'a> ContentEq for TSTypeName<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::IdentifierReference(it) => {
                matches!(other, Self::IdentifierReference(other) if it.content_eq(other))
            }
            Self::QualifiedName(it) => {
                matches!(other, Self::QualifiedName(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for TSQualifiedName<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.left.content_eq(&other.left) && self.right.content_eq(&other.right)
    }
}

impl<'a> ContentEq for TSTypeParameterInstantiation<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.params.content_eq(&other.params)
    }
}

impl<'a> ContentEq for TSTypeParameter<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.name.content_eq(&other.name)
            && self.constraint.content_eq(&other.constraint)
            && self.default.content_eq(&other.default)
            && self.r#in.content_eq(&other.r#in)
            && self.out.content_eq(&other.out)
            && self.r#const.content_eq(&other.r#const)
    }
}

impl<'a> ContentEq for TSTypeParameterDeclaration<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.params.content_eq(&other.params)
    }
}

impl<'a> ContentEq for TSTypeAliasDeclaration<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.id.content_eq(&other.id)
            && self.type_parameters.content_eq(&other.type_parameters)
            && self.type_annotation.content_eq(&other.type_annotation)
            && self.declare.content_eq(&other.declare)
    }
}

impl ContentEq for TSAccessibility {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentEq for TSClassImplements<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expression.content_eq(&other.expression)
            && self.type_parameters.content_eq(&other.type_parameters)
    }
}

impl<'a> ContentEq for TSInterfaceDeclaration<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.id.content_eq(&other.id)
            && self.extends.content_eq(&other.extends)
            && self.type_parameters.content_eq(&other.type_parameters)
            && self.body.content_eq(&other.body)
            && self.declare.content_eq(&other.declare)
    }
}

impl<'a> ContentEq for TSInterfaceBody<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for TSPropertySignature<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.computed.content_eq(&other.computed)
            && self.optional.content_eq(&other.optional)
            && self.readonly.content_eq(&other.readonly)
            && self.key.content_eq(&other.key)
            && self.type_annotation.content_eq(&other.type_annotation)
    }
}

impl<'a> ContentEq for TSSignature<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::TSIndexSignature(it) => {
                matches!(other, Self::TSIndexSignature(other) if it.content_eq(other))
            }
            Self::TSPropertySignature(it) => {
                matches!(other, Self::TSPropertySignature(other) if it.content_eq(other))
            }
            Self::TSCallSignatureDeclaration(it) => {
                matches!(
                    other, Self::TSCallSignatureDeclaration(other) if it
                    .content_eq(other)
                )
            }
            Self::TSConstructSignatureDeclaration(it) => {
                matches!(
                    other, Self::TSConstructSignatureDeclaration(other) if it
                    .content_eq(other)
                )
            }
            Self::TSMethodSignature(it) => {
                matches!(other, Self::TSMethodSignature(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for TSIndexSignature<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.parameters.content_eq(&other.parameters)
            && self.type_annotation.content_eq(&other.type_annotation)
            && self.readonly.content_eq(&other.readonly)
    }
}

impl<'a> ContentEq for TSCallSignatureDeclaration<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.this_param.content_eq(&other.this_param)
            && self.params.content_eq(&other.params)
            && self.return_type.content_eq(&other.return_type)
            && self.type_parameters.content_eq(&other.type_parameters)
    }
}

impl ContentEq for TSMethodSignatureKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentEq for TSMethodSignature<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.key.content_eq(&other.key)
            && self.computed.content_eq(&other.computed)
            && self.optional.content_eq(&other.optional)
            && self.kind.content_eq(&other.kind)
            && self.this_param.content_eq(&other.this_param)
            && self.params.content_eq(&other.params)
            && self.return_type.content_eq(&other.return_type)
            && self.type_parameters.content_eq(&other.type_parameters)
    }
}

impl<'a> ContentEq for TSConstructSignatureDeclaration<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.params.content_eq(&other.params)
            && self.return_type.content_eq(&other.return_type)
            && self.type_parameters.content_eq(&other.type_parameters)
    }
}

impl<'a> ContentEq for TSIndexSignatureName<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.name.content_eq(&other.name) && self.type_annotation.content_eq(&other.type_annotation)
    }
}

impl<'a> ContentEq for TSInterfaceHeritage<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expression.content_eq(&other.expression)
            && self.type_parameters.content_eq(&other.type_parameters)
    }
}

impl<'a> ContentEq for TSTypePredicate<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.parameter_name.content_eq(&other.parameter_name)
            && self.asserts.content_eq(&other.asserts)
            && self.type_annotation.content_eq(&other.type_annotation)
    }
}

impl<'a> ContentEq for TSTypePredicateName<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::Identifier(it) => {
                matches!(other, Self::Identifier(other) if it.content_eq(other))
            }
            Self::This(it) => matches!(other, Self::This(other) if it.content_eq(other)),
        }
    }
}

impl<'a> ContentEq for TSModuleDeclaration<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.id.content_eq(&other.id)
            && self.body.content_eq(&other.body)
            && self.kind.content_eq(&other.kind)
            && self.declare.content_eq(&other.declare)
    }
}

impl ContentEq for TSModuleDeclarationKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentEq for TSModuleDeclarationName<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::Identifier(it) => {
                matches!(other, Self::Identifier(other) if it.content_eq(other))
            }
            Self::StringLiteral(it) => {
                matches!(other, Self::StringLiteral(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for TSModuleDeclarationBody<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::TSModuleDeclaration(it) => {
                matches!(other, Self::TSModuleDeclaration(other) if it.content_eq(other))
            }
            Self::TSModuleBlock(it) => {
                matches!(other, Self::TSModuleBlock(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for TSModuleBlock<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.directives.content_eq(&other.directives) && self.body.content_eq(&other.body)
    }
}

impl<'a> ContentEq for TSTypeLiteral<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.members.content_eq(&other.members)
    }
}

impl<'a> ContentEq for TSInferType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.type_parameter.content_eq(&other.type_parameter)
    }
}

impl<'a> ContentEq for TSTypeQuery<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expr_name.content_eq(&other.expr_name)
            && self.type_parameters.content_eq(&other.type_parameters)
    }
}

impl<'a> ContentEq for TSTypeQueryExprName<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::TSImportType(it) => {
                matches!(other, Self::TSImportType(other) if it.content_eq(other))
            }
            Self::IdentifierReference(it) => {
                matches!(other, Self::IdentifierReference(other) if it.content_eq(other))
            }
            Self::QualifiedName(it) => {
                matches!(other, Self::QualifiedName(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for TSImportType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.is_type_of.content_eq(&other.is_type_of)
            && self.parameter.content_eq(&other.parameter)
            && self.qualifier.content_eq(&other.qualifier)
            && self.attributes.content_eq(&other.attributes)
            && self.type_parameters.content_eq(&other.type_parameters)
    }
}

impl<'a> ContentEq for TSImportAttributes<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.attributes_keyword.content_eq(&other.attributes_keyword)
            && self.elements.content_eq(&other.elements)
    }
}

impl<'a> ContentEq for TSImportAttribute<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.name.content_eq(&other.name) && self.value.content_eq(&other.value)
    }
}

impl<'a> ContentEq for TSImportAttributeName<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::Identifier(it) => {
                matches!(other, Self::Identifier(other) if it.content_eq(other))
            }
            Self::StringLiteral(it) => {
                matches!(other, Self::StringLiteral(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for TSFunctionType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.this_param.content_eq(&other.this_param)
            && self.params.content_eq(&other.params)
            && self.return_type.content_eq(&other.return_type)
            && self.type_parameters.content_eq(&other.type_parameters)
    }
}

impl<'a> ContentEq for TSConstructorType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.r#abstract.content_eq(&other.r#abstract)
            && self.params.content_eq(&other.params)
            && self.return_type.content_eq(&other.return_type)
            && self.type_parameters.content_eq(&other.type_parameters)
    }
}

impl<'a> ContentEq for TSMappedType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.type_parameter.content_eq(&other.type_parameter)
            && self.name_type.content_eq(&other.name_type)
            && self.type_annotation.content_eq(&other.type_annotation)
            && self.optional.content_eq(&other.optional)
            && self.readonly.content_eq(&other.readonly)
    }
}

impl ContentEq for TSMappedTypeModifierOperator {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentEq for TSTemplateLiteralType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.quasis.content_eq(&other.quasis) && self.types.content_eq(&other.types)
    }
}

impl<'a> ContentEq for TSAsExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expression.content_eq(&other.expression)
            && self.type_annotation.content_eq(&other.type_annotation)
    }
}

impl<'a> ContentEq for TSSatisfiesExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expression.content_eq(&other.expression)
            && self.type_annotation.content_eq(&other.type_annotation)
    }
}

impl<'a> ContentEq for TSTypeAssertion<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expression.content_eq(&other.expression)
            && self.type_annotation.content_eq(&other.type_annotation)
    }
}

impl<'a> ContentEq for TSImportEqualsDeclaration<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.id.content_eq(&other.id)
            && self.module_reference.content_eq(&other.module_reference)
            && self.import_kind.content_eq(&other.import_kind)
    }
}

impl<'a> ContentEq for TSModuleReference<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::ExternalModuleReference(it) => {
                matches!(
                    other, Self::ExternalModuleReference(other) if it.content_eq(other)
                )
            }
            Self::IdentifierReference(it) => {
                matches!(other, Self::IdentifierReference(other) if it.content_eq(other))
            }
            Self::QualifiedName(it) => {
                matches!(other, Self::QualifiedName(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for TSExternalModuleReference<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expression.content_eq(&other.expression)
    }
}

impl<'a> ContentEq for TSNonNullExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expression.content_eq(&other.expression)
    }
}

impl<'a> ContentEq for Decorator<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expression.content_eq(&other.expression)
    }
}

impl<'a> ContentEq for TSExportAssignment<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expression.content_eq(&other.expression)
    }
}

impl<'a> ContentEq for TSNamespaceExportDeclaration<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.id.content_eq(&other.id)
    }
}

impl<'a> ContentEq for TSInstantiationExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expression.content_eq(&other.expression)
            && self.type_parameters.content_eq(&other.type_parameters)
    }
}

impl ContentEq for ImportOrExportKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentEq for JSDocNullableType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.type_annotation.content_eq(&other.type_annotation)
            && self.postfix.content_eq(&other.postfix)
    }
}

impl<'a> ContentEq for JSDocNonNullableType<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.type_annotation.content_eq(&other.type_annotation)
            && self.postfix.content_eq(&other.postfix)
    }
}

impl ContentEq for JSDocUnknownType {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl<'a> ContentEq for JSXElement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.opening_element.content_eq(&other.opening_element)
            && self.closing_element.content_eq(&other.closing_element)
            && self.children.content_eq(&other.children)
    }
}

impl<'a> ContentEq for JSXOpeningElement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.self_closing.content_eq(&other.self_closing)
            && self.name.content_eq(&other.name)
            && self.attributes.content_eq(&other.attributes)
            && self.type_parameters.content_eq(&other.type_parameters)
    }
}

impl<'a> ContentEq for JSXClosingElement<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.name.content_eq(&other.name)
    }
}

impl<'a> ContentEq for JSXFragment<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.opening_fragment.content_eq(&other.opening_fragment)
            && self.closing_fragment.content_eq(&other.closing_fragment)
            && self.children.content_eq(&other.children)
    }
}

impl ContentEq for JSXOpeningFragment {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for JSXClosingFragment {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl<'a> ContentEq for JSXElementName<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::Identifier(it) => {
                matches!(other, Self::Identifier(other) if it.content_eq(other))
            }
            Self::IdentifierReference(it) => {
                matches!(other, Self::IdentifierReference(other) if it.content_eq(other))
            }
            Self::NamespacedName(it) => {
                matches!(other, Self::NamespacedName(other) if it.content_eq(other))
            }
            Self::MemberExpression(it) => {
                matches!(other, Self::MemberExpression(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for JSXNamespacedName<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.namespace.content_eq(&other.namespace) && self.property.content_eq(&other.property)
    }
}

impl<'a> ContentEq for JSXMemberExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.object.content_eq(&other.object) && self.property.content_eq(&other.property)
    }
}

impl<'a> ContentEq for JSXMemberExpressionObject<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::IdentifierReference(it) => {
                matches!(other, Self::IdentifierReference(other) if it.content_eq(other))
            }
            Self::MemberExpression(it) => {
                matches!(other, Self::MemberExpression(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for JSXExpressionContainer<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expression.content_eq(&other.expression)
    }
}

impl<'a> ContentEq for JSXExpression<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::EmptyExpression(it) => {
                matches!(other, Self::EmptyExpression(other) if it.content_eq(other))
            }
            Self::BooleanLiteral(it) => {
                matches!(other, Self::BooleanLiteral(other) if it.content_eq(other))
            }
            Self::NullLiteral(it) => {
                matches!(other, Self::NullLiteral(other) if it.content_eq(other))
            }
            Self::NumericLiteral(it) => {
                matches!(other, Self::NumericLiteral(other) if it.content_eq(other))
            }
            Self::BigIntLiteral(it) => {
                matches!(other, Self::BigIntLiteral(other) if it.content_eq(other))
            }
            Self::RegExpLiteral(it) => {
                matches!(other, Self::RegExpLiteral(other) if it.content_eq(other))
            }
            Self::StringLiteral(it) => {
                matches!(other, Self::StringLiteral(other) if it.content_eq(other))
            }
            Self::TemplateLiteral(it) => {
                matches!(other, Self::TemplateLiteral(other) if it.content_eq(other))
            }
            Self::Identifier(it) => {
                matches!(other, Self::Identifier(other) if it.content_eq(other))
            }
            Self::MetaProperty(it) => {
                matches!(other, Self::MetaProperty(other) if it.content_eq(other))
            }
            Self::Super(it) => {
                matches!(other, Self::Super(other) if it.content_eq(other))
            }
            Self::ArrayExpression(it) => {
                matches!(other, Self::ArrayExpression(other) if it.content_eq(other))
            }
            Self::ArrowFunctionExpression(it) => {
                matches!(
                    other, Self::ArrowFunctionExpression(other) if it.content_eq(other)
                )
            }
            Self::AssignmentExpression(it) => {
                matches!(
                    other, Self::AssignmentExpression(other) if it.content_eq(other)
                )
            }
            Self::AwaitExpression(it) => {
                matches!(other, Self::AwaitExpression(other) if it.content_eq(other))
            }
            Self::BinaryExpression(it) => {
                matches!(other, Self::BinaryExpression(other) if it.content_eq(other))
            }
            Self::CallExpression(it) => {
                matches!(other, Self::CallExpression(other) if it.content_eq(other))
            }
            Self::ChainExpression(it) => {
                matches!(other, Self::ChainExpression(other) if it.content_eq(other))
            }
            Self::ClassExpression(it) => {
                matches!(other, Self::ClassExpression(other) if it.content_eq(other))
            }
            Self::ConditionalExpression(it) => {
                matches!(
                    other, Self::ConditionalExpression(other) if it.content_eq(other)
                )
            }
            Self::FunctionExpression(it) => {
                matches!(other, Self::FunctionExpression(other) if it.content_eq(other))
            }
            Self::ImportExpression(it) => {
                matches!(other, Self::ImportExpression(other) if it.content_eq(other))
            }
            Self::LogicalExpression(it) => {
                matches!(other, Self::LogicalExpression(other) if it.content_eq(other))
            }
            Self::NewExpression(it) => {
                matches!(other, Self::NewExpression(other) if it.content_eq(other))
            }
            Self::ObjectExpression(it) => {
                matches!(other, Self::ObjectExpression(other) if it.content_eq(other))
            }
            Self::ParenthesizedExpression(it) => {
                matches!(
                    other, Self::ParenthesizedExpression(other) if it.content_eq(other)
                )
            }
            Self::SequenceExpression(it) => {
                matches!(other, Self::SequenceExpression(other) if it.content_eq(other))
            }
            Self::TaggedTemplateExpression(it) => {
                matches!(
                    other, Self::TaggedTemplateExpression(other) if it.content_eq(other)
                )
            }
            Self::ThisExpression(it) => {
                matches!(other, Self::ThisExpression(other) if it.content_eq(other))
            }
            Self::UnaryExpression(it) => {
                matches!(other, Self::UnaryExpression(other) if it.content_eq(other))
            }
            Self::UpdateExpression(it) => {
                matches!(other, Self::UpdateExpression(other) if it.content_eq(other))
            }
            Self::YieldExpression(it) => {
                matches!(other, Self::YieldExpression(other) if it.content_eq(other))
            }
            Self::PrivateInExpression(it) => {
                matches!(other, Self::PrivateInExpression(other) if it.content_eq(other))
            }
            Self::JSXElement(it) => {
                matches!(other, Self::JSXElement(other) if it.content_eq(other))
            }
            Self::JSXFragment(it) => {
                matches!(other, Self::JSXFragment(other) if it.content_eq(other))
            }
            Self::TSAsExpression(it) => {
                matches!(other, Self::TSAsExpression(other) if it.content_eq(other))
            }
            Self::TSSatisfiesExpression(it) => {
                matches!(
                    other, Self::TSSatisfiesExpression(other) if it.content_eq(other)
                )
            }
            Self::TSTypeAssertion(it) => {
                matches!(other, Self::TSTypeAssertion(other) if it.content_eq(other))
            }
            Self::TSNonNullExpression(it) => {
                matches!(other, Self::TSNonNullExpression(other) if it.content_eq(other))
            }
            Self::TSInstantiationExpression(it) => {
                matches!(
                    other, Self::TSInstantiationExpression(other) if it.content_eq(other)
                )
            }
            Self::ComputedMemberExpression(it) => {
                matches!(
                    other, Self::ComputedMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::StaticMemberExpression(it) => {
                matches!(
                    other, Self::StaticMemberExpression(other) if it.content_eq(other)
                )
            }
            Self::PrivateFieldExpression(it) => {
                matches!(
                    other, Self::PrivateFieldExpression(other) if it.content_eq(other)
                )
            }
        }
    }
}

impl ContentEq for JSXEmptyExpression {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl<'a> ContentEq for JSXAttributeItem<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::Attribute(it) => {
                matches!(other, Self::Attribute(other) if it.content_eq(other))
            }
            Self::SpreadAttribute(it) => {
                matches!(other, Self::SpreadAttribute(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for JSXAttribute<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.name.content_eq(&other.name) && self.value.content_eq(&other.value)
    }
}

impl<'a> ContentEq for JSXSpreadAttribute<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.argument.content_eq(&other.argument)
    }
}

impl<'a> ContentEq for JSXAttributeName<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::Identifier(it) => {
                matches!(other, Self::Identifier(other) if it.content_eq(other))
            }
            Self::NamespacedName(it) => {
                matches!(other, Self::NamespacedName(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for JSXAttributeValue<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::StringLiteral(it) => {
                matches!(other, Self::StringLiteral(other) if it.content_eq(other))
            }
            Self::ExpressionContainer(it) => {
                matches!(other, Self::ExpressionContainer(other) if it.content_eq(other))
            }
            Self::Element(it) => {
                matches!(other, Self::Element(other) if it.content_eq(other))
            }
            Self::Fragment(it) => {
                matches!(other, Self::Fragment(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for JSXIdentifier<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.name.content_eq(&other.name)
    }
}

impl<'a> ContentEq for JSXChild<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        match self {
            Self::Text(it) => matches!(other, Self::Text(other) if it.content_eq(other)),
            Self::Element(it) => {
                matches!(other, Self::Element(other) if it.content_eq(other))
            }
            Self::Fragment(it) => {
                matches!(other, Self::Fragment(other) if it.content_eq(other))
            }
            Self::ExpressionContainer(it) => {
                matches!(other, Self::ExpressionContainer(other) if it.content_eq(other))
            }
            Self::Spread(it) => {
                matches!(other, Self::Spread(other) if it.content_eq(other))
            }
        }
    }
}

impl<'a> ContentEq for JSXSpreadChild<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.expression.content_eq(&other.expression)
    }
}

impl<'a> ContentEq for JSXText<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self.value.content_eq(&other.value)
    }
}
