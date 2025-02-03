// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/content_eq.rs`

#![allow(clippy::match_like_matches_macro)]

use oxc_span::cmp::ContentEq;

use crate::ast::comment::*;
use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
use crate::ast::ts::*;

impl ContentEq for BooleanLiteral {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.value, &other.value)
    }
}

impl ContentEq for NullLiteral {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for RegExp<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.pattern, &other.pattern)
            && ContentEq::content_eq(&self.flags, &other.flags)
    }
}

impl ContentEq for Program<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.source_type, &other.source_type)
            && ContentEq::content_eq(&self.source_text, &other.source_text)
            && ContentEq::content_eq(&self.comments, &other.comments)
            && ContentEq::content_eq(&self.hashbang, &other.hashbang)
            && ContentEq::content_eq(&self.directives, &other.directives)
            && ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for Expression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::BooleanLiteral(a), Self::BooleanLiteral(b)) => a.content_eq(b),
            (Self::NullLiteral(a), Self::NullLiteral(b)) => a.content_eq(b),
            (Self::NumericLiteral(a), Self::NumericLiteral(b)) => a.content_eq(b),
            (Self::BigIntLiteral(a), Self::BigIntLiteral(b)) => a.content_eq(b),
            (Self::RegExpLiteral(a), Self::RegExpLiteral(b)) => a.content_eq(b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => a.content_eq(b),
            (Self::TemplateLiteral(a), Self::TemplateLiteral(b)) => a.content_eq(b),
            (Self::Identifier(a), Self::Identifier(b)) => a.content_eq(b),
            (Self::MetaProperty(a), Self::MetaProperty(b)) => a.content_eq(b),
            (Self::Super(a), Self::Super(b)) => a.content_eq(b),
            (Self::ArrayExpression(a), Self::ArrayExpression(b)) => a.content_eq(b),
            (Self::ArrowFunctionExpression(a), Self::ArrowFunctionExpression(b)) => a.content_eq(b),
            (Self::AssignmentExpression(a), Self::AssignmentExpression(b)) => a.content_eq(b),
            (Self::AwaitExpression(a), Self::AwaitExpression(b)) => a.content_eq(b),
            (Self::BinaryExpression(a), Self::BinaryExpression(b)) => a.content_eq(b),
            (Self::CallExpression(a), Self::CallExpression(b)) => a.content_eq(b),
            (Self::ChainExpression(a), Self::ChainExpression(b)) => a.content_eq(b),
            (Self::ClassExpression(a), Self::ClassExpression(b)) => a.content_eq(b),
            (Self::ConditionalExpression(a), Self::ConditionalExpression(b)) => a.content_eq(b),
            (Self::FunctionExpression(a), Self::FunctionExpression(b)) => a.content_eq(b),
            (Self::ImportExpression(a), Self::ImportExpression(b)) => a.content_eq(b),
            (Self::LogicalExpression(a), Self::LogicalExpression(b)) => a.content_eq(b),
            (Self::NewExpression(a), Self::NewExpression(b)) => a.content_eq(b),
            (Self::ObjectExpression(a), Self::ObjectExpression(b)) => a.content_eq(b),
            (Self::ParenthesizedExpression(a), Self::ParenthesizedExpression(b)) => a.content_eq(b),
            (Self::SequenceExpression(a), Self::SequenceExpression(b)) => a.content_eq(b),
            (Self::TaggedTemplateExpression(a), Self::TaggedTemplateExpression(b)) => {
                a.content_eq(b)
            }
            (Self::ThisExpression(a), Self::ThisExpression(b)) => a.content_eq(b),
            (Self::UnaryExpression(a), Self::UnaryExpression(b)) => a.content_eq(b),
            (Self::UpdateExpression(a), Self::UpdateExpression(b)) => a.content_eq(b),
            (Self::YieldExpression(a), Self::YieldExpression(b)) => a.content_eq(b),
            (Self::PrivateInExpression(a), Self::PrivateInExpression(b)) => a.content_eq(b),
            (Self::JSXElement(a), Self::JSXElement(b)) => a.content_eq(b),
            (Self::JSXFragment(a), Self::JSXFragment(b)) => a.content_eq(b),
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => a.content_eq(b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => a.content_eq(b),
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => a.content_eq(b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => a.content_eq(b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                a.content_eq(b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                a.content_eq(b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => a.content_eq(b),
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for IdentifierName<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.name, &other.name)
    }
}

impl ContentEq for IdentifierReference<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.name, &other.name)
    }
}

impl ContentEq for BindingIdentifier<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.name, &other.name)
    }
}

impl ContentEq for LabelIdentifier<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.name, &other.name)
    }
}

impl ContentEq for ThisExpression {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for ArrayExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.elements, &other.elements)
    }
}

impl ContentEq for ArrayExpressionElement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::SpreadElement(a), Self::SpreadElement(b)) => a.content_eq(b),
            (Self::Elision(a), Self::Elision(b)) => a.content_eq(b),
            (Self::BooleanLiteral(a), Self::BooleanLiteral(b)) => a.content_eq(b),
            (Self::NullLiteral(a), Self::NullLiteral(b)) => a.content_eq(b),
            (Self::NumericLiteral(a), Self::NumericLiteral(b)) => a.content_eq(b),
            (Self::BigIntLiteral(a), Self::BigIntLiteral(b)) => a.content_eq(b),
            (Self::RegExpLiteral(a), Self::RegExpLiteral(b)) => a.content_eq(b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => a.content_eq(b),
            (Self::TemplateLiteral(a), Self::TemplateLiteral(b)) => a.content_eq(b),
            (Self::Identifier(a), Self::Identifier(b)) => a.content_eq(b),
            (Self::MetaProperty(a), Self::MetaProperty(b)) => a.content_eq(b),
            (Self::Super(a), Self::Super(b)) => a.content_eq(b),
            (Self::ArrayExpression(a), Self::ArrayExpression(b)) => a.content_eq(b),
            (Self::ArrowFunctionExpression(a), Self::ArrowFunctionExpression(b)) => a.content_eq(b),
            (Self::AssignmentExpression(a), Self::AssignmentExpression(b)) => a.content_eq(b),
            (Self::AwaitExpression(a), Self::AwaitExpression(b)) => a.content_eq(b),
            (Self::BinaryExpression(a), Self::BinaryExpression(b)) => a.content_eq(b),
            (Self::CallExpression(a), Self::CallExpression(b)) => a.content_eq(b),
            (Self::ChainExpression(a), Self::ChainExpression(b)) => a.content_eq(b),
            (Self::ClassExpression(a), Self::ClassExpression(b)) => a.content_eq(b),
            (Self::ConditionalExpression(a), Self::ConditionalExpression(b)) => a.content_eq(b),
            (Self::FunctionExpression(a), Self::FunctionExpression(b)) => a.content_eq(b),
            (Self::ImportExpression(a), Self::ImportExpression(b)) => a.content_eq(b),
            (Self::LogicalExpression(a), Self::LogicalExpression(b)) => a.content_eq(b),
            (Self::NewExpression(a), Self::NewExpression(b)) => a.content_eq(b),
            (Self::ObjectExpression(a), Self::ObjectExpression(b)) => a.content_eq(b),
            (Self::ParenthesizedExpression(a), Self::ParenthesizedExpression(b)) => a.content_eq(b),
            (Self::SequenceExpression(a), Self::SequenceExpression(b)) => a.content_eq(b),
            (Self::TaggedTemplateExpression(a), Self::TaggedTemplateExpression(b)) => {
                a.content_eq(b)
            }
            (Self::ThisExpression(a), Self::ThisExpression(b)) => a.content_eq(b),
            (Self::UnaryExpression(a), Self::UnaryExpression(b)) => a.content_eq(b),
            (Self::UpdateExpression(a), Self::UpdateExpression(b)) => a.content_eq(b),
            (Self::YieldExpression(a), Self::YieldExpression(b)) => a.content_eq(b),
            (Self::PrivateInExpression(a), Self::PrivateInExpression(b)) => a.content_eq(b),
            (Self::JSXElement(a), Self::JSXElement(b)) => a.content_eq(b),
            (Self::JSXFragment(a), Self::JSXFragment(b)) => a.content_eq(b),
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => a.content_eq(b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => a.content_eq(b),
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => a.content_eq(b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => a.content_eq(b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                a.content_eq(b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                a.content_eq(b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => a.content_eq(b),
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for Elision {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for ObjectExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.properties, &other.properties)
    }
}

impl ContentEq for ObjectPropertyKind<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::ObjectProperty(a), Self::ObjectProperty(b)) => a.content_eq(b),
            (Self::SpreadProperty(a), Self::SpreadProperty(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for ObjectProperty<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.kind, &other.kind)
            && ContentEq::content_eq(&self.key, &other.key)
            && ContentEq::content_eq(&self.value, &other.value)
            && ContentEq::content_eq(&self.method, &other.method)
            && ContentEq::content_eq(&self.shorthand, &other.shorthand)
            && ContentEq::content_eq(&self.computed, &other.computed)
    }
}

impl ContentEq for PropertyKey<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::StaticIdentifier(a), Self::StaticIdentifier(b)) => a.content_eq(b),
            (Self::PrivateIdentifier(a), Self::PrivateIdentifier(b)) => a.content_eq(b),
            (Self::BooleanLiteral(a), Self::BooleanLiteral(b)) => a.content_eq(b),
            (Self::NullLiteral(a), Self::NullLiteral(b)) => a.content_eq(b),
            (Self::NumericLiteral(a), Self::NumericLiteral(b)) => a.content_eq(b),
            (Self::BigIntLiteral(a), Self::BigIntLiteral(b)) => a.content_eq(b),
            (Self::RegExpLiteral(a), Self::RegExpLiteral(b)) => a.content_eq(b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => a.content_eq(b),
            (Self::TemplateLiteral(a), Self::TemplateLiteral(b)) => a.content_eq(b),
            (Self::Identifier(a), Self::Identifier(b)) => a.content_eq(b),
            (Self::MetaProperty(a), Self::MetaProperty(b)) => a.content_eq(b),
            (Self::Super(a), Self::Super(b)) => a.content_eq(b),
            (Self::ArrayExpression(a), Self::ArrayExpression(b)) => a.content_eq(b),
            (Self::ArrowFunctionExpression(a), Self::ArrowFunctionExpression(b)) => a.content_eq(b),
            (Self::AssignmentExpression(a), Self::AssignmentExpression(b)) => a.content_eq(b),
            (Self::AwaitExpression(a), Self::AwaitExpression(b)) => a.content_eq(b),
            (Self::BinaryExpression(a), Self::BinaryExpression(b)) => a.content_eq(b),
            (Self::CallExpression(a), Self::CallExpression(b)) => a.content_eq(b),
            (Self::ChainExpression(a), Self::ChainExpression(b)) => a.content_eq(b),
            (Self::ClassExpression(a), Self::ClassExpression(b)) => a.content_eq(b),
            (Self::ConditionalExpression(a), Self::ConditionalExpression(b)) => a.content_eq(b),
            (Self::FunctionExpression(a), Self::FunctionExpression(b)) => a.content_eq(b),
            (Self::ImportExpression(a), Self::ImportExpression(b)) => a.content_eq(b),
            (Self::LogicalExpression(a), Self::LogicalExpression(b)) => a.content_eq(b),
            (Self::NewExpression(a), Self::NewExpression(b)) => a.content_eq(b),
            (Self::ObjectExpression(a), Self::ObjectExpression(b)) => a.content_eq(b),
            (Self::ParenthesizedExpression(a), Self::ParenthesizedExpression(b)) => a.content_eq(b),
            (Self::SequenceExpression(a), Self::SequenceExpression(b)) => a.content_eq(b),
            (Self::TaggedTemplateExpression(a), Self::TaggedTemplateExpression(b)) => {
                a.content_eq(b)
            }
            (Self::ThisExpression(a), Self::ThisExpression(b)) => a.content_eq(b),
            (Self::UnaryExpression(a), Self::UnaryExpression(b)) => a.content_eq(b),
            (Self::UpdateExpression(a), Self::UpdateExpression(b)) => a.content_eq(b),
            (Self::YieldExpression(a), Self::YieldExpression(b)) => a.content_eq(b),
            (Self::PrivateInExpression(a), Self::PrivateInExpression(b)) => a.content_eq(b),
            (Self::JSXElement(a), Self::JSXElement(b)) => a.content_eq(b),
            (Self::JSXFragment(a), Self::JSXFragment(b)) => a.content_eq(b),
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => a.content_eq(b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => a.content_eq(b),
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => a.content_eq(b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => a.content_eq(b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                a.content_eq(b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                a.content_eq(b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => a.content_eq(b),
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for PropertyKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for TemplateLiteral<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.quasis, &other.quasis)
            && ContentEq::content_eq(&self.expressions, &other.expressions)
    }
}

impl ContentEq for TaggedTemplateExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.tag, &other.tag)
            && ContentEq::content_eq(&self.quasi, &other.quasi)
            && ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
    }
}

impl ContentEq for TemplateElement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.tail, &other.tail)
            && ContentEq::content_eq(&self.value, &other.value)
    }
}

impl ContentEq for TemplateElementValue<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.raw, &other.raw)
            && ContentEq::content_eq(&self.cooked, &other.cooked)
    }
}

impl ContentEq for MemberExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                a.content_eq(b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => a.content_eq(b),
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for ComputedMemberExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.object, &other.object)
            && ContentEq::content_eq(&self.expression, &other.expression)
            && ContentEq::content_eq(&self.optional, &other.optional)
    }
}

impl ContentEq for StaticMemberExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.object, &other.object)
            && ContentEq::content_eq(&self.property, &other.property)
            && ContentEq::content_eq(&self.optional, &other.optional)
    }
}

impl ContentEq for PrivateFieldExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.object, &other.object)
            && ContentEq::content_eq(&self.field, &other.field)
            && ContentEq::content_eq(&self.optional, &other.optional)
    }
}

impl ContentEq for CallExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.callee, &other.callee)
            && ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
            && ContentEq::content_eq(&self.arguments, &other.arguments)
            && ContentEq::content_eq(&self.optional, &other.optional)
    }
}

impl ContentEq for NewExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.callee, &other.callee)
            && ContentEq::content_eq(&self.arguments, &other.arguments)
            && ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
    }
}

impl ContentEq for MetaProperty<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.meta, &other.meta)
            && ContentEq::content_eq(&self.property, &other.property)
    }
}

impl ContentEq for SpreadElement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.argument, &other.argument)
    }
}

impl ContentEq for Argument<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::SpreadElement(a), Self::SpreadElement(b)) => a.content_eq(b),
            (Self::BooleanLiteral(a), Self::BooleanLiteral(b)) => a.content_eq(b),
            (Self::NullLiteral(a), Self::NullLiteral(b)) => a.content_eq(b),
            (Self::NumericLiteral(a), Self::NumericLiteral(b)) => a.content_eq(b),
            (Self::BigIntLiteral(a), Self::BigIntLiteral(b)) => a.content_eq(b),
            (Self::RegExpLiteral(a), Self::RegExpLiteral(b)) => a.content_eq(b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => a.content_eq(b),
            (Self::TemplateLiteral(a), Self::TemplateLiteral(b)) => a.content_eq(b),
            (Self::Identifier(a), Self::Identifier(b)) => a.content_eq(b),
            (Self::MetaProperty(a), Self::MetaProperty(b)) => a.content_eq(b),
            (Self::Super(a), Self::Super(b)) => a.content_eq(b),
            (Self::ArrayExpression(a), Self::ArrayExpression(b)) => a.content_eq(b),
            (Self::ArrowFunctionExpression(a), Self::ArrowFunctionExpression(b)) => a.content_eq(b),
            (Self::AssignmentExpression(a), Self::AssignmentExpression(b)) => a.content_eq(b),
            (Self::AwaitExpression(a), Self::AwaitExpression(b)) => a.content_eq(b),
            (Self::BinaryExpression(a), Self::BinaryExpression(b)) => a.content_eq(b),
            (Self::CallExpression(a), Self::CallExpression(b)) => a.content_eq(b),
            (Self::ChainExpression(a), Self::ChainExpression(b)) => a.content_eq(b),
            (Self::ClassExpression(a), Self::ClassExpression(b)) => a.content_eq(b),
            (Self::ConditionalExpression(a), Self::ConditionalExpression(b)) => a.content_eq(b),
            (Self::FunctionExpression(a), Self::FunctionExpression(b)) => a.content_eq(b),
            (Self::ImportExpression(a), Self::ImportExpression(b)) => a.content_eq(b),
            (Self::LogicalExpression(a), Self::LogicalExpression(b)) => a.content_eq(b),
            (Self::NewExpression(a), Self::NewExpression(b)) => a.content_eq(b),
            (Self::ObjectExpression(a), Self::ObjectExpression(b)) => a.content_eq(b),
            (Self::ParenthesizedExpression(a), Self::ParenthesizedExpression(b)) => a.content_eq(b),
            (Self::SequenceExpression(a), Self::SequenceExpression(b)) => a.content_eq(b),
            (Self::TaggedTemplateExpression(a), Self::TaggedTemplateExpression(b)) => {
                a.content_eq(b)
            }
            (Self::ThisExpression(a), Self::ThisExpression(b)) => a.content_eq(b),
            (Self::UnaryExpression(a), Self::UnaryExpression(b)) => a.content_eq(b),
            (Self::UpdateExpression(a), Self::UpdateExpression(b)) => a.content_eq(b),
            (Self::YieldExpression(a), Self::YieldExpression(b)) => a.content_eq(b),
            (Self::PrivateInExpression(a), Self::PrivateInExpression(b)) => a.content_eq(b),
            (Self::JSXElement(a), Self::JSXElement(b)) => a.content_eq(b),
            (Self::JSXFragment(a), Self::JSXFragment(b)) => a.content_eq(b),
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => a.content_eq(b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => a.content_eq(b),
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => a.content_eq(b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => a.content_eq(b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                a.content_eq(b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                a.content_eq(b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => a.content_eq(b),
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for UpdateExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.operator, &other.operator)
            && ContentEq::content_eq(&self.prefix, &other.prefix)
            && ContentEq::content_eq(&self.argument, &other.argument)
    }
}

impl ContentEq for UnaryExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.operator, &other.operator)
            && ContentEq::content_eq(&self.argument, &other.argument)
    }
}

impl ContentEq for BinaryExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.left, &other.left)
            && ContentEq::content_eq(&self.operator, &other.operator)
            && ContentEq::content_eq(&self.right, &other.right)
    }
}

impl ContentEq for PrivateInExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.left, &other.left)
            && ContentEq::content_eq(&self.operator, &other.operator)
            && ContentEq::content_eq(&self.right, &other.right)
    }
}

impl ContentEq for LogicalExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.left, &other.left)
            && ContentEq::content_eq(&self.operator, &other.operator)
            && ContentEq::content_eq(&self.right, &other.right)
    }
}

impl ContentEq for ConditionalExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.test, &other.test)
            && ContentEq::content_eq(&self.consequent, &other.consequent)
            && ContentEq::content_eq(&self.alternate, &other.alternate)
    }
}

impl ContentEq for AssignmentExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.operator, &other.operator)
            && ContentEq::content_eq(&self.left, &other.left)
            && ContentEq::content_eq(&self.right, &other.right)
    }
}

impl ContentEq for AssignmentTarget<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::AssignmentTargetIdentifier(a), Self::AssignmentTargetIdentifier(b)) => {
                a.content_eq(b)
            }
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => a.content_eq(b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => a.content_eq(b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => a.content_eq(b),
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => a.content_eq(b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                a.content_eq(b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                a.content_eq(b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => a.content_eq(b),
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => a.content_eq(b),
            (Self::ArrayAssignmentTarget(a), Self::ArrayAssignmentTarget(b)) => a.content_eq(b),
            (Self::ObjectAssignmentTarget(a), Self::ObjectAssignmentTarget(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for SimpleAssignmentTarget<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::AssignmentTargetIdentifier(a), Self::AssignmentTargetIdentifier(b)) => {
                a.content_eq(b)
            }
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => a.content_eq(b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => a.content_eq(b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => a.content_eq(b),
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => a.content_eq(b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                a.content_eq(b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                a.content_eq(b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => a.content_eq(b),
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for AssignmentTargetPattern<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::ArrayAssignmentTarget(a), Self::ArrayAssignmentTarget(b)) => a.content_eq(b),
            (Self::ObjectAssignmentTarget(a), Self::ObjectAssignmentTarget(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for ArrayAssignmentTarget<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.elements, &other.elements)
            && ContentEq::content_eq(&self.rest, &other.rest)
    }
}

impl ContentEq for ObjectAssignmentTarget<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.properties, &other.properties)
            && ContentEq::content_eq(&self.rest, &other.rest)
    }
}

impl ContentEq for AssignmentTargetRest<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.target, &other.target)
    }
}

impl ContentEq for AssignmentTargetMaybeDefault<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::AssignmentTargetWithDefault(a), Self::AssignmentTargetWithDefault(b)) => {
                a.content_eq(b)
            }
            (Self::AssignmentTargetIdentifier(a), Self::AssignmentTargetIdentifier(b)) => {
                a.content_eq(b)
            }
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => a.content_eq(b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => a.content_eq(b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => a.content_eq(b),
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => a.content_eq(b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                a.content_eq(b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                a.content_eq(b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => a.content_eq(b),
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => a.content_eq(b),
            (Self::ArrayAssignmentTarget(a), Self::ArrayAssignmentTarget(b)) => a.content_eq(b),
            (Self::ObjectAssignmentTarget(a), Self::ObjectAssignmentTarget(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for AssignmentTargetWithDefault<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.binding, &other.binding)
            && ContentEq::content_eq(&self.init, &other.init)
    }
}

impl ContentEq for AssignmentTargetProperty<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (
                Self::AssignmentTargetPropertyIdentifier(a),
                Self::AssignmentTargetPropertyIdentifier(b),
            ) => a.content_eq(b),
            (
                Self::AssignmentTargetPropertyProperty(a),
                Self::AssignmentTargetPropertyProperty(b),
            ) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for AssignmentTargetPropertyIdentifier<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.binding, &other.binding)
            && ContentEq::content_eq(&self.init, &other.init)
    }
}

impl ContentEq for AssignmentTargetPropertyProperty<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.name, &other.name)
            && ContentEq::content_eq(&self.binding, &other.binding)
            && ContentEq::content_eq(&self.computed, &other.computed)
    }
}

impl ContentEq for SequenceExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expressions, &other.expressions)
    }
}

impl ContentEq for Super {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for AwaitExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.argument, &other.argument)
    }
}

impl ContentEq for ChainExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expression, &other.expression)
    }
}

impl ContentEq for ChainElement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::CallExpression(a), Self::CallExpression(b)) => a.content_eq(b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => a.content_eq(b),
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                a.content_eq(b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => a.content_eq(b),
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for ParenthesizedExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expression, &other.expression)
    }
}

impl ContentEq for Statement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::BlockStatement(a), Self::BlockStatement(b)) => a.content_eq(b),
            (Self::BreakStatement(a), Self::BreakStatement(b)) => a.content_eq(b),
            (Self::ContinueStatement(a), Self::ContinueStatement(b)) => a.content_eq(b),
            (Self::DebuggerStatement(a), Self::DebuggerStatement(b)) => a.content_eq(b),
            (Self::DoWhileStatement(a), Self::DoWhileStatement(b)) => a.content_eq(b),
            (Self::EmptyStatement(a), Self::EmptyStatement(b)) => a.content_eq(b),
            (Self::ExpressionStatement(a), Self::ExpressionStatement(b)) => a.content_eq(b),
            (Self::ForInStatement(a), Self::ForInStatement(b)) => a.content_eq(b),
            (Self::ForOfStatement(a), Self::ForOfStatement(b)) => a.content_eq(b),
            (Self::ForStatement(a), Self::ForStatement(b)) => a.content_eq(b),
            (Self::IfStatement(a), Self::IfStatement(b)) => a.content_eq(b),
            (Self::LabeledStatement(a), Self::LabeledStatement(b)) => a.content_eq(b),
            (Self::ReturnStatement(a), Self::ReturnStatement(b)) => a.content_eq(b),
            (Self::SwitchStatement(a), Self::SwitchStatement(b)) => a.content_eq(b),
            (Self::ThrowStatement(a), Self::ThrowStatement(b)) => a.content_eq(b),
            (Self::TryStatement(a), Self::TryStatement(b)) => a.content_eq(b),
            (Self::WhileStatement(a), Self::WhileStatement(b)) => a.content_eq(b),
            (Self::WithStatement(a), Self::WithStatement(b)) => a.content_eq(b),
            (Self::VariableDeclaration(a), Self::VariableDeclaration(b)) => a.content_eq(b),
            (Self::FunctionDeclaration(a), Self::FunctionDeclaration(b)) => a.content_eq(b),
            (Self::ClassDeclaration(a), Self::ClassDeclaration(b)) => a.content_eq(b),
            (Self::TSTypeAliasDeclaration(a), Self::TSTypeAliasDeclaration(b)) => a.content_eq(b),
            (Self::TSInterfaceDeclaration(a), Self::TSInterfaceDeclaration(b)) => a.content_eq(b),
            (Self::TSEnumDeclaration(a), Self::TSEnumDeclaration(b)) => a.content_eq(b),
            (Self::TSModuleDeclaration(a), Self::TSModuleDeclaration(b)) => a.content_eq(b),
            (Self::TSImportEqualsDeclaration(a), Self::TSImportEqualsDeclaration(b)) => {
                a.content_eq(b)
            }
            (Self::ImportDeclaration(a), Self::ImportDeclaration(b)) => a.content_eq(b),
            (Self::ExportAllDeclaration(a), Self::ExportAllDeclaration(b)) => a.content_eq(b),
            (Self::ExportDefaultDeclaration(a), Self::ExportDefaultDeclaration(b)) => {
                a.content_eq(b)
            }
            (Self::ExportNamedDeclaration(a), Self::ExportNamedDeclaration(b)) => a.content_eq(b),
            (Self::TSExportAssignment(a), Self::TSExportAssignment(b)) => a.content_eq(b),
            (Self::TSNamespaceExportDeclaration(a), Self::TSNamespaceExportDeclaration(b)) => {
                a.content_eq(b)
            }
            _ => false,
        }
    }
}

impl ContentEq for Directive<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expression, &other.expression)
            && ContentEq::content_eq(&self.directive, &other.directive)
    }
}

impl ContentEq for Hashbang<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.value, &other.value)
    }
}

impl ContentEq for BlockStatement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for Declaration<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::VariableDeclaration(a), Self::VariableDeclaration(b)) => a.content_eq(b),
            (Self::FunctionDeclaration(a), Self::FunctionDeclaration(b)) => a.content_eq(b),
            (Self::ClassDeclaration(a), Self::ClassDeclaration(b)) => a.content_eq(b),
            (Self::TSTypeAliasDeclaration(a), Self::TSTypeAliasDeclaration(b)) => a.content_eq(b),
            (Self::TSInterfaceDeclaration(a), Self::TSInterfaceDeclaration(b)) => a.content_eq(b),
            (Self::TSEnumDeclaration(a), Self::TSEnumDeclaration(b)) => a.content_eq(b),
            (Self::TSModuleDeclaration(a), Self::TSModuleDeclaration(b)) => a.content_eq(b),
            (Self::TSImportEqualsDeclaration(a), Self::TSImportEqualsDeclaration(b)) => {
                a.content_eq(b)
            }
            _ => false,
        }
    }
}

impl ContentEq for VariableDeclaration<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.kind, &other.kind)
            && ContentEq::content_eq(&self.declarations, &other.declarations)
            && ContentEq::content_eq(&self.declare, &other.declare)
    }
}

impl ContentEq for VariableDeclarationKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for VariableDeclarator<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.kind, &other.kind)
            && ContentEq::content_eq(&self.id, &other.id)
            && ContentEq::content_eq(&self.init, &other.init)
            && ContentEq::content_eq(&self.definite, &other.definite)
    }
}

impl ContentEq for EmptyStatement {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for ExpressionStatement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expression, &other.expression)
    }
}

impl ContentEq for IfStatement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.test, &other.test)
            && ContentEq::content_eq(&self.consequent, &other.consequent)
            && ContentEq::content_eq(&self.alternate, &other.alternate)
    }
}

impl ContentEq for DoWhileStatement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.body, &other.body)
            && ContentEq::content_eq(&self.test, &other.test)
    }
}

impl ContentEq for WhileStatement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.test, &other.test)
            && ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for ForStatement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.init, &other.init)
            && ContentEq::content_eq(&self.test, &other.test)
            && ContentEq::content_eq(&self.update, &other.update)
            && ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for ForStatementInit<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::VariableDeclaration(a), Self::VariableDeclaration(b)) => a.content_eq(b),
            (Self::BooleanLiteral(a), Self::BooleanLiteral(b)) => a.content_eq(b),
            (Self::NullLiteral(a), Self::NullLiteral(b)) => a.content_eq(b),
            (Self::NumericLiteral(a), Self::NumericLiteral(b)) => a.content_eq(b),
            (Self::BigIntLiteral(a), Self::BigIntLiteral(b)) => a.content_eq(b),
            (Self::RegExpLiteral(a), Self::RegExpLiteral(b)) => a.content_eq(b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => a.content_eq(b),
            (Self::TemplateLiteral(a), Self::TemplateLiteral(b)) => a.content_eq(b),
            (Self::Identifier(a), Self::Identifier(b)) => a.content_eq(b),
            (Self::MetaProperty(a), Self::MetaProperty(b)) => a.content_eq(b),
            (Self::Super(a), Self::Super(b)) => a.content_eq(b),
            (Self::ArrayExpression(a), Self::ArrayExpression(b)) => a.content_eq(b),
            (Self::ArrowFunctionExpression(a), Self::ArrowFunctionExpression(b)) => a.content_eq(b),
            (Self::AssignmentExpression(a), Self::AssignmentExpression(b)) => a.content_eq(b),
            (Self::AwaitExpression(a), Self::AwaitExpression(b)) => a.content_eq(b),
            (Self::BinaryExpression(a), Self::BinaryExpression(b)) => a.content_eq(b),
            (Self::CallExpression(a), Self::CallExpression(b)) => a.content_eq(b),
            (Self::ChainExpression(a), Self::ChainExpression(b)) => a.content_eq(b),
            (Self::ClassExpression(a), Self::ClassExpression(b)) => a.content_eq(b),
            (Self::ConditionalExpression(a), Self::ConditionalExpression(b)) => a.content_eq(b),
            (Self::FunctionExpression(a), Self::FunctionExpression(b)) => a.content_eq(b),
            (Self::ImportExpression(a), Self::ImportExpression(b)) => a.content_eq(b),
            (Self::LogicalExpression(a), Self::LogicalExpression(b)) => a.content_eq(b),
            (Self::NewExpression(a), Self::NewExpression(b)) => a.content_eq(b),
            (Self::ObjectExpression(a), Self::ObjectExpression(b)) => a.content_eq(b),
            (Self::ParenthesizedExpression(a), Self::ParenthesizedExpression(b)) => a.content_eq(b),
            (Self::SequenceExpression(a), Self::SequenceExpression(b)) => a.content_eq(b),
            (Self::TaggedTemplateExpression(a), Self::TaggedTemplateExpression(b)) => {
                a.content_eq(b)
            }
            (Self::ThisExpression(a), Self::ThisExpression(b)) => a.content_eq(b),
            (Self::UnaryExpression(a), Self::UnaryExpression(b)) => a.content_eq(b),
            (Self::UpdateExpression(a), Self::UpdateExpression(b)) => a.content_eq(b),
            (Self::YieldExpression(a), Self::YieldExpression(b)) => a.content_eq(b),
            (Self::PrivateInExpression(a), Self::PrivateInExpression(b)) => a.content_eq(b),
            (Self::JSXElement(a), Self::JSXElement(b)) => a.content_eq(b),
            (Self::JSXFragment(a), Self::JSXFragment(b)) => a.content_eq(b),
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => a.content_eq(b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => a.content_eq(b),
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => a.content_eq(b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => a.content_eq(b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                a.content_eq(b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                a.content_eq(b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => a.content_eq(b),
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for ForInStatement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.left, &other.left)
            && ContentEq::content_eq(&self.right, &other.right)
            && ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for ForStatementLeft<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::VariableDeclaration(a), Self::VariableDeclaration(b)) => a.content_eq(b),
            (Self::AssignmentTargetIdentifier(a), Self::AssignmentTargetIdentifier(b)) => {
                a.content_eq(b)
            }
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => a.content_eq(b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => a.content_eq(b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => a.content_eq(b),
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => a.content_eq(b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                a.content_eq(b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                a.content_eq(b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => a.content_eq(b),
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => a.content_eq(b),
            (Self::ArrayAssignmentTarget(a), Self::ArrayAssignmentTarget(b)) => a.content_eq(b),
            (Self::ObjectAssignmentTarget(a), Self::ObjectAssignmentTarget(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for ForOfStatement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.r#await, &other.r#await)
            && ContentEq::content_eq(&self.left, &other.left)
            && ContentEq::content_eq(&self.right, &other.right)
            && ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for ContinueStatement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.label, &other.label)
    }
}

impl ContentEq for BreakStatement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.label, &other.label)
    }
}

impl ContentEq for ReturnStatement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.argument, &other.argument)
    }
}

impl ContentEq for WithStatement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.object, &other.object)
            && ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for SwitchStatement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.discriminant, &other.discriminant)
            && ContentEq::content_eq(&self.cases, &other.cases)
    }
}

impl ContentEq for SwitchCase<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.test, &other.test)
            && ContentEq::content_eq(&self.consequent, &other.consequent)
    }
}

impl ContentEq for LabeledStatement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.label, &other.label)
            && ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for ThrowStatement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.argument, &other.argument)
    }
}

impl ContentEq for TryStatement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.block, &other.block)
            && ContentEq::content_eq(&self.handler, &other.handler)
            && ContentEq::content_eq(&self.finalizer, &other.finalizer)
    }
}

impl ContentEq for CatchClause<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.param, &other.param)
            && ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for CatchParameter<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.pattern, &other.pattern)
    }
}

impl ContentEq for DebuggerStatement {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for BindingPattern<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.kind, &other.kind)
            && ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
            && ContentEq::content_eq(&self.optional, &other.optional)
    }
}

impl ContentEq for BindingPatternKind<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::BindingIdentifier(a), Self::BindingIdentifier(b)) => a.content_eq(b),
            (Self::ObjectPattern(a), Self::ObjectPattern(b)) => a.content_eq(b),
            (Self::ArrayPattern(a), Self::ArrayPattern(b)) => a.content_eq(b),
            (Self::AssignmentPattern(a), Self::AssignmentPattern(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for AssignmentPattern<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.left, &other.left)
            && ContentEq::content_eq(&self.right, &other.right)
    }
}

impl ContentEq for ObjectPattern<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.properties, &other.properties)
            && ContentEq::content_eq(&self.rest, &other.rest)
    }
}

impl ContentEq for BindingProperty<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.key, &other.key)
            && ContentEq::content_eq(&self.value, &other.value)
            && ContentEq::content_eq(&self.shorthand, &other.shorthand)
            && ContentEq::content_eq(&self.computed, &other.computed)
    }
}

impl ContentEq for ArrayPattern<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.elements, &other.elements)
            && ContentEq::content_eq(&self.rest, &other.rest)
    }
}

impl ContentEq for BindingRestElement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.argument, &other.argument)
    }
}

impl ContentEq for Function<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.r#type, &other.r#type)
            && ContentEq::content_eq(&self.id, &other.id)
            && ContentEq::content_eq(&self.generator, &other.generator)
            && ContentEq::content_eq(&self.r#async, &other.r#async)
            && ContentEq::content_eq(&self.declare, &other.declare)
            && ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
            && ContentEq::content_eq(&self.this_param, &other.this_param)
            && ContentEq::content_eq(&self.params, &other.params)
            && ContentEq::content_eq(&self.return_type, &other.return_type)
            && ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for FunctionType {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for FormalParameters<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.kind, &other.kind)
            && ContentEq::content_eq(&self.items, &other.items)
            && ContentEq::content_eq(&self.rest, &other.rest)
    }
}

impl ContentEq for FormalParameter<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.decorators, &other.decorators)
            && ContentEq::content_eq(&self.pattern, &other.pattern)
            && ContentEq::content_eq(&self.accessibility, &other.accessibility)
            && ContentEq::content_eq(&self.readonly, &other.readonly)
            && ContentEq::content_eq(&self.r#override, &other.r#override)
    }
}

impl ContentEq for FormalParameterKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for FunctionBody<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.directives, &other.directives)
            && ContentEq::content_eq(&self.statements, &other.statements)
    }
}

impl ContentEq for ArrowFunctionExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expression, &other.expression)
            && ContentEq::content_eq(&self.r#async, &other.r#async)
            && ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
            && ContentEq::content_eq(&self.params, &other.params)
            && ContentEq::content_eq(&self.return_type, &other.return_type)
            && ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for YieldExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.delegate, &other.delegate)
            && ContentEq::content_eq(&self.argument, &other.argument)
    }
}

impl ContentEq for Class<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.r#type, &other.r#type)
            && ContentEq::content_eq(&self.decorators, &other.decorators)
            && ContentEq::content_eq(&self.id, &other.id)
            && ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
            && ContentEq::content_eq(&self.super_class, &other.super_class)
            && ContentEq::content_eq(&self.super_type_parameters, &other.super_type_parameters)
            && ContentEq::content_eq(&self.implements, &other.implements)
            && ContentEq::content_eq(&self.body, &other.body)
            && ContentEq::content_eq(&self.r#abstract, &other.r#abstract)
            && ContentEq::content_eq(&self.declare, &other.declare)
    }
}

impl ContentEq for ClassType {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for ClassBody<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for ClassElement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::StaticBlock(a), Self::StaticBlock(b)) => a.content_eq(b),
            (Self::MethodDefinition(a), Self::MethodDefinition(b)) => a.content_eq(b),
            (Self::PropertyDefinition(a), Self::PropertyDefinition(b)) => a.content_eq(b),
            (Self::AccessorProperty(a), Self::AccessorProperty(b)) => a.content_eq(b),
            (Self::TSIndexSignature(a), Self::TSIndexSignature(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for MethodDefinition<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.r#type, &other.r#type)
            && ContentEq::content_eq(&self.decorators, &other.decorators)
            && ContentEq::content_eq(&self.key, &other.key)
            && ContentEq::content_eq(&self.value, &other.value)
            && ContentEq::content_eq(&self.kind, &other.kind)
            && ContentEq::content_eq(&self.computed, &other.computed)
            && ContentEq::content_eq(&self.r#static, &other.r#static)
            && ContentEq::content_eq(&self.r#override, &other.r#override)
            && ContentEq::content_eq(&self.optional, &other.optional)
            && ContentEq::content_eq(&self.accessibility, &other.accessibility)
    }
}

impl ContentEq for MethodDefinitionType {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for PropertyDefinition<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.r#type, &other.r#type)
            && ContentEq::content_eq(&self.decorators, &other.decorators)
            && ContentEq::content_eq(&self.key, &other.key)
            && ContentEq::content_eq(&self.value, &other.value)
            && ContentEq::content_eq(&self.computed, &other.computed)
            && ContentEq::content_eq(&self.r#static, &other.r#static)
            && ContentEq::content_eq(&self.declare, &other.declare)
            && ContentEq::content_eq(&self.r#override, &other.r#override)
            && ContentEq::content_eq(&self.optional, &other.optional)
            && ContentEq::content_eq(&self.definite, &other.definite)
            && ContentEq::content_eq(&self.readonly, &other.readonly)
            && ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
            && ContentEq::content_eq(&self.accessibility, &other.accessibility)
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

impl ContentEq for PrivateIdentifier<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.name, &other.name)
    }
}

impl ContentEq for StaticBlock<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for ModuleDeclaration<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::ImportDeclaration(a), Self::ImportDeclaration(b)) => a.content_eq(b),
            (Self::ExportAllDeclaration(a), Self::ExportAllDeclaration(b)) => a.content_eq(b),
            (Self::ExportDefaultDeclaration(a), Self::ExportDefaultDeclaration(b)) => {
                a.content_eq(b)
            }
            (Self::ExportNamedDeclaration(a), Self::ExportNamedDeclaration(b)) => a.content_eq(b),
            (Self::TSExportAssignment(a), Self::TSExportAssignment(b)) => a.content_eq(b),
            (Self::TSNamespaceExportDeclaration(a), Self::TSNamespaceExportDeclaration(b)) => {
                a.content_eq(b)
            }
            _ => false,
        }
    }
}

impl ContentEq for AccessorPropertyType {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for AccessorProperty<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.r#type, &other.r#type)
            && ContentEq::content_eq(&self.decorators, &other.decorators)
            && ContentEq::content_eq(&self.key, &other.key)
            && ContentEq::content_eq(&self.value, &other.value)
            && ContentEq::content_eq(&self.computed, &other.computed)
            && ContentEq::content_eq(&self.r#static, &other.r#static)
            && ContentEq::content_eq(&self.definite, &other.definite)
            && ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
            && ContentEq::content_eq(&self.accessibility, &other.accessibility)
    }
}

impl ContentEq for ImportExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.source, &other.source)
            && ContentEq::content_eq(&self.arguments, &other.arguments)
            && ContentEq::content_eq(&self.phase, &other.phase)
    }
}

impl ContentEq for ImportDeclaration<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.specifiers, &other.specifiers)
            && ContentEq::content_eq(&self.source, &other.source)
            && ContentEq::content_eq(&self.phase, &other.phase)
            && ContentEq::content_eq(&self.with_clause, &other.with_clause)
            && ContentEq::content_eq(&self.import_kind, &other.import_kind)
    }
}

impl ContentEq for ImportPhase {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for ImportDeclarationSpecifier<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::ImportSpecifier(a), Self::ImportSpecifier(b)) => a.content_eq(b),
            (Self::ImportDefaultSpecifier(a), Self::ImportDefaultSpecifier(b)) => a.content_eq(b),
            (Self::ImportNamespaceSpecifier(a), Self::ImportNamespaceSpecifier(b)) => {
                a.content_eq(b)
            }
            _ => false,
        }
    }
}

impl ContentEq for ImportSpecifier<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.imported, &other.imported)
            && ContentEq::content_eq(&self.local, &other.local)
            && ContentEq::content_eq(&self.import_kind, &other.import_kind)
    }
}

impl ContentEq for ImportDefaultSpecifier<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.local, &other.local)
    }
}

impl ContentEq for ImportNamespaceSpecifier<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.local, &other.local)
    }
}

impl ContentEq for WithClause<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.attributes_keyword, &other.attributes_keyword)
            && ContentEq::content_eq(&self.with_entries, &other.with_entries)
    }
}

impl ContentEq for ImportAttribute<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.key, &other.key)
            && ContentEq::content_eq(&self.value, &other.value)
    }
}

impl ContentEq for ImportAttributeKey<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::Identifier(a), Self::Identifier(b)) => a.content_eq(b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for ExportNamedDeclaration<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.declaration, &other.declaration)
            && ContentEq::content_eq(&self.specifiers, &other.specifiers)
            && ContentEq::content_eq(&self.source, &other.source)
            && ContentEq::content_eq(&self.export_kind, &other.export_kind)
            && ContentEq::content_eq(&self.with_clause, &other.with_clause)
    }
}

impl ContentEq for ExportDefaultDeclaration<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.declaration, &other.declaration)
            && ContentEq::content_eq(&self.exported, &other.exported)
    }
}

impl ContentEq for ExportAllDeclaration<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.exported, &other.exported)
            && ContentEq::content_eq(&self.source, &other.source)
            && ContentEq::content_eq(&self.with_clause, &other.with_clause)
            && ContentEq::content_eq(&self.export_kind, &other.export_kind)
    }
}

impl ContentEq for ExportSpecifier<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.local, &other.local)
            && ContentEq::content_eq(&self.exported, &other.exported)
            && ContentEq::content_eq(&self.export_kind, &other.export_kind)
    }
}

impl ContentEq for ExportDefaultDeclarationKind<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::FunctionDeclaration(a), Self::FunctionDeclaration(b)) => a.content_eq(b),
            (Self::ClassDeclaration(a), Self::ClassDeclaration(b)) => a.content_eq(b),
            (Self::TSInterfaceDeclaration(a), Self::TSInterfaceDeclaration(b)) => a.content_eq(b),
            (Self::BooleanLiteral(a), Self::BooleanLiteral(b)) => a.content_eq(b),
            (Self::NullLiteral(a), Self::NullLiteral(b)) => a.content_eq(b),
            (Self::NumericLiteral(a), Self::NumericLiteral(b)) => a.content_eq(b),
            (Self::BigIntLiteral(a), Self::BigIntLiteral(b)) => a.content_eq(b),
            (Self::RegExpLiteral(a), Self::RegExpLiteral(b)) => a.content_eq(b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => a.content_eq(b),
            (Self::TemplateLiteral(a), Self::TemplateLiteral(b)) => a.content_eq(b),
            (Self::Identifier(a), Self::Identifier(b)) => a.content_eq(b),
            (Self::MetaProperty(a), Self::MetaProperty(b)) => a.content_eq(b),
            (Self::Super(a), Self::Super(b)) => a.content_eq(b),
            (Self::ArrayExpression(a), Self::ArrayExpression(b)) => a.content_eq(b),
            (Self::ArrowFunctionExpression(a), Self::ArrowFunctionExpression(b)) => a.content_eq(b),
            (Self::AssignmentExpression(a), Self::AssignmentExpression(b)) => a.content_eq(b),
            (Self::AwaitExpression(a), Self::AwaitExpression(b)) => a.content_eq(b),
            (Self::BinaryExpression(a), Self::BinaryExpression(b)) => a.content_eq(b),
            (Self::CallExpression(a), Self::CallExpression(b)) => a.content_eq(b),
            (Self::ChainExpression(a), Self::ChainExpression(b)) => a.content_eq(b),
            (Self::ClassExpression(a), Self::ClassExpression(b)) => a.content_eq(b),
            (Self::ConditionalExpression(a), Self::ConditionalExpression(b)) => a.content_eq(b),
            (Self::FunctionExpression(a), Self::FunctionExpression(b)) => a.content_eq(b),
            (Self::ImportExpression(a), Self::ImportExpression(b)) => a.content_eq(b),
            (Self::LogicalExpression(a), Self::LogicalExpression(b)) => a.content_eq(b),
            (Self::NewExpression(a), Self::NewExpression(b)) => a.content_eq(b),
            (Self::ObjectExpression(a), Self::ObjectExpression(b)) => a.content_eq(b),
            (Self::ParenthesizedExpression(a), Self::ParenthesizedExpression(b)) => a.content_eq(b),
            (Self::SequenceExpression(a), Self::SequenceExpression(b)) => a.content_eq(b),
            (Self::TaggedTemplateExpression(a), Self::TaggedTemplateExpression(b)) => {
                a.content_eq(b)
            }
            (Self::ThisExpression(a), Self::ThisExpression(b)) => a.content_eq(b),
            (Self::UnaryExpression(a), Self::UnaryExpression(b)) => a.content_eq(b),
            (Self::UpdateExpression(a), Self::UpdateExpression(b)) => a.content_eq(b),
            (Self::YieldExpression(a), Self::YieldExpression(b)) => a.content_eq(b),
            (Self::PrivateInExpression(a), Self::PrivateInExpression(b)) => a.content_eq(b),
            (Self::JSXElement(a), Self::JSXElement(b)) => a.content_eq(b),
            (Self::JSXFragment(a), Self::JSXFragment(b)) => a.content_eq(b),
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => a.content_eq(b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => a.content_eq(b),
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => a.content_eq(b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => a.content_eq(b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                a.content_eq(b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                a.content_eq(b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => a.content_eq(b),
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for ModuleExportName<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::IdentifierName(a), Self::IdentifierName(b)) => a.content_eq(b),
            (Self::IdentifierReference(a), Self::IdentifierReference(b)) => a.content_eq(b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for TSThisParameter<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl ContentEq for TSEnumDeclaration<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.id, &other.id)
            && ContentEq::content_eq(&self.members, &other.members)
            && ContentEq::content_eq(&self.r#const, &other.r#const)
            && ContentEq::content_eq(&self.declare, &other.declare)
    }
}

impl ContentEq for TSEnumMember<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.id, &other.id)
            && ContentEq::content_eq(&self.initializer, &other.initializer)
    }
}

impl ContentEq for TSEnumMemberName<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::Identifier(a), Self::Identifier(b)) => a.content_eq(b),
            (Self::String(a), Self::String(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for TSTypeAnnotation<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl ContentEq for TSLiteralType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.literal, &other.literal)
    }
}

impl ContentEq for TSLiteral<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::BooleanLiteral(a), Self::BooleanLiteral(b)) => a.content_eq(b),
            (Self::NullLiteral(a), Self::NullLiteral(b)) => a.content_eq(b),
            (Self::NumericLiteral(a), Self::NumericLiteral(b)) => a.content_eq(b),
            (Self::BigIntLiteral(a), Self::BigIntLiteral(b)) => a.content_eq(b),
            (Self::RegExpLiteral(a), Self::RegExpLiteral(b)) => a.content_eq(b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => a.content_eq(b),
            (Self::TemplateLiteral(a), Self::TemplateLiteral(b)) => a.content_eq(b),
            (Self::UnaryExpression(a), Self::UnaryExpression(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for TSType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::TSAnyKeyword(a), Self::TSAnyKeyword(b)) => a.content_eq(b),
            (Self::TSBigIntKeyword(a), Self::TSBigIntKeyword(b)) => a.content_eq(b),
            (Self::TSBooleanKeyword(a), Self::TSBooleanKeyword(b)) => a.content_eq(b),
            (Self::TSIntrinsicKeyword(a), Self::TSIntrinsicKeyword(b)) => a.content_eq(b),
            (Self::TSNeverKeyword(a), Self::TSNeverKeyword(b)) => a.content_eq(b),
            (Self::TSNullKeyword(a), Self::TSNullKeyword(b)) => a.content_eq(b),
            (Self::TSNumberKeyword(a), Self::TSNumberKeyword(b)) => a.content_eq(b),
            (Self::TSObjectKeyword(a), Self::TSObjectKeyword(b)) => a.content_eq(b),
            (Self::TSStringKeyword(a), Self::TSStringKeyword(b)) => a.content_eq(b),
            (Self::TSSymbolKeyword(a), Self::TSSymbolKeyword(b)) => a.content_eq(b),
            (Self::TSUndefinedKeyword(a), Self::TSUndefinedKeyword(b)) => a.content_eq(b),
            (Self::TSUnknownKeyword(a), Self::TSUnknownKeyword(b)) => a.content_eq(b),
            (Self::TSVoidKeyword(a), Self::TSVoidKeyword(b)) => a.content_eq(b),
            (Self::TSArrayType(a), Self::TSArrayType(b)) => a.content_eq(b),
            (Self::TSConditionalType(a), Self::TSConditionalType(b)) => a.content_eq(b),
            (Self::TSConstructorType(a), Self::TSConstructorType(b)) => a.content_eq(b),
            (Self::TSFunctionType(a), Self::TSFunctionType(b)) => a.content_eq(b),
            (Self::TSImportType(a), Self::TSImportType(b)) => a.content_eq(b),
            (Self::TSIndexedAccessType(a), Self::TSIndexedAccessType(b)) => a.content_eq(b),
            (Self::TSInferType(a), Self::TSInferType(b)) => a.content_eq(b),
            (Self::TSIntersectionType(a), Self::TSIntersectionType(b)) => a.content_eq(b),
            (Self::TSLiteralType(a), Self::TSLiteralType(b)) => a.content_eq(b),
            (Self::TSMappedType(a), Self::TSMappedType(b)) => a.content_eq(b),
            (Self::TSNamedTupleMember(a), Self::TSNamedTupleMember(b)) => a.content_eq(b),
            (Self::TSQualifiedName(a), Self::TSQualifiedName(b)) => a.content_eq(b),
            (Self::TSTemplateLiteralType(a), Self::TSTemplateLiteralType(b)) => a.content_eq(b),
            (Self::TSThisType(a), Self::TSThisType(b)) => a.content_eq(b),
            (Self::TSTupleType(a), Self::TSTupleType(b)) => a.content_eq(b),
            (Self::TSTypeLiteral(a), Self::TSTypeLiteral(b)) => a.content_eq(b),
            (Self::TSTypeOperatorType(a), Self::TSTypeOperatorType(b)) => a.content_eq(b),
            (Self::TSTypePredicate(a), Self::TSTypePredicate(b)) => a.content_eq(b),
            (Self::TSTypeQuery(a), Self::TSTypeQuery(b)) => a.content_eq(b),
            (Self::TSTypeReference(a), Self::TSTypeReference(b)) => a.content_eq(b),
            (Self::TSUnionType(a), Self::TSUnionType(b)) => a.content_eq(b),
            (Self::TSParenthesizedType(a), Self::TSParenthesizedType(b)) => a.content_eq(b),
            (Self::JSDocNullableType(a), Self::JSDocNullableType(b)) => a.content_eq(b),
            (Self::JSDocNonNullableType(a), Self::JSDocNonNullableType(b)) => a.content_eq(b),
            (Self::JSDocUnknownType(a), Self::JSDocUnknownType(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for TSConditionalType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.check_type, &other.check_type)
            && ContentEq::content_eq(&self.extends_type, &other.extends_type)
            && ContentEq::content_eq(&self.true_type, &other.true_type)
            && ContentEq::content_eq(&self.false_type, &other.false_type)
    }
}

impl ContentEq for TSUnionType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.types, &other.types)
    }
}

impl ContentEq for TSIntersectionType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.types, &other.types)
    }
}

impl ContentEq for TSParenthesizedType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl ContentEq for TSTypeOperator<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.operator, &other.operator)
            && ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl ContentEq for TSTypeOperatorOperator {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for TSArrayType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.element_type, &other.element_type)
    }
}

impl ContentEq for TSIndexedAccessType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.object_type, &other.object_type)
            && ContentEq::content_eq(&self.index_type, &other.index_type)
    }
}

impl ContentEq for TSTupleType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.element_types, &other.element_types)
    }
}

impl ContentEq for TSNamedTupleMember<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.element_type, &other.element_type)
            && ContentEq::content_eq(&self.label, &other.label)
            && ContentEq::content_eq(&self.optional, &other.optional)
    }
}

impl ContentEq for TSOptionalType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl ContentEq for TSRestType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl ContentEq for TSTupleElement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::TSOptionalType(a), Self::TSOptionalType(b)) => a.content_eq(b),
            (Self::TSRestType(a), Self::TSRestType(b)) => a.content_eq(b),
            (Self::TSAnyKeyword(a), Self::TSAnyKeyword(b)) => a.content_eq(b),
            (Self::TSBigIntKeyword(a), Self::TSBigIntKeyword(b)) => a.content_eq(b),
            (Self::TSBooleanKeyword(a), Self::TSBooleanKeyword(b)) => a.content_eq(b),
            (Self::TSIntrinsicKeyword(a), Self::TSIntrinsicKeyword(b)) => a.content_eq(b),
            (Self::TSNeverKeyword(a), Self::TSNeverKeyword(b)) => a.content_eq(b),
            (Self::TSNullKeyword(a), Self::TSNullKeyword(b)) => a.content_eq(b),
            (Self::TSNumberKeyword(a), Self::TSNumberKeyword(b)) => a.content_eq(b),
            (Self::TSObjectKeyword(a), Self::TSObjectKeyword(b)) => a.content_eq(b),
            (Self::TSStringKeyword(a), Self::TSStringKeyword(b)) => a.content_eq(b),
            (Self::TSSymbolKeyword(a), Self::TSSymbolKeyword(b)) => a.content_eq(b),
            (Self::TSUndefinedKeyword(a), Self::TSUndefinedKeyword(b)) => a.content_eq(b),
            (Self::TSUnknownKeyword(a), Self::TSUnknownKeyword(b)) => a.content_eq(b),
            (Self::TSVoidKeyword(a), Self::TSVoidKeyword(b)) => a.content_eq(b),
            (Self::TSArrayType(a), Self::TSArrayType(b)) => a.content_eq(b),
            (Self::TSConditionalType(a), Self::TSConditionalType(b)) => a.content_eq(b),
            (Self::TSConstructorType(a), Self::TSConstructorType(b)) => a.content_eq(b),
            (Self::TSFunctionType(a), Self::TSFunctionType(b)) => a.content_eq(b),
            (Self::TSImportType(a), Self::TSImportType(b)) => a.content_eq(b),
            (Self::TSIndexedAccessType(a), Self::TSIndexedAccessType(b)) => a.content_eq(b),
            (Self::TSInferType(a), Self::TSInferType(b)) => a.content_eq(b),
            (Self::TSIntersectionType(a), Self::TSIntersectionType(b)) => a.content_eq(b),
            (Self::TSLiteralType(a), Self::TSLiteralType(b)) => a.content_eq(b),
            (Self::TSMappedType(a), Self::TSMappedType(b)) => a.content_eq(b),
            (Self::TSNamedTupleMember(a), Self::TSNamedTupleMember(b)) => a.content_eq(b),
            (Self::TSQualifiedName(a), Self::TSQualifiedName(b)) => a.content_eq(b),
            (Self::TSTemplateLiteralType(a), Self::TSTemplateLiteralType(b)) => a.content_eq(b),
            (Self::TSThisType(a), Self::TSThisType(b)) => a.content_eq(b),
            (Self::TSTupleType(a), Self::TSTupleType(b)) => a.content_eq(b),
            (Self::TSTypeLiteral(a), Self::TSTypeLiteral(b)) => a.content_eq(b),
            (Self::TSTypeOperatorType(a), Self::TSTypeOperatorType(b)) => a.content_eq(b),
            (Self::TSTypePredicate(a), Self::TSTypePredicate(b)) => a.content_eq(b),
            (Self::TSTypeQuery(a), Self::TSTypeQuery(b)) => a.content_eq(b),
            (Self::TSTypeReference(a), Self::TSTypeReference(b)) => a.content_eq(b),
            (Self::TSUnionType(a), Self::TSUnionType(b)) => a.content_eq(b),
            (Self::TSParenthesizedType(a), Self::TSParenthesizedType(b)) => a.content_eq(b),
            (Self::JSDocNullableType(a), Self::JSDocNullableType(b)) => a.content_eq(b),
            (Self::JSDocNonNullableType(a), Self::JSDocNonNullableType(b)) => a.content_eq(b),
            (Self::JSDocUnknownType(a), Self::JSDocUnknownType(b)) => a.content_eq(b),
            _ => false,
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

impl ContentEq for TSTypeReference<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.type_name, &other.type_name)
            && ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
    }
}

impl ContentEq for TSTypeName<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::IdentifierReference(a), Self::IdentifierReference(b)) => a.content_eq(b),
            (Self::QualifiedName(a), Self::QualifiedName(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for TSQualifiedName<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.left, &other.left)
            && ContentEq::content_eq(&self.right, &other.right)
    }
}

impl ContentEq for TSTypeParameterInstantiation<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.params, &other.params)
    }
}

impl ContentEq for TSTypeParameter<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.name, &other.name)
            && ContentEq::content_eq(&self.constraint, &other.constraint)
            && ContentEq::content_eq(&self.default, &other.default)
            && ContentEq::content_eq(&self.r#in, &other.r#in)
            && ContentEq::content_eq(&self.out, &other.out)
            && ContentEq::content_eq(&self.r#const, &other.r#const)
    }
}

impl ContentEq for TSTypeParameterDeclaration<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.params, &other.params)
    }
}

impl ContentEq for TSTypeAliasDeclaration<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.id, &other.id)
            && ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
            && ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
            && ContentEq::content_eq(&self.declare, &other.declare)
    }
}

impl ContentEq for TSAccessibility {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for TSClassImplements<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expression, &other.expression)
            && ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
    }
}

impl ContentEq for TSInterfaceDeclaration<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.id, &other.id)
            && ContentEq::content_eq(&self.extends, &other.extends)
            && ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
            && ContentEq::content_eq(&self.body, &other.body)
            && ContentEq::content_eq(&self.declare, &other.declare)
    }
}

impl ContentEq for TSInterfaceBody<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for TSPropertySignature<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.computed, &other.computed)
            && ContentEq::content_eq(&self.optional, &other.optional)
            && ContentEq::content_eq(&self.readonly, &other.readonly)
            && ContentEq::content_eq(&self.key, &other.key)
            && ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl ContentEq for TSSignature<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::TSIndexSignature(a), Self::TSIndexSignature(b)) => a.content_eq(b),
            (Self::TSPropertySignature(a), Self::TSPropertySignature(b)) => a.content_eq(b),
            (Self::TSCallSignatureDeclaration(a), Self::TSCallSignatureDeclaration(b)) => {
                a.content_eq(b)
            }
            (
                Self::TSConstructSignatureDeclaration(a),
                Self::TSConstructSignatureDeclaration(b),
            ) => a.content_eq(b),
            (Self::TSMethodSignature(a), Self::TSMethodSignature(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for TSIndexSignature<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.parameters, &other.parameters)
            && ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
            && ContentEq::content_eq(&self.readonly, &other.readonly)
            && ContentEq::content_eq(&self.r#static, &other.r#static)
    }
}

impl ContentEq for TSCallSignatureDeclaration<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
            && ContentEq::content_eq(&self.this_param, &other.this_param)
            && ContentEq::content_eq(&self.params, &other.params)
            && ContentEq::content_eq(&self.return_type, &other.return_type)
    }
}

impl ContentEq for TSMethodSignatureKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for TSMethodSignature<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.key, &other.key)
            && ContentEq::content_eq(&self.computed, &other.computed)
            && ContentEq::content_eq(&self.optional, &other.optional)
            && ContentEq::content_eq(&self.kind, &other.kind)
            && ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
            && ContentEq::content_eq(&self.this_param, &other.this_param)
            && ContentEq::content_eq(&self.params, &other.params)
            && ContentEq::content_eq(&self.return_type, &other.return_type)
    }
}

impl ContentEq for TSConstructSignatureDeclaration<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
            && ContentEq::content_eq(&self.params, &other.params)
            && ContentEq::content_eq(&self.return_type, &other.return_type)
    }
}

impl ContentEq for TSIndexSignatureName<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.name, &other.name)
            && ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl ContentEq for TSInterfaceHeritage<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expression, &other.expression)
            && ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
    }
}

impl ContentEq for TSTypePredicate<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.parameter_name, &other.parameter_name)
            && ContentEq::content_eq(&self.asserts, &other.asserts)
            && ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl ContentEq for TSTypePredicateName<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::Identifier(a), Self::Identifier(b)) => a.content_eq(b),
            (Self::This(a), Self::This(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for TSModuleDeclaration<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.id, &other.id)
            && ContentEq::content_eq(&self.body, &other.body)
            && ContentEq::content_eq(&self.kind, &other.kind)
            && ContentEq::content_eq(&self.declare, &other.declare)
    }
}

impl ContentEq for TSModuleDeclarationKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for TSModuleDeclarationName<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::Identifier(a), Self::Identifier(b)) => a.content_eq(b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for TSModuleDeclarationBody<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::TSModuleDeclaration(a), Self::TSModuleDeclaration(b)) => a.content_eq(b),
            (Self::TSModuleBlock(a), Self::TSModuleBlock(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for TSModuleBlock<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.directives, &other.directives)
            && ContentEq::content_eq(&self.body, &other.body)
    }
}

impl ContentEq for TSTypeLiteral<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.members, &other.members)
    }
}

impl ContentEq for TSInferType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.type_parameter, &other.type_parameter)
    }
}

impl ContentEq for TSTypeQuery<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expr_name, &other.expr_name)
            && ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
    }
}

impl ContentEq for TSTypeQueryExprName<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::TSImportType(a), Self::TSImportType(b)) => a.content_eq(b),
            (Self::IdentifierReference(a), Self::IdentifierReference(b)) => a.content_eq(b),
            (Self::QualifiedName(a), Self::QualifiedName(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for TSImportType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.is_type_of, &other.is_type_of)
            && ContentEq::content_eq(&self.parameter, &other.parameter)
            && ContentEq::content_eq(&self.qualifier, &other.qualifier)
            && ContentEq::content_eq(&self.attributes, &other.attributes)
            && ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
    }
}

impl ContentEq for TSImportAttributes<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.attributes_keyword, &other.attributes_keyword)
            && ContentEq::content_eq(&self.elements, &other.elements)
    }
}

impl ContentEq for TSImportAttribute<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.name, &other.name)
            && ContentEq::content_eq(&self.value, &other.value)
    }
}

impl ContentEq for TSImportAttributeName<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::Identifier(a), Self::Identifier(b)) => a.content_eq(b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for TSFunctionType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
            && ContentEq::content_eq(&self.this_param, &other.this_param)
            && ContentEq::content_eq(&self.params, &other.params)
            && ContentEq::content_eq(&self.return_type, &other.return_type)
    }
}

impl ContentEq for TSConstructorType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.r#abstract, &other.r#abstract)
            && ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
            && ContentEq::content_eq(&self.params, &other.params)
            && ContentEq::content_eq(&self.return_type, &other.return_type)
    }
}

impl ContentEq for TSMappedType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.type_parameter, &other.type_parameter)
            && ContentEq::content_eq(&self.name_type, &other.name_type)
            && ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
            && ContentEq::content_eq(&self.optional, &other.optional)
            && ContentEq::content_eq(&self.readonly, &other.readonly)
    }
}

impl ContentEq for TSMappedTypeModifierOperator {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for TSTemplateLiteralType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.quasis, &other.quasis)
            && ContentEq::content_eq(&self.types, &other.types)
    }
}

impl ContentEq for TSAsExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expression, &other.expression)
            && ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl ContentEq for TSSatisfiesExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expression, &other.expression)
            && ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl ContentEq for TSTypeAssertion<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expression, &other.expression)
            && ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
    }
}

impl ContentEq for TSImportEqualsDeclaration<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.id, &other.id)
            && ContentEq::content_eq(&self.module_reference, &other.module_reference)
            && ContentEq::content_eq(&self.import_kind, &other.import_kind)
    }
}

impl ContentEq for TSModuleReference<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::ExternalModuleReference(a), Self::ExternalModuleReference(b)) => a.content_eq(b),
            (Self::IdentifierReference(a), Self::IdentifierReference(b)) => a.content_eq(b),
            (Self::QualifiedName(a), Self::QualifiedName(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for TSExternalModuleReference<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expression, &other.expression)
    }
}

impl ContentEq for TSNonNullExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expression, &other.expression)
    }
}

impl ContentEq for Decorator<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expression, &other.expression)
    }
}

impl ContentEq for TSExportAssignment<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expression, &other.expression)
    }
}

impl ContentEq for TSNamespaceExportDeclaration<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.id, &other.id)
    }
}

impl ContentEq for TSInstantiationExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expression, &other.expression)
            && ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
    }
}

impl ContentEq for ImportOrExportKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for JSDocNullableType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
            && ContentEq::content_eq(&self.postfix, &other.postfix)
    }
}

impl ContentEq for JSDocNonNullableType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.type_annotation, &other.type_annotation)
            && ContentEq::content_eq(&self.postfix, &other.postfix)
    }
}

impl ContentEq for JSDocUnknownType {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for JSXElement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.opening_element, &other.opening_element)
            && ContentEq::content_eq(&self.closing_element, &other.closing_element)
            && ContentEq::content_eq(&self.children, &other.children)
    }
}

impl ContentEq for JSXOpeningElement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.self_closing, &other.self_closing)
            && ContentEq::content_eq(&self.name, &other.name)
            && ContentEq::content_eq(&self.attributes, &other.attributes)
            && ContentEq::content_eq(&self.type_parameters, &other.type_parameters)
    }
}

impl ContentEq for JSXClosingElement<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.name, &other.name)
    }
}

impl ContentEq for JSXFragment<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.opening_fragment, &other.opening_fragment)
            && ContentEq::content_eq(&self.closing_fragment, &other.closing_fragment)
            && ContentEq::content_eq(&self.children, &other.children)
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

impl ContentEq for JSXElementName<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::Identifier(a), Self::Identifier(b)) => a.content_eq(b),
            (Self::IdentifierReference(a), Self::IdentifierReference(b)) => a.content_eq(b),
            (Self::NamespacedName(a), Self::NamespacedName(b)) => a.content_eq(b),
            (Self::MemberExpression(a), Self::MemberExpression(b)) => a.content_eq(b),
            (Self::ThisExpression(a), Self::ThisExpression(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for JSXNamespacedName<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.namespace, &other.namespace)
            && ContentEq::content_eq(&self.property, &other.property)
    }
}

impl ContentEq for JSXMemberExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.object, &other.object)
            && ContentEq::content_eq(&self.property, &other.property)
    }
}

impl ContentEq for JSXMemberExpressionObject<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::IdentifierReference(a), Self::IdentifierReference(b)) => a.content_eq(b),
            (Self::MemberExpression(a), Self::MemberExpression(b)) => a.content_eq(b),
            (Self::ThisExpression(a), Self::ThisExpression(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for JSXExpressionContainer<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expression, &other.expression)
    }
}

impl ContentEq for JSXExpression<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::EmptyExpression(a), Self::EmptyExpression(b)) => a.content_eq(b),
            (Self::BooleanLiteral(a), Self::BooleanLiteral(b)) => a.content_eq(b),
            (Self::NullLiteral(a), Self::NullLiteral(b)) => a.content_eq(b),
            (Self::NumericLiteral(a), Self::NumericLiteral(b)) => a.content_eq(b),
            (Self::BigIntLiteral(a), Self::BigIntLiteral(b)) => a.content_eq(b),
            (Self::RegExpLiteral(a), Self::RegExpLiteral(b)) => a.content_eq(b),
            (Self::StringLiteral(a), Self::StringLiteral(b)) => a.content_eq(b),
            (Self::TemplateLiteral(a), Self::TemplateLiteral(b)) => a.content_eq(b),
            (Self::Identifier(a), Self::Identifier(b)) => a.content_eq(b),
            (Self::MetaProperty(a), Self::MetaProperty(b)) => a.content_eq(b),
            (Self::Super(a), Self::Super(b)) => a.content_eq(b),
            (Self::ArrayExpression(a), Self::ArrayExpression(b)) => a.content_eq(b),
            (Self::ArrowFunctionExpression(a), Self::ArrowFunctionExpression(b)) => a.content_eq(b),
            (Self::AssignmentExpression(a), Self::AssignmentExpression(b)) => a.content_eq(b),
            (Self::AwaitExpression(a), Self::AwaitExpression(b)) => a.content_eq(b),
            (Self::BinaryExpression(a), Self::BinaryExpression(b)) => a.content_eq(b),
            (Self::CallExpression(a), Self::CallExpression(b)) => a.content_eq(b),
            (Self::ChainExpression(a), Self::ChainExpression(b)) => a.content_eq(b),
            (Self::ClassExpression(a), Self::ClassExpression(b)) => a.content_eq(b),
            (Self::ConditionalExpression(a), Self::ConditionalExpression(b)) => a.content_eq(b),
            (Self::FunctionExpression(a), Self::FunctionExpression(b)) => a.content_eq(b),
            (Self::ImportExpression(a), Self::ImportExpression(b)) => a.content_eq(b),
            (Self::LogicalExpression(a), Self::LogicalExpression(b)) => a.content_eq(b),
            (Self::NewExpression(a), Self::NewExpression(b)) => a.content_eq(b),
            (Self::ObjectExpression(a), Self::ObjectExpression(b)) => a.content_eq(b),
            (Self::ParenthesizedExpression(a), Self::ParenthesizedExpression(b)) => a.content_eq(b),
            (Self::SequenceExpression(a), Self::SequenceExpression(b)) => a.content_eq(b),
            (Self::TaggedTemplateExpression(a), Self::TaggedTemplateExpression(b)) => {
                a.content_eq(b)
            }
            (Self::ThisExpression(a), Self::ThisExpression(b)) => a.content_eq(b),
            (Self::UnaryExpression(a), Self::UnaryExpression(b)) => a.content_eq(b),
            (Self::UpdateExpression(a), Self::UpdateExpression(b)) => a.content_eq(b),
            (Self::YieldExpression(a), Self::YieldExpression(b)) => a.content_eq(b),
            (Self::PrivateInExpression(a), Self::PrivateInExpression(b)) => a.content_eq(b),
            (Self::JSXElement(a), Self::JSXElement(b)) => a.content_eq(b),
            (Self::JSXFragment(a), Self::JSXFragment(b)) => a.content_eq(b),
            (Self::TSAsExpression(a), Self::TSAsExpression(b)) => a.content_eq(b),
            (Self::TSSatisfiesExpression(a), Self::TSSatisfiesExpression(b)) => a.content_eq(b),
            (Self::TSTypeAssertion(a), Self::TSTypeAssertion(b)) => a.content_eq(b),
            (Self::TSNonNullExpression(a), Self::TSNonNullExpression(b)) => a.content_eq(b),
            (Self::TSInstantiationExpression(a), Self::TSInstantiationExpression(b)) => {
                a.content_eq(b)
            }
            (Self::ComputedMemberExpression(a), Self::ComputedMemberExpression(b)) => {
                a.content_eq(b)
            }
            (Self::StaticMemberExpression(a), Self::StaticMemberExpression(b)) => a.content_eq(b),
            (Self::PrivateFieldExpression(a), Self::PrivateFieldExpression(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for JSXEmptyExpression {
    fn content_eq(&self, _: &Self) -> bool {
        true
    }
}

impl ContentEq for JSXAttributeItem<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::Attribute(a), Self::Attribute(b)) => a.content_eq(b),
            (Self::SpreadAttribute(a), Self::SpreadAttribute(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for JSXAttribute<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.name, &other.name)
            && ContentEq::content_eq(&self.value, &other.value)
    }
}

impl ContentEq for JSXSpreadAttribute<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.argument, &other.argument)
    }
}

impl ContentEq for JSXAttributeName<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::Identifier(a), Self::Identifier(b)) => a.content_eq(b),
            (Self::NamespacedName(a), Self::NamespacedName(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for JSXAttributeValue<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::StringLiteral(a), Self::StringLiteral(b)) => a.content_eq(b),
            (Self::ExpressionContainer(a), Self::ExpressionContainer(b)) => a.content_eq(b),
            (Self::Element(a), Self::Element(b)) => a.content_eq(b),
            (Self::Fragment(a), Self::Fragment(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for JSXIdentifier<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.name, &other.name)
    }
}

impl ContentEq for JSXChild<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        #[allow(clippy::match_same_arms)]
        match (self, other) {
            (Self::Text(a), Self::Text(b)) => a.content_eq(b),
            (Self::Element(a), Self::Element(b)) => a.content_eq(b),
            (Self::Fragment(a), Self::Fragment(b)) => a.content_eq(b),
            (Self::ExpressionContainer(a), Self::ExpressionContainer(b)) => a.content_eq(b),
            (Self::Spread(a), Self::Spread(b)) => a.content_eq(b),
            _ => false,
        }
    }
}

impl ContentEq for JSXSpreadChild<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.expression, &other.expression)
    }
}

impl ContentEq for JSXText<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.value, &other.value)
    }
}

impl ContentEq for CommentKind {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for CommentPosition {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for Comment {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.attached_to, &other.attached_to)
            && ContentEq::content_eq(&self.kind, &other.kind)
            && ContentEq::content_eq(&self.position, &other.position)
            && ContentEq::content_eq(&self.preceded_by_newline, &other.preceded_by_newline)
            && ContentEq::content_eq(&self.followed_by_newline, &other.followed_by_newline)
    }
}
