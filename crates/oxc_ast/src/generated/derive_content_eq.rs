// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/content_eq.rs`

#![allow(clippy::match_like_matches_macro)]
#[allow(unused)]
use std::mem;

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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::BooleanLiteral(it) => match other {
                Self::BooleanLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NullLiteral(it) => match other {
                Self::NullLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NumericLiteral(it) => match other {
                Self::NumericLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BigIntLiteral(it) => match other {
                Self::BigIntLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::RegExpLiteral(it) => match other {
                Self::RegExpLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StringLiteral(it) => match other {
                Self::StringLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TemplateLiteral(it) => match other {
                Self::TemplateLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Identifier(it) => match other {
                Self::Identifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::MetaProperty(it) => match other {
                Self::MetaProperty(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Super(it) => match other {
                Self::Super(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ArrayExpression(it) => match other {
                Self::ArrayExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ArrowFunctionExpression(it) => match other {
                Self::ArrowFunctionExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::AssignmentExpression(it) => match other {
                Self::AssignmentExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::AwaitExpression(it) => match other {
                Self::AwaitExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BinaryExpression(it) => match other {
                Self::BinaryExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::CallExpression(it) => match other {
                Self::CallExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ChainExpression(it) => match other {
                Self::ChainExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ClassExpression(it) => match other {
                Self::ClassExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ConditionalExpression(it) => match other {
                Self::ConditionalExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::FunctionExpression(it) => match other {
                Self::FunctionExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ImportExpression(it) => match other {
                Self::ImportExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::LogicalExpression(it) => match other {
                Self::LogicalExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NewExpression(it) => match other {
                Self::NewExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ObjectExpression(it) => match other {
                Self::ObjectExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ParenthesizedExpression(it) => match other {
                Self::ParenthesizedExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::SequenceExpression(it) => match other {
                Self::SequenceExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TaggedTemplateExpression(it) => match other {
                Self::TaggedTemplateExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ThisExpression(it) => match other {
                Self::ThisExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::UnaryExpression(it) => match other {
                Self::UnaryExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::UpdateExpression(it) => match other {
                Self::UpdateExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::YieldExpression(it) => match other {
                Self::YieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateInExpression(it) => match other {
                Self::PrivateInExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSXElement(it) => match other {
                Self::JSXElement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSXFragment(it) => match other {
                Self::JSXFragment(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSAsExpression(it) => match other {
                Self::TSAsExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSSatisfiesExpression(it) => match other {
                Self::TSSatisfiesExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeAssertion(it) => match other {
                Self::TSTypeAssertion(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNonNullExpression(it) => match other {
                Self::TSNonNullExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSInstantiationExpression(it) => match other {
                Self::TSInstantiationExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ComputedMemberExpression(it) => match other {
                Self::ComputedMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StaticMemberExpression(it) => match other {
                Self::StaticMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateFieldExpression(it) => match other {
                Self::PrivateFieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::SpreadElement(it) => match other {
                Self::SpreadElement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Elision(it) => match other {
                Self::Elision(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BooleanLiteral(it) => match other {
                Self::BooleanLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NullLiteral(it) => match other {
                Self::NullLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NumericLiteral(it) => match other {
                Self::NumericLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BigIntLiteral(it) => match other {
                Self::BigIntLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::RegExpLiteral(it) => match other {
                Self::RegExpLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StringLiteral(it) => match other {
                Self::StringLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TemplateLiteral(it) => match other {
                Self::TemplateLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Identifier(it) => match other {
                Self::Identifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::MetaProperty(it) => match other {
                Self::MetaProperty(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Super(it) => match other {
                Self::Super(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ArrayExpression(it) => match other {
                Self::ArrayExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ArrowFunctionExpression(it) => match other {
                Self::ArrowFunctionExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::AssignmentExpression(it) => match other {
                Self::AssignmentExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::AwaitExpression(it) => match other {
                Self::AwaitExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BinaryExpression(it) => match other {
                Self::BinaryExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::CallExpression(it) => match other {
                Self::CallExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ChainExpression(it) => match other {
                Self::ChainExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ClassExpression(it) => match other {
                Self::ClassExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ConditionalExpression(it) => match other {
                Self::ConditionalExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::FunctionExpression(it) => match other {
                Self::FunctionExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ImportExpression(it) => match other {
                Self::ImportExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::LogicalExpression(it) => match other {
                Self::LogicalExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NewExpression(it) => match other {
                Self::NewExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ObjectExpression(it) => match other {
                Self::ObjectExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ParenthesizedExpression(it) => match other {
                Self::ParenthesizedExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::SequenceExpression(it) => match other {
                Self::SequenceExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TaggedTemplateExpression(it) => match other {
                Self::TaggedTemplateExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ThisExpression(it) => match other {
                Self::ThisExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::UnaryExpression(it) => match other {
                Self::UnaryExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::UpdateExpression(it) => match other {
                Self::UpdateExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::YieldExpression(it) => match other {
                Self::YieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateInExpression(it) => match other {
                Self::PrivateInExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSXElement(it) => match other {
                Self::JSXElement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSXFragment(it) => match other {
                Self::JSXFragment(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSAsExpression(it) => match other {
                Self::TSAsExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSSatisfiesExpression(it) => match other {
                Self::TSSatisfiesExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeAssertion(it) => match other {
                Self::TSTypeAssertion(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNonNullExpression(it) => match other {
                Self::TSNonNullExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSInstantiationExpression(it) => match other {
                Self::TSInstantiationExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ComputedMemberExpression(it) => match other {
                Self::ComputedMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StaticMemberExpression(it) => match other {
                Self::StaticMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateFieldExpression(it) => match other {
                Self::PrivateFieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::ObjectProperty(it) => match other {
                Self::ObjectProperty(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::SpreadProperty(it) => match other {
                Self::SpreadProperty(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::StaticIdentifier(it) => match other {
                Self::StaticIdentifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateIdentifier(it) => match other {
                Self::PrivateIdentifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BooleanLiteral(it) => match other {
                Self::BooleanLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NullLiteral(it) => match other {
                Self::NullLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NumericLiteral(it) => match other {
                Self::NumericLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BigIntLiteral(it) => match other {
                Self::BigIntLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::RegExpLiteral(it) => match other {
                Self::RegExpLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StringLiteral(it) => match other {
                Self::StringLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TemplateLiteral(it) => match other {
                Self::TemplateLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Identifier(it) => match other {
                Self::Identifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::MetaProperty(it) => match other {
                Self::MetaProperty(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Super(it) => match other {
                Self::Super(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ArrayExpression(it) => match other {
                Self::ArrayExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ArrowFunctionExpression(it) => match other {
                Self::ArrowFunctionExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::AssignmentExpression(it) => match other {
                Self::AssignmentExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::AwaitExpression(it) => match other {
                Self::AwaitExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BinaryExpression(it) => match other {
                Self::BinaryExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::CallExpression(it) => match other {
                Self::CallExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ChainExpression(it) => match other {
                Self::ChainExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ClassExpression(it) => match other {
                Self::ClassExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ConditionalExpression(it) => match other {
                Self::ConditionalExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::FunctionExpression(it) => match other {
                Self::FunctionExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ImportExpression(it) => match other {
                Self::ImportExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::LogicalExpression(it) => match other {
                Self::LogicalExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NewExpression(it) => match other {
                Self::NewExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ObjectExpression(it) => match other {
                Self::ObjectExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ParenthesizedExpression(it) => match other {
                Self::ParenthesizedExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::SequenceExpression(it) => match other {
                Self::SequenceExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TaggedTemplateExpression(it) => match other {
                Self::TaggedTemplateExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ThisExpression(it) => match other {
                Self::ThisExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::UnaryExpression(it) => match other {
                Self::UnaryExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::UpdateExpression(it) => match other {
                Self::UpdateExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::YieldExpression(it) => match other {
                Self::YieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateInExpression(it) => match other {
                Self::PrivateInExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSXElement(it) => match other {
                Self::JSXElement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSXFragment(it) => match other {
                Self::JSXFragment(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSAsExpression(it) => match other {
                Self::TSAsExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSSatisfiesExpression(it) => match other {
                Self::TSSatisfiesExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeAssertion(it) => match other {
                Self::TSTypeAssertion(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNonNullExpression(it) => match other {
                Self::TSNonNullExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSInstantiationExpression(it) => match other {
                Self::TSInstantiationExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ComputedMemberExpression(it) => match other {
                Self::ComputedMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StaticMemberExpression(it) => match other {
                Self::StaticMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateFieldExpression(it) => match other {
                Self::PrivateFieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::ComputedMemberExpression(it) => match other {
                Self::ComputedMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StaticMemberExpression(it) => match other {
                Self::StaticMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateFieldExpression(it) => match other {
                Self::PrivateFieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::SpreadElement(it) => match other {
                Self::SpreadElement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BooleanLiteral(it) => match other {
                Self::BooleanLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NullLiteral(it) => match other {
                Self::NullLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NumericLiteral(it) => match other {
                Self::NumericLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BigIntLiteral(it) => match other {
                Self::BigIntLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::RegExpLiteral(it) => match other {
                Self::RegExpLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StringLiteral(it) => match other {
                Self::StringLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TemplateLiteral(it) => match other {
                Self::TemplateLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Identifier(it) => match other {
                Self::Identifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::MetaProperty(it) => match other {
                Self::MetaProperty(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Super(it) => match other {
                Self::Super(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ArrayExpression(it) => match other {
                Self::ArrayExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ArrowFunctionExpression(it) => match other {
                Self::ArrowFunctionExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::AssignmentExpression(it) => match other {
                Self::AssignmentExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::AwaitExpression(it) => match other {
                Self::AwaitExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BinaryExpression(it) => match other {
                Self::BinaryExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::CallExpression(it) => match other {
                Self::CallExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ChainExpression(it) => match other {
                Self::ChainExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ClassExpression(it) => match other {
                Self::ClassExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ConditionalExpression(it) => match other {
                Self::ConditionalExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::FunctionExpression(it) => match other {
                Self::FunctionExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ImportExpression(it) => match other {
                Self::ImportExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::LogicalExpression(it) => match other {
                Self::LogicalExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NewExpression(it) => match other {
                Self::NewExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ObjectExpression(it) => match other {
                Self::ObjectExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ParenthesizedExpression(it) => match other {
                Self::ParenthesizedExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::SequenceExpression(it) => match other {
                Self::SequenceExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TaggedTemplateExpression(it) => match other {
                Self::TaggedTemplateExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ThisExpression(it) => match other {
                Self::ThisExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::UnaryExpression(it) => match other {
                Self::UnaryExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::UpdateExpression(it) => match other {
                Self::UpdateExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::YieldExpression(it) => match other {
                Self::YieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateInExpression(it) => match other {
                Self::PrivateInExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSXElement(it) => match other {
                Self::JSXElement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSXFragment(it) => match other {
                Self::JSXFragment(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSAsExpression(it) => match other {
                Self::TSAsExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSSatisfiesExpression(it) => match other {
                Self::TSSatisfiesExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeAssertion(it) => match other {
                Self::TSTypeAssertion(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNonNullExpression(it) => match other {
                Self::TSNonNullExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSInstantiationExpression(it) => match other {
                Self::TSInstantiationExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ComputedMemberExpression(it) => match other {
                Self::ComputedMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StaticMemberExpression(it) => match other {
                Self::StaticMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateFieldExpression(it) => match other {
                Self::PrivateFieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::AssignmentTargetIdentifier(it) => match other {
                Self::AssignmentTargetIdentifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSAsExpression(it) => match other {
                Self::TSAsExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSSatisfiesExpression(it) => match other {
                Self::TSSatisfiesExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNonNullExpression(it) => match other {
                Self::TSNonNullExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeAssertion(it) => match other {
                Self::TSTypeAssertion(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSInstantiationExpression(it) => match other {
                Self::TSInstantiationExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ComputedMemberExpression(it) => match other {
                Self::ComputedMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StaticMemberExpression(it) => match other {
                Self::StaticMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateFieldExpression(it) => match other {
                Self::PrivateFieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ArrayAssignmentTarget(it) => match other {
                Self::ArrayAssignmentTarget(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ObjectAssignmentTarget(it) => match other {
                Self::ObjectAssignmentTarget(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
        }
    }
}

impl ContentEq for SimpleAssignmentTarget<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::AssignmentTargetIdentifier(it) => match other {
                Self::AssignmentTargetIdentifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSAsExpression(it) => match other {
                Self::TSAsExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSSatisfiesExpression(it) => match other {
                Self::TSSatisfiesExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNonNullExpression(it) => match other {
                Self::TSNonNullExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeAssertion(it) => match other {
                Self::TSTypeAssertion(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSInstantiationExpression(it) => match other {
                Self::TSInstantiationExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ComputedMemberExpression(it) => match other {
                Self::ComputedMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StaticMemberExpression(it) => match other {
                Self::StaticMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateFieldExpression(it) => match other {
                Self::PrivateFieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
        }
    }
}

impl ContentEq for AssignmentTargetPattern<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::ArrayAssignmentTarget(it) => match other {
                Self::ArrayAssignmentTarget(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ObjectAssignmentTarget(it) => match other {
                Self::ObjectAssignmentTarget(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::AssignmentTargetWithDefault(it) => match other {
                Self::AssignmentTargetWithDefault(other) if ContentEq::content_eq(it, other) => {
                    true
                }
                _ => false,
            },
            Self::AssignmentTargetIdentifier(it) => match other {
                Self::AssignmentTargetIdentifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSAsExpression(it) => match other {
                Self::TSAsExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSSatisfiesExpression(it) => match other {
                Self::TSSatisfiesExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNonNullExpression(it) => match other {
                Self::TSNonNullExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeAssertion(it) => match other {
                Self::TSTypeAssertion(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSInstantiationExpression(it) => match other {
                Self::TSInstantiationExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ComputedMemberExpression(it) => match other {
                Self::ComputedMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StaticMemberExpression(it) => match other {
                Self::StaticMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateFieldExpression(it) => match other {
                Self::PrivateFieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ArrayAssignmentTarget(it) => match other {
                Self::ArrayAssignmentTarget(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ObjectAssignmentTarget(it) => match other {
                Self::ObjectAssignmentTarget(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => match other {
                Self::AssignmentTargetPropertyIdentifier(other)
                    if ContentEq::content_eq(it, other) =>
                {
                    true
                }
                _ => false,
            },
            Self::AssignmentTargetPropertyProperty(it) => match other {
                Self::AssignmentTargetPropertyProperty(other)
                    if ContentEq::content_eq(it, other) =>
                {
                    true
                }
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::CallExpression(it) => match other {
                Self::CallExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNonNullExpression(it) => match other {
                Self::TSNonNullExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ComputedMemberExpression(it) => match other {
                Self::ComputedMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StaticMemberExpression(it) => match other {
                Self::StaticMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateFieldExpression(it) => match other {
                Self::PrivateFieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::BlockStatement(it) => match other {
                Self::BlockStatement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BreakStatement(it) => match other {
                Self::BreakStatement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ContinueStatement(it) => match other {
                Self::ContinueStatement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::DebuggerStatement(it) => match other {
                Self::DebuggerStatement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::DoWhileStatement(it) => match other {
                Self::DoWhileStatement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::EmptyStatement(it) => match other {
                Self::EmptyStatement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ExpressionStatement(it) => match other {
                Self::ExpressionStatement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ForInStatement(it) => match other {
                Self::ForInStatement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ForOfStatement(it) => match other {
                Self::ForOfStatement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ForStatement(it) => match other {
                Self::ForStatement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::IfStatement(it) => match other {
                Self::IfStatement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::LabeledStatement(it) => match other {
                Self::LabeledStatement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ReturnStatement(it) => match other {
                Self::ReturnStatement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::SwitchStatement(it) => match other {
                Self::SwitchStatement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ThrowStatement(it) => match other {
                Self::ThrowStatement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TryStatement(it) => match other {
                Self::TryStatement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::WhileStatement(it) => match other {
                Self::WhileStatement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::WithStatement(it) => match other {
                Self::WithStatement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::VariableDeclaration(it) => match other {
                Self::VariableDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::FunctionDeclaration(it) => match other {
                Self::FunctionDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ClassDeclaration(it) => match other {
                Self::ClassDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeAliasDeclaration(it) => match other {
                Self::TSTypeAliasDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSInterfaceDeclaration(it) => match other {
                Self::TSInterfaceDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSEnumDeclaration(it) => match other {
                Self::TSEnumDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSModuleDeclaration(it) => match other {
                Self::TSModuleDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSImportEqualsDeclaration(it) => match other {
                Self::TSImportEqualsDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ImportDeclaration(it) => match other {
                Self::ImportDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ExportAllDeclaration(it) => match other {
                Self::ExportAllDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ExportDefaultDeclaration(it) => match other {
                Self::ExportDefaultDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ExportNamedDeclaration(it) => match other {
                Self::ExportNamedDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSExportAssignment(it) => match other {
                Self::TSExportAssignment(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNamespaceExportDeclaration(it) => match other {
                Self::TSNamespaceExportDeclaration(other) if ContentEq::content_eq(it, other) => {
                    true
                }
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::VariableDeclaration(it) => match other {
                Self::VariableDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::FunctionDeclaration(it) => match other {
                Self::FunctionDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ClassDeclaration(it) => match other {
                Self::ClassDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeAliasDeclaration(it) => match other {
                Self::TSTypeAliasDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSInterfaceDeclaration(it) => match other {
                Self::TSInterfaceDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSEnumDeclaration(it) => match other {
                Self::TSEnumDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSModuleDeclaration(it) => match other {
                Self::TSModuleDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSImportEqualsDeclaration(it) => match other {
                Self::TSImportEqualsDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::VariableDeclaration(it) => match other {
                Self::VariableDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BooleanLiteral(it) => match other {
                Self::BooleanLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NullLiteral(it) => match other {
                Self::NullLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NumericLiteral(it) => match other {
                Self::NumericLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BigIntLiteral(it) => match other {
                Self::BigIntLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::RegExpLiteral(it) => match other {
                Self::RegExpLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StringLiteral(it) => match other {
                Self::StringLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TemplateLiteral(it) => match other {
                Self::TemplateLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Identifier(it) => match other {
                Self::Identifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::MetaProperty(it) => match other {
                Self::MetaProperty(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Super(it) => match other {
                Self::Super(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ArrayExpression(it) => match other {
                Self::ArrayExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ArrowFunctionExpression(it) => match other {
                Self::ArrowFunctionExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::AssignmentExpression(it) => match other {
                Self::AssignmentExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::AwaitExpression(it) => match other {
                Self::AwaitExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BinaryExpression(it) => match other {
                Self::BinaryExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::CallExpression(it) => match other {
                Self::CallExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ChainExpression(it) => match other {
                Self::ChainExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ClassExpression(it) => match other {
                Self::ClassExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ConditionalExpression(it) => match other {
                Self::ConditionalExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::FunctionExpression(it) => match other {
                Self::FunctionExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ImportExpression(it) => match other {
                Self::ImportExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::LogicalExpression(it) => match other {
                Self::LogicalExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NewExpression(it) => match other {
                Self::NewExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ObjectExpression(it) => match other {
                Self::ObjectExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ParenthesizedExpression(it) => match other {
                Self::ParenthesizedExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::SequenceExpression(it) => match other {
                Self::SequenceExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TaggedTemplateExpression(it) => match other {
                Self::TaggedTemplateExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ThisExpression(it) => match other {
                Self::ThisExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::UnaryExpression(it) => match other {
                Self::UnaryExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::UpdateExpression(it) => match other {
                Self::UpdateExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::YieldExpression(it) => match other {
                Self::YieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateInExpression(it) => match other {
                Self::PrivateInExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSXElement(it) => match other {
                Self::JSXElement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSXFragment(it) => match other {
                Self::JSXFragment(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSAsExpression(it) => match other {
                Self::TSAsExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSSatisfiesExpression(it) => match other {
                Self::TSSatisfiesExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeAssertion(it) => match other {
                Self::TSTypeAssertion(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNonNullExpression(it) => match other {
                Self::TSNonNullExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSInstantiationExpression(it) => match other {
                Self::TSInstantiationExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ComputedMemberExpression(it) => match other {
                Self::ComputedMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StaticMemberExpression(it) => match other {
                Self::StaticMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateFieldExpression(it) => match other {
                Self::PrivateFieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::VariableDeclaration(it) => match other {
                Self::VariableDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::AssignmentTargetIdentifier(it) => match other {
                Self::AssignmentTargetIdentifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSAsExpression(it) => match other {
                Self::TSAsExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSSatisfiesExpression(it) => match other {
                Self::TSSatisfiesExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNonNullExpression(it) => match other {
                Self::TSNonNullExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeAssertion(it) => match other {
                Self::TSTypeAssertion(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSInstantiationExpression(it) => match other {
                Self::TSInstantiationExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ComputedMemberExpression(it) => match other {
                Self::ComputedMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StaticMemberExpression(it) => match other {
                Self::StaticMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateFieldExpression(it) => match other {
                Self::PrivateFieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ArrayAssignmentTarget(it) => match other {
                Self::ArrayAssignmentTarget(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ObjectAssignmentTarget(it) => match other {
                Self::ObjectAssignmentTarget(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::BindingIdentifier(it) => match other {
                Self::BindingIdentifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ObjectPattern(it) => match other {
                Self::ObjectPattern(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ArrayPattern(it) => match other {
                Self::ArrayPattern(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::AssignmentPattern(it) => match other {
                Self::AssignmentPattern(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::StaticBlock(it) => match other {
                Self::StaticBlock(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::MethodDefinition(it) => match other {
                Self::MethodDefinition(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PropertyDefinition(it) => match other {
                Self::PropertyDefinition(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::AccessorProperty(it) => match other {
                Self::AccessorProperty(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSIndexSignature(it) => match other {
                Self::TSIndexSignature(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::ImportDeclaration(it) => match other {
                Self::ImportDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ExportAllDeclaration(it) => match other {
                Self::ExportAllDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ExportDefaultDeclaration(it) => match other {
                Self::ExportDefaultDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ExportNamedDeclaration(it) => match other {
                Self::ExportNamedDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSExportAssignment(it) => match other {
                Self::TSExportAssignment(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNamespaceExportDeclaration(it) => match other {
                Self::TSNamespaceExportDeclaration(other) if ContentEq::content_eq(it, other) => {
                    true
                }
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::ImportSpecifier(it) => match other {
                Self::ImportSpecifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ImportDefaultSpecifier(it) => match other {
                Self::ImportDefaultSpecifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ImportNamespaceSpecifier(it) => match other {
                Self::ImportNamespaceSpecifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::Identifier(it) => match other {
                Self::Identifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StringLiteral(it) => match other {
                Self::StringLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::FunctionDeclaration(it) => match other {
                Self::FunctionDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ClassDeclaration(it) => match other {
                Self::ClassDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSInterfaceDeclaration(it) => match other {
                Self::TSInterfaceDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BooleanLiteral(it) => match other {
                Self::BooleanLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NullLiteral(it) => match other {
                Self::NullLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NumericLiteral(it) => match other {
                Self::NumericLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BigIntLiteral(it) => match other {
                Self::BigIntLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::RegExpLiteral(it) => match other {
                Self::RegExpLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StringLiteral(it) => match other {
                Self::StringLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TemplateLiteral(it) => match other {
                Self::TemplateLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Identifier(it) => match other {
                Self::Identifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::MetaProperty(it) => match other {
                Self::MetaProperty(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Super(it) => match other {
                Self::Super(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ArrayExpression(it) => match other {
                Self::ArrayExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ArrowFunctionExpression(it) => match other {
                Self::ArrowFunctionExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::AssignmentExpression(it) => match other {
                Self::AssignmentExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::AwaitExpression(it) => match other {
                Self::AwaitExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BinaryExpression(it) => match other {
                Self::BinaryExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::CallExpression(it) => match other {
                Self::CallExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ChainExpression(it) => match other {
                Self::ChainExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ClassExpression(it) => match other {
                Self::ClassExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ConditionalExpression(it) => match other {
                Self::ConditionalExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::FunctionExpression(it) => match other {
                Self::FunctionExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ImportExpression(it) => match other {
                Self::ImportExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::LogicalExpression(it) => match other {
                Self::LogicalExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NewExpression(it) => match other {
                Self::NewExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ObjectExpression(it) => match other {
                Self::ObjectExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ParenthesizedExpression(it) => match other {
                Self::ParenthesizedExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::SequenceExpression(it) => match other {
                Self::SequenceExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TaggedTemplateExpression(it) => match other {
                Self::TaggedTemplateExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ThisExpression(it) => match other {
                Self::ThisExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::UnaryExpression(it) => match other {
                Self::UnaryExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::UpdateExpression(it) => match other {
                Self::UpdateExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::YieldExpression(it) => match other {
                Self::YieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateInExpression(it) => match other {
                Self::PrivateInExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSXElement(it) => match other {
                Self::JSXElement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSXFragment(it) => match other {
                Self::JSXFragment(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSAsExpression(it) => match other {
                Self::TSAsExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSSatisfiesExpression(it) => match other {
                Self::TSSatisfiesExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeAssertion(it) => match other {
                Self::TSTypeAssertion(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNonNullExpression(it) => match other {
                Self::TSNonNullExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSInstantiationExpression(it) => match other {
                Self::TSInstantiationExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ComputedMemberExpression(it) => match other {
                Self::ComputedMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StaticMemberExpression(it) => match other {
                Self::StaticMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateFieldExpression(it) => match other {
                Self::PrivateFieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
        }
    }
}

impl ContentEq for ModuleExportName<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::IdentifierName(it) => match other {
                Self::IdentifierName(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::IdentifierReference(it) => match other {
                Self::IdentifierReference(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StringLiteral(it) => match other {
                Self::StringLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::Identifier(it) => match other {
                Self::Identifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::String(it) => match other {
                Self::String(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::BooleanLiteral(it) => match other {
                Self::BooleanLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NullLiteral(it) => match other {
                Self::NullLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NumericLiteral(it) => match other {
                Self::NumericLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BigIntLiteral(it) => match other {
                Self::BigIntLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::RegExpLiteral(it) => match other {
                Self::RegExpLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StringLiteral(it) => match other {
                Self::StringLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TemplateLiteral(it) => match other {
                Self::TemplateLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::UnaryExpression(it) => match other {
                Self::UnaryExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
        }
    }
}

impl ContentEq for TSType<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::TSAnyKeyword(it) => match other {
                Self::TSAnyKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSBigIntKeyword(it) => match other {
                Self::TSBigIntKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSBooleanKeyword(it) => match other {
                Self::TSBooleanKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSIntrinsicKeyword(it) => match other {
                Self::TSIntrinsicKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNeverKeyword(it) => match other {
                Self::TSNeverKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNullKeyword(it) => match other {
                Self::TSNullKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNumberKeyword(it) => match other {
                Self::TSNumberKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSObjectKeyword(it) => match other {
                Self::TSObjectKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSStringKeyword(it) => match other {
                Self::TSStringKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSSymbolKeyword(it) => match other {
                Self::TSSymbolKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSUndefinedKeyword(it) => match other {
                Self::TSUndefinedKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSUnknownKeyword(it) => match other {
                Self::TSUnknownKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSVoidKeyword(it) => match other {
                Self::TSVoidKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSArrayType(it) => match other {
                Self::TSArrayType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSConditionalType(it) => match other {
                Self::TSConditionalType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSConstructorType(it) => match other {
                Self::TSConstructorType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSFunctionType(it) => match other {
                Self::TSFunctionType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSImportType(it) => match other {
                Self::TSImportType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSIndexedAccessType(it) => match other {
                Self::TSIndexedAccessType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSInferType(it) => match other {
                Self::TSInferType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSIntersectionType(it) => match other {
                Self::TSIntersectionType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSLiteralType(it) => match other {
                Self::TSLiteralType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSMappedType(it) => match other {
                Self::TSMappedType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNamedTupleMember(it) => match other {
                Self::TSNamedTupleMember(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSQualifiedName(it) => match other {
                Self::TSQualifiedName(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTemplateLiteralType(it) => match other {
                Self::TSTemplateLiteralType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSThisType(it) => match other {
                Self::TSThisType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTupleType(it) => match other {
                Self::TSTupleType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeLiteral(it) => match other {
                Self::TSTypeLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeOperatorType(it) => match other {
                Self::TSTypeOperatorType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypePredicate(it) => match other {
                Self::TSTypePredicate(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeQuery(it) => match other {
                Self::TSTypeQuery(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeReference(it) => match other {
                Self::TSTypeReference(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSUnionType(it) => match other {
                Self::TSUnionType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSParenthesizedType(it) => match other {
                Self::TSParenthesizedType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSDocNullableType(it) => match other {
                Self::JSDocNullableType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSDocNonNullableType(it) => match other {
                Self::JSDocNonNullableType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSDocUnknownType(it) => match other {
                Self::JSDocUnknownType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::TSOptionalType(it) => match other {
                Self::TSOptionalType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSRestType(it) => match other {
                Self::TSRestType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSAnyKeyword(it) => match other {
                Self::TSAnyKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSBigIntKeyword(it) => match other {
                Self::TSBigIntKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSBooleanKeyword(it) => match other {
                Self::TSBooleanKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSIntrinsicKeyword(it) => match other {
                Self::TSIntrinsicKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNeverKeyword(it) => match other {
                Self::TSNeverKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNullKeyword(it) => match other {
                Self::TSNullKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNumberKeyword(it) => match other {
                Self::TSNumberKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSObjectKeyword(it) => match other {
                Self::TSObjectKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSStringKeyword(it) => match other {
                Self::TSStringKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSSymbolKeyword(it) => match other {
                Self::TSSymbolKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSUndefinedKeyword(it) => match other {
                Self::TSUndefinedKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSUnknownKeyword(it) => match other {
                Self::TSUnknownKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSVoidKeyword(it) => match other {
                Self::TSVoidKeyword(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSArrayType(it) => match other {
                Self::TSArrayType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSConditionalType(it) => match other {
                Self::TSConditionalType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSConstructorType(it) => match other {
                Self::TSConstructorType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSFunctionType(it) => match other {
                Self::TSFunctionType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSImportType(it) => match other {
                Self::TSImportType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSIndexedAccessType(it) => match other {
                Self::TSIndexedAccessType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSInferType(it) => match other {
                Self::TSInferType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSIntersectionType(it) => match other {
                Self::TSIntersectionType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSLiteralType(it) => match other {
                Self::TSLiteralType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSMappedType(it) => match other {
                Self::TSMappedType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNamedTupleMember(it) => match other {
                Self::TSNamedTupleMember(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSQualifiedName(it) => match other {
                Self::TSQualifiedName(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTemplateLiteralType(it) => match other {
                Self::TSTemplateLiteralType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSThisType(it) => match other {
                Self::TSThisType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTupleType(it) => match other {
                Self::TSTupleType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeLiteral(it) => match other {
                Self::TSTypeLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeOperatorType(it) => match other {
                Self::TSTypeOperatorType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypePredicate(it) => match other {
                Self::TSTypePredicate(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeQuery(it) => match other {
                Self::TSTypeQuery(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeReference(it) => match other {
                Self::TSTypeReference(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSUnionType(it) => match other {
                Self::TSUnionType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSParenthesizedType(it) => match other {
                Self::TSParenthesizedType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSDocNullableType(it) => match other {
                Self::JSDocNullableType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSDocNonNullableType(it) => match other {
                Self::JSDocNonNullableType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSDocUnknownType(it) => match other {
                Self::JSDocUnknownType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::IdentifierReference(it) => match other {
                Self::IdentifierReference(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::QualifiedName(it) => match other {
                Self::QualifiedName(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::TSIndexSignature(it) => match other {
                Self::TSIndexSignature(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSPropertySignature(it) => match other {
                Self::TSPropertySignature(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSCallSignatureDeclaration(it) => match other {
                Self::TSCallSignatureDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSConstructSignatureDeclaration(it) => match other {
                Self::TSConstructSignatureDeclaration(other)
                    if ContentEq::content_eq(it, other) =>
                {
                    true
                }
                _ => false,
            },
            Self::TSMethodSignature(it) => match other {
                Self::TSMethodSignature(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::Identifier(it) => match other {
                Self::Identifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::This(it) => match other {
                Self::This(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::Identifier(it) => match other {
                Self::Identifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StringLiteral(it) => match other {
                Self::StringLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
        }
    }
}

impl ContentEq for TSModuleDeclarationBody<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::TSModuleDeclaration(it) => match other {
                Self::TSModuleDeclaration(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSModuleBlock(it) => match other {
                Self::TSModuleBlock(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::TSImportType(it) => match other {
                Self::TSImportType(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::IdentifierReference(it) => match other {
                Self::IdentifierReference(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::QualifiedName(it) => match other {
                Self::QualifiedName(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::Identifier(it) => match other {
                Self::Identifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StringLiteral(it) => match other {
                Self::StringLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::ExternalModuleReference(it) => match other {
                Self::ExternalModuleReference(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::IdentifierReference(it) => match other {
                Self::IdentifierReference(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::QualifiedName(it) => match other {
                Self::QualifiedName(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::Identifier(it) => match other {
                Self::Identifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::IdentifierReference(it) => match other {
                Self::IdentifierReference(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NamespacedName(it) => match other {
                Self::NamespacedName(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::MemberExpression(it) => match other {
                Self::MemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ThisExpression(it) => match other {
                Self::ThisExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::IdentifierReference(it) => match other {
                Self::IdentifierReference(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::MemberExpression(it) => match other {
                Self::MemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ThisExpression(it) => match other {
                Self::ThisExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::EmptyExpression(it) => match other {
                Self::EmptyExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BooleanLiteral(it) => match other {
                Self::BooleanLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NullLiteral(it) => match other {
                Self::NullLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NumericLiteral(it) => match other {
                Self::NumericLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BigIntLiteral(it) => match other {
                Self::BigIntLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::RegExpLiteral(it) => match other {
                Self::RegExpLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StringLiteral(it) => match other {
                Self::StringLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TemplateLiteral(it) => match other {
                Self::TemplateLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Identifier(it) => match other {
                Self::Identifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::MetaProperty(it) => match other {
                Self::MetaProperty(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Super(it) => match other {
                Self::Super(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ArrayExpression(it) => match other {
                Self::ArrayExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ArrowFunctionExpression(it) => match other {
                Self::ArrowFunctionExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::AssignmentExpression(it) => match other {
                Self::AssignmentExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::AwaitExpression(it) => match other {
                Self::AwaitExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::BinaryExpression(it) => match other {
                Self::BinaryExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::CallExpression(it) => match other {
                Self::CallExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ChainExpression(it) => match other {
                Self::ChainExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ClassExpression(it) => match other {
                Self::ClassExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ConditionalExpression(it) => match other {
                Self::ConditionalExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::FunctionExpression(it) => match other {
                Self::FunctionExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ImportExpression(it) => match other {
                Self::ImportExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::LogicalExpression(it) => match other {
                Self::LogicalExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NewExpression(it) => match other {
                Self::NewExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ObjectExpression(it) => match other {
                Self::ObjectExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ParenthesizedExpression(it) => match other {
                Self::ParenthesizedExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::SequenceExpression(it) => match other {
                Self::SequenceExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TaggedTemplateExpression(it) => match other {
                Self::TaggedTemplateExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ThisExpression(it) => match other {
                Self::ThisExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::UnaryExpression(it) => match other {
                Self::UnaryExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::UpdateExpression(it) => match other {
                Self::UpdateExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::YieldExpression(it) => match other {
                Self::YieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateInExpression(it) => match other {
                Self::PrivateInExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSXElement(it) => match other {
                Self::JSXElement(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::JSXFragment(it) => match other {
                Self::JSXFragment(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSAsExpression(it) => match other {
                Self::TSAsExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSSatisfiesExpression(it) => match other {
                Self::TSSatisfiesExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSTypeAssertion(it) => match other {
                Self::TSTypeAssertion(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSNonNullExpression(it) => match other {
                Self::TSNonNullExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::TSInstantiationExpression(it) => match other {
                Self::TSInstantiationExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ComputedMemberExpression(it) => match other {
                Self::ComputedMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::StaticMemberExpression(it) => match other {
                Self::StaticMemberExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::PrivateFieldExpression(it) => match other {
                Self::PrivateFieldExpression(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::Attribute(it) => match other {
                Self::Attribute(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::SpreadAttribute(it) => match other {
                Self::SpreadAttribute(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::Identifier(it) => match other {
                Self::Identifier(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::NamespacedName(it) => match other {
                Self::NamespacedName(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
        }
    }
}

impl ContentEq for JSXAttributeValue<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::StringLiteral(it) => match other {
                Self::StringLiteral(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ExpressionContainer(it) => match other {
                Self::ExpressionContainer(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Element(it) => match other {
                Self::Element(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Fragment(it) => match other {
                Self::Fragment(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        match self {
            Self::Text(it) => match other {
                Self::Text(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Element(it) => match other {
                Self::Element(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Fragment(it) => match other {
                Self::Fragment(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::ExpressionContainer(it) => match other {
                Self::ExpressionContainer(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
            Self::Spread(it) => match other {
                Self::Spread(other) if ContentEq::content_eq(it, other) => true,
                _ => false,
            },
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
