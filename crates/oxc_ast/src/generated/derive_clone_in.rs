// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/clone_in.rs`

#![allow(clippy::default_trait_access)]

use oxc_allocator::{Allocator, CloneIn};

#[allow(clippy::wildcard_imports)]
use crate::ast::comment::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::js::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::jsx::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::literal::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::ts::*;

impl<'alloc> CloneIn<'alloc> for BooleanLiteral {
    type Cloned = BooleanLiteral;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        BooleanLiteral {
            span: CloneIn::clone_in(&self.span, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for NullLiteral {
    type Cloned = NullLiteral;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        NullLiteral { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for NumericLiteral<'old_alloc> {
    type Cloned = NumericLiteral<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        NumericLiteral {
            span: CloneIn::clone_in(&self.span, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
            raw: CloneIn::clone_in(&self.raw, allocator),
            base: CloneIn::clone_in(&self.base, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BigIntLiteral<'old_alloc> {
    type Cloned = BigIntLiteral<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BigIntLiteral {
            span: CloneIn::clone_in(&self.span, allocator),
            raw: CloneIn::clone_in(&self.raw, allocator),
            base: CloneIn::clone_in(&self.base, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for RegExpLiteral<'old_alloc> {
    type Cloned = RegExpLiteral<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        RegExpLiteral {
            span: CloneIn::clone_in(&self.span, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
            regex: CloneIn::clone_in(&self.regex, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for RegExp<'old_alloc> {
    type Cloned = RegExp<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        RegExp {
            pattern: CloneIn::clone_in(&self.pattern, allocator),
            flags: CloneIn::clone_in(&self.flags, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for RegExpPattern<'old_alloc> {
    type Cloned = RegExpPattern<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Raw(it) => RegExpPattern::Raw(CloneIn::clone_in(it, allocator)),
            Self::Invalid(it) => RegExpPattern::Invalid(CloneIn::clone_in(it, allocator)),
            Self::Pattern(it) => RegExpPattern::Pattern(CloneIn::clone_in(it, allocator)),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for EmptyObject {
    type Cloned = EmptyObject;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        EmptyObject
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for StringLiteral<'old_alloc> {
    type Cloned = StringLiteral<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        StringLiteral {
            span: CloneIn::clone_in(&self.span, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Program<'old_alloc> {
    type Cloned = Program<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Program {
            span: CloneIn::clone_in(&self.span, allocator),
            source_type: CloneIn::clone_in(&self.source_type, allocator),
            source_text: CloneIn::clone_in(&self.source_text, allocator),
            comments: CloneIn::clone_in(&self.comments, allocator),
            hashbang: CloneIn::clone_in(&self.hashbang, allocator),
            directives: CloneIn::clone_in(&self.directives, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Expression<'old_alloc> {
    type Cloned = Expression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::BooleanLiteral(it) => {
                Expression::BooleanLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::NullLiteral(it) => Expression::NullLiteral(CloneIn::clone_in(it, allocator)),
            Self::NumericLiteral(it) => {
                Expression::NumericLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::BigIntLiteral(it) => Expression::BigIntLiteral(CloneIn::clone_in(it, allocator)),
            Self::RegExpLiteral(it) => Expression::RegExpLiteral(CloneIn::clone_in(it, allocator)),
            Self::StringLiteral(it) => Expression::StringLiteral(CloneIn::clone_in(it, allocator)),
            Self::TemplateLiteral(it) => {
                Expression::TemplateLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::Identifier(it) => Expression::Identifier(CloneIn::clone_in(it, allocator)),
            Self::MetaProperty(it) => Expression::MetaProperty(CloneIn::clone_in(it, allocator)),
            Self::Super(it) => Expression::Super(CloneIn::clone_in(it, allocator)),
            Self::ArrayExpression(it) => {
                Expression::ArrayExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ArrowFunctionExpression(it) => {
                Expression::ArrowFunctionExpression(CloneIn::clone_in(it, allocator))
            }
            Self::AssignmentExpression(it) => {
                Expression::AssignmentExpression(CloneIn::clone_in(it, allocator))
            }
            Self::AwaitExpression(it) => {
                Expression::AwaitExpression(CloneIn::clone_in(it, allocator))
            }
            Self::BinaryExpression(it) => {
                Expression::BinaryExpression(CloneIn::clone_in(it, allocator))
            }
            Self::CallExpression(it) => {
                Expression::CallExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ChainExpression(it) => {
                Expression::ChainExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ClassExpression(it) => {
                Expression::ClassExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ConditionalExpression(it) => {
                Expression::ConditionalExpression(CloneIn::clone_in(it, allocator))
            }
            Self::FunctionExpression(it) => {
                Expression::FunctionExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ImportExpression(it) => {
                Expression::ImportExpression(CloneIn::clone_in(it, allocator))
            }
            Self::LogicalExpression(it) => {
                Expression::LogicalExpression(CloneIn::clone_in(it, allocator))
            }
            Self::NewExpression(it) => Expression::NewExpression(CloneIn::clone_in(it, allocator)),
            Self::ObjectExpression(it) => {
                Expression::ObjectExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ParenthesizedExpression(it) => {
                Expression::ParenthesizedExpression(CloneIn::clone_in(it, allocator))
            }
            Self::SequenceExpression(it) => {
                Expression::SequenceExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TaggedTemplateExpression(it) => {
                Expression::TaggedTemplateExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ThisExpression(it) => {
                Expression::ThisExpression(CloneIn::clone_in(it, allocator))
            }
            Self::UnaryExpression(it) => {
                Expression::UnaryExpression(CloneIn::clone_in(it, allocator))
            }
            Self::UpdateExpression(it) => {
                Expression::UpdateExpression(CloneIn::clone_in(it, allocator))
            }
            Self::YieldExpression(it) => {
                Expression::YieldExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateInExpression(it) => {
                Expression::PrivateInExpression(CloneIn::clone_in(it, allocator))
            }
            Self::JSXElement(it) => Expression::JSXElement(CloneIn::clone_in(it, allocator)),
            Self::JSXFragment(it) => Expression::JSXFragment(CloneIn::clone_in(it, allocator)),
            Self::TSAsExpression(it) => {
                Expression::TSAsExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSSatisfiesExpression(it) => {
                Expression::TSSatisfiesExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSTypeAssertion(it) => {
                Expression::TSTypeAssertion(CloneIn::clone_in(it, allocator))
            }
            Self::TSNonNullExpression(it) => {
                Expression::TSNonNullExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSInstantiationExpression(it) => {
                Expression::TSInstantiationExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ComputedMemberExpression(it) => {
                Expression::ComputedMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::StaticMemberExpression(it) => {
                Expression::StaticMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateFieldExpression(it) => {
                Expression::PrivateFieldExpression(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for IdentifierName<'old_alloc> {
    type Cloned = IdentifierName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        IdentifierName {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for IdentifierReference<'old_alloc> {
    type Cloned = IdentifierReference<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        IdentifierReference {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
            reference_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BindingIdentifier<'old_alloc> {
    type Cloned = BindingIdentifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BindingIdentifier {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
            symbol_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for LabelIdentifier<'old_alloc> {
    type Cloned = LabelIdentifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        LabelIdentifier {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for ThisExpression {
    type Cloned = ThisExpression;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        ThisExpression { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ArrayExpression<'old_alloc> {
    type Cloned = ArrayExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ArrayExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            elements: CloneIn::clone_in(&self.elements, allocator),
            trailing_comma: CloneIn::clone_in(&self.trailing_comma, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ArrayExpressionElement<'old_alloc> {
    type Cloned = ArrayExpressionElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::SpreadElement(it) => {
                ArrayExpressionElement::SpreadElement(CloneIn::clone_in(it, allocator))
            }
            Self::Elision(it) => ArrayExpressionElement::Elision(CloneIn::clone_in(it, allocator)),
            Self::BooleanLiteral(it) => {
                ArrayExpressionElement::BooleanLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::NullLiteral(it) => {
                ArrayExpressionElement::NullLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::NumericLiteral(it) => {
                ArrayExpressionElement::NumericLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::BigIntLiteral(it) => {
                ArrayExpressionElement::BigIntLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::RegExpLiteral(it) => {
                ArrayExpressionElement::RegExpLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::StringLiteral(it) => {
                ArrayExpressionElement::StringLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::TemplateLiteral(it) => {
                ArrayExpressionElement::TemplateLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::Identifier(it) => {
                ArrayExpressionElement::Identifier(CloneIn::clone_in(it, allocator))
            }
            Self::MetaProperty(it) => {
                ArrayExpressionElement::MetaProperty(CloneIn::clone_in(it, allocator))
            }
            Self::Super(it) => ArrayExpressionElement::Super(CloneIn::clone_in(it, allocator)),
            Self::ArrayExpression(it) => {
                ArrayExpressionElement::ArrayExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ArrowFunctionExpression(it) => {
                ArrayExpressionElement::ArrowFunctionExpression(CloneIn::clone_in(it, allocator))
            }
            Self::AssignmentExpression(it) => {
                ArrayExpressionElement::AssignmentExpression(CloneIn::clone_in(it, allocator))
            }
            Self::AwaitExpression(it) => {
                ArrayExpressionElement::AwaitExpression(CloneIn::clone_in(it, allocator))
            }
            Self::BinaryExpression(it) => {
                ArrayExpressionElement::BinaryExpression(CloneIn::clone_in(it, allocator))
            }
            Self::CallExpression(it) => {
                ArrayExpressionElement::CallExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ChainExpression(it) => {
                ArrayExpressionElement::ChainExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ClassExpression(it) => {
                ArrayExpressionElement::ClassExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ConditionalExpression(it) => {
                ArrayExpressionElement::ConditionalExpression(CloneIn::clone_in(it, allocator))
            }
            Self::FunctionExpression(it) => {
                ArrayExpressionElement::FunctionExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ImportExpression(it) => {
                ArrayExpressionElement::ImportExpression(CloneIn::clone_in(it, allocator))
            }
            Self::LogicalExpression(it) => {
                ArrayExpressionElement::LogicalExpression(CloneIn::clone_in(it, allocator))
            }
            Self::NewExpression(it) => {
                ArrayExpressionElement::NewExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ObjectExpression(it) => {
                ArrayExpressionElement::ObjectExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ParenthesizedExpression(it) => {
                ArrayExpressionElement::ParenthesizedExpression(CloneIn::clone_in(it, allocator))
            }
            Self::SequenceExpression(it) => {
                ArrayExpressionElement::SequenceExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TaggedTemplateExpression(it) => {
                ArrayExpressionElement::TaggedTemplateExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ThisExpression(it) => {
                ArrayExpressionElement::ThisExpression(CloneIn::clone_in(it, allocator))
            }
            Self::UnaryExpression(it) => {
                ArrayExpressionElement::UnaryExpression(CloneIn::clone_in(it, allocator))
            }
            Self::UpdateExpression(it) => {
                ArrayExpressionElement::UpdateExpression(CloneIn::clone_in(it, allocator))
            }
            Self::YieldExpression(it) => {
                ArrayExpressionElement::YieldExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateInExpression(it) => {
                ArrayExpressionElement::PrivateInExpression(CloneIn::clone_in(it, allocator))
            }
            Self::JSXElement(it) => {
                ArrayExpressionElement::JSXElement(CloneIn::clone_in(it, allocator))
            }
            Self::JSXFragment(it) => {
                ArrayExpressionElement::JSXFragment(CloneIn::clone_in(it, allocator))
            }
            Self::TSAsExpression(it) => {
                ArrayExpressionElement::TSAsExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSSatisfiesExpression(it) => {
                ArrayExpressionElement::TSSatisfiesExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSTypeAssertion(it) => {
                ArrayExpressionElement::TSTypeAssertion(CloneIn::clone_in(it, allocator))
            }
            Self::TSNonNullExpression(it) => {
                ArrayExpressionElement::TSNonNullExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSInstantiationExpression(it) => {
                ArrayExpressionElement::TSInstantiationExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ComputedMemberExpression(it) => {
                ArrayExpressionElement::ComputedMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::StaticMemberExpression(it) => {
                ArrayExpressionElement::StaticMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateFieldExpression(it) => {
                ArrayExpressionElement::PrivateFieldExpression(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'alloc> CloneIn<'alloc> for Elision {
    type Cloned = Elision;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        Elision { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ObjectExpression<'old_alloc> {
    type Cloned = ObjectExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ObjectExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            properties: CloneIn::clone_in(&self.properties, allocator),
            trailing_comma: CloneIn::clone_in(&self.trailing_comma, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ObjectPropertyKind<'old_alloc> {
    type Cloned = ObjectPropertyKind<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::ObjectProperty(it) => {
                ObjectPropertyKind::ObjectProperty(CloneIn::clone_in(it, allocator))
            }
            Self::SpreadProperty(it) => {
                ObjectPropertyKind::SpreadProperty(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ObjectProperty<'old_alloc> {
    type Cloned = ObjectProperty<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ObjectProperty {
            span: CloneIn::clone_in(&self.span, allocator),
            kind: CloneIn::clone_in(&self.kind, allocator),
            key: CloneIn::clone_in(&self.key, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
            init: CloneIn::clone_in(&self.init, allocator),
            method: CloneIn::clone_in(&self.method, allocator),
            shorthand: CloneIn::clone_in(&self.shorthand, allocator),
            computed: CloneIn::clone_in(&self.computed, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for PropertyKey<'old_alloc> {
    type Cloned = PropertyKey<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::StaticIdentifier(it) => {
                PropertyKey::StaticIdentifier(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateIdentifier(it) => {
                PropertyKey::PrivateIdentifier(CloneIn::clone_in(it, allocator))
            }
            Self::BooleanLiteral(it) => {
                PropertyKey::BooleanLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::NullLiteral(it) => PropertyKey::NullLiteral(CloneIn::clone_in(it, allocator)),
            Self::NumericLiteral(it) => {
                PropertyKey::NumericLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::BigIntLiteral(it) => PropertyKey::BigIntLiteral(CloneIn::clone_in(it, allocator)),
            Self::RegExpLiteral(it) => PropertyKey::RegExpLiteral(CloneIn::clone_in(it, allocator)),
            Self::StringLiteral(it) => PropertyKey::StringLiteral(CloneIn::clone_in(it, allocator)),
            Self::TemplateLiteral(it) => {
                PropertyKey::TemplateLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::Identifier(it) => PropertyKey::Identifier(CloneIn::clone_in(it, allocator)),
            Self::MetaProperty(it) => PropertyKey::MetaProperty(CloneIn::clone_in(it, allocator)),
            Self::Super(it) => PropertyKey::Super(CloneIn::clone_in(it, allocator)),
            Self::ArrayExpression(it) => {
                PropertyKey::ArrayExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ArrowFunctionExpression(it) => {
                PropertyKey::ArrowFunctionExpression(CloneIn::clone_in(it, allocator))
            }
            Self::AssignmentExpression(it) => {
                PropertyKey::AssignmentExpression(CloneIn::clone_in(it, allocator))
            }
            Self::AwaitExpression(it) => {
                PropertyKey::AwaitExpression(CloneIn::clone_in(it, allocator))
            }
            Self::BinaryExpression(it) => {
                PropertyKey::BinaryExpression(CloneIn::clone_in(it, allocator))
            }
            Self::CallExpression(it) => {
                PropertyKey::CallExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ChainExpression(it) => {
                PropertyKey::ChainExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ClassExpression(it) => {
                PropertyKey::ClassExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ConditionalExpression(it) => {
                PropertyKey::ConditionalExpression(CloneIn::clone_in(it, allocator))
            }
            Self::FunctionExpression(it) => {
                PropertyKey::FunctionExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ImportExpression(it) => {
                PropertyKey::ImportExpression(CloneIn::clone_in(it, allocator))
            }
            Self::LogicalExpression(it) => {
                PropertyKey::LogicalExpression(CloneIn::clone_in(it, allocator))
            }
            Self::NewExpression(it) => PropertyKey::NewExpression(CloneIn::clone_in(it, allocator)),
            Self::ObjectExpression(it) => {
                PropertyKey::ObjectExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ParenthesizedExpression(it) => {
                PropertyKey::ParenthesizedExpression(CloneIn::clone_in(it, allocator))
            }
            Self::SequenceExpression(it) => {
                PropertyKey::SequenceExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TaggedTemplateExpression(it) => {
                PropertyKey::TaggedTemplateExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ThisExpression(it) => {
                PropertyKey::ThisExpression(CloneIn::clone_in(it, allocator))
            }
            Self::UnaryExpression(it) => {
                PropertyKey::UnaryExpression(CloneIn::clone_in(it, allocator))
            }
            Self::UpdateExpression(it) => {
                PropertyKey::UpdateExpression(CloneIn::clone_in(it, allocator))
            }
            Self::YieldExpression(it) => {
                PropertyKey::YieldExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateInExpression(it) => {
                PropertyKey::PrivateInExpression(CloneIn::clone_in(it, allocator))
            }
            Self::JSXElement(it) => PropertyKey::JSXElement(CloneIn::clone_in(it, allocator)),
            Self::JSXFragment(it) => PropertyKey::JSXFragment(CloneIn::clone_in(it, allocator)),
            Self::TSAsExpression(it) => {
                PropertyKey::TSAsExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSSatisfiesExpression(it) => {
                PropertyKey::TSSatisfiesExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSTypeAssertion(it) => {
                PropertyKey::TSTypeAssertion(CloneIn::clone_in(it, allocator))
            }
            Self::TSNonNullExpression(it) => {
                PropertyKey::TSNonNullExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSInstantiationExpression(it) => {
                PropertyKey::TSInstantiationExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ComputedMemberExpression(it) => {
                PropertyKey::ComputedMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::StaticMemberExpression(it) => {
                PropertyKey::StaticMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateFieldExpression(it) => {
                PropertyKey::PrivateFieldExpression(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'alloc> CloneIn<'alloc> for PropertyKind {
    type Cloned = PropertyKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Init => PropertyKind::Init,
            Self::Get => PropertyKind::Get,
            Self::Set => PropertyKind::Set,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TemplateLiteral<'old_alloc> {
    type Cloned = TemplateLiteral<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TemplateLiteral {
            span: CloneIn::clone_in(&self.span, allocator),
            quasis: CloneIn::clone_in(&self.quasis, allocator),
            expressions: CloneIn::clone_in(&self.expressions, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TaggedTemplateExpression<'old_alloc> {
    type Cloned = TaggedTemplateExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TaggedTemplateExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            tag: CloneIn::clone_in(&self.tag, allocator),
            quasi: CloneIn::clone_in(&self.quasi, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TemplateElement<'old_alloc> {
    type Cloned = TemplateElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TemplateElement {
            span: CloneIn::clone_in(&self.span, allocator),
            tail: CloneIn::clone_in(&self.tail, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TemplateElementValue<'old_alloc> {
    type Cloned = TemplateElementValue<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TemplateElementValue {
            raw: CloneIn::clone_in(&self.raw, allocator),
            cooked: CloneIn::clone_in(&self.cooked, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for MemberExpression<'old_alloc> {
    type Cloned = MemberExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::ComputedMemberExpression(it) => {
                MemberExpression::ComputedMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::StaticMemberExpression(it) => {
                MemberExpression::StaticMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateFieldExpression(it) => {
                MemberExpression::PrivateFieldExpression(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ComputedMemberExpression<'old_alloc> {
    type Cloned = ComputedMemberExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ComputedMemberExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            object: CloneIn::clone_in(&self.object, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
            optional: CloneIn::clone_in(&self.optional, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for StaticMemberExpression<'old_alloc> {
    type Cloned = StaticMemberExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        StaticMemberExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            object: CloneIn::clone_in(&self.object, allocator),
            property: CloneIn::clone_in(&self.property, allocator),
            optional: CloneIn::clone_in(&self.optional, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for PrivateFieldExpression<'old_alloc> {
    type Cloned = PrivateFieldExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        PrivateFieldExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            object: CloneIn::clone_in(&self.object, allocator),
            field: CloneIn::clone_in(&self.field, allocator),
            optional: CloneIn::clone_in(&self.optional, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for CallExpression<'old_alloc> {
    type Cloned = CallExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        CallExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            callee: CloneIn::clone_in(&self.callee, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
            arguments: CloneIn::clone_in(&self.arguments, allocator),
            optional: CloneIn::clone_in(&self.optional, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for NewExpression<'old_alloc> {
    type Cloned = NewExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        NewExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            callee: CloneIn::clone_in(&self.callee, allocator),
            arguments: CloneIn::clone_in(&self.arguments, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for MetaProperty<'old_alloc> {
    type Cloned = MetaProperty<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        MetaProperty {
            span: CloneIn::clone_in(&self.span, allocator),
            meta: CloneIn::clone_in(&self.meta, allocator),
            property: CloneIn::clone_in(&self.property, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for SpreadElement<'old_alloc> {
    type Cloned = SpreadElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        SpreadElement {
            span: CloneIn::clone_in(&self.span, allocator),
            argument: CloneIn::clone_in(&self.argument, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Argument<'old_alloc> {
    type Cloned = Argument<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::SpreadElement(it) => Argument::SpreadElement(CloneIn::clone_in(it, allocator)),
            Self::BooleanLiteral(it) => Argument::BooleanLiteral(CloneIn::clone_in(it, allocator)),
            Self::NullLiteral(it) => Argument::NullLiteral(CloneIn::clone_in(it, allocator)),
            Self::NumericLiteral(it) => Argument::NumericLiteral(CloneIn::clone_in(it, allocator)),
            Self::BigIntLiteral(it) => Argument::BigIntLiteral(CloneIn::clone_in(it, allocator)),
            Self::RegExpLiteral(it) => Argument::RegExpLiteral(CloneIn::clone_in(it, allocator)),
            Self::StringLiteral(it) => Argument::StringLiteral(CloneIn::clone_in(it, allocator)),
            Self::TemplateLiteral(it) => {
                Argument::TemplateLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::Identifier(it) => Argument::Identifier(CloneIn::clone_in(it, allocator)),
            Self::MetaProperty(it) => Argument::MetaProperty(CloneIn::clone_in(it, allocator)),
            Self::Super(it) => Argument::Super(CloneIn::clone_in(it, allocator)),
            Self::ArrayExpression(it) => {
                Argument::ArrayExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ArrowFunctionExpression(it) => {
                Argument::ArrowFunctionExpression(CloneIn::clone_in(it, allocator))
            }
            Self::AssignmentExpression(it) => {
                Argument::AssignmentExpression(CloneIn::clone_in(it, allocator))
            }
            Self::AwaitExpression(it) => {
                Argument::AwaitExpression(CloneIn::clone_in(it, allocator))
            }
            Self::BinaryExpression(it) => {
                Argument::BinaryExpression(CloneIn::clone_in(it, allocator))
            }
            Self::CallExpression(it) => Argument::CallExpression(CloneIn::clone_in(it, allocator)),
            Self::ChainExpression(it) => {
                Argument::ChainExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ClassExpression(it) => {
                Argument::ClassExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ConditionalExpression(it) => {
                Argument::ConditionalExpression(CloneIn::clone_in(it, allocator))
            }
            Self::FunctionExpression(it) => {
                Argument::FunctionExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ImportExpression(it) => {
                Argument::ImportExpression(CloneIn::clone_in(it, allocator))
            }
            Self::LogicalExpression(it) => {
                Argument::LogicalExpression(CloneIn::clone_in(it, allocator))
            }
            Self::NewExpression(it) => Argument::NewExpression(CloneIn::clone_in(it, allocator)),
            Self::ObjectExpression(it) => {
                Argument::ObjectExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ParenthesizedExpression(it) => {
                Argument::ParenthesizedExpression(CloneIn::clone_in(it, allocator))
            }
            Self::SequenceExpression(it) => {
                Argument::SequenceExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TaggedTemplateExpression(it) => {
                Argument::TaggedTemplateExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ThisExpression(it) => Argument::ThisExpression(CloneIn::clone_in(it, allocator)),
            Self::UnaryExpression(it) => {
                Argument::UnaryExpression(CloneIn::clone_in(it, allocator))
            }
            Self::UpdateExpression(it) => {
                Argument::UpdateExpression(CloneIn::clone_in(it, allocator))
            }
            Self::YieldExpression(it) => {
                Argument::YieldExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateInExpression(it) => {
                Argument::PrivateInExpression(CloneIn::clone_in(it, allocator))
            }
            Self::JSXElement(it) => Argument::JSXElement(CloneIn::clone_in(it, allocator)),
            Self::JSXFragment(it) => Argument::JSXFragment(CloneIn::clone_in(it, allocator)),
            Self::TSAsExpression(it) => Argument::TSAsExpression(CloneIn::clone_in(it, allocator)),
            Self::TSSatisfiesExpression(it) => {
                Argument::TSSatisfiesExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSTypeAssertion(it) => {
                Argument::TSTypeAssertion(CloneIn::clone_in(it, allocator))
            }
            Self::TSNonNullExpression(it) => {
                Argument::TSNonNullExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSInstantiationExpression(it) => {
                Argument::TSInstantiationExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ComputedMemberExpression(it) => {
                Argument::ComputedMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::StaticMemberExpression(it) => {
                Argument::StaticMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateFieldExpression(it) => {
                Argument::PrivateFieldExpression(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for UpdateExpression<'old_alloc> {
    type Cloned = UpdateExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        UpdateExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            operator: CloneIn::clone_in(&self.operator, allocator),
            prefix: CloneIn::clone_in(&self.prefix, allocator),
            argument: CloneIn::clone_in(&self.argument, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for UnaryExpression<'old_alloc> {
    type Cloned = UnaryExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        UnaryExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            operator: CloneIn::clone_in(&self.operator, allocator),
            argument: CloneIn::clone_in(&self.argument, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BinaryExpression<'old_alloc> {
    type Cloned = BinaryExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BinaryExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            left: CloneIn::clone_in(&self.left, allocator),
            operator: CloneIn::clone_in(&self.operator, allocator),
            right: CloneIn::clone_in(&self.right, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for PrivateInExpression<'old_alloc> {
    type Cloned = PrivateInExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        PrivateInExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            left: CloneIn::clone_in(&self.left, allocator),
            operator: CloneIn::clone_in(&self.operator, allocator),
            right: CloneIn::clone_in(&self.right, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for LogicalExpression<'old_alloc> {
    type Cloned = LogicalExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        LogicalExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            left: CloneIn::clone_in(&self.left, allocator),
            operator: CloneIn::clone_in(&self.operator, allocator),
            right: CloneIn::clone_in(&self.right, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ConditionalExpression<'old_alloc> {
    type Cloned = ConditionalExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ConditionalExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            test: CloneIn::clone_in(&self.test, allocator),
            consequent: CloneIn::clone_in(&self.consequent, allocator),
            alternate: CloneIn::clone_in(&self.alternate, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentExpression<'old_alloc> {
    type Cloned = AssignmentExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            operator: CloneIn::clone_in(&self.operator, allocator),
            left: CloneIn::clone_in(&self.left, allocator),
            right: CloneIn::clone_in(&self.right, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTarget<'old_alloc> {
    type Cloned = AssignmentTarget<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::AssignmentTargetIdentifier(it) => {
                AssignmentTarget::AssignmentTargetIdentifier(CloneIn::clone_in(it, allocator))
            }
            Self::TSAsExpression(it) => {
                AssignmentTarget::TSAsExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSSatisfiesExpression(it) => {
                AssignmentTarget::TSSatisfiesExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSNonNullExpression(it) => {
                AssignmentTarget::TSNonNullExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSTypeAssertion(it) => {
                AssignmentTarget::TSTypeAssertion(CloneIn::clone_in(it, allocator))
            }
            Self::TSInstantiationExpression(it) => {
                AssignmentTarget::TSInstantiationExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ComputedMemberExpression(it) => {
                AssignmentTarget::ComputedMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::StaticMemberExpression(it) => {
                AssignmentTarget::StaticMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateFieldExpression(it) => {
                AssignmentTarget::PrivateFieldExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ArrayAssignmentTarget(it) => {
                AssignmentTarget::ArrayAssignmentTarget(CloneIn::clone_in(it, allocator))
            }
            Self::ObjectAssignmentTarget(it) => {
                AssignmentTarget::ObjectAssignmentTarget(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for SimpleAssignmentTarget<'old_alloc> {
    type Cloned = SimpleAssignmentTarget<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::AssignmentTargetIdentifier(it) => {
                SimpleAssignmentTarget::AssignmentTargetIdentifier(CloneIn::clone_in(it, allocator))
            }
            Self::TSAsExpression(it) => {
                SimpleAssignmentTarget::TSAsExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSSatisfiesExpression(it) => {
                SimpleAssignmentTarget::TSSatisfiesExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSNonNullExpression(it) => {
                SimpleAssignmentTarget::TSNonNullExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSTypeAssertion(it) => {
                SimpleAssignmentTarget::TSTypeAssertion(CloneIn::clone_in(it, allocator))
            }
            Self::TSInstantiationExpression(it) => {
                SimpleAssignmentTarget::TSInstantiationExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ComputedMemberExpression(it) => {
                SimpleAssignmentTarget::ComputedMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::StaticMemberExpression(it) => {
                SimpleAssignmentTarget::StaticMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateFieldExpression(it) => {
                SimpleAssignmentTarget::PrivateFieldExpression(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTargetPattern<'old_alloc> {
    type Cloned = AssignmentTargetPattern<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::ArrayAssignmentTarget(it) => {
                AssignmentTargetPattern::ArrayAssignmentTarget(CloneIn::clone_in(it, allocator))
            }
            Self::ObjectAssignmentTarget(it) => {
                AssignmentTargetPattern::ObjectAssignmentTarget(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ArrayAssignmentTarget<'old_alloc> {
    type Cloned = ArrayAssignmentTarget<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ArrayAssignmentTarget {
            span: CloneIn::clone_in(&self.span, allocator),
            elements: CloneIn::clone_in(&self.elements, allocator),
            rest: CloneIn::clone_in(&self.rest, allocator),
            trailing_comma: CloneIn::clone_in(&self.trailing_comma, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ObjectAssignmentTarget<'old_alloc> {
    type Cloned = ObjectAssignmentTarget<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ObjectAssignmentTarget {
            span: CloneIn::clone_in(&self.span, allocator),
            properties: CloneIn::clone_in(&self.properties, allocator),
            rest: CloneIn::clone_in(&self.rest, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTargetRest<'old_alloc> {
    type Cloned = AssignmentTargetRest<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentTargetRest {
            span: CloneIn::clone_in(&self.span, allocator),
            target: CloneIn::clone_in(&self.target, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTargetMaybeDefault<'old_alloc> {
    type Cloned = AssignmentTargetMaybeDefault<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::AssignmentTargetWithDefault(it) => {
                AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(CloneIn::clone_in(
                    it, allocator,
                ))
            }
            Self::AssignmentTargetIdentifier(it) => {
                AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(CloneIn::clone_in(
                    it, allocator,
                ))
            }
            Self::TSAsExpression(it) => {
                AssignmentTargetMaybeDefault::TSAsExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSSatisfiesExpression(it) => AssignmentTargetMaybeDefault::TSSatisfiesExpression(
                CloneIn::clone_in(it, allocator),
            ),
            Self::TSNonNullExpression(it) => {
                AssignmentTargetMaybeDefault::TSNonNullExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSTypeAssertion(it) => {
                AssignmentTargetMaybeDefault::TSTypeAssertion(CloneIn::clone_in(it, allocator))
            }
            Self::TSInstantiationExpression(it) => {
                AssignmentTargetMaybeDefault::TSInstantiationExpression(CloneIn::clone_in(
                    it, allocator,
                ))
            }
            Self::ComputedMemberExpression(it) => {
                AssignmentTargetMaybeDefault::ComputedMemberExpression(CloneIn::clone_in(
                    it, allocator,
                ))
            }
            Self::StaticMemberExpression(it) => {
                AssignmentTargetMaybeDefault::StaticMemberExpression(CloneIn::clone_in(
                    it, allocator,
                ))
            }
            Self::PrivateFieldExpression(it) => {
                AssignmentTargetMaybeDefault::PrivateFieldExpression(CloneIn::clone_in(
                    it, allocator,
                ))
            }
            Self::ArrayAssignmentTarget(it) => AssignmentTargetMaybeDefault::ArrayAssignmentTarget(
                CloneIn::clone_in(it, allocator),
            ),
            Self::ObjectAssignmentTarget(it) => {
                AssignmentTargetMaybeDefault::ObjectAssignmentTarget(CloneIn::clone_in(
                    it, allocator,
                ))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTargetWithDefault<'old_alloc> {
    type Cloned = AssignmentTargetWithDefault<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentTargetWithDefault {
            span: CloneIn::clone_in(&self.span, allocator),
            binding: CloneIn::clone_in(&self.binding, allocator),
            init: CloneIn::clone_in(&self.init, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTargetProperty<'old_alloc> {
    type Cloned = AssignmentTargetProperty<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => {
                AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(CloneIn::clone_in(
                    it, allocator,
                ))
            }
            Self::AssignmentTargetPropertyProperty(it) => {
                AssignmentTargetProperty::AssignmentTargetPropertyProperty(CloneIn::clone_in(
                    it, allocator,
                ))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc>
    for AssignmentTargetPropertyIdentifier<'old_alloc>
{
    type Cloned = AssignmentTargetPropertyIdentifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentTargetPropertyIdentifier {
            span: CloneIn::clone_in(&self.span, allocator),
            binding: CloneIn::clone_in(&self.binding, allocator),
            init: CloneIn::clone_in(&self.init, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTargetPropertyProperty<'old_alloc> {
    type Cloned = AssignmentTargetPropertyProperty<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentTargetPropertyProperty {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
            binding: CloneIn::clone_in(&self.binding, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for SequenceExpression<'old_alloc> {
    type Cloned = SequenceExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        SequenceExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            expressions: CloneIn::clone_in(&self.expressions, allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for Super {
    type Cloned = Super;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        Super { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AwaitExpression<'old_alloc> {
    type Cloned = AwaitExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AwaitExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            argument: CloneIn::clone_in(&self.argument, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ChainExpression<'old_alloc> {
    type Cloned = ChainExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ChainExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ChainElement<'old_alloc> {
    type Cloned = ChainElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::CallExpression(it) => {
                ChainElement::CallExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ComputedMemberExpression(it) => {
                ChainElement::ComputedMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::StaticMemberExpression(it) => {
                ChainElement::StaticMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateFieldExpression(it) => {
                ChainElement::PrivateFieldExpression(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ParenthesizedExpression<'old_alloc> {
    type Cloned = ParenthesizedExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ParenthesizedExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Statement<'old_alloc> {
    type Cloned = Statement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::BlockStatement(it) => Statement::BlockStatement(CloneIn::clone_in(it, allocator)),
            Self::BreakStatement(it) => Statement::BreakStatement(CloneIn::clone_in(it, allocator)),
            Self::ContinueStatement(it) => {
                Statement::ContinueStatement(CloneIn::clone_in(it, allocator))
            }
            Self::DebuggerStatement(it) => {
                Statement::DebuggerStatement(CloneIn::clone_in(it, allocator))
            }
            Self::DoWhileStatement(it) => {
                Statement::DoWhileStatement(CloneIn::clone_in(it, allocator))
            }
            Self::EmptyStatement(it) => Statement::EmptyStatement(CloneIn::clone_in(it, allocator)),
            Self::ExpressionStatement(it) => {
                Statement::ExpressionStatement(CloneIn::clone_in(it, allocator))
            }
            Self::ForInStatement(it) => Statement::ForInStatement(CloneIn::clone_in(it, allocator)),
            Self::ForOfStatement(it) => Statement::ForOfStatement(CloneIn::clone_in(it, allocator)),
            Self::ForStatement(it) => Statement::ForStatement(CloneIn::clone_in(it, allocator)),
            Self::IfStatement(it) => Statement::IfStatement(CloneIn::clone_in(it, allocator)),
            Self::LabeledStatement(it) => {
                Statement::LabeledStatement(CloneIn::clone_in(it, allocator))
            }
            Self::ReturnStatement(it) => {
                Statement::ReturnStatement(CloneIn::clone_in(it, allocator))
            }
            Self::SwitchStatement(it) => {
                Statement::SwitchStatement(CloneIn::clone_in(it, allocator))
            }
            Self::ThrowStatement(it) => Statement::ThrowStatement(CloneIn::clone_in(it, allocator)),
            Self::TryStatement(it) => Statement::TryStatement(CloneIn::clone_in(it, allocator)),
            Self::WhileStatement(it) => Statement::WhileStatement(CloneIn::clone_in(it, allocator)),
            Self::WithStatement(it) => Statement::WithStatement(CloneIn::clone_in(it, allocator)),
            Self::VariableDeclaration(it) => {
                Statement::VariableDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::FunctionDeclaration(it) => {
                Statement::FunctionDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::ClassDeclaration(it) => {
                Statement::ClassDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::TSTypeAliasDeclaration(it) => {
                Statement::TSTypeAliasDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::TSInterfaceDeclaration(it) => {
                Statement::TSInterfaceDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::TSEnumDeclaration(it) => {
                Statement::TSEnumDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::TSModuleDeclaration(it) => {
                Statement::TSModuleDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::TSImportEqualsDeclaration(it) => {
                Statement::TSImportEqualsDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::ImportDeclaration(it) => {
                Statement::ImportDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::ExportAllDeclaration(it) => {
                Statement::ExportAllDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::ExportDefaultDeclaration(it) => {
                Statement::ExportDefaultDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::ExportNamedDeclaration(it) => {
                Statement::ExportNamedDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::TSExportAssignment(it) => {
                Statement::TSExportAssignment(CloneIn::clone_in(it, allocator))
            }
            Self::TSNamespaceExportDeclaration(it) => {
                Statement::TSNamespaceExportDeclaration(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Directive<'old_alloc> {
    type Cloned = Directive<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Directive {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
            directive: CloneIn::clone_in(&self.directive, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Hashbang<'old_alloc> {
    type Cloned = Hashbang<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Hashbang {
            span: CloneIn::clone_in(&self.span, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BlockStatement<'old_alloc> {
    type Cloned = BlockStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BlockStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Declaration<'old_alloc> {
    type Cloned = Declaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::VariableDeclaration(it) => {
                Declaration::VariableDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::FunctionDeclaration(it) => {
                Declaration::FunctionDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::ClassDeclaration(it) => {
                Declaration::ClassDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::TSTypeAliasDeclaration(it) => {
                Declaration::TSTypeAliasDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::TSInterfaceDeclaration(it) => {
                Declaration::TSInterfaceDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::TSEnumDeclaration(it) => {
                Declaration::TSEnumDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::TSModuleDeclaration(it) => {
                Declaration::TSModuleDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::TSImportEqualsDeclaration(it) => {
                Declaration::TSImportEqualsDeclaration(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for VariableDeclaration<'old_alloc> {
    type Cloned = VariableDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        VariableDeclaration {
            span: CloneIn::clone_in(&self.span, allocator),
            kind: CloneIn::clone_in(&self.kind, allocator),
            declarations: CloneIn::clone_in(&self.declarations, allocator),
            declare: CloneIn::clone_in(&self.declare, allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for VariableDeclarationKind {
    type Cloned = VariableDeclarationKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Var => VariableDeclarationKind::Var,
            Self::Const => VariableDeclarationKind::Const,
            Self::Let => VariableDeclarationKind::Let,
            Self::Using => VariableDeclarationKind::Using,
            Self::AwaitUsing => VariableDeclarationKind::AwaitUsing,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for VariableDeclarator<'old_alloc> {
    type Cloned = VariableDeclarator<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        VariableDeclarator {
            span: CloneIn::clone_in(&self.span, allocator),
            kind: CloneIn::clone_in(&self.kind, allocator),
            id: CloneIn::clone_in(&self.id, allocator),
            init: CloneIn::clone_in(&self.init, allocator),
            definite: CloneIn::clone_in(&self.definite, allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for EmptyStatement {
    type Cloned = EmptyStatement;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        EmptyStatement { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ExpressionStatement<'old_alloc> {
    type Cloned = ExpressionStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ExpressionStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for IfStatement<'old_alloc> {
    type Cloned = IfStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        IfStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            test: CloneIn::clone_in(&self.test, allocator),
            consequent: CloneIn::clone_in(&self.consequent, allocator),
            alternate: CloneIn::clone_in(&self.alternate, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for DoWhileStatement<'old_alloc> {
    type Cloned = DoWhileStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        DoWhileStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
            test: CloneIn::clone_in(&self.test, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for WhileStatement<'old_alloc> {
    type Cloned = WhileStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        WhileStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            test: CloneIn::clone_in(&self.test, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ForStatement<'old_alloc> {
    type Cloned = ForStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ForStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            init: CloneIn::clone_in(&self.init, allocator),
            test: CloneIn::clone_in(&self.test, allocator),
            update: CloneIn::clone_in(&self.update, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ForStatementInit<'old_alloc> {
    type Cloned = ForStatementInit<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::VariableDeclaration(it) => {
                ForStatementInit::VariableDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::BooleanLiteral(it) => {
                ForStatementInit::BooleanLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::NullLiteral(it) => {
                ForStatementInit::NullLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::NumericLiteral(it) => {
                ForStatementInit::NumericLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::BigIntLiteral(it) => {
                ForStatementInit::BigIntLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::RegExpLiteral(it) => {
                ForStatementInit::RegExpLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::StringLiteral(it) => {
                ForStatementInit::StringLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::TemplateLiteral(it) => {
                ForStatementInit::TemplateLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::Identifier(it) => ForStatementInit::Identifier(CloneIn::clone_in(it, allocator)),
            Self::MetaProperty(it) => {
                ForStatementInit::MetaProperty(CloneIn::clone_in(it, allocator))
            }
            Self::Super(it) => ForStatementInit::Super(CloneIn::clone_in(it, allocator)),
            Self::ArrayExpression(it) => {
                ForStatementInit::ArrayExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ArrowFunctionExpression(it) => {
                ForStatementInit::ArrowFunctionExpression(CloneIn::clone_in(it, allocator))
            }
            Self::AssignmentExpression(it) => {
                ForStatementInit::AssignmentExpression(CloneIn::clone_in(it, allocator))
            }
            Self::AwaitExpression(it) => {
                ForStatementInit::AwaitExpression(CloneIn::clone_in(it, allocator))
            }
            Self::BinaryExpression(it) => {
                ForStatementInit::BinaryExpression(CloneIn::clone_in(it, allocator))
            }
            Self::CallExpression(it) => {
                ForStatementInit::CallExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ChainExpression(it) => {
                ForStatementInit::ChainExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ClassExpression(it) => {
                ForStatementInit::ClassExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ConditionalExpression(it) => {
                ForStatementInit::ConditionalExpression(CloneIn::clone_in(it, allocator))
            }
            Self::FunctionExpression(it) => {
                ForStatementInit::FunctionExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ImportExpression(it) => {
                ForStatementInit::ImportExpression(CloneIn::clone_in(it, allocator))
            }
            Self::LogicalExpression(it) => {
                ForStatementInit::LogicalExpression(CloneIn::clone_in(it, allocator))
            }
            Self::NewExpression(it) => {
                ForStatementInit::NewExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ObjectExpression(it) => {
                ForStatementInit::ObjectExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ParenthesizedExpression(it) => {
                ForStatementInit::ParenthesizedExpression(CloneIn::clone_in(it, allocator))
            }
            Self::SequenceExpression(it) => {
                ForStatementInit::SequenceExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TaggedTemplateExpression(it) => {
                ForStatementInit::TaggedTemplateExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ThisExpression(it) => {
                ForStatementInit::ThisExpression(CloneIn::clone_in(it, allocator))
            }
            Self::UnaryExpression(it) => {
                ForStatementInit::UnaryExpression(CloneIn::clone_in(it, allocator))
            }
            Self::UpdateExpression(it) => {
                ForStatementInit::UpdateExpression(CloneIn::clone_in(it, allocator))
            }
            Self::YieldExpression(it) => {
                ForStatementInit::YieldExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateInExpression(it) => {
                ForStatementInit::PrivateInExpression(CloneIn::clone_in(it, allocator))
            }
            Self::JSXElement(it) => ForStatementInit::JSXElement(CloneIn::clone_in(it, allocator)),
            Self::JSXFragment(it) => {
                ForStatementInit::JSXFragment(CloneIn::clone_in(it, allocator))
            }
            Self::TSAsExpression(it) => {
                ForStatementInit::TSAsExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSSatisfiesExpression(it) => {
                ForStatementInit::TSSatisfiesExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSTypeAssertion(it) => {
                ForStatementInit::TSTypeAssertion(CloneIn::clone_in(it, allocator))
            }
            Self::TSNonNullExpression(it) => {
                ForStatementInit::TSNonNullExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSInstantiationExpression(it) => {
                ForStatementInit::TSInstantiationExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ComputedMemberExpression(it) => {
                ForStatementInit::ComputedMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::StaticMemberExpression(it) => {
                ForStatementInit::StaticMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateFieldExpression(it) => {
                ForStatementInit::PrivateFieldExpression(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ForInStatement<'old_alloc> {
    type Cloned = ForInStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ForInStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            left: CloneIn::clone_in(&self.left, allocator),
            right: CloneIn::clone_in(&self.right, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ForStatementLeft<'old_alloc> {
    type Cloned = ForStatementLeft<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::VariableDeclaration(it) => {
                ForStatementLeft::VariableDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::AssignmentTargetIdentifier(it) => {
                ForStatementLeft::AssignmentTargetIdentifier(CloneIn::clone_in(it, allocator))
            }
            Self::TSAsExpression(it) => {
                ForStatementLeft::TSAsExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSSatisfiesExpression(it) => {
                ForStatementLeft::TSSatisfiesExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSNonNullExpression(it) => {
                ForStatementLeft::TSNonNullExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSTypeAssertion(it) => {
                ForStatementLeft::TSTypeAssertion(CloneIn::clone_in(it, allocator))
            }
            Self::TSInstantiationExpression(it) => {
                ForStatementLeft::TSInstantiationExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ComputedMemberExpression(it) => {
                ForStatementLeft::ComputedMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::StaticMemberExpression(it) => {
                ForStatementLeft::StaticMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateFieldExpression(it) => {
                ForStatementLeft::PrivateFieldExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ArrayAssignmentTarget(it) => {
                ForStatementLeft::ArrayAssignmentTarget(CloneIn::clone_in(it, allocator))
            }
            Self::ObjectAssignmentTarget(it) => {
                ForStatementLeft::ObjectAssignmentTarget(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ForOfStatement<'old_alloc> {
    type Cloned = ForOfStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ForOfStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            r#await: CloneIn::clone_in(&self.r#await, allocator),
            left: CloneIn::clone_in(&self.left, allocator),
            right: CloneIn::clone_in(&self.right, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ContinueStatement<'old_alloc> {
    type Cloned = ContinueStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ContinueStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            label: CloneIn::clone_in(&self.label, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BreakStatement<'old_alloc> {
    type Cloned = BreakStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BreakStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            label: CloneIn::clone_in(&self.label, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ReturnStatement<'old_alloc> {
    type Cloned = ReturnStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ReturnStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            argument: CloneIn::clone_in(&self.argument, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for WithStatement<'old_alloc> {
    type Cloned = WithStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        WithStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            object: CloneIn::clone_in(&self.object, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for SwitchStatement<'old_alloc> {
    type Cloned = SwitchStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        SwitchStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            discriminant: CloneIn::clone_in(&self.discriminant, allocator),
            cases: CloneIn::clone_in(&self.cases, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for SwitchCase<'old_alloc> {
    type Cloned = SwitchCase<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        SwitchCase {
            span: CloneIn::clone_in(&self.span, allocator),
            test: CloneIn::clone_in(&self.test, allocator),
            consequent: CloneIn::clone_in(&self.consequent, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for LabeledStatement<'old_alloc> {
    type Cloned = LabeledStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        LabeledStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            label: CloneIn::clone_in(&self.label, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ThrowStatement<'old_alloc> {
    type Cloned = ThrowStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ThrowStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            argument: CloneIn::clone_in(&self.argument, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TryStatement<'old_alloc> {
    type Cloned = TryStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TryStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            block: CloneIn::clone_in(&self.block, allocator),
            handler: CloneIn::clone_in(&self.handler, allocator),
            finalizer: CloneIn::clone_in(&self.finalizer, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for CatchClause<'old_alloc> {
    type Cloned = CatchClause<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        CatchClause {
            span: CloneIn::clone_in(&self.span, allocator),
            param: CloneIn::clone_in(&self.param, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for CatchParameter<'old_alloc> {
    type Cloned = CatchParameter<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        CatchParameter {
            span: CloneIn::clone_in(&self.span, allocator),
            pattern: CloneIn::clone_in(&self.pattern, allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for DebuggerStatement {
    type Cloned = DebuggerStatement;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        DebuggerStatement { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BindingPattern<'old_alloc> {
    type Cloned = BindingPattern<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BindingPattern {
            kind: CloneIn::clone_in(&self.kind, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
            optional: CloneIn::clone_in(&self.optional, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BindingPatternKind<'old_alloc> {
    type Cloned = BindingPatternKind<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::BindingIdentifier(it) => {
                BindingPatternKind::BindingIdentifier(CloneIn::clone_in(it, allocator))
            }
            Self::ObjectPattern(it) => {
                BindingPatternKind::ObjectPattern(CloneIn::clone_in(it, allocator))
            }
            Self::ArrayPattern(it) => {
                BindingPatternKind::ArrayPattern(CloneIn::clone_in(it, allocator))
            }
            Self::AssignmentPattern(it) => {
                BindingPatternKind::AssignmentPattern(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentPattern<'old_alloc> {
    type Cloned = AssignmentPattern<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentPattern {
            span: CloneIn::clone_in(&self.span, allocator),
            left: CloneIn::clone_in(&self.left, allocator),
            right: CloneIn::clone_in(&self.right, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ObjectPattern<'old_alloc> {
    type Cloned = ObjectPattern<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ObjectPattern {
            span: CloneIn::clone_in(&self.span, allocator),
            properties: CloneIn::clone_in(&self.properties, allocator),
            rest: CloneIn::clone_in(&self.rest, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BindingProperty<'old_alloc> {
    type Cloned = BindingProperty<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BindingProperty {
            span: CloneIn::clone_in(&self.span, allocator),
            key: CloneIn::clone_in(&self.key, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
            shorthand: CloneIn::clone_in(&self.shorthand, allocator),
            computed: CloneIn::clone_in(&self.computed, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ArrayPattern<'old_alloc> {
    type Cloned = ArrayPattern<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ArrayPattern {
            span: CloneIn::clone_in(&self.span, allocator),
            elements: CloneIn::clone_in(&self.elements, allocator),
            rest: CloneIn::clone_in(&self.rest, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BindingRestElement<'old_alloc> {
    type Cloned = BindingRestElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BindingRestElement {
            span: CloneIn::clone_in(&self.span, allocator),
            argument: CloneIn::clone_in(&self.argument, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Function<'old_alloc> {
    type Cloned = Function<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Function {
            r#type: CloneIn::clone_in(&self.r#type, allocator),
            span: CloneIn::clone_in(&self.span, allocator),
            id: CloneIn::clone_in(&self.id, allocator),
            generator: CloneIn::clone_in(&self.generator, allocator),
            r#async: CloneIn::clone_in(&self.r#async, allocator),
            declare: CloneIn::clone_in(&self.declare, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
            this_param: CloneIn::clone_in(&self.this_param, allocator),
            params: CloneIn::clone_in(&self.params, allocator),
            return_type: CloneIn::clone_in(&self.return_type, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for FunctionType {
    type Cloned = FunctionType;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::FunctionDeclaration => FunctionType::FunctionDeclaration,
            Self::FunctionExpression => FunctionType::FunctionExpression,
            Self::TSDeclareFunction => FunctionType::TSDeclareFunction,
            Self::TSEmptyBodyFunctionExpression => FunctionType::TSEmptyBodyFunctionExpression,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for FormalParameters<'old_alloc> {
    type Cloned = FormalParameters<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        FormalParameters {
            span: CloneIn::clone_in(&self.span, allocator),
            kind: CloneIn::clone_in(&self.kind, allocator),
            items: CloneIn::clone_in(&self.items, allocator),
            rest: CloneIn::clone_in(&self.rest, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for FormalParameter<'old_alloc> {
    type Cloned = FormalParameter<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        FormalParameter {
            span: CloneIn::clone_in(&self.span, allocator),
            decorators: CloneIn::clone_in(&self.decorators, allocator),
            pattern: CloneIn::clone_in(&self.pattern, allocator),
            accessibility: CloneIn::clone_in(&self.accessibility, allocator),
            readonly: CloneIn::clone_in(&self.readonly, allocator),
            r#override: CloneIn::clone_in(&self.r#override, allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for FormalParameterKind {
    type Cloned = FormalParameterKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::FormalParameter => FormalParameterKind::FormalParameter,
            Self::UniqueFormalParameters => FormalParameterKind::UniqueFormalParameters,
            Self::ArrowFormalParameters => FormalParameterKind::ArrowFormalParameters,
            Self::Signature => FormalParameterKind::Signature,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for FunctionBody<'old_alloc> {
    type Cloned = FunctionBody<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        FunctionBody {
            span: CloneIn::clone_in(&self.span, allocator),
            directives: CloneIn::clone_in(&self.directives, allocator),
            statements: CloneIn::clone_in(&self.statements, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ArrowFunctionExpression<'old_alloc> {
    type Cloned = ArrowFunctionExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ArrowFunctionExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
            r#async: CloneIn::clone_in(&self.r#async, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
            params: CloneIn::clone_in(&self.params, allocator),
            return_type: CloneIn::clone_in(&self.return_type, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for YieldExpression<'old_alloc> {
    type Cloned = YieldExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        YieldExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            delegate: CloneIn::clone_in(&self.delegate, allocator),
            argument: CloneIn::clone_in(&self.argument, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Class<'old_alloc> {
    type Cloned = Class<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Class {
            r#type: CloneIn::clone_in(&self.r#type, allocator),
            span: CloneIn::clone_in(&self.span, allocator),
            decorators: CloneIn::clone_in(&self.decorators, allocator),
            id: CloneIn::clone_in(&self.id, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
            super_class: CloneIn::clone_in(&self.super_class, allocator),
            super_type_parameters: CloneIn::clone_in(&self.super_type_parameters, allocator),
            implements: CloneIn::clone_in(&self.implements, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
            r#abstract: CloneIn::clone_in(&self.r#abstract, allocator),
            declare: CloneIn::clone_in(&self.declare, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for ClassType {
    type Cloned = ClassType;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::ClassDeclaration => ClassType::ClassDeclaration,
            Self::ClassExpression => ClassType::ClassExpression,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ClassBody<'old_alloc> {
    type Cloned = ClassBody<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ClassBody {
            span: CloneIn::clone_in(&self.span, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ClassElement<'old_alloc> {
    type Cloned = ClassElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::StaticBlock(it) => ClassElement::StaticBlock(CloneIn::clone_in(it, allocator)),
            Self::MethodDefinition(it) => {
                ClassElement::MethodDefinition(CloneIn::clone_in(it, allocator))
            }
            Self::PropertyDefinition(it) => {
                ClassElement::PropertyDefinition(CloneIn::clone_in(it, allocator))
            }
            Self::AccessorProperty(it) => {
                ClassElement::AccessorProperty(CloneIn::clone_in(it, allocator))
            }
            Self::TSIndexSignature(it) => {
                ClassElement::TSIndexSignature(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for MethodDefinition<'old_alloc> {
    type Cloned = MethodDefinition<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        MethodDefinition {
            r#type: CloneIn::clone_in(&self.r#type, allocator),
            span: CloneIn::clone_in(&self.span, allocator),
            decorators: CloneIn::clone_in(&self.decorators, allocator),
            key: CloneIn::clone_in(&self.key, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
            kind: CloneIn::clone_in(&self.kind, allocator),
            computed: CloneIn::clone_in(&self.computed, allocator),
            r#static: CloneIn::clone_in(&self.r#static, allocator),
            r#override: CloneIn::clone_in(&self.r#override, allocator),
            optional: CloneIn::clone_in(&self.optional, allocator),
            accessibility: CloneIn::clone_in(&self.accessibility, allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for MethodDefinitionType {
    type Cloned = MethodDefinitionType;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::MethodDefinition => MethodDefinitionType::MethodDefinition,
            Self::TSAbstractMethodDefinition => MethodDefinitionType::TSAbstractMethodDefinition,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for PropertyDefinition<'old_alloc> {
    type Cloned = PropertyDefinition<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        PropertyDefinition {
            r#type: CloneIn::clone_in(&self.r#type, allocator),
            span: CloneIn::clone_in(&self.span, allocator),
            decorators: CloneIn::clone_in(&self.decorators, allocator),
            key: CloneIn::clone_in(&self.key, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
            computed: CloneIn::clone_in(&self.computed, allocator),
            r#static: CloneIn::clone_in(&self.r#static, allocator),
            declare: CloneIn::clone_in(&self.declare, allocator),
            r#override: CloneIn::clone_in(&self.r#override, allocator),
            optional: CloneIn::clone_in(&self.optional, allocator),
            definite: CloneIn::clone_in(&self.definite, allocator),
            readonly: CloneIn::clone_in(&self.readonly, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
            accessibility: CloneIn::clone_in(&self.accessibility, allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for PropertyDefinitionType {
    type Cloned = PropertyDefinitionType;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::PropertyDefinition => PropertyDefinitionType::PropertyDefinition,
            Self::TSAbstractPropertyDefinition => {
                PropertyDefinitionType::TSAbstractPropertyDefinition
            }
        }
    }
}

impl<'alloc> CloneIn<'alloc> for MethodDefinitionKind {
    type Cloned = MethodDefinitionKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Constructor => MethodDefinitionKind::Constructor,
            Self::Method => MethodDefinitionKind::Method,
            Self::Get => MethodDefinitionKind::Get,
            Self::Set => MethodDefinitionKind::Set,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for PrivateIdentifier<'old_alloc> {
    type Cloned = PrivateIdentifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        PrivateIdentifier {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for StaticBlock<'old_alloc> {
    type Cloned = StaticBlock<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        StaticBlock {
            span: CloneIn::clone_in(&self.span, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ModuleDeclaration<'old_alloc> {
    type Cloned = ModuleDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::ImportDeclaration(it) => {
                ModuleDeclaration::ImportDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::ExportAllDeclaration(it) => {
                ModuleDeclaration::ExportAllDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::ExportDefaultDeclaration(it) => {
                ModuleDeclaration::ExportDefaultDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::ExportNamedDeclaration(it) => {
                ModuleDeclaration::ExportNamedDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::TSExportAssignment(it) => {
                ModuleDeclaration::TSExportAssignment(CloneIn::clone_in(it, allocator))
            }
            Self::TSNamespaceExportDeclaration(it) => {
                ModuleDeclaration::TSNamespaceExportDeclaration(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'alloc> CloneIn<'alloc> for AccessorPropertyType {
    type Cloned = AccessorPropertyType;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::AccessorProperty => AccessorPropertyType::AccessorProperty,
            Self::TSAbstractAccessorProperty => AccessorPropertyType::TSAbstractAccessorProperty,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AccessorProperty<'old_alloc> {
    type Cloned = AccessorProperty<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AccessorProperty {
            r#type: CloneIn::clone_in(&self.r#type, allocator),
            span: CloneIn::clone_in(&self.span, allocator),
            decorators: CloneIn::clone_in(&self.decorators, allocator),
            key: CloneIn::clone_in(&self.key, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
            computed: CloneIn::clone_in(&self.computed, allocator),
            r#static: CloneIn::clone_in(&self.r#static, allocator),
            definite: CloneIn::clone_in(&self.definite, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
            accessibility: CloneIn::clone_in(&self.accessibility, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportExpression<'old_alloc> {
    type Cloned = ImportExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ImportExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            source: CloneIn::clone_in(&self.source, allocator),
            arguments: CloneIn::clone_in(&self.arguments, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportDeclaration<'old_alloc> {
    type Cloned = ImportDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ImportDeclaration {
            span: CloneIn::clone_in(&self.span, allocator),
            specifiers: CloneIn::clone_in(&self.specifiers, allocator),
            source: CloneIn::clone_in(&self.source, allocator),
            with_clause: CloneIn::clone_in(&self.with_clause, allocator),
            import_kind: CloneIn::clone_in(&self.import_kind, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportDeclarationSpecifier<'old_alloc> {
    type Cloned = ImportDeclarationSpecifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::ImportSpecifier(it) => {
                ImportDeclarationSpecifier::ImportSpecifier(CloneIn::clone_in(it, allocator))
            }
            Self::ImportDefaultSpecifier(it) => {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(CloneIn::clone_in(it, allocator))
            }
            Self::ImportNamespaceSpecifier(it) => {
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(CloneIn::clone_in(
                    it, allocator,
                ))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportSpecifier<'old_alloc> {
    type Cloned = ImportSpecifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ImportSpecifier {
            span: CloneIn::clone_in(&self.span, allocator),
            imported: CloneIn::clone_in(&self.imported, allocator),
            local: CloneIn::clone_in(&self.local, allocator),
            import_kind: CloneIn::clone_in(&self.import_kind, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportDefaultSpecifier<'old_alloc> {
    type Cloned = ImportDefaultSpecifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ImportDefaultSpecifier {
            span: CloneIn::clone_in(&self.span, allocator),
            local: CloneIn::clone_in(&self.local, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportNamespaceSpecifier<'old_alloc> {
    type Cloned = ImportNamespaceSpecifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ImportNamespaceSpecifier {
            span: CloneIn::clone_in(&self.span, allocator),
            local: CloneIn::clone_in(&self.local, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for WithClause<'old_alloc> {
    type Cloned = WithClause<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        WithClause {
            span: CloneIn::clone_in(&self.span, allocator),
            attributes_keyword: CloneIn::clone_in(&self.attributes_keyword, allocator),
            with_entries: CloneIn::clone_in(&self.with_entries, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportAttribute<'old_alloc> {
    type Cloned = ImportAttribute<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ImportAttribute {
            span: CloneIn::clone_in(&self.span, allocator),
            key: CloneIn::clone_in(&self.key, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportAttributeKey<'old_alloc> {
    type Cloned = ImportAttributeKey<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => {
                ImportAttributeKey::Identifier(CloneIn::clone_in(it, allocator))
            }
            Self::StringLiteral(it) => {
                ImportAttributeKey::StringLiteral(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ExportNamedDeclaration<'old_alloc> {
    type Cloned = ExportNamedDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ExportNamedDeclaration {
            span: CloneIn::clone_in(&self.span, allocator),
            declaration: CloneIn::clone_in(&self.declaration, allocator),
            specifiers: CloneIn::clone_in(&self.specifiers, allocator),
            source: CloneIn::clone_in(&self.source, allocator),
            export_kind: CloneIn::clone_in(&self.export_kind, allocator),
            with_clause: CloneIn::clone_in(&self.with_clause, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ExportDefaultDeclaration<'old_alloc> {
    type Cloned = ExportDefaultDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ExportDefaultDeclaration {
            span: CloneIn::clone_in(&self.span, allocator),
            declaration: CloneIn::clone_in(&self.declaration, allocator),
            exported: CloneIn::clone_in(&self.exported, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ExportAllDeclaration<'old_alloc> {
    type Cloned = ExportAllDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ExportAllDeclaration {
            span: CloneIn::clone_in(&self.span, allocator),
            exported: CloneIn::clone_in(&self.exported, allocator),
            source: CloneIn::clone_in(&self.source, allocator),
            with_clause: CloneIn::clone_in(&self.with_clause, allocator),
            export_kind: CloneIn::clone_in(&self.export_kind, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ExportSpecifier<'old_alloc> {
    type Cloned = ExportSpecifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ExportSpecifier {
            span: CloneIn::clone_in(&self.span, allocator),
            local: CloneIn::clone_in(&self.local, allocator),
            exported: CloneIn::clone_in(&self.exported, allocator),
            export_kind: CloneIn::clone_in(&self.export_kind, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ExportDefaultDeclarationKind<'old_alloc> {
    type Cloned = ExportDefaultDeclarationKind<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::FunctionDeclaration(it) => {
                ExportDefaultDeclarationKind::FunctionDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::ClassDeclaration(it) => {
                ExportDefaultDeclarationKind::ClassDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::TSInterfaceDeclaration(it) => {
                ExportDefaultDeclarationKind::TSInterfaceDeclaration(CloneIn::clone_in(
                    it, allocator,
                ))
            }
            Self::BooleanLiteral(it) => {
                ExportDefaultDeclarationKind::BooleanLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::NullLiteral(it) => {
                ExportDefaultDeclarationKind::NullLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::NumericLiteral(it) => {
                ExportDefaultDeclarationKind::NumericLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::BigIntLiteral(it) => {
                ExportDefaultDeclarationKind::BigIntLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::RegExpLiteral(it) => {
                ExportDefaultDeclarationKind::RegExpLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::StringLiteral(it) => {
                ExportDefaultDeclarationKind::StringLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::TemplateLiteral(it) => {
                ExportDefaultDeclarationKind::TemplateLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::Identifier(it) => {
                ExportDefaultDeclarationKind::Identifier(CloneIn::clone_in(it, allocator))
            }
            Self::MetaProperty(it) => {
                ExportDefaultDeclarationKind::MetaProperty(CloneIn::clone_in(it, allocator))
            }
            Self::Super(it) => {
                ExportDefaultDeclarationKind::Super(CloneIn::clone_in(it, allocator))
            }
            Self::ArrayExpression(it) => {
                ExportDefaultDeclarationKind::ArrayExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ArrowFunctionExpression(it) => {
                ExportDefaultDeclarationKind::ArrowFunctionExpression(CloneIn::clone_in(
                    it, allocator,
                ))
            }
            Self::AssignmentExpression(it) => {
                ExportDefaultDeclarationKind::AssignmentExpression(CloneIn::clone_in(it, allocator))
            }
            Self::AwaitExpression(it) => {
                ExportDefaultDeclarationKind::AwaitExpression(CloneIn::clone_in(it, allocator))
            }
            Self::BinaryExpression(it) => {
                ExportDefaultDeclarationKind::BinaryExpression(CloneIn::clone_in(it, allocator))
            }
            Self::CallExpression(it) => {
                ExportDefaultDeclarationKind::CallExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ChainExpression(it) => {
                ExportDefaultDeclarationKind::ChainExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ClassExpression(it) => {
                ExportDefaultDeclarationKind::ClassExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ConditionalExpression(it) => ExportDefaultDeclarationKind::ConditionalExpression(
                CloneIn::clone_in(it, allocator),
            ),
            Self::FunctionExpression(it) => {
                ExportDefaultDeclarationKind::FunctionExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ImportExpression(it) => {
                ExportDefaultDeclarationKind::ImportExpression(CloneIn::clone_in(it, allocator))
            }
            Self::LogicalExpression(it) => {
                ExportDefaultDeclarationKind::LogicalExpression(CloneIn::clone_in(it, allocator))
            }
            Self::NewExpression(it) => {
                ExportDefaultDeclarationKind::NewExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ObjectExpression(it) => {
                ExportDefaultDeclarationKind::ObjectExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ParenthesizedExpression(it) => {
                ExportDefaultDeclarationKind::ParenthesizedExpression(CloneIn::clone_in(
                    it, allocator,
                ))
            }
            Self::SequenceExpression(it) => {
                ExportDefaultDeclarationKind::SequenceExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TaggedTemplateExpression(it) => {
                ExportDefaultDeclarationKind::TaggedTemplateExpression(CloneIn::clone_in(
                    it, allocator,
                ))
            }
            Self::ThisExpression(it) => {
                ExportDefaultDeclarationKind::ThisExpression(CloneIn::clone_in(it, allocator))
            }
            Self::UnaryExpression(it) => {
                ExportDefaultDeclarationKind::UnaryExpression(CloneIn::clone_in(it, allocator))
            }
            Self::UpdateExpression(it) => {
                ExportDefaultDeclarationKind::UpdateExpression(CloneIn::clone_in(it, allocator))
            }
            Self::YieldExpression(it) => {
                ExportDefaultDeclarationKind::YieldExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateInExpression(it) => {
                ExportDefaultDeclarationKind::PrivateInExpression(CloneIn::clone_in(it, allocator))
            }
            Self::JSXElement(it) => {
                ExportDefaultDeclarationKind::JSXElement(CloneIn::clone_in(it, allocator))
            }
            Self::JSXFragment(it) => {
                ExportDefaultDeclarationKind::JSXFragment(CloneIn::clone_in(it, allocator))
            }
            Self::TSAsExpression(it) => {
                ExportDefaultDeclarationKind::TSAsExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSSatisfiesExpression(it) => ExportDefaultDeclarationKind::TSSatisfiesExpression(
                CloneIn::clone_in(it, allocator),
            ),
            Self::TSTypeAssertion(it) => {
                ExportDefaultDeclarationKind::TSTypeAssertion(CloneIn::clone_in(it, allocator))
            }
            Self::TSNonNullExpression(it) => {
                ExportDefaultDeclarationKind::TSNonNullExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSInstantiationExpression(it) => {
                ExportDefaultDeclarationKind::TSInstantiationExpression(CloneIn::clone_in(
                    it, allocator,
                ))
            }
            Self::ComputedMemberExpression(it) => {
                ExportDefaultDeclarationKind::ComputedMemberExpression(CloneIn::clone_in(
                    it, allocator,
                ))
            }
            Self::StaticMemberExpression(it) => {
                ExportDefaultDeclarationKind::StaticMemberExpression(CloneIn::clone_in(
                    it, allocator,
                ))
            }
            Self::PrivateFieldExpression(it) => {
                ExportDefaultDeclarationKind::PrivateFieldExpression(CloneIn::clone_in(
                    it, allocator,
                ))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ModuleExportName<'old_alloc> {
    type Cloned = ModuleExportName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::IdentifierName(it) => {
                ModuleExportName::IdentifierName(CloneIn::clone_in(it, allocator))
            }
            Self::IdentifierReference(it) => {
                ModuleExportName::IdentifierReference(CloneIn::clone_in(it, allocator))
            }
            Self::StringLiteral(it) => {
                ModuleExportName::StringLiteral(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSThisParameter<'old_alloc> {
    type Cloned = TSThisParameter<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSThisParameter {
            span: CloneIn::clone_in(&self.span, allocator),
            this_span: CloneIn::clone_in(&self.this_span, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSEnumDeclaration<'old_alloc> {
    type Cloned = TSEnumDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSEnumDeclaration {
            span: CloneIn::clone_in(&self.span, allocator),
            id: CloneIn::clone_in(&self.id, allocator),
            members: CloneIn::clone_in(&self.members, allocator),
            r#const: CloneIn::clone_in(&self.r#const, allocator),
            declare: CloneIn::clone_in(&self.declare, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSEnumMember<'old_alloc> {
    type Cloned = TSEnumMember<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSEnumMember {
            span: CloneIn::clone_in(&self.span, allocator),
            id: CloneIn::clone_in(&self.id, allocator),
            initializer: CloneIn::clone_in(&self.initializer, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSEnumMemberName<'old_alloc> {
    type Cloned = TSEnumMemberName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::StaticIdentifier(it) => {
                TSEnumMemberName::StaticIdentifier(CloneIn::clone_in(it, allocator))
            }
            Self::StaticStringLiteral(it) => {
                TSEnumMemberName::StaticStringLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::StaticTemplateLiteral(it) => {
                TSEnumMemberName::StaticTemplateLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::StaticNumericLiteral(it) => {
                TSEnumMemberName::StaticNumericLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::BooleanLiteral(it) => {
                TSEnumMemberName::BooleanLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::NullLiteral(it) => {
                TSEnumMemberName::NullLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::NumericLiteral(it) => {
                TSEnumMemberName::NumericLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::BigIntLiteral(it) => {
                TSEnumMemberName::BigIntLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::RegExpLiteral(it) => {
                TSEnumMemberName::RegExpLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::StringLiteral(it) => {
                TSEnumMemberName::StringLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::TemplateLiteral(it) => {
                TSEnumMemberName::TemplateLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::Identifier(it) => TSEnumMemberName::Identifier(CloneIn::clone_in(it, allocator)),
            Self::MetaProperty(it) => {
                TSEnumMemberName::MetaProperty(CloneIn::clone_in(it, allocator))
            }
            Self::Super(it) => TSEnumMemberName::Super(CloneIn::clone_in(it, allocator)),
            Self::ArrayExpression(it) => {
                TSEnumMemberName::ArrayExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ArrowFunctionExpression(it) => {
                TSEnumMemberName::ArrowFunctionExpression(CloneIn::clone_in(it, allocator))
            }
            Self::AssignmentExpression(it) => {
                TSEnumMemberName::AssignmentExpression(CloneIn::clone_in(it, allocator))
            }
            Self::AwaitExpression(it) => {
                TSEnumMemberName::AwaitExpression(CloneIn::clone_in(it, allocator))
            }
            Self::BinaryExpression(it) => {
                TSEnumMemberName::BinaryExpression(CloneIn::clone_in(it, allocator))
            }
            Self::CallExpression(it) => {
                TSEnumMemberName::CallExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ChainExpression(it) => {
                TSEnumMemberName::ChainExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ClassExpression(it) => {
                TSEnumMemberName::ClassExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ConditionalExpression(it) => {
                TSEnumMemberName::ConditionalExpression(CloneIn::clone_in(it, allocator))
            }
            Self::FunctionExpression(it) => {
                TSEnumMemberName::FunctionExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ImportExpression(it) => {
                TSEnumMemberName::ImportExpression(CloneIn::clone_in(it, allocator))
            }
            Self::LogicalExpression(it) => {
                TSEnumMemberName::LogicalExpression(CloneIn::clone_in(it, allocator))
            }
            Self::NewExpression(it) => {
                TSEnumMemberName::NewExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ObjectExpression(it) => {
                TSEnumMemberName::ObjectExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ParenthesizedExpression(it) => {
                TSEnumMemberName::ParenthesizedExpression(CloneIn::clone_in(it, allocator))
            }
            Self::SequenceExpression(it) => {
                TSEnumMemberName::SequenceExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TaggedTemplateExpression(it) => {
                TSEnumMemberName::TaggedTemplateExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ThisExpression(it) => {
                TSEnumMemberName::ThisExpression(CloneIn::clone_in(it, allocator))
            }
            Self::UnaryExpression(it) => {
                TSEnumMemberName::UnaryExpression(CloneIn::clone_in(it, allocator))
            }
            Self::UpdateExpression(it) => {
                TSEnumMemberName::UpdateExpression(CloneIn::clone_in(it, allocator))
            }
            Self::YieldExpression(it) => {
                TSEnumMemberName::YieldExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateInExpression(it) => {
                TSEnumMemberName::PrivateInExpression(CloneIn::clone_in(it, allocator))
            }
            Self::JSXElement(it) => TSEnumMemberName::JSXElement(CloneIn::clone_in(it, allocator)),
            Self::JSXFragment(it) => {
                TSEnumMemberName::JSXFragment(CloneIn::clone_in(it, allocator))
            }
            Self::TSAsExpression(it) => {
                TSEnumMemberName::TSAsExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSSatisfiesExpression(it) => {
                TSEnumMemberName::TSSatisfiesExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSTypeAssertion(it) => {
                TSEnumMemberName::TSTypeAssertion(CloneIn::clone_in(it, allocator))
            }
            Self::TSNonNullExpression(it) => {
                TSEnumMemberName::TSNonNullExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSInstantiationExpression(it) => {
                TSEnumMemberName::TSInstantiationExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ComputedMemberExpression(it) => {
                TSEnumMemberName::ComputedMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::StaticMemberExpression(it) => {
                TSEnumMemberName::StaticMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateFieldExpression(it) => {
                TSEnumMemberName::PrivateFieldExpression(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeAnnotation<'old_alloc> {
    type Cloned = TSTypeAnnotation<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeAnnotation {
            span: CloneIn::clone_in(&self.span, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSLiteralType<'old_alloc> {
    type Cloned = TSLiteralType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSLiteralType {
            span: CloneIn::clone_in(&self.span, allocator),
            literal: CloneIn::clone_in(&self.literal, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSLiteral<'old_alloc> {
    type Cloned = TSLiteral<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::BooleanLiteral(it) => TSLiteral::BooleanLiteral(CloneIn::clone_in(it, allocator)),
            Self::NullLiteral(it) => TSLiteral::NullLiteral(CloneIn::clone_in(it, allocator)),
            Self::NumericLiteral(it) => TSLiteral::NumericLiteral(CloneIn::clone_in(it, allocator)),
            Self::BigIntLiteral(it) => TSLiteral::BigIntLiteral(CloneIn::clone_in(it, allocator)),
            Self::RegExpLiteral(it) => TSLiteral::RegExpLiteral(CloneIn::clone_in(it, allocator)),
            Self::StringLiteral(it) => TSLiteral::StringLiteral(CloneIn::clone_in(it, allocator)),
            Self::TemplateLiteral(it) => {
                TSLiteral::TemplateLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::UnaryExpression(it) => {
                TSLiteral::UnaryExpression(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSType<'old_alloc> {
    type Cloned = TSType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::TSAnyKeyword(it) => TSType::TSAnyKeyword(CloneIn::clone_in(it, allocator)),
            Self::TSBigIntKeyword(it) => TSType::TSBigIntKeyword(CloneIn::clone_in(it, allocator)),
            Self::TSBooleanKeyword(it) => {
                TSType::TSBooleanKeyword(CloneIn::clone_in(it, allocator))
            }
            Self::TSIntrinsicKeyword(it) => {
                TSType::TSIntrinsicKeyword(CloneIn::clone_in(it, allocator))
            }
            Self::TSNeverKeyword(it) => TSType::TSNeverKeyword(CloneIn::clone_in(it, allocator)),
            Self::TSNullKeyword(it) => TSType::TSNullKeyword(CloneIn::clone_in(it, allocator)),
            Self::TSNumberKeyword(it) => TSType::TSNumberKeyword(CloneIn::clone_in(it, allocator)),
            Self::TSObjectKeyword(it) => TSType::TSObjectKeyword(CloneIn::clone_in(it, allocator)),
            Self::TSStringKeyword(it) => TSType::TSStringKeyword(CloneIn::clone_in(it, allocator)),
            Self::TSSymbolKeyword(it) => TSType::TSSymbolKeyword(CloneIn::clone_in(it, allocator)),
            Self::TSUndefinedKeyword(it) => {
                TSType::TSUndefinedKeyword(CloneIn::clone_in(it, allocator))
            }
            Self::TSUnknownKeyword(it) => {
                TSType::TSUnknownKeyword(CloneIn::clone_in(it, allocator))
            }
            Self::TSVoidKeyword(it) => TSType::TSVoidKeyword(CloneIn::clone_in(it, allocator)),
            Self::TSArrayType(it) => TSType::TSArrayType(CloneIn::clone_in(it, allocator)),
            Self::TSConditionalType(it) => {
                TSType::TSConditionalType(CloneIn::clone_in(it, allocator))
            }
            Self::TSConstructorType(it) => {
                TSType::TSConstructorType(CloneIn::clone_in(it, allocator))
            }
            Self::TSFunctionType(it) => TSType::TSFunctionType(CloneIn::clone_in(it, allocator)),
            Self::TSImportType(it) => TSType::TSImportType(CloneIn::clone_in(it, allocator)),
            Self::TSIndexedAccessType(it) => {
                TSType::TSIndexedAccessType(CloneIn::clone_in(it, allocator))
            }
            Self::TSInferType(it) => TSType::TSInferType(CloneIn::clone_in(it, allocator)),
            Self::TSIntersectionType(it) => {
                TSType::TSIntersectionType(CloneIn::clone_in(it, allocator))
            }
            Self::TSLiteralType(it) => TSType::TSLiteralType(CloneIn::clone_in(it, allocator)),
            Self::TSMappedType(it) => TSType::TSMappedType(CloneIn::clone_in(it, allocator)),
            Self::TSNamedTupleMember(it) => {
                TSType::TSNamedTupleMember(CloneIn::clone_in(it, allocator))
            }
            Self::TSQualifiedName(it) => TSType::TSQualifiedName(CloneIn::clone_in(it, allocator)),
            Self::TSTemplateLiteralType(it) => {
                TSType::TSTemplateLiteralType(CloneIn::clone_in(it, allocator))
            }
            Self::TSThisType(it) => TSType::TSThisType(CloneIn::clone_in(it, allocator)),
            Self::TSTupleType(it) => TSType::TSTupleType(CloneIn::clone_in(it, allocator)),
            Self::TSTypeLiteral(it) => TSType::TSTypeLiteral(CloneIn::clone_in(it, allocator)),
            Self::TSTypeOperatorType(it) => {
                TSType::TSTypeOperatorType(CloneIn::clone_in(it, allocator))
            }
            Self::TSTypePredicate(it) => TSType::TSTypePredicate(CloneIn::clone_in(it, allocator)),
            Self::TSTypeQuery(it) => TSType::TSTypeQuery(CloneIn::clone_in(it, allocator)),
            Self::TSTypeReference(it) => TSType::TSTypeReference(CloneIn::clone_in(it, allocator)),
            Self::TSUnionType(it) => TSType::TSUnionType(CloneIn::clone_in(it, allocator)),
            Self::TSParenthesizedType(it) => {
                TSType::TSParenthesizedType(CloneIn::clone_in(it, allocator))
            }
            Self::JSDocNullableType(it) => {
                TSType::JSDocNullableType(CloneIn::clone_in(it, allocator))
            }
            Self::JSDocNonNullableType(it) => {
                TSType::JSDocNonNullableType(CloneIn::clone_in(it, allocator))
            }
            Self::JSDocUnknownType(it) => {
                TSType::JSDocUnknownType(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSConditionalType<'old_alloc> {
    type Cloned = TSConditionalType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSConditionalType {
            span: CloneIn::clone_in(&self.span, allocator),
            check_type: CloneIn::clone_in(&self.check_type, allocator),
            extends_type: CloneIn::clone_in(&self.extends_type, allocator),
            true_type: CloneIn::clone_in(&self.true_type, allocator),
            false_type: CloneIn::clone_in(&self.false_type, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSUnionType<'old_alloc> {
    type Cloned = TSUnionType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSUnionType {
            span: CloneIn::clone_in(&self.span, allocator),
            types: CloneIn::clone_in(&self.types, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSIntersectionType<'old_alloc> {
    type Cloned = TSIntersectionType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSIntersectionType {
            span: CloneIn::clone_in(&self.span, allocator),
            types: CloneIn::clone_in(&self.types, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSParenthesizedType<'old_alloc> {
    type Cloned = TSParenthesizedType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSParenthesizedType {
            span: CloneIn::clone_in(&self.span, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeOperator<'old_alloc> {
    type Cloned = TSTypeOperator<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeOperator {
            span: CloneIn::clone_in(&self.span, allocator),
            operator: CloneIn::clone_in(&self.operator, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for TSTypeOperatorOperator {
    type Cloned = TSTypeOperatorOperator;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Keyof => TSTypeOperatorOperator::Keyof,
            Self::Unique => TSTypeOperatorOperator::Unique,
            Self::Readonly => TSTypeOperatorOperator::Readonly,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSArrayType<'old_alloc> {
    type Cloned = TSArrayType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSArrayType {
            span: CloneIn::clone_in(&self.span, allocator),
            element_type: CloneIn::clone_in(&self.element_type, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSIndexedAccessType<'old_alloc> {
    type Cloned = TSIndexedAccessType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSIndexedAccessType {
            span: CloneIn::clone_in(&self.span, allocator),
            object_type: CloneIn::clone_in(&self.object_type, allocator),
            index_type: CloneIn::clone_in(&self.index_type, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTupleType<'old_alloc> {
    type Cloned = TSTupleType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTupleType {
            span: CloneIn::clone_in(&self.span, allocator),
            element_types: CloneIn::clone_in(&self.element_types, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSNamedTupleMember<'old_alloc> {
    type Cloned = TSNamedTupleMember<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSNamedTupleMember {
            span: CloneIn::clone_in(&self.span, allocator),
            element_type: CloneIn::clone_in(&self.element_type, allocator),
            label: CloneIn::clone_in(&self.label, allocator),
            optional: CloneIn::clone_in(&self.optional, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSOptionalType<'old_alloc> {
    type Cloned = TSOptionalType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSOptionalType {
            span: CloneIn::clone_in(&self.span, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSRestType<'old_alloc> {
    type Cloned = TSRestType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSRestType {
            span: CloneIn::clone_in(&self.span, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTupleElement<'old_alloc> {
    type Cloned = TSTupleElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::TSOptionalType(it) => {
                TSTupleElement::TSOptionalType(CloneIn::clone_in(it, allocator))
            }
            Self::TSRestType(it) => TSTupleElement::TSRestType(CloneIn::clone_in(it, allocator)),
            Self::TSAnyKeyword(it) => {
                TSTupleElement::TSAnyKeyword(CloneIn::clone_in(it, allocator))
            }
            Self::TSBigIntKeyword(it) => {
                TSTupleElement::TSBigIntKeyword(CloneIn::clone_in(it, allocator))
            }
            Self::TSBooleanKeyword(it) => {
                TSTupleElement::TSBooleanKeyword(CloneIn::clone_in(it, allocator))
            }
            Self::TSIntrinsicKeyword(it) => {
                TSTupleElement::TSIntrinsicKeyword(CloneIn::clone_in(it, allocator))
            }
            Self::TSNeverKeyword(it) => {
                TSTupleElement::TSNeverKeyword(CloneIn::clone_in(it, allocator))
            }
            Self::TSNullKeyword(it) => {
                TSTupleElement::TSNullKeyword(CloneIn::clone_in(it, allocator))
            }
            Self::TSNumberKeyword(it) => {
                TSTupleElement::TSNumberKeyword(CloneIn::clone_in(it, allocator))
            }
            Self::TSObjectKeyword(it) => {
                TSTupleElement::TSObjectKeyword(CloneIn::clone_in(it, allocator))
            }
            Self::TSStringKeyword(it) => {
                TSTupleElement::TSStringKeyword(CloneIn::clone_in(it, allocator))
            }
            Self::TSSymbolKeyword(it) => {
                TSTupleElement::TSSymbolKeyword(CloneIn::clone_in(it, allocator))
            }
            Self::TSUndefinedKeyword(it) => {
                TSTupleElement::TSUndefinedKeyword(CloneIn::clone_in(it, allocator))
            }
            Self::TSUnknownKeyword(it) => {
                TSTupleElement::TSUnknownKeyword(CloneIn::clone_in(it, allocator))
            }
            Self::TSVoidKeyword(it) => {
                TSTupleElement::TSVoidKeyword(CloneIn::clone_in(it, allocator))
            }
            Self::TSArrayType(it) => TSTupleElement::TSArrayType(CloneIn::clone_in(it, allocator)),
            Self::TSConditionalType(it) => {
                TSTupleElement::TSConditionalType(CloneIn::clone_in(it, allocator))
            }
            Self::TSConstructorType(it) => {
                TSTupleElement::TSConstructorType(CloneIn::clone_in(it, allocator))
            }
            Self::TSFunctionType(it) => {
                TSTupleElement::TSFunctionType(CloneIn::clone_in(it, allocator))
            }
            Self::TSImportType(it) => {
                TSTupleElement::TSImportType(CloneIn::clone_in(it, allocator))
            }
            Self::TSIndexedAccessType(it) => {
                TSTupleElement::TSIndexedAccessType(CloneIn::clone_in(it, allocator))
            }
            Self::TSInferType(it) => TSTupleElement::TSInferType(CloneIn::clone_in(it, allocator)),
            Self::TSIntersectionType(it) => {
                TSTupleElement::TSIntersectionType(CloneIn::clone_in(it, allocator))
            }
            Self::TSLiteralType(it) => {
                TSTupleElement::TSLiteralType(CloneIn::clone_in(it, allocator))
            }
            Self::TSMappedType(it) => {
                TSTupleElement::TSMappedType(CloneIn::clone_in(it, allocator))
            }
            Self::TSNamedTupleMember(it) => {
                TSTupleElement::TSNamedTupleMember(CloneIn::clone_in(it, allocator))
            }
            Self::TSQualifiedName(it) => {
                TSTupleElement::TSQualifiedName(CloneIn::clone_in(it, allocator))
            }
            Self::TSTemplateLiteralType(it) => {
                TSTupleElement::TSTemplateLiteralType(CloneIn::clone_in(it, allocator))
            }
            Self::TSThisType(it) => TSTupleElement::TSThisType(CloneIn::clone_in(it, allocator)),
            Self::TSTupleType(it) => TSTupleElement::TSTupleType(CloneIn::clone_in(it, allocator)),
            Self::TSTypeLiteral(it) => {
                TSTupleElement::TSTypeLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::TSTypeOperatorType(it) => {
                TSTupleElement::TSTypeOperatorType(CloneIn::clone_in(it, allocator))
            }
            Self::TSTypePredicate(it) => {
                TSTupleElement::TSTypePredicate(CloneIn::clone_in(it, allocator))
            }
            Self::TSTypeQuery(it) => TSTupleElement::TSTypeQuery(CloneIn::clone_in(it, allocator)),
            Self::TSTypeReference(it) => {
                TSTupleElement::TSTypeReference(CloneIn::clone_in(it, allocator))
            }
            Self::TSUnionType(it) => TSTupleElement::TSUnionType(CloneIn::clone_in(it, allocator)),
            Self::TSParenthesizedType(it) => {
                TSTupleElement::TSParenthesizedType(CloneIn::clone_in(it, allocator))
            }
            Self::JSDocNullableType(it) => {
                TSTupleElement::JSDocNullableType(CloneIn::clone_in(it, allocator))
            }
            Self::JSDocNonNullableType(it) => {
                TSTupleElement::JSDocNonNullableType(CloneIn::clone_in(it, allocator))
            }
            Self::JSDocUnknownType(it) => {
                TSTupleElement::JSDocUnknownType(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'alloc> CloneIn<'alloc> for TSAnyKeyword {
    type Cloned = TSAnyKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSAnyKeyword { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSStringKeyword {
    type Cloned = TSStringKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSStringKeyword { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSBooleanKeyword {
    type Cloned = TSBooleanKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSBooleanKeyword { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSNumberKeyword {
    type Cloned = TSNumberKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSNumberKeyword { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSNeverKeyword {
    type Cloned = TSNeverKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSNeverKeyword { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSIntrinsicKeyword {
    type Cloned = TSIntrinsicKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSIntrinsicKeyword { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSUnknownKeyword {
    type Cloned = TSUnknownKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSUnknownKeyword { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSNullKeyword {
    type Cloned = TSNullKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSNullKeyword { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSUndefinedKeyword {
    type Cloned = TSUndefinedKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSUndefinedKeyword { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSVoidKeyword {
    type Cloned = TSVoidKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSVoidKeyword { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSSymbolKeyword {
    type Cloned = TSSymbolKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSSymbolKeyword { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSThisType {
    type Cloned = TSThisType;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSThisType { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSObjectKeyword {
    type Cloned = TSObjectKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSObjectKeyword { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSBigIntKeyword {
    type Cloned = TSBigIntKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSBigIntKeyword { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeReference<'old_alloc> {
    type Cloned = TSTypeReference<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeReference {
            span: CloneIn::clone_in(&self.span, allocator),
            type_name: CloneIn::clone_in(&self.type_name, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeName<'old_alloc> {
    type Cloned = TSTypeName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::IdentifierReference(it) => {
                TSTypeName::IdentifierReference(CloneIn::clone_in(it, allocator))
            }
            Self::QualifiedName(it) => TSTypeName::QualifiedName(CloneIn::clone_in(it, allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSQualifiedName<'old_alloc> {
    type Cloned = TSQualifiedName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSQualifiedName {
            span: CloneIn::clone_in(&self.span, allocator),
            left: CloneIn::clone_in(&self.left, allocator),
            right: CloneIn::clone_in(&self.right, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeParameterInstantiation<'old_alloc> {
    type Cloned = TSTypeParameterInstantiation<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeParameterInstantiation {
            span: CloneIn::clone_in(&self.span, allocator),
            params: CloneIn::clone_in(&self.params, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeParameter<'old_alloc> {
    type Cloned = TSTypeParameter<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeParameter {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
            constraint: CloneIn::clone_in(&self.constraint, allocator),
            default: CloneIn::clone_in(&self.default, allocator),
            r#in: CloneIn::clone_in(&self.r#in, allocator),
            out: CloneIn::clone_in(&self.out, allocator),
            r#const: CloneIn::clone_in(&self.r#const, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeParameterDeclaration<'old_alloc> {
    type Cloned = TSTypeParameterDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeParameterDeclaration {
            span: CloneIn::clone_in(&self.span, allocator),
            params: CloneIn::clone_in(&self.params, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeAliasDeclaration<'old_alloc> {
    type Cloned = TSTypeAliasDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeAliasDeclaration {
            span: CloneIn::clone_in(&self.span, allocator),
            id: CloneIn::clone_in(&self.id, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
            declare: CloneIn::clone_in(&self.declare, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for TSAccessibility {
    type Cloned = TSAccessibility;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Private => TSAccessibility::Private,
            Self::Protected => TSAccessibility::Protected,
            Self::Public => TSAccessibility::Public,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSClassImplements<'old_alloc> {
    type Cloned = TSClassImplements<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSClassImplements {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSInterfaceDeclaration<'old_alloc> {
    type Cloned = TSInterfaceDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSInterfaceDeclaration {
            span: CloneIn::clone_in(&self.span, allocator),
            id: CloneIn::clone_in(&self.id, allocator),
            extends: CloneIn::clone_in(&self.extends, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
            declare: CloneIn::clone_in(&self.declare, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSInterfaceBody<'old_alloc> {
    type Cloned = TSInterfaceBody<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSInterfaceBody {
            span: CloneIn::clone_in(&self.span, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSPropertySignature<'old_alloc> {
    type Cloned = TSPropertySignature<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSPropertySignature {
            span: CloneIn::clone_in(&self.span, allocator),
            computed: CloneIn::clone_in(&self.computed, allocator),
            optional: CloneIn::clone_in(&self.optional, allocator),
            readonly: CloneIn::clone_in(&self.readonly, allocator),
            key: CloneIn::clone_in(&self.key, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSSignature<'old_alloc> {
    type Cloned = TSSignature<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::TSIndexSignature(it) => {
                TSSignature::TSIndexSignature(CloneIn::clone_in(it, allocator))
            }
            Self::TSPropertySignature(it) => {
                TSSignature::TSPropertySignature(CloneIn::clone_in(it, allocator))
            }
            Self::TSCallSignatureDeclaration(it) => {
                TSSignature::TSCallSignatureDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::TSConstructSignatureDeclaration(it) => {
                TSSignature::TSConstructSignatureDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::TSMethodSignature(it) => {
                TSSignature::TSMethodSignature(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSIndexSignature<'old_alloc> {
    type Cloned = TSIndexSignature<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSIndexSignature {
            span: CloneIn::clone_in(&self.span, allocator),
            parameters: CloneIn::clone_in(&self.parameters, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
            readonly: CloneIn::clone_in(&self.readonly, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSCallSignatureDeclaration<'old_alloc> {
    type Cloned = TSCallSignatureDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSCallSignatureDeclaration {
            span: CloneIn::clone_in(&self.span, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
            this_param: CloneIn::clone_in(&self.this_param, allocator),
            params: CloneIn::clone_in(&self.params, allocator),
            return_type: CloneIn::clone_in(&self.return_type, allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for TSMethodSignatureKind {
    type Cloned = TSMethodSignatureKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Method => TSMethodSignatureKind::Method,
            Self::Get => TSMethodSignatureKind::Get,
            Self::Set => TSMethodSignatureKind::Set,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSMethodSignature<'old_alloc> {
    type Cloned = TSMethodSignature<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSMethodSignature {
            span: CloneIn::clone_in(&self.span, allocator),
            key: CloneIn::clone_in(&self.key, allocator),
            computed: CloneIn::clone_in(&self.computed, allocator),
            optional: CloneIn::clone_in(&self.optional, allocator),
            kind: CloneIn::clone_in(&self.kind, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
            this_param: CloneIn::clone_in(&self.this_param, allocator),
            params: CloneIn::clone_in(&self.params, allocator),
            return_type: CloneIn::clone_in(&self.return_type, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSConstructSignatureDeclaration<'old_alloc> {
    type Cloned = TSConstructSignatureDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSConstructSignatureDeclaration {
            span: CloneIn::clone_in(&self.span, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
            params: CloneIn::clone_in(&self.params, allocator),
            return_type: CloneIn::clone_in(&self.return_type, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSIndexSignatureName<'old_alloc> {
    type Cloned = TSIndexSignatureName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSIndexSignatureName {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSInterfaceHeritage<'old_alloc> {
    type Cloned = TSInterfaceHeritage<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSInterfaceHeritage {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypePredicate<'old_alloc> {
    type Cloned = TSTypePredicate<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypePredicate {
            span: CloneIn::clone_in(&self.span, allocator),
            parameter_name: CloneIn::clone_in(&self.parameter_name, allocator),
            asserts: CloneIn::clone_in(&self.asserts, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypePredicateName<'old_alloc> {
    type Cloned = TSTypePredicateName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => {
                TSTypePredicateName::Identifier(CloneIn::clone_in(it, allocator))
            }
            Self::This(it) => TSTypePredicateName::This(CloneIn::clone_in(it, allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSModuleDeclaration<'old_alloc> {
    type Cloned = TSModuleDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSModuleDeclaration {
            span: CloneIn::clone_in(&self.span, allocator),
            id: CloneIn::clone_in(&self.id, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
            kind: CloneIn::clone_in(&self.kind, allocator),
            declare: CloneIn::clone_in(&self.declare, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for TSModuleDeclarationKind {
    type Cloned = TSModuleDeclarationKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Global => TSModuleDeclarationKind::Global,
            Self::Module => TSModuleDeclarationKind::Module,
            Self::Namespace => TSModuleDeclarationKind::Namespace,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSModuleDeclarationName<'old_alloc> {
    type Cloned = TSModuleDeclarationName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => {
                TSModuleDeclarationName::Identifier(CloneIn::clone_in(it, allocator))
            }
            Self::StringLiteral(it) => {
                TSModuleDeclarationName::StringLiteral(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSModuleDeclarationBody<'old_alloc> {
    type Cloned = TSModuleDeclarationBody<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::TSModuleDeclaration(it) => {
                TSModuleDeclarationBody::TSModuleDeclaration(CloneIn::clone_in(it, allocator))
            }
            Self::TSModuleBlock(it) => {
                TSModuleDeclarationBody::TSModuleBlock(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSModuleBlock<'old_alloc> {
    type Cloned = TSModuleBlock<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSModuleBlock {
            span: CloneIn::clone_in(&self.span, allocator),
            directives: CloneIn::clone_in(&self.directives, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeLiteral<'old_alloc> {
    type Cloned = TSTypeLiteral<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeLiteral {
            span: CloneIn::clone_in(&self.span, allocator),
            members: CloneIn::clone_in(&self.members, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSInferType<'old_alloc> {
    type Cloned = TSInferType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSInferType {
            span: CloneIn::clone_in(&self.span, allocator),
            type_parameter: CloneIn::clone_in(&self.type_parameter, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeQuery<'old_alloc> {
    type Cloned = TSTypeQuery<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeQuery {
            span: CloneIn::clone_in(&self.span, allocator),
            expr_name: CloneIn::clone_in(&self.expr_name, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeQueryExprName<'old_alloc> {
    type Cloned = TSTypeQueryExprName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::TSImportType(it) => {
                TSTypeQueryExprName::TSImportType(CloneIn::clone_in(it, allocator))
            }
            Self::IdentifierReference(it) => {
                TSTypeQueryExprName::IdentifierReference(CloneIn::clone_in(it, allocator))
            }
            Self::QualifiedName(it) => {
                TSTypeQueryExprName::QualifiedName(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSImportType<'old_alloc> {
    type Cloned = TSImportType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSImportType {
            span: CloneIn::clone_in(&self.span, allocator),
            is_type_of: CloneIn::clone_in(&self.is_type_of, allocator),
            parameter: CloneIn::clone_in(&self.parameter, allocator),
            qualifier: CloneIn::clone_in(&self.qualifier, allocator),
            attributes: CloneIn::clone_in(&self.attributes, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSImportAttributes<'old_alloc> {
    type Cloned = TSImportAttributes<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSImportAttributes {
            span: CloneIn::clone_in(&self.span, allocator),
            attributes_keyword: CloneIn::clone_in(&self.attributes_keyword, allocator),
            elements: CloneIn::clone_in(&self.elements, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSImportAttribute<'old_alloc> {
    type Cloned = TSImportAttribute<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSImportAttribute {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSImportAttributeName<'old_alloc> {
    type Cloned = TSImportAttributeName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => {
                TSImportAttributeName::Identifier(CloneIn::clone_in(it, allocator))
            }
            Self::StringLiteral(it) => {
                TSImportAttributeName::StringLiteral(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSFunctionType<'old_alloc> {
    type Cloned = TSFunctionType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSFunctionType {
            span: CloneIn::clone_in(&self.span, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
            this_param: CloneIn::clone_in(&self.this_param, allocator),
            params: CloneIn::clone_in(&self.params, allocator),
            return_type: CloneIn::clone_in(&self.return_type, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSConstructorType<'old_alloc> {
    type Cloned = TSConstructorType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSConstructorType {
            span: CloneIn::clone_in(&self.span, allocator),
            r#abstract: CloneIn::clone_in(&self.r#abstract, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
            params: CloneIn::clone_in(&self.params, allocator),
            return_type: CloneIn::clone_in(&self.return_type, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSMappedType<'old_alloc> {
    type Cloned = TSMappedType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSMappedType {
            span: CloneIn::clone_in(&self.span, allocator),
            type_parameter: CloneIn::clone_in(&self.type_parameter, allocator),
            name_type: CloneIn::clone_in(&self.name_type, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
            optional: CloneIn::clone_in(&self.optional, allocator),
            readonly: CloneIn::clone_in(&self.readonly, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for TSMappedTypeModifierOperator {
    type Cloned = TSMappedTypeModifierOperator;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::True => TSMappedTypeModifierOperator::True,
            Self::Plus => TSMappedTypeModifierOperator::Plus,
            Self::Minus => TSMappedTypeModifierOperator::Minus,
            Self::None => TSMappedTypeModifierOperator::None,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTemplateLiteralType<'old_alloc> {
    type Cloned = TSTemplateLiteralType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTemplateLiteralType {
            span: CloneIn::clone_in(&self.span, allocator),
            quasis: CloneIn::clone_in(&self.quasis, allocator),
            types: CloneIn::clone_in(&self.types, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSAsExpression<'old_alloc> {
    type Cloned = TSAsExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSAsExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSSatisfiesExpression<'old_alloc> {
    type Cloned = TSSatisfiesExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSSatisfiesExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeAssertion<'old_alloc> {
    type Cloned = TSTypeAssertion<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeAssertion {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSImportEqualsDeclaration<'old_alloc> {
    type Cloned = TSImportEqualsDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSImportEqualsDeclaration {
            span: CloneIn::clone_in(&self.span, allocator),
            id: CloneIn::clone_in(&self.id, allocator),
            module_reference: CloneIn::clone_in(&self.module_reference, allocator),
            import_kind: CloneIn::clone_in(&self.import_kind, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSModuleReference<'old_alloc> {
    type Cloned = TSModuleReference<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::ExternalModuleReference(it) => {
                TSModuleReference::ExternalModuleReference(CloneIn::clone_in(it, allocator))
            }
            Self::IdentifierReference(it) => {
                TSModuleReference::IdentifierReference(CloneIn::clone_in(it, allocator))
            }
            Self::QualifiedName(it) => {
                TSModuleReference::QualifiedName(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSExternalModuleReference<'old_alloc> {
    type Cloned = TSExternalModuleReference<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSExternalModuleReference {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSNonNullExpression<'old_alloc> {
    type Cloned = TSNonNullExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSNonNullExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Decorator<'old_alloc> {
    type Cloned = Decorator<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Decorator {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSExportAssignment<'old_alloc> {
    type Cloned = TSExportAssignment<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSExportAssignment {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSNamespaceExportDeclaration<'old_alloc> {
    type Cloned = TSNamespaceExportDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSNamespaceExportDeclaration {
            span: CloneIn::clone_in(&self.span, allocator),
            id: CloneIn::clone_in(&self.id, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSInstantiationExpression<'old_alloc> {
    type Cloned = TSInstantiationExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSInstantiationExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for ImportOrExportKind {
    type Cloned = ImportOrExportKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Value => ImportOrExportKind::Value,
            Self::Type => ImportOrExportKind::Type,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSDocNullableType<'old_alloc> {
    type Cloned = JSDocNullableType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSDocNullableType {
            span: CloneIn::clone_in(&self.span, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
            postfix: CloneIn::clone_in(&self.postfix, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSDocNonNullableType<'old_alloc> {
    type Cloned = JSDocNonNullableType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSDocNonNullableType {
            span: CloneIn::clone_in(&self.span, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
            postfix: CloneIn::clone_in(&self.postfix, allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for JSDocUnknownType {
    type Cloned = JSDocUnknownType;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        JSDocUnknownType { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXElement<'old_alloc> {
    type Cloned = JSXElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXElement {
            span: CloneIn::clone_in(&self.span, allocator),
            opening_element: CloneIn::clone_in(&self.opening_element, allocator),
            closing_element: CloneIn::clone_in(&self.closing_element, allocator),
            children: CloneIn::clone_in(&self.children, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXOpeningElement<'old_alloc> {
    type Cloned = JSXOpeningElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXOpeningElement {
            span: CloneIn::clone_in(&self.span, allocator),
            self_closing: CloneIn::clone_in(&self.self_closing, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
            attributes: CloneIn::clone_in(&self.attributes, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXClosingElement<'old_alloc> {
    type Cloned = JSXClosingElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXClosingElement {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXFragment<'old_alloc> {
    type Cloned = JSXFragment<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXFragment {
            span: CloneIn::clone_in(&self.span, allocator),
            opening_fragment: CloneIn::clone_in(&self.opening_fragment, allocator),
            closing_fragment: CloneIn::clone_in(&self.closing_fragment, allocator),
            children: CloneIn::clone_in(&self.children, allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for JSXOpeningFragment {
    type Cloned = JSXOpeningFragment;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        JSXOpeningFragment { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for JSXClosingFragment {
    type Cloned = JSXClosingFragment;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        JSXClosingFragment { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXElementName<'old_alloc> {
    type Cloned = JSXElementName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => JSXElementName::Identifier(CloneIn::clone_in(it, allocator)),
            Self::IdentifierReference(it) => {
                JSXElementName::IdentifierReference(CloneIn::clone_in(it, allocator))
            }
            Self::NamespacedName(it) => {
                JSXElementName::NamespacedName(CloneIn::clone_in(it, allocator))
            }
            Self::MemberExpression(it) => {
                JSXElementName::MemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ThisExpression(it) => {
                JSXElementName::ThisExpression(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXNamespacedName<'old_alloc> {
    type Cloned = JSXNamespacedName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXNamespacedName {
            span: CloneIn::clone_in(&self.span, allocator),
            namespace: CloneIn::clone_in(&self.namespace, allocator),
            property: CloneIn::clone_in(&self.property, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXMemberExpression<'old_alloc> {
    type Cloned = JSXMemberExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXMemberExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            object: CloneIn::clone_in(&self.object, allocator),
            property: CloneIn::clone_in(&self.property, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXMemberExpressionObject<'old_alloc> {
    type Cloned = JSXMemberExpressionObject<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::IdentifierReference(it) => {
                JSXMemberExpressionObject::IdentifierReference(CloneIn::clone_in(it, allocator))
            }
            Self::MemberExpression(it) => {
                JSXMemberExpressionObject::MemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ThisExpression(it) => {
                JSXMemberExpressionObject::ThisExpression(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXExpressionContainer<'old_alloc> {
    type Cloned = JSXExpressionContainer<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXExpressionContainer {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXExpression<'old_alloc> {
    type Cloned = JSXExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::EmptyExpression(it) => {
                JSXExpression::EmptyExpression(CloneIn::clone_in(it, allocator))
            }
            Self::BooleanLiteral(it) => {
                JSXExpression::BooleanLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::NullLiteral(it) => JSXExpression::NullLiteral(CloneIn::clone_in(it, allocator)),
            Self::NumericLiteral(it) => {
                JSXExpression::NumericLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::BigIntLiteral(it) => {
                JSXExpression::BigIntLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::RegExpLiteral(it) => {
                JSXExpression::RegExpLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::StringLiteral(it) => {
                JSXExpression::StringLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::TemplateLiteral(it) => {
                JSXExpression::TemplateLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::Identifier(it) => JSXExpression::Identifier(CloneIn::clone_in(it, allocator)),
            Self::MetaProperty(it) => JSXExpression::MetaProperty(CloneIn::clone_in(it, allocator)),
            Self::Super(it) => JSXExpression::Super(CloneIn::clone_in(it, allocator)),
            Self::ArrayExpression(it) => {
                JSXExpression::ArrayExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ArrowFunctionExpression(it) => {
                JSXExpression::ArrowFunctionExpression(CloneIn::clone_in(it, allocator))
            }
            Self::AssignmentExpression(it) => {
                JSXExpression::AssignmentExpression(CloneIn::clone_in(it, allocator))
            }
            Self::AwaitExpression(it) => {
                JSXExpression::AwaitExpression(CloneIn::clone_in(it, allocator))
            }
            Self::BinaryExpression(it) => {
                JSXExpression::BinaryExpression(CloneIn::clone_in(it, allocator))
            }
            Self::CallExpression(it) => {
                JSXExpression::CallExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ChainExpression(it) => {
                JSXExpression::ChainExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ClassExpression(it) => {
                JSXExpression::ClassExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ConditionalExpression(it) => {
                JSXExpression::ConditionalExpression(CloneIn::clone_in(it, allocator))
            }
            Self::FunctionExpression(it) => {
                JSXExpression::FunctionExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ImportExpression(it) => {
                JSXExpression::ImportExpression(CloneIn::clone_in(it, allocator))
            }
            Self::LogicalExpression(it) => {
                JSXExpression::LogicalExpression(CloneIn::clone_in(it, allocator))
            }
            Self::NewExpression(it) => {
                JSXExpression::NewExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ObjectExpression(it) => {
                JSXExpression::ObjectExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ParenthesizedExpression(it) => {
                JSXExpression::ParenthesizedExpression(CloneIn::clone_in(it, allocator))
            }
            Self::SequenceExpression(it) => {
                JSXExpression::SequenceExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TaggedTemplateExpression(it) => {
                JSXExpression::TaggedTemplateExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ThisExpression(it) => {
                JSXExpression::ThisExpression(CloneIn::clone_in(it, allocator))
            }
            Self::UnaryExpression(it) => {
                JSXExpression::UnaryExpression(CloneIn::clone_in(it, allocator))
            }
            Self::UpdateExpression(it) => {
                JSXExpression::UpdateExpression(CloneIn::clone_in(it, allocator))
            }
            Self::YieldExpression(it) => {
                JSXExpression::YieldExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateInExpression(it) => {
                JSXExpression::PrivateInExpression(CloneIn::clone_in(it, allocator))
            }
            Self::JSXElement(it) => JSXExpression::JSXElement(CloneIn::clone_in(it, allocator)),
            Self::JSXFragment(it) => JSXExpression::JSXFragment(CloneIn::clone_in(it, allocator)),
            Self::TSAsExpression(it) => {
                JSXExpression::TSAsExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSSatisfiesExpression(it) => {
                JSXExpression::TSSatisfiesExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSTypeAssertion(it) => {
                JSXExpression::TSTypeAssertion(CloneIn::clone_in(it, allocator))
            }
            Self::TSNonNullExpression(it) => {
                JSXExpression::TSNonNullExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSInstantiationExpression(it) => {
                JSXExpression::TSInstantiationExpression(CloneIn::clone_in(it, allocator))
            }
            Self::ComputedMemberExpression(it) => {
                JSXExpression::ComputedMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::StaticMemberExpression(it) => {
                JSXExpression::StaticMemberExpression(CloneIn::clone_in(it, allocator))
            }
            Self::PrivateFieldExpression(it) => {
                JSXExpression::PrivateFieldExpression(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'alloc> CloneIn<'alloc> for JSXEmptyExpression {
    type Cloned = JSXEmptyExpression;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        JSXEmptyExpression { span: CloneIn::clone_in(&self.span, allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXAttributeItem<'old_alloc> {
    type Cloned = JSXAttributeItem<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Attribute(it) => JSXAttributeItem::Attribute(CloneIn::clone_in(it, allocator)),
            Self::SpreadAttribute(it) => {
                JSXAttributeItem::SpreadAttribute(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXAttribute<'old_alloc> {
    type Cloned = JSXAttribute<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXAttribute {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXSpreadAttribute<'old_alloc> {
    type Cloned = JSXSpreadAttribute<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXSpreadAttribute {
            span: CloneIn::clone_in(&self.span, allocator),
            argument: CloneIn::clone_in(&self.argument, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXAttributeName<'old_alloc> {
    type Cloned = JSXAttributeName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => JSXAttributeName::Identifier(CloneIn::clone_in(it, allocator)),
            Self::NamespacedName(it) => {
                JSXAttributeName::NamespacedName(CloneIn::clone_in(it, allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXAttributeValue<'old_alloc> {
    type Cloned = JSXAttributeValue<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::StringLiteral(it) => {
                JSXAttributeValue::StringLiteral(CloneIn::clone_in(it, allocator))
            }
            Self::ExpressionContainer(it) => {
                JSXAttributeValue::ExpressionContainer(CloneIn::clone_in(it, allocator))
            }
            Self::Element(it) => JSXAttributeValue::Element(CloneIn::clone_in(it, allocator)),
            Self::Fragment(it) => JSXAttributeValue::Fragment(CloneIn::clone_in(it, allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXIdentifier<'old_alloc> {
    type Cloned = JSXIdentifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXIdentifier {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXChild<'old_alloc> {
    type Cloned = JSXChild<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Text(it) => JSXChild::Text(CloneIn::clone_in(it, allocator)),
            Self::Element(it) => JSXChild::Element(CloneIn::clone_in(it, allocator)),
            Self::Fragment(it) => JSXChild::Fragment(CloneIn::clone_in(it, allocator)),
            Self::ExpressionContainer(it) => {
                JSXChild::ExpressionContainer(CloneIn::clone_in(it, allocator))
            }
            Self::Spread(it) => JSXChild::Spread(CloneIn::clone_in(it, allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXSpreadChild<'old_alloc> {
    type Cloned = JSXSpreadChild<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXSpreadChild {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXText<'old_alloc> {
    type Cloned = JSXText<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXText {
            span: CloneIn::clone_in(&self.span, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for CommentKind {
    type Cloned = CommentKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Line => CommentKind::Line,
            Self::Block => CommentKind::Block,
        }
    }
}

impl<'alloc> CloneIn<'alloc> for CommentPosition {
    type Cloned = CommentPosition;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Leading => CommentPosition::Leading,
            Self::Trailing => CommentPosition::Trailing,
        }
    }
}

impl<'alloc> CloneIn<'alloc> for Comment {
    type Cloned = Comment;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        Comment {
            span: CloneIn::clone_in(&self.span, allocator),
            kind: CloneIn::clone_in(&self.kind, allocator),
            position: CloneIn::clone_in(&self.position, allocator),
            attached_to: CloneIn::clone_in(&self.attached_to, allocator),
            preceded_by_newline: CloneIn::clone_in(&self.preceded_by_newline, allocator),
            followed_by_newline: CloneIn::clone_in(&self.followed_by_newline, allocator),
        }
    }
}
