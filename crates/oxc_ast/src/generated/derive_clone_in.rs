// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/clone_in.rs`

#![allow(clippy::default_trait_access)]

use oxc_allocator::{Allocator, CloneIn};

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
            span: self.span.clone_in(allocator),
            value: self.value.clone_in(allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for NullLiteral {
    type Cloned = NullLiteral;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        NullLiteral { span: self.span.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for NumericLiteral<'old_alloc> {
    type Cloned = NumericLiteral<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        NumericLiteral {
            span: self.span.clone_in(allocator),
            value: self.value.clone_in(allocator),
            raw: self.raw.clone_in(allocator),
            base: self.base.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BigIntLiteral<'old_alloc> {
    type Cloned = BigIntLiteral<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BigIntLiteral {
            span: self.span.clone_in(allocator),
            raw: self.raw.clone_in(allocator),
            base: self.base.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for RegExpLiteral<'old_alloc> {
    type Cloned = RegExpLiteral<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        RegExpLiteral {
            span: self.span.clone_in(allocator),
            value: self.value.clone_in(allocator),
            regex: self.regex.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for RegExp<'old_alloc> {
    type Cloned = RegExp<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        RegExp { pattern: self.pattern.clone_in(allocator), flags: self.flags.clone_in(allocator) }
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
        StringLiteral { span: self.span.clone_in(allocator), value: self.value.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Program<'old_alloc> {
    type Cloned = Program<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Program {
            span: self.span.clone_in(allocator),
            source_type: self.source_type.clone_in(allocator),
            hashbang: self.hashbang.clone_in(allocator),
            directives: self.directives.clone_in(allocator),
            body: self.body.clone_in(allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Expression<'old_alloc> {
    type Cloned = Expression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::BooleanLiteral(it) => Expression::BooleanLiteral(it.clone_in(allocator)),
            Self::NullLiteral(it) => Expression::NullLiteral(it.clone_in(allocator)),
            Self::NumericLiteral(it) => Expression::NumericLiteral(it.clone_in(allocator)),
            Self::BigIntLiteral(it) => Expression::BigIntLiteral(it.clone_in(allocator)),
            Self::RegExpLiteral(it) => Expression::RegExpLiteral(it.clone_in(allocator)),
            Self::StringLiteral(it) => Expression::StringLiteral(it.clone_in(allocator)),
            Self::TemplateLiteral(it) => Expression::TemplateLiteral(it.clone_in(allocator)),
            Self::Identifier(it) => Expression::Identifier(it.clone_in(allocator)),
            Self::MetaProperty(it) => Expression::MetaProperty(it.clone_in(allocator)),
            Self::Super(it) => Expression::Super(it.clone_in(allocator)),
            Self::ArrayExpression(it) => Expression::ArrayExpression(it.clone_in(allocator)),
            Self::ArrowFunctionExpression(it) => {
                Expression::ArrowFunctionExpression(it.clone_in(allocator))
            }
            Self::AssignmentExpression(it) => {
                Expression::AssignmentExpression(it.clone_in(allocator))
            }
            Self::AwaitExpression(it) => Expression::AwaitExpression(it.clone_in(allocator)),
            Self::BinaryExpression(it) => Expression::BinaryExpression(it.clone_in(allocator)),
            Self::CallExpression(it) => Expression::CallExpression(it.clone_in(allocator)),
            Self::ChainExpression(it) => Expression::ChainExpression(it.clone_in(allocator)),
            Self::ClassExpression(it) => Expression::ClassExpression(it.clone_in(allocator)),
            Self::ConditionalExpression(it) => {
                Expression::ConditionalExpression(it.clone_in(allocator))
            }
            Self::FunctionExpression(it) => Expression::FunctionExpression(it.clone_in(allocator)),
            Self::ImportExpression(it) => Expression::ImportExpression(it.clone_in(allocator)),
            Self::LogicalExpression(it) => Expression::LogicalExpression(it.clone_in(allocator)),
            Self::NewExpression(it) => Expression::NewExpression(it.clone_in(allocator)),
            Self::ObjectExpression(it) => Expression::ObjectExpression(it.clone_in(allocator)),
            Self::ParenthesizedExpression(it) => {
                Expression::ParenthesizedExpression(it.clone_in(allocator))
            }
            Self::SequenceExpression(it) => Expression::SequenceExpression(it.clone_in(allocator)),
            Self::TaggedTemplateExpression(it) => {
                Expression::TaggedTemplateExpression(it.clone_in(allocator))
            }
            Self::ThisExpression(it) => Expression::ThisExpression(it.clone_in(allocator)),
            Self::UnaryExpression(it) => Expression::UnaryExpression(it.clone_in(allocator)),
            Self::UpdateExpression(it) => Expression::UpdateExpression(it.clone_in(allocator)),
            Self::YieldExpression(it) => Expression::YieldExpression(it.clone_in(allocator)),
            Self::PrivateInExpression(it) => {
                Expression::PrivateInExpression(it.clone_in(allocator))
            }
            Self::JSXElement(it) => Expression::JSXElement(it.clone_in(allocator)),
            Self::JSXFragment(it) => Expression::JSXFragment(it.clone_in(allocator)),
            Self::TSAsExpression(it) => Expression::TSAsExpression(it.clone_in(allocator)),
            Self::TSSatisfiesExpression(it) => {
                Expression::TSSatisfiesExpression(it.clone_in(allocator))
            }
            Self::TSTypeAssertion(it) => Expression::TSTypeAssertion(it.clone_in(allocator)),
            Self::TSNonNullExpression(it) => {
                Expression::TSNonNullExpression(it.clone_in(allocator))
            }
            Self::TSInstantiationExpression(it) => {
                Expression::TSInstantiationExpression(it.clone_in(allocator))
            }
            Self::ComputedMemberExpression(it) => {
                Expression::ComputedMemberExpression(it.clone_in(allocator))
            }
            Self::StaticMemberExpression(it) => {
                Expression::StaticMemberExpression(it.clone_in(allocator))
            }
            Self::PrivateFieldExpression(it) => {
                Expression::PrivateFieldExpression(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for IdentifierName<'old_alloc> {
    type Cloned = IdentifierName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        IdentifierName { span: self.span.clone_in(allocator), name: self.name.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for IdentifierReference<'old_alloc> {
    type Cloned = IdentifierReference<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        IdentifierReference {
            span: self.span.clone_in(allocator),
            name: self.name.clone_in(allocator),
            reference_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BindingIdentifier<'old_alloc> {
    type Cloned = BindingIdentifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BindingIdentifier {
            span: self.span.clone_in(allocator),
            name: self.name.clone_in(allocator),
            symbol_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for LabelIdentifier<'old_alloc> {
    type Cloned = LabelIdentifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        LabelIdentifier { span: self.span.clone_in(allocator), name: self.name.clone_in(allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for ThisExpression {
    type Cloned = ThisExpression;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        ThisExpression { span: self.span.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ArrayExpression<'old_alloc> {
    type Cloned = ArrayExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ArrayExpression {
            span: self.span.clone_in(allocator),
            elements: self.elements.clone_in(allocator),
            trailing_comma: self.trailing_comma.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ArrayExpressionElement<'old_alloc> {
    type Cloned = ArrayExpressionElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::SpreadElement(it) => {
                ArrayExpressionElement::SpreadElement(it.clone_in(allocator))
            }
            Self::Elision(it) => ArrayExpressionElement::Elision(it.clone_in(allocator)),
            Self::BooleanLiteral(it) => {
                ArrayExpressionElement::BooleanLiteral(it.clone_in(allocator))
            }
            Self::NullLiteral(it) => ArrayExpressionElement::NullLiteral(it.clone_in(allocator)),
            Self::NumericLiteral(it) => {
                ArrayExpressionElement::NumericLiteral(it.clone_in(allocator))
            }
            Self::BigIntLiteral(it) => {
                ArrayExpressionElement::BigIntLiteral(it.clone_in(allocator))
            }
            Self::RegExpLiteral(it) => {
                ArrayExpressionElement::RegExpLiteral(it.clone_in(allocator))
            }
            Self::StringLiteral(it) => {
                ArrayExpressionElement::StringLiteral(it.clone_in(allocator))
            }
            Self::TemplateLiteral(it) => {
                ArrayExpressionElement::TemplateLiteral(it.clone_in(allocator))
            }
            Self::Identifier(it) => ArrayExpressionElement::Identifier(it.clone_in(allocator)),
            Self::MetaProperty(it) => ArrayExpressionElement::MetaProperty(it.clone_in(allocator)),
            Self::Super(it) => ArrayExpressionElement::Super(it.clone_in(allocator)),
            Self::ArrayExpression(it) => {
                ArrayExpressionElement::ArrayExpression(it.clone_in(allocator))
            }
            Self::ArrowFunctionExpression(it) => {
                ArrayExpressionElement::ArrowFunctionExpression(it.clone_in(allocator))
            }
            Self::AssignmentExpression(it) => {
                ArrayExpressionElement::AssignmentExpression(it.clone_in(allocator))
            }
            Self::AwaitExpression(it) => {
                ArrayExpressionElement::AwaitExpression(it.clone_in(allocator))
            }
            Self::BinaryExpression(it) => {
                ArrayExpressionElement::BinaryExpression(it.clone_in(allocator))
            }
            Self::CallExpression(it) => {
                ArrayExpressionElement::CallExpression(it.clone_in(allocator))
            }
            Self::ChainExpression(it) => {
                ArrayExpressionElement::ChainExpression(it.clone_in(allocator))
            }
            Self::ClassExpression(it) => {
                ArrayExpressionElement::ClassExpression(it.clone_in(allocator))
            }
            Self::ConditionalExpression(it) => {
                ArrayExpressionElement::ConditionalExpression(it.clone_in(allocator))
            }
            Self::FunctionExpression(it) => {
                ArrayExpressionElement::FunctionExpression(it.clone_in(allocator))
            }
            Self::ImportExpression(it) => {
                ArrayExpressionElement::ImportExpression(it.clone_in(allocator))
            }
            Self::LogicalExpression(it) => {
                ArrayExpressionElement::LogicalExpression(it.clone_in(allocator))
            }
            Self::NewExpression(it) => {
                ArrayExpressionElement::NewExpression(it.clone_in(allocator))
            }
            Self::ObjectExpression(it) => {
                ArrayExpressionElement::ObjectExpression(it.clone_in(allocator))
            }
            Self::ParenthesizedExpression(it) => {
                ArrayExpressionElement::ParenthesizedExpression(it.clone_in(allocator))
            }
            Self::SequenceExpression(it) => {
                ArrayExpressionElement::SequenceExpression(it.clone_in(allocator))
            }
            Self::TaggedTemplateExpression(it) => {
                ArrayExpressionElement::TaggedTemplateExpression(it.clone_in(allocator))
            }
            Self::ThisExpression(it) => {
                ArrayExpressionElement::ThisExpression(it.clone_in(allocator))
            }
            Self::UnaryExpression(it) => {
                ArrayExpressionElement::UnaryExpression(it.clone_in(allocator))
            }
            Self::UpdateExpression(it) => {
                ArrayExpressionElement::UpdateExpression(it.clone_in(allocator))
            }
            Self::YieldExpression(it) => {
                ArrayExpressionElement::YieldExpression(it.clone_in(allocator))
            }
            Self::PrivateInExpression(it) => {
                ArrayExpressionElement::PrivateInExpression(it.clone_in(allocator))
            }
            Self::JSXElement(it) => ArrayExpressionElement::JSXElement(it.clone_in(allocator)),
            Self::JSXFragment(it) => ArrayExpressionElement::JSXFragment(it.clone_in(allocator)),
            Self::TSAsExpression(it) => {
                ArrayExpressionElement::TSAsExpression(it.clone_in(allocator))
            }
            Self::TSSatisfiesExpression(it) => {
                ArrayExpressionElement::TSSatisfiesExpression(it.clone_in(allocator))
            }
            Self::TSTypeAssertion(it) => {
                ArrayExpressionElement::TSTypeAssertion(it.clone_in(allocator))
            }
            Self::TSNonNullExpression(it) => {
                ArrayExpressionElement::TSNonNullExpression(it.clone_in(allocator))
            }
            Self::TSInstantiationExpression(it) => {
                ArrayExpressionElement::TSInstantiationExpression(it.clone_in(allocator))
            }
            Self::ComputedMemberExpression(it) => {
                ArrayExpressionElement::ComputedMemberExpression(it.clone_in(allocator))
            }
            Self::StaticMemberExpression(it) => {
                ArrayExpressionElement::StaticMemberExpression(it.clone_in(allocator))
            }
            Self::PrivateFieldExpression(it) => {
                ArrayExpressionElement::PrivateFieldExpression(it.clone_in(allocator))
            }
        }
    }
}

impl<'alloc> CloneIn<'alloc> for Elision {
    type Cloned = Elision;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        Elision { span: self.span.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ObjectExpression<'old_alloc> {
    type Cloned = ObjectExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ObjectExpression {
            span: self.span.clone_in(allocator),
            properties: self.properties.clone_in(allocator),
            trailing_comma: self.trailing_comma.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ObjectPropertyKind<'old_alloc> {
    type Cloned = ObjectPropertyKind<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::ObjectProperty(it) => ObjectPropertyKind::ObjectProperty(it.clone_in(allocator)),
            Self::SpreadProperty(it) => ObjectPropertyKind::SpreadProperty(it.clone_in(allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ObjectProperty<'old_alloc> {
    type Cloned = ObjectProperty<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ObjectProperty {
            span: self.span.clone_in(allocator),
            kind: self.kind.clone_in(allocator),
            key: self.key.clone_in(allocator),
            value: self.value.clone_in(allocator),
            init: self.init.clone_in(allocator),
            method: self.method.clone_in(allocator),
            shorthand: self.shorthand.clone_in(allocator),
            computed: self.computed.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for PropertyKey<'old_alloc> {
    type Cloned = PropertyKey<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::StaticIdentifier(it) => PropertyKey::StaticIdentifier(it.clone_in(allocator)),
            Self::PrivateIdentifier(it) => PropertyKey::PrivateIdentifier(it.clone_in(allocator)),
            Self::BooleanLiteral(it) => PropertyKey::BooleanLiteral(it.clone_in(allocator)),
            Self::NullLiteral(it) => PropertyKey::NullLiteral(it.clone_in(allocator)),
            Self::NumericLiteral(it) => PropertyKey::NumericLiteral(it.clone_in(allocator)),
            Self::BigIntLiteral(it) => PropertyKey::BigIntLiteral(it.clone_in(allocator)),
            Self::RegExpLiteral(it) => PropertyKey::RegExpLiteral(it.clone_in(allocator)),
            Self::StringLiteral(it) => PropertyKey::StringLiteral(it.clone_in(allocator)),
            Self::TemplateLiteral(it) => PropertyKey::TemplateLiteral(it.clone_in(allocator)),
            Self::Identifier(it) => PropertyKey::Identifier(it.clone_in(allocator)),
            Self::MetaProperty(it) => PropertyKey::MetaProperty(it.clone_in(allocator)),
            Self::Super(it) => PropertyKey::Super(it.clone_in(allocator)),
            Self::ArrayExpression(it) => PropertyKey::ArrayExpression(it.clone_in(allocator)),
            Self::ArrowFunctionExpression(it) => {
                PropertyKey::ArrowFunctionExpression(it.clone_in(allocator))
            }
            Self::AssignmentExpression(it) => {
                PropertyKey::AssignmentExpression(it.clone_in(allocator))
            }
            Self::AwaitExpression(it) => PropertyKey::AwaitExpression(it.clone_in(allocator)),
            Self::BinaryExpression(it) => PropertyKey::BinaryExpression(it.clone_in(allocator)),
            Self::CallExpression(it) => PropertyKey::CallExpression(it.clone_in(allocator)),
            Self::ChainExpression(it) => PropertyKey::ChainExpression(it.clone_in(allocator)),
            Self::ClassExpression(it) => PropertyKey::ClassExpression(it.clone_in(allocator)),
            Self::ConditionalExpression(it) => {
                PropertyKey::ConditionalExpression(it.clone_in(allocator))
            }
            Self::FunctionExpression(it) => PropertyKey::FunctionExpression(it.clone_in(allocator)),
            Self::ImportExpression(it) => PropertyKey::ImportExpression(it.clone_in(allocator)),
            Self::LogicalExpression(it) => PropertyKey::LogicalExpression(it.clone_in(allocator)),
            Self::NewExpression(it) => PropertyKey::NewExpression(it.clone_in(allocator)),
            Self::ObjectExpression(it) => PropertyKey::ObjectExpression(it.clone_in(allocator)),
            Self::ParenthesizedExpression(it) => {
                PropertyKey::ParenthesizedExpression(it.clone_in(allocator))
            }
            Self::SequenceExpression(it) => PropertyKey::SequenceExpression(it.clone_in(allocator)),
            Self::TaggedTemplateExpression(it) => {
                PropertyKey::TaggedTemplateExpression(it.clone_in(allocator))
            }
            Self::ThisExpression(it) => PropertyKey::ThisExpression(it.clone_in(allocator)),
            Self::UnaryExpression(it) => PropertyKey::UnaryExpression(it.clone_in(allocator)),
            Self::UpdateExpression(it) => PropertyKey::UpdateExpression(it.clone_in(allocator)),
            Self::YieldExpression(it) => PropertyKey::YieldExpression(it.clone_in(allocator)),
            Self::PrivateInExpression(it) => {
                PropertyKey::PrivateInExpression(it.clone_in(allocator))
            }
            Self::JSXElement(it) => PropertyKey::JSXElement(it.clone_in(allocator)),
            Self::JSXFragment(it) => PropertyKey::JSXFragment(it.clone_in(allocator)),
            Self::TSAsExpression(it) => PropertyKey::TSAsExpression(it.clone_in(allocator)),
            Self::TSSatisfiesExpression(it) => {
                PropertyKey::TSSatisfiesExpression(it.clone_in(allocator))
            }
            Self::TSTypeAssertion(it) => PropertyKey::TSTypeAssertion(it.clone_in(allocator)),
            Self::TSNonNullExpression(it) => {
                PropertyKey::TSNonNullExpression(it.clone_in(allocator))
            }
            Self::TSInstantiationExpression(it) => {
                PropertyKey::TSInstantiationExpression(it.clone_in(allocator))
            }
            Self::ComputedMemberExpression(it) => {
                PropertyKey::ComputedMemberExpression(it.clone_in(allocator))
            }
            Self::StaticMemberExpression(it) => {
                PropertyKey::StaticMemberExpression(it.clone_in(allocator))
            }
            Self::PrivateFieldExpression(it) => {
                PropertyKey::PrivateFieldExpression(it.clone_in(allocator))
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
            span: self.span.clone_in(allocator),
            quasis: self.quasis.clone_in(allocator),
            expressions: self.expressions.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TaggedTemplateExpression<'old_alloc> {
    type Cloned = TaggedTemplateExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TaggedTemplateExpression {
            span: self.span.clone_in(allocator),
            tag: self.tag.clone_in(allocator),
            quasi: self.quasi.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TemplateElement<'old_alloc> {
    type Cloned = TemplateElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TemplateElement {
            span: self.span.clone_in(allocator),
            tail: self.tail.clone_in(allocator),
            value: self.value.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TemplateElementValue<'old_alloc> {
    type Cloned = TemplateElementValue<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TemplateElementValue {
            raw: self.raw.clone_in(allocator),
            cooked: self.cooked.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for MemberExpression<'old_alloc> {
    type Cloned = MemberExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::ComputedMemberExpression(it) => {
                MemberExpression::ComputedMemberExpression(it.clone_in(allocator))
            }
            Self::StaticMemberExpression(it) => {
                MemberExpression::StaticMemberExpression(it.clone_in(allocator))
            }
            Self::PrivateFieldExpression(it) => {
                MemberExpression::PrivateFieldExpression(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ComputedMemberExpression<'old_alloc> {
    type Cloned = ComputedMemberExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ComputedMemberExpression {
            span: self.span.clone_in(allocator),
            object: self.object.clone_in(allocator),
            expression: self.expression.clone_in(allocator),
            optional: self.optional.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for StaticMemberExpression<'old_alloc> {
    type Cloned = StaticMemberExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        StaticMemberExpression {
            span: self.span.clone_in(allocator),
            object: self.object.clone_in(allocator),
            property: self.property.clone_in(allocator),
            optional: self.optional.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for PrivateFieldExpression<'old_alloc> {
    type Cloned = PrivateFieldExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        PrivateFieldExpression {
            span: self.span.clone_in(allocator),
            object: self.object.clone_in(allocator),
            field: self.field.clone_in(allocator),
            optional: self.optional.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for CallExpression<'old_alloc> {
    type Cloned = CallExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        CallExpression {
            span: self.span.clone_in(allocator),
            callee: self.callee.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
            arguments: self.arguments.clone_in(allocator),
            optional: self.optional.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for NewExpression<'old_alloc> {
    type Cloned = NewExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        NewExpression {
            span: self.span.clone_in(allocator),
            callee: self.callee.clone_in(allocator),
            arguments: self.arguments.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for MetaProperty<'old_alloc> {
    type Cloned = MetaProperty<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        MetaProperty {
            span: self.span.clone_in(allocator),
            meta: self.meta.clone_in(allocator),
            property: self.property.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for SpreadElement<'old_alloc> {
    type Cloned = SpreadElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        SpreadElement {
            span: self.span.clone_in(allocator),
            argument: self.argument.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Argument<'old_alloc> {
    type Cloned = Argument<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::SpreadElement(it) => Argument::SpreadElement(it.clone_in(allocator)),
            Self::BooleanLiteral(it) => Argument::BooleanLiteral(it.clone_in(allocator)),
            Self::NullLiteral(it) => Argument::NullLiteral(it.clone_in(allocator)),
            Self::NumericLiteral(it) => Argument::NumericLiteral(it.clone_in(allocator)),
            Self::BigIntLiteral(it) => Argument::BigIntLiteral(it.clone_in(allocator)),
            Self::RegExpLiteral(it) => Argument::RegExpLiteral(it.clone_in(allocator)),
            Self::StringLiteral(it) => Argument::StringLiteral(it.clone_in(allocator)),
            Self::TemplateLiteral(it) => Argument::TemplateLiteral(it.clone_in(allocator)),
            Self::Identifier(it) => Argument::Identifier(it.clone_in(allocator)),
            Self::MetaProperty(it) => Argument::MetaProperty(it.clone_in(allocator)),
            Self::Super(it) => Argument::Super(it.clone_in(allocator)),
            Self::ArrayExpression(it) => Argument::ArrayExpression(it.clone_in(allocator)),
            Self::ArrowFunctionExpression(it) => {
                Argument::ArrowFunctionExpression(it.clone_in(allocator))
            }
            Self::AssignmentExpression(it) => {
                Argument::AssignmentExpression(it.clone_in(allocator))
            }
            Self::AwaitExpression(it) => Argument::AwaitExpression(it.clone_in(allocator)),
            Self::BinaryExpression(it) => Argument::BinaryExpression(it.clone_in(allocator)),
            Self::CallExpression(it) => Argument::CallExpression(it.clone_in(allocator)),
            Self::ChainExpression(it) => Argument::ChainExpression(it.clone_in(allocator)),
            Self::ClassExpression(it) => Argument::ClassExpression(it.clone_in(allocator)),
            Self::ConditionalExpression(it) => {
                Argument::ConditionalExpression(it.clone_in(allocator))
            }
            Self::FunctionExpression(it) => Argument::FunctionExpression(it.clone_in(allocator)),
            Self::ImportExpression(it) => Argument::ImportExpression(it.clone_in(allocator)),
            Self::LogicalExpression(it) => Argument::LogicalExpression(it.clone_in(allocator)),
            Self::NewExpression(it) => Argument::NewExpression(it.clone_in(allocator)),
            Self::ObjectExpression(it) => Argument::ObjectExpression(it.clone_in(allocator)),
            Self::ParenthesizedExpression(it) => {
                Argument::ParenthesizedExpression(it.clone_in(allocator))
            }
            Self::SequenceExpression(it) => Argument::SequenceExpression(it.clone_in(allocator)),
            Self::TaggedTemplateExpression(it) => {
                Argument::TaggedTemplateExpression(it.clone_in(allocator))
            }
            Self::ThisExpression(it) => Argument::ThisExpression(it.clone_in(allocator)),
            Self::UnaryExpression(it) => Argument::UnaryExpression(it.clone_in(allocator)),
            Self::UpdateExpression(it) => Argument::UpdateExpression(it.clone_in(allocator)),
            Self::YieldExpression(it) => Argument::YieldExpression(it.clone_in(allocator)),
            Self::PrivateInExpression(it) => Argument::PrivateInExpression(it.clone_in(allocator)),
            Self::JSXElement(it) => Argument::JSXElement(it.clone_in(allocator)),
            Self::JSXFragment(it) => Argument::JSXFragment(it.clone_in(allocator)),
            Self::TSAsExpression(it) => Argument::TSAsExpression(it.clone_in(allocator)),
            Self::TSSatisfiesExpression(it) => {
                Argument::TSSatisfiesExpression(it.clone_in(allocator))
            }
            Self::TSTypeAssertion(it) => Argument::TSTypeAssertion(it.clone_in(allocator)),
            Self::TSNonNullExpression(it) => Argument::TSNonNullExpression(it.clone_in(allocator)),
            Self::TSInstantiationExpression(it) => {
                Argument::TSInstantiationExpression(it.clone_in(allocator))
            }
            Self::ComputedMemberExpression(it) => {
                Argument::ComputedMemberExpression(it.clone_in(allocator))
            }
            Self::StaticMemberExpression(it) => {
                Argument::StaticMemberExpression(it.clone_in(allocator))
            }
            Self::PrivateFieldExpression(it) => {
                Argument::PrivateFieldExpression(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for UpdateExpression<'old_alloc> {
    type Cloned = UpdateExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        UpdateExpression {
            span: self.span.clone_in(allocator),
            operator: self.operator.clone_in(allocator),
            prefix: self.prefix.clone_in(allocator),
            argument: self.argument.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for UnaryExpression<'old_alloc> {
    type Cloned = UnaryExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        UnaryExpression {
            span: self.span.clone_in(allocator),
            operator: self.operator.clone_in(allocator),
            argument: self.argument.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BinaryExpression<'old_alloc> {
    type Cloned = BinaryExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BinaryExpression {
            span: self.span.clone_in(allocator),
            left: self.left.clone_in(allocator),
            operator: self.operator.clone_in(allocator),
            right: self.right.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for PrivateInExpression<'old_alloc> {
    type Cloned = PrivateInExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        PrivateInExpression {
            span: self.span.clone_in(allocator),
            left: self.left.clone_in(allocator),
            operator: self.operator.clone_in(allocator),
            right: self.right.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for LogicalExpression<'old_alloc> {
    type Cloned = LogicalExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        LogicalExpression {
            span: self.span.clone_in(allocator),
            left: self.left.clone_in(allocator),
            operator: self.operator.clone_in(allocator),
            right: self.right.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ConditionalExpression<'old_alloc> {
    type Cloned = ConditionalExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ConditionalExpression {
            span: self.span.clone_in(allocator),
            test: self.test.clone_in(allocator),
            consequent: self.consequent.clone_in(allocator),
            alternate: self.alternate.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentExpression<'old_alloc> {
    type Cloned = AssignmentExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentExpression {
            span: self.span.clone_in(allocator),
            operator: self.operator.clone_in(allocator),
            left: self.left.clone_in(allocator),
            right: self.right.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTarget<'old_alloc> {
    type Cloned = AssignmentTarget<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::AssignmentTargetIdentifier(it) => {
                AssignmentTarget::AssignmentTargetIdentifier(it.clone_in(allocator))
            }
            Self::TSAsExpression(it) => AssignmentTarget::TSAsExpression(it.clone_in(allocator)),
            Self::TSSatisfiesExpression(it) => {
                AssignmentTarget::TSSatisfiesExpression(it.clone_in(allocator))
            }
            Self::TSNonNullExpression(it) => {
                AssignmentTarget::TSNonNullExpression(it.clone_in(allocator))
            }
            Self::TSTypeAssertion(it) => AssignmentTarget::TSTypeAssertion(it.clone_in(allocator)),
            Self::TSInstantiationExpression(it) => {
                AssignmentTarget::TSInstantiationExpression(it.clone_in(allocator))
            }
            Self::ComputedMemberExpression(it) => {
                AssignmentTarget::ComputedMemberExpression(it.clone_in(allocator))
            }
            Self::StaticMemberExpression(it) => {
                AssignmentTarget::StaticMemberExpression(it.clone_in(allocator))
            }
            Self::PrivateFieldExpression(it) => {
                AssignmentTarget::PrivateFieldExpression(it.clone_in(allocator))
            }
            Self::ArrayAssignmentTarget(it) => {
                AssignmentTarget::ArrayAssignmentTarget(it.clone_in(allocator))
            }
            Self::ObjectAssignmentTarget(it) => {
                AssignmentTarget::ObjectAssignmentTarget(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for SimpleAssignmentTarget<'old_alloc> {
    type Cloned = SimpleAssignmentTarget<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::AssignmentTargetIdentifier(it) => {
                SimpleAssignmentTarget::AssignmentTargetIdentifier(it.clone_in(allocator))
            }
            Self::TSAsExpression(it) => {
                SimpleAssignmentTarget::TSAsExpression(it.clone_in(allocator))
            }
            Self::TSSatisfiesExpression(it) => {
                SimpleAssignmentTarget::TSSatisfiesExpression(it.clone_in(allocator))
            }
            Self::TSNonNullExpression(it) => {
                SimpleAssignmentTarget::TSNonNullExpression(it.clone_in(allocator))
            }
            Self::TSTypeAssertion(it) => {
                SimpleAssignmentTarget::TSTypeAssertion(it.clone_in(allocator))
            }
            Self::TSInstantiationExpression(it) => {
                SimpleAssignmentTarget::TSInstantiationExpression(it.clone_in(allocator))
            }
            Self::ComputedMemberExpression(it) => {
                SimpleAssignmentTarget::ComputedMemberExpression(it.clone_in(allocator))
            }
            Self::StaticMemberExpression(it) => {
                SimpleAssignmentTarget::StaticMemberExpression(it.clone_in(allocator))
            }
            Self::PrivateFieldExpression(it) => {
                SimpleAssignmentTarget::PrivateFieldExpression(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTargetPattern<'old_alloc> {
    type Cloned = AssignmentTargetPattern<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::ArrayAssignmentTarget(it) => {
                AssignmentTargetPattern::ArrayAssignmentTarget(it.clone_in(allocator))
            }
            Self::ObjectAssignmentTarget(it) => {
                AssignmentTargetPattern::ObjectAssignmentTarget(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ArrayAssignmentTarget<'old_alloc> {
    type Cloned = ArrayAssignmentTarget<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ArrayAssignmentTarget {
            span: self.span.clone_in(allocator),
            elements: self.elements.clone_in(allocator),
            rest: self.rest.clone_in(allocator),
            trailing_comma: self.trailing_comma.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ObjectAssignmentTarget<'old_alloc> {
    type Cloned = ObjectAssignmentTarget<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ObjectAssignmentTarget {
            span: self.span.clone_in(allocator),
            properties: self.properties.clone_in(allocator),
            rest: self.rest.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTargetRest<'old_alloc> {
    type Cloned = AssignmentTargetRest<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentTargetRest {
            span: self.span.clone_in(allocator),
            target: self.target.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTargetMaybeDefault<'old_alloc> {
    type Cloned = AssignmentTargetMaybeDefault<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::AssignmentTargetWithDefault(it) => {
                AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(it.clone_in(allocator))
            }
            Self::AssignmentTargetIdentifier(it) => {
                AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(it.clone_in(allocator))
            }
            Self::TSAsExpression(it) => {
                AssignmentTargetMaybeDefault::TSAsExpression(it.clone_in(allocator))
            }
            Self::TSSatisfiesExpression(it) => {
                AssignmentTargetMaybeDefault::TSSatisfiesExpression(it.clone_in(allocator))
            }
            Self::TSNonNullExpression(it) => {
                AssignmentTargetMaybeDefault::TSNonNullExpression(it.clone_in(allocator))
            }
            Self::TSTypeAssertion(it) => {
                AssignmentTargetMaybeDefault::TSTypeAssertion(it.clone_in(allocator))
            }
            Self::TSInstantiationExpression(it) => {
                AssignmentTargetMaybeDefault::TSInstantiationExpression(it.clone_in(allocator))
            }
            Self::ComputedMemberExpression(it) => {
                AssignmentTargetMaybeDefault::ComputedMemberExpression(it.clone_in(allocator))
            }
            Self::StaticMemberExpression(it) => {
                AssignmentTargetMaybeDefault::StaticMemberExpression(it.clone_in(allocator))
            }
            Self::PrivateFieldExpression(it) => {
                AssignmentTargetMaybeDefault::PrivateFieldExpression(it.clone_in(allocator))
            }
            Self::ArrayAssignmentTarget(it) => {
                AssignmentTargetMaybeDefault::ArrayAssignmentTarget(it.clone_in(allocator))
            }
            Self::ObjectAssignmentTarget(it) => {
                AssignmentTargetMaybeDefault::ObjectAssignmentTarget(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTargetWithDefault<'old_alloc> {
    type Cloned = AssignmentTargetWithDefault<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentTargetWithDefault {
            span: self.span.clone_in(allocator),
            binding: self.binding.clone_in(allocator),
            init: self.init.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTargetProperty<'old_alloc> {
    type Cloned = AssignmentTargetProperty<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => {
                AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(it.clone_in(allocator))
            }
            Self::AssignmentTargetPropertyProperty(it) => {
                AssignmentTargetProperty::AssignmentTargetPropertyProperty(it.clone_in(allocator))
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
            span: self.span.clone_in(allocator),
            binding: self.binding.clone_in(allocator),
            init: self.init.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTargetPropertyProperty<'old_alloc> {
    type Cloned = AssignmentTargetPropertyProperty<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentTargetPropertyProperty {
            span: self.span.clone_in(allocator),
            name: self.name.clone_in(allocator),
            binding: self.binding.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for SequenceExpression<'old_alloc> {
    type Cloned = SequenceExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        SequenceExpression {
            span: self.span.clone_in(allocator),
            expressions: self.expressions.clone_in(allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for Super {
    type Cloned = Super;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        Super { span: self.span.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AwaitExpression<'old_alloc> {
    type Cloned = AwaitExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AwaitExpression {
            span: self.span.clone_in(allocator),
            argument: self.argument.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ChainExpression<'old_alloc> {
    type Cloned = ChainExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ChainExpression {
            span: self.span.clone_in(allocator),
            expression: self.expression.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ChainElement<'old_alloc> {
    type Cloned = ChainElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::CallExpression(it) => ChainElement::CallExpression(it.clone_in(allocator)),
            Self::ComputedMemberExpression(it) => {
                ChainElement::ComputedMemberExpression(it.clone_in(allocator))
            }
            Self::StaticMemberExpression(it) => {
                ChainElement::StaticMemberExpression(it.clone_in(allocator))
            }
            Self::PrivateFieldExpression(it) => {
                ChainElement::PrivateFieldExpression(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ParenthesizedExpression<'old_alloc> {
    type Cloned = ParenthesizedExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ParenthesizedExpression {
            span: self.span.clone_in(allocator),
            expression: self.expression.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Statement<'old_alloc> {
    type Cloned = Statement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::BlockStatement(it) => Statement::BlockStatement(it.clone_in(allocator)),
            Self::BreakStatement(it) => Statement::BreakStatement(it.clone_in(allocator)),
            Self::ContinueStatement(it) => Statement::ContinueStatement(it.clone_in(allocator)),
            Self::DebuggerStatement(it) => Statement::DebuggerStatement(it.clone_in(allocator)),
            Self::DoWhileStatement(it) => Statement::DoWhileStatement(it.clone_in(allocator)),
            Self::EmptyStatement(it) => Statement::EmptyStatement(it.clone_in(allocator)),
            Self::ExpressionStatement(it) => Statement::ExpressionStatement(it.clone_in(allocator)),
            Self::ForInStatement(it) => Statement::ForInStatement(it.clone_in(allocator)),
            Self::ForOfStatement(it) => Statement::ForOfStatement(it.clone_in(allocator)),
            Self::ForStatement(it) => Statement::ForStatement(it.clone_in(allocator)),
            Self::IfStatement(it) => Statement::IfStatement(it.clone_in(allocator)),
            Self::LabeledStatement(it) => Statement::LabeledStatement(it.clone_in(allocator)),
            Self::ReturnStatement(it) => Statement::ReturnStatement(it.clone_in(allocator)),
            Self::SwitchStatement(it) => Statement::SwitchStatement(it.clone_in(allocator)),
            Self::ThrowStatement(it) => Statement::ThrowStatement(it.clone_in(allocator)),
            Self::TryStatement(it) => Statement::TryStatement(it.clone_in(allocator)),
            Self::WhileStatement(it) => Statement::WhileStatement(it.clone_in(allocator)),
            Self::WithStatement(it) => Statement::WithStatement(it.clone_in(allocator)),
            Self::VariableDeclaration(it) => Statement::VariableDeclaration(it.clone_in(allocator)),
            Self::FunctionDeclaration(it) => Statement::FunctionDeclaration(it.clone_in(allocator)),
            Self::ClassDeclaration(it) => Statement::ClassDeclaration(it.clone_in(allocator)),
            Self::TSTypeAliasDeclaration(it) => {
                Statement::TSTypeAliasDeclaration(it.clone_in(allocator))
            }
            Self::TSInterfaceDeclaration(it) => {
                Statement::TSInterfaceDeclaration(it.clone_in(allocator))
            }
            Self::TSEnumDeclaration(it) => Statement::TSEnumDeclaration(it.clone_in(allocator)),
            Self::TSModuleDeclaration(it) => Statement::TSModuleDeclaration(it.clone_in(allocator)),
            Self::TSImportEqualsDeclaration(it) => {
                Statement::TSImportEqualsDeclaration(it.clone_in(allocator))
            }
            Self::ImportDeclaration(it) => Statement::ImportDeclaration(it.clone_in(allocator)),
            Self::ExportAllDeclaration(it) => {
                Statement::ExportAllDeclaration(it.clone_in(allocator))
            }
            Self::ExportDefaultDeclaration(it) => {
                Statement::ExportDefaultDeclaration(it.clone_in(allocator))
            }
            Self::ExportNamedDeclaration(it) => {
                Statement::ExportNamedDeclaration(it.clone_in(allocator))
            }
            Self::TSExportAssignment(it) => Statement::TSExportAssignment(it.clone_in(allocator)),
            Self::TSNamespaceExportDeclaration(it) => {
                Statement::TSNamespaceExportDeclaration(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Directive<'old_alloc> {
    type Cloned = Directive<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Directive {
            span: self.span.clone_in(allocator),
            expression: self.expression.clone_in(allocator),
            directive: self.directive.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Hashbang<'old_alloc> {
    type Cloned = Hashbang<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Hashbang { span: self.span.clone_in(allocator), value: self.value.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BlockStatement<'old_alloc> {
    type Cloned = BlockStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BlockStatement {
            span: self.span.clone_in(allocator),
            body: self.body.clone_in(allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Declaration<'old_alloc> {
    type Cloned = Declaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::VariableDeclaration(it) => {
                Declaration::VariableDeclaration(it.clone_in(allocator))
            }
            Self::FunctionDeclaration(it) => {
                Declaration::FunctionDeclaration(it.clone_in(allocator))
            }
            Self::ClassDeclaration(it) => Declaration::ClassDeclaration(it.clone_in(allocator)),
            Self::TSTypeAliasDeclaration(it) => {
                Declaration::TSTypeAliasDeclaration(it.clone_in(allocator))
            }
            Self::TSInterfaceDeclaration(it) => {
                Declaration::TSInterfaceDeclaration(it.clone_in(allocator))
            }
            Self::TSEnumDeclaration(it) => Declaration::TSEnumDeclaration(it.clone_in(allocator)),
            Self::TSModuleDeclaration(it) => {
                Declaration::TSModuleDeclaration(it.clone_in(allocator))
            }
            Self::TSImportEqualsDeclaration(it) => {
                Declaration::TSImportEqualsDeclaration(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for VariableDeclaration<'old_alloc> {
    type Cloned = VariableDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        VariableDeclaration {
            span: self.span.clone_in(allocator),
            kind: self.kind.clone_in(allocator),
            declarations: self.declarations.clone_in(allocator),
            declare: self.declare.clone_in(allocator),
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
            span: self.span.clone_in(allocator),
            kind: self.kind.clone_in(allocator),
            id: self.id.clone_in(allocator),
            init: self.init.clone_in(allocator),
            definite: self.definite.clone_in(allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for EmptyStatement {
    type Cloned = EmptyStatement;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        EmptyStatement { span: self.span.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ExpressionStatement<'old_alloc> {
    type Cloned = ExpressionStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ExpressionStatement {
            span: self.span.clone_in(allocator),
            expression: self.expression.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for IfStatement<'old_alloc> {
    type Cloned = IfStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        IfStatement {
            span: self.span.clone_in(allocator),
            test: self.test.clone_in(allocator),
            consequent: self.consequent.clone_in(allocator),
            alternate: self.alternate.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for DoWhileStatement<'old_alloc> {
    type Cloned = DoWhileStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        DoWhileStatement {
            span: self.span.clone_in(allocator),
            body: self.body.clone_in(allocator),
            test: self.test.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for WhileStatement<'old_alloc> {
    type Cloned = WhileStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        WhileStatement {
            span: self.span.clone_in(allocator),
            test: self.test.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ForStatement<'old_alloc> {
    type Cloned = ForStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ForStatement {
            span: self.span.clone_in(allocator),
            init: self.init.clone_in(allocator),
            test: self.test.clone_in(allocator),
            update: self.update.clone_in(allocator),
            body: self.body.clone_in(allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ForStatementInit<'old_alloc> {
    type Cloned = ForStatementInit<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::VariableDeclaration(it) => {
                ForStatementInit::VariableDeclaration(it.clone_in(allocator))
            }
            Self::BooleanLiteral(it) => ForStatementInit::BooleanLiteral(it.clone_in(allocator)),
            Self::NullLiteral(it) => ForStatementInit::NullLiteral(it.clone_in(allocator)),
            Self::NumericLiteral(it) => ForStatementInit::NumericLiteral(it.clone_in(allocator)),
            Self::BigIntLiteral(it) => ForStatementInit::BigIntLiteral(it.clone_in(allocator)),
            Self::RegExpLiteral(it) => ForStatementInit::RegExpLiteral(it.clone_in(allocator)),
            Self::StringLiteral(it) => ForStatementInit::StringLiteral(it.clone_in(allocator)),
            Self::TemplateLiteral(it) => ForStatementInit::TemplateLiteral(it.clone_in(allocator)),
            Self::Identifier(it) => ForStatementInit::Identifier(it.clone_in(allocator)),
            Self::MetaProperty(it) => ForStatementInit::MetaProperty(it.clone_in(allocator)),
            Self::Super(it) => ForStatementInit::Super(it.clone_in(allocator)),
            Self::ArrayExpression(it) => ForStatementInit::ArrayExpression(it.clone_in(allocator)),
            Self::ArrowFunctionExpression(it) => {
                ForStatementInit::ArrowFunctionExpression(it.clone_in(allocator))
            }
            Self::AssignmentExpression(it) => {
                ForStatementInit::AssignmentExpression(it.clone_in(allocator))
            }
            Self::AwaitExpression(it) => ForStatementInit::AwaitExpression(it.clone_in(allocator)),
            Self::BinaryExpression(it) => {
                ForStatementInit::BinaryExpression(it.clone_in(allocator))
            }
            Self::CallExpression(it) => ForStatementInit::CallExpression(it.clone_in(allocator)),
            Self::ChainExpression(it) => ForStatementInit::ChainExpression(it.clone_in(allocator)),
            Self::ClassExpression(it) => ForStatementInit::ClassExpression(it.clone_in(allocator)),
            Self::ConditionalExpression(it) => {
                ForStatementInit::ConditionalExpression(it.clone_in(allocator))
            }
            Self::FunctionExpression(it) => {
                ForStatementInit::FunctionExpression(it.clone_in(allocator))
            }
            Self::ImportExpression(it) => {
                ForStatementInit::ImportExpression(it.clone_in(allocator))
            }
            Self::LogicalExpression(it) => {
                ForStatementInit::LogicalExpression(it.clone_in(allocator))
            }
            Self::NewExpression(it) => ForStatementInit::NewExpression(it.clone_in(allocator)),
            Self::ObjectExpression(it) => {
                ForStatementInit::ObjectExpression(it.clone_in(allocator))
            }
            Self::ParenthesizedExpression(it) => {
                ForStatementInit::ParenthesizedExpression(it.clone_in(allocator))
            }
            Self::SequenceExpression(it) => {
                ForStatementInit::SequenceExpression(it.clone_in(allocator))
            }
            Self::TaggedTemplateExpression(it) => {
                ForStatementInit::TaggedTemplateExpression(it.clone_in(allocator))
            }
            Self::ThisExpression(it) => ForStatementInit::ThisExpression(it.clone_in(allocator)),
            Self::UnaryExpression(it) => ForStatementInit::UnaryExpression(it.clone_in(allocator)),
            Self::UpdateExpression(it) => {
                ForStatementInit::UpdateExpression(it.clone_in(allocator))
            }
            Self::YieldExpression(it) => ForStatementInit::YieldExpression(it.clone_in(allocator)),
            Self::PrivateInExpression(it) => {
                ForStatementInit::PrivateInExpression(it.clone_in(allocator))
            }
            Self::JSXElement(it) => ForStatementInit::JSXElement(it.clone_in(allocator)),
            Self::JSXFragment(it) => ForStatementInit::JSXFragment(it.clone_in(allocator)),
            Self::TSAsExpression(it) => ForStatementInit::TSAsExpression(it.clone_in(allocator)),
            Self::TSSatisfiesExpression(it) => {
                ForStatementInit::TSSatisfiesExpression(it.clone_in(allocator))
            }
            Self::TSTypeAssertion(it) => ForStatementInit::TSTypeAssertion(it.clone_in(allocator)),
            Self::TSNonNullExpression(it) => {
                ForStatementInit::TSNonNullExpression(it.clone_in(allocator))
            }
            Self::TSInstantiationExpression(it) => {
                ForStatementInit::TSInstantiationExpression(it.clone_in(allocator))
            }
            Self::ComputedMemberExpression(it) => {
                ForStatementInit::ComputedMemberExpression(it.clone_in(allocator))
            }
            Self::StaticMemberExpression(it) => {
                ForStatementInit::StaticMemberExpression(it.clone_in(allocator))
            }
            Self::PrivateFieldExpression(it) => {
                ForStatementInit::PrivateFieldExpression(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ForInStatement<'old_alloc> {
    type Cloned = ForInStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ForInStatement {
            span: self.span.clone_in(allocator),
            left: self.left.clone_in(allocator),
            right: self.right.clone_in(allocator),
            body: self.body.clone_in(allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ForStatementLeft<'old_alloc> {
    type Cloned = ForStatementLeft<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::VariableDeclaration(it) => {
                ForStatementLeft::VariableDeclaration(it.clone_in(allocator))
            }
            Self::AssignmentTargetIdentifier(it) => {
                ForStatementLeft::AssignmentTargetIdentifier(it.clone_in(allocator))
            }
            Self::TSAsExpression(it) => ForStatementLeft::TSAsExpression(it.clone_in(allocator)),
            Self::TSSatisfiesExpression(it) => {
                ForStatementLeft::TSSatisfiesExpression(it.clone_in(allocator))
            }
            Self::TSNonNullExpression(it) => {
                ForStatementLeft::TSNonNullExpression(it.clone_in(allocator))
            }
            Self::TSTypeAssertion(it) => ForStatementLeft::TSTypeAssertion(it.clone_in(allocator)),
            Self::TSInstantiationExpression(it) => {
                ForStatementLeft::TSInstantiationExpression(it.clone_in(allocator))
            }
            Self::ComputedMemberExpression(it) => {
                ForStatementLeft::ComputedMemberExpression(it.clone_in(allocator))
            }
            Self::StaticMemberExpression(it) => {
                ForStatementLeft::StaticMemberExpression(it.clone_in(allocator))
            }
            Self::PrivateFieldExpression(it) => {
                ForStatementLeft::PrivateFieldExpression(it.clone_in(allocator))
            }
            Self::ArrayAssignmentTarget(it) => {
                ForStatementLeft::ArrayAssignmentTarget(it.clone_in(allocator))
            }
            Self::ObjectAssignmentTarget(it) => {
                ForStatementLeft::ObjectAssignmentTarget(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ForOfStatement<'old_alloc> {
    type Cloned = ForOfStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ForOfStatement {
            span: self.span.clone_in(allocator),
            r#await: self.r#await.clone_in(allocator),
            left: self.left.clone_in(allocator),
            right: self.right.clone_in(allocator),
            body: self.body.clone_in(allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ContinueStatement<'old_alloc> {
    type Cloned = ContinueStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ContinueStatement {
            span: self.span.clone_in(allocator),
            label: self.label.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BreakStatement<'old_alloc> {
    type Cloned = BreakStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BreakStatement {
            span: self.span.clone_in(allocator),
            label: self.label.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ReturnStatement<'old_alloc> {
    type Cloned = ReturnStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ReturnStatement {
            span: self.span.clone_in(allocator),
            argument: self.argument.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for WithStatement<'old_alloc> {
    type Cloned = WithStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        WithStatement {
            span: self.span.clone_in(allocator),
            object: self.object.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for SwitchStatement<'old_alloc> {
    type Cloned = SwitchStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        SwitchStatement {
            span: self.span.clone_in(allocator),
            discriminant: self.discriminant.clone_in(allocator),
            cases: self.cases.clone_in(allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for SwitchCase<'old_alloc> {
    type Cloned = SwitchCase<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        SwitchCase {
            span: self.span.clone_in(allocator),
            test: self.test.clone_in(allocator),
            consequent: self.consequent.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for LabeledStatement<'old_alloc> {
    type Cloned = LabeledStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        LabeledStatement {
            span: self.span.clone_in(allocator),
            label: self.label.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ThrowStatement<'old_alloc> {
    type Cloned = ThrowStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ThrowStatement {
            span: self.span.clone_in(allocator),
            argument: self.argument.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TryStatement<'old_alloc> {
    type Cloned = TryStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TryStatement {
            span: self.span.clone_in(allocator),
            block: self.block.clone_in(allocator),
            handler: self.handler.clone_in(allocator),
            finalizer: self.finalizer.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for CatchClause<'old_alloc> {
    type Cloned = CatchClause<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        CatchClause {
            span: self.span.clone_in(allocator),
            param: self.param.clone_in(allocator),
            body: self.body.clone_in(allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for CatchParameter<'old_alloc> {
    type Cloned = CatchParameter<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        CatchParameter {
            span: self.span.clone_in(allocator),
            pattern: self.pattern.clone_in(allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for DebuggerStatement {
    type Cloned = DebuggerStatement;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        DebuggerStatement { span: self.span.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BindingPattern<'old_alloc> {
    type Cloned = BindingPattern<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BindingPattern {
            kind: self.kind.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
            optional: self.optional.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BindingPatternKind<'old_alloc> {
    type Cloned = BindingPatternKind<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::BindingIdentifier(it) => {
                BindingPatternKind::BindingIdentifier(it.clone_in(allocator))
            }
            Self::ObjectPattern(it) => BindingPatternKind::ObjectPattern(it.clone_in(allocator)),
            Self::ArrayPattern(it) => BindingPatternKind::ArrayPattern(it.clone_in(allocator)),
            Self::AssignmentPattern(it) => {
                BindingPatternKind::AssignmentPattern(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentPattern<'old_alloc> {
    type Cloned = AssignmentPattern<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentPattern {
            span: self.span.clone_in(allocator),
            left: self.left.clone_in(allocator),
            right: self.right.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ObjectPattern<'old_alloc> {
    type Cloned = ObjectPattern<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ObjectPattern {
            span: self.span.clone_in(allocator),
            properties: self.properties.clone_in(allocator),
            rest: self.rest.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BindingProperty<'old_alloc> {
    type Cloned = BindingProperty<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BindingProperty {
            span: self.span.clone_in(allocator),
            key: self.key.clone_in(allocator),
            value: self.value.clone_in(allocator),
            shorthand: self.shorthand.clone_in(allocator),
            computed: self.computed.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ArrayPattern<'old_alloc> {
    type Cloned = ArrayPattern<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ArrayPattern {
            span: self.span.clone_in(allocator),
            elements: self.elements.clone_in(allocator),
            rest: self.rest.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BindingRestElement<'old_alloc> {
    type Cloned = BindingRestElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BindingRestElement {
            span: self.span.clone_in(allocator),
            argument: self.argument.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Function<'old_alloc> {
    type Cloned = Function<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Function {
            r#type: self.r#type.clone_in(allocator),
            span: self.span.clone_in(allocator),
            id: self.id.clone_in(allocator),
            generator: self.generator.clone_in(allocator),
            r#async: self.r#async.clone_in(allocator),
            declare: self.declare.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
            this_param: self.this_param.clone_in(allocator),
            params: self.params.clone_in(allocator),
            return_type: self.return_type.clone_in(allocator),
            body: self.body.clone_in(allocator),
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
            span: self.span.clone_in(allocator),
            kind: self.kind.clone_in(allocator),
            items: self.items.clone_in(allocator),
            rest: self.rest.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for FormalParameter<'old_alloc> {
    type Cloned = FormalParameter<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        FormalParameter {
            span: self.span.clone_in(allocator),
            decorators: self.decorators.clone_in(allocator),
            pattern: self.pattern.clone_in(allocator),
            accessibility: self.accessibility.clone_in(allocator),
            readonly: self.readonly.clone_in(allocator),
            r#override: self.r#override.clone_in(allocator),
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
            span: self.span.clone_in(allocator),
            directives: self.directives.clone_in(allocator),
            statements: self.statements.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ArrowFunctionExpression<'old_alloc> {
    type Cloned = ArrowFunctionExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ArrowFunctionExpression {
            span: self.span.clone_in(allocator),
            expression: self.expression.clone_in(allocator),
            r#async: self.r#async.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
            params: self.params.clone_in(allocator),
            return_type: self.return_type.clone_in(allocator),
            body: self.body.clone_in(allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for YieldExpression<'old_alloc> {
    type Cloned = YieldExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        YieldExpression {
            span: self.span.clone_in(allocator),
            delegate: self.delegate.clone_in(allocator),
            argument: self.argument.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Class<'old_alloc> {
    type Cloned = Class<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Class {
            r#type: self.r#type.clone_in(allocator),
            span: self.span.clone_in(allocator),
            decorators: self.decorators.clone_in(allocator),
            id: self.id.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
            super_class: self.super_class.clone_in(allocator),
            super_type_parameters: self.super_type_parameters.clone_in(allocator),
            implements: self.implements.clone_in(allocator),
            body: self.body.clone_in(allocator),
            r#abstract: self.r#abstract.clone_in(allocator),
            declare: self.declare.clone_in(allocator),
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
        ClassBody { span: self.span.clone_in(allocator), body: self.body.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ClassElement<'old_alloc> {
    type Cloned = ClassElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::StaticBlock(it) => ClassElement::StaticBlock(it.clone_in(allocator)),
            Self::MethodDefinition(it) => ClassElement::MethodDefinition(it.clone_in(allocator)),
            Self::PropertyDefinition(it) => {
                ClassElement::PropertyDefinition(it.clone_in(allocator))
            }
            Self::AccessorProperty(it) => ClassElement::AccessorProperty(it.clone_in(allocator)),
            Self::TSIndexSignature(it) => ClassElement::TSIndexSignature(it.clone_in(allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for MethodDefinition<'old_alloc> {
    type Cloned = MethodDefinition<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        MethodDefinition {
            r#type: self.r#type.clone_in(allocator),
            span: self.span.clone_in(allocator),
            decorators: self.decorators.clone_in(allocator),
            key: self.key.clone_in(allocator),
            value: self.value.clone_in(allocator),
            kind: self.kind.clone_in(allocator),
            computed: self.computed.clone_in(allocator),
            r#static: self.r#static.clone_in(allocator),
            r#override: self.r#override.clone_in(allocator),
            optional: self.optional.clone_in(allocator),
            accessibility: self.accessibility.clone_in(allocator),
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
            r#type: self.r#type.clone_in(allocator),
            span: self.span.clone_in(allocator),
            decorators: self.decorators.clone_in(allocator),
            key: self.key.clone_in(allocator),
            value: self.value.clone_in(allocator),
            computed: self.computed.clone_in(allocator),
            r#static: self.r#static.clone_in(allocator),
            declare: self.declare.clone_in(allocator),
            r#override: self.r#override.clone_in(allocator),
            optional: self.optional.clone_in(allocator),
            definite: self.definite.clone_in(allocator),
            readonly: self.readonly.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
            accessibility: self.accessibility.clone_in(allocator),
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
            span: self.span.clone_in(allocator),
            name: self.name.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for StaticBlock<'old_alloc> {
    type Cloned = StaticBlock<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        StaticBlock {
            span: self.span.clone_in(allocator),
            body: self.body.clone_in(allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ModuleDeclaration<'old_alloc> {
    type Cloned = ModuleDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::ImportDeclaration(it) => {
                ModuleDeclaration::ImportDeclaration(it.clone_in(allocator))
            }
            Self::ExportAllDeclaration(it) => {
                ModuleDeclaration::ExportAllDeclaration(it.clone_in(allocator))
            }
            Self::ExportDefaultDeclaration(it) => {
                ModuleDeclaration::ExportDefaultDeclaration(it.clone_in(allocator))
            }
            Self::ExportNamedDeclaration(it) => {
                ModuleDeclaration::ExportNamedDeclaration(it.clone_in(allocator))
            }
            Self::TSExportAssignment(it) => {
                ModuleDeclaration::TSExportAssignment(it.clone_in(allocator))
            }
            Self::TSNamespaceExportDeclaration(it) => {
                ModuleDeclaration::TSNamespaceExportDeclaration(it.clone_in(allocator))
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
            r#type: self.r#type.clone_in(allocator),
            span: self.span.clone_in(allocator),
            decorators: self.decorators.clone_in(allocator),
            key: self.key.clone_in(allocator),
            value: self.value.clone_in(allocator),
            computed: self.computed.clone_in(allocator),
            r#static: self.r#static.clone_in(allocator),
            definite: self.definite.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
            accessibility: self.accessibility.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportExpression<'old_alloc> {
    type Cloned = ImportExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ImportExpression {
            span: self.span.clone_in(allocator),
            source: self.source.clone_in(allocator),
            arguments: self.arguments.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportDeclaration<'old_alloc> {
    type Cloned = ImportDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ImportDeclaration {
            span: self.span.clone_in(allocator),
            specifiers: self.specifiers.clone_in(allocator),
            source: self.source.clone_in(allocator),
            with_clause: self.with_clause.clone_in(allocator),
            import_kind: self.import_kind.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportDeclarationSpecifier<'old_alloc> {
    type Cloned = ImportDeclarationSpecifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::ImportSpecifier(it) => {
                ImportDeclarationSpecifier::ImportSpecifier(it.clone_in(allocator))
            }
            Self::ImportDefaultSpecifier(it) => {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(it.clone_in(allocator))
            }
            Self::ImportNamespaceSpecifier(it) => {
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportSpecifier<'old_alloc> {
    type Cloned = ImportSpecifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ImportSpecifier {
            span: self.span.clone_in(allocator),
            imported: self.imported.clone_in(allocator),
            local: self.local.clone_in(allocator),
            import_kind: self.import_kind.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportDefaultSpecifier<'old_alloc> {
    type Cloned = ImportDefaultSpecifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ImportDefaultSpecifier {
            span: self.span.clone_in(allocator),
            local: self.local.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportNamespaceSpecifier<'old_alloc> {
    type Cloned = ImportNamespaceSpecifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ImportNamespaceSpecifier {
            span: self.span.clone_in(allocator),
            local: self.local.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for WithClause<'old_alloc> {
    type Cloned = WithClause<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        WithClause {
            span: self.span.clone_in(allocator),
            attributes_keyword: self.attributes_keyword.clone_in(allocator),
            with_entries: self.with_entries.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportAttribute<'old_alloc> {
    type Cloned = ImportAttribute<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ImportAttribute {
            span: self.span.clone_in(allocator),
            key: self.key.clone_in(allocator),
            value: self.value.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportAttributeKey<'old_alloc> {
    type Cloned = ImportAttributeKey<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => ImportAttributeKey::Identifier(it.clone_in(allocator)),
            Self::StringLiteral(it) => ImportAttributeKey::StringLiteral(it.clone_in(allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ExportNamedDeclaration<'old_alloc> {
    type Cloned = ExportNamedDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ExportNamedDeclaration {
            span: self.span.clone_in(allocator),
            declaration: self.declaration.clone_in(allocator),
            specifiers: self.specifiers.clone_in(allocator),
            source: self.source.clone_in(allocator),
            export_kind: self.export_kind.clone_in(allocator),
            with_clause: self.with_clause.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ExportDefaultDeclaration<'old_alloc> {
    type Cloned = ExportDefaultDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ExportDefaultDeclaration {
            span: self.span.clone_in(allocator),
            declaration: self.declaration.clone_in(allocator),
            exported: self.exported.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ExportAllDeclaration<'old_alloc> {
    type Cloned = ExportAllDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ExportAllDeclaration {
            span: self.span.clone_in(allocator),
            exported: self.exported.clone_in(allocator),
            source: self.source.clone_in(allocator),
            with_clause: self.with_clause.clone_in(allocator),
            export_kind: self.export_kind.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ExportSpecifier<'old_alloc> {
    type Cloned = ExportSpecifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ExportSpecifier {
            span: self.span.clone_in(allocator),
            local: self.local.clone_in(allocator),
            exported: self.exported.clone_in(allocator),
            export_kind: self.export_kind.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ExportDefaultDeclarationKind<'old_alloc> {
    type Cloned = ExportDefaultDeclarationKind<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::FunctionDeclaration(it) => {
                ExportDefaultDeclarationKind::FunctionDeclaration(it.clone_in(allocator))
            }
            Self::ClassDeclaration(it) => {
                ExportDefaultDeclarationKind::ClassDeclaration(it.clone_in(allocator))
            }
            Self::TSInterfaceDeclaration(it) => {
                ExportDefaultDeclarationKind::TSInterfaceDeclaration(it.clone_in(allocator))
            }
            Self::BooleanLiteral(it) => {
                ExportDefaultDeclarationKind::BooleanLiteral(it.clone_in(allocator))
            }
            Self::NullLiteral(it) => {
                ExportDefaultDeclarationKind::NullLiteral(it.clone_in(allocator))
            }
            Self::NumericLiteral(it) => {
                ExportDefaultDeclarationKind::NumericLiteral(it.clone_in(allocator))
            }
            Self::BigIntLiteral(it) => {
                ExportDefaultDeclarationKind::BigIntLiteral(it.clone_in(allocator))
            }
            Self::RegExpLiteral(it) => {
                ExportDefaultDeclarationKind::RegExpLiteral(it.clone_in(allocator))
            }
            Self::StringLiteral(it) => {
                ExportDefaultDeclarationKind::StringLiteral(it.clone_in(allocator))
            }
            Self::TemplateLiteral(it) => {
                ExportDefaultDeclarationKind::TemplateLiteral(it.clone_in(allocator))
            }
            Self::Identifier(it) => {
                ExportDefaultDeclarationKind::Identifier(it.clone_in(allocator))
            }
            Self::MetaProperty(it) => {
                ExportDefaultDeclarationKind::MetaProperty(it.clone_in(allocator))
            }
            Self::Super(it) => ExportDefaultDeclarationKind::Super(it.clone_in(allocator)),
            Self::ArrayExpression(it) => {
                ExportDefaultDeclarationKind::ArrayExpression(it.clone_in(allocator))
            }
            Self::ArrowFunctionExpression(it) => {
                ExportDefaultDeclarationKind::ArrowFunctionExpression(it.clone_in(allocator))
            }
            Self::AssignmentExpression(it) => {
                ExportDefaultDeclarationKind::AssignmentExpression(it.clone_in(allocator))
            }
            Self::AwaitExpression(it) => {
                ExportDefaultDeclarationKind::AwaitExpression(it.clone_in(allocator))
            }
            Self::BinaryExpression(it) => {
                ExportDefaultDeclarationKind::BinaryExpression(it.clone_in(allocator))
            }
            Self::CallExpression(it) => {
                ExportDefaultDeclarationKind::CallExpression(it.clone_in(allocator))
            }
            Self::ChainExpression(it) => {
                ExportDefaultDeclarationKind::ChainExpression(it.clone_in(allocator))
            }
            Self::ClassExpression(it) => {
                ExportDefaultDeclarationKind::ClassExpression(it.clone_in(allocator))
            }
            Self::ConditionalExpression(it) => {
                ExportDefaultDeclarationKind::ConditionalExpression(it.clone_in(allocator))
            }
            Self::FunctionExpression(it) => {
                ExportDefaultDeclarationKind::FunctionExpression(it.clone_in(allocator))
            }
            Self::ImportExpression(it) => {
                ExportDefaultDeclarationKind::ImportExpression(it.clone_in(allocator))
            }
            Self::LogicalExpression(it) => {
                ExportDefaultDeclarationKind::LogicalExpression(it.clone_in(allocator))
            }
            Self::NewExpression(it) => {
                ExportDefaultDeclarationKind::NewExpression(it.clone_in(allocator))
            }
            Self::ObjectExpression(it) => {
                ExportDefaultDeclarationKind::ObjectExpression(it.clone_in(allocator))
            }
            Self::ParenthesizedExpression(it) => {
                ExportDefaultDeclarationKind::ParenthesizedExpression(it.clone_in(allocator))
            }
            Self::SequenceExpression(it) => {
                ExportDefaultDeclarationKind::SequenceExpression(it.clone_in(allocator))
            }
            Self::TaggedTemplateExpression(it) => {
                ExportDefaultDeclarationKind::TaggedTemplateExpression(it.clone_in(allocator))
            }
            Self::ThisExpression(it) => {
                ExportDefaultDeclarationKind::ThisExpression(it.clone_in(allocator))
            }
            Self::UnaryExpression(it) => {
                ExportDefaultDeclarationKind::UnaryExpression(it.clone_in(allocator))
            }
            Self::UpdateExpression(it) => {
                ExportDefaultDeclarationKind::UpdateExpression(it.clone_in(allocator))
            }
            Self::YieldExpression(it) => {
                ExportDefaultDeclarationKind::YieldExpression(it.clone_in(allocator))
            }
            Self::PrivateInExpression(it) => {
                ExportDefaultDeclarationKind::PrivateInExpression(it.clone_in(allocator))
            }
            Self::JSXElement(it) => {
                ExportDefaultDeclarationKind::JSXElement(it.clone_in(allocator))
            }
            Self::JSXFragment(it) => {
                ExportDefaultDeclarationKind::JSXFragment(it.clone_in(allocator))
            }
            Self::TSAsExpression(it) => {
                ExportDefaultDeclarationKind::TSAsExpression(it.clone_in(allocator))
            }
            Self::TSSatisfiesExpression(it) => {
                ExportDefaultDeclarationKind::TSSatisfiesExpression(it.clone_in(allocator))
            }
            Self::TSTypeAssertion(it) => {
                ExportDefaultDeclarationKind::TSTypeAssertion(it.clone_in(allocator))
            }
            Self::TSNonNullExpression(it) => {
                ExportDefaultDeclarationKind::TSNonNullExpression(it.clone_in(allocator))
            }
            Self::TSInstantiationExpression(it) => {
                ExportDefaultDeclarationKind::TSInstantiationExpression(it.clone_in(allocator))
            }
            Self::ComputedMemberExpression(it) => {
                ExportDefaultDeclarationKind::ComputedMemberExpression(it.clone_in(allocator))
            }
            Self::StaticMemberExpression(it) => {
                ExportDefaultDeclarationKind::StaticMemberExpression(it.clone_in(allocator))
            }
            Self::PrivateFieldExpression(it) => {
                ExportDefaultDeclarationKind::PrivateFieldExpression(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ModuleExportName<'old_alloc> {
    type Cloned = ModuleExportName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::IdentifierName(it) => ModuleExportName::IdentifierName(it.clone_in(allocator)),
            Self::IdentifierReference(it) => {
                ModuleExportName::IdentifierReference(it.clone_in(allocator))
            }
            Self::StringLiteral(it) => ModuleExportName::StringLiteral(it.clone_in(allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSThisParameter<'old_alloc> {
    type Cloned = TSThisParameter<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSThisParameter {
            span: self.span.clone_in(allocator),
            this: self.this.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSEnumDeclaration<'old_alloc> {
    type Cloned = TSEnumDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSEnumDeclaration {
            span: self.span.clone_in(allocator),
            id: self.id.clone_in(allocator),
            members: self.members.clone_in(allocator),
            r#const: self.r#const.clone_in(allocator),
            declare: self.declare.clone_in(allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSEnumMember<'old_alloc> {
    type Cloned = TSEnumMember<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSEnumMember {
            span: self.span.clone_in(allocator),
            id: self.id.clone_in(allocator),
            initializer: self.initializer.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSEnumMemberName<'old_alloc> {
    type Cloned = TSEnumMemberName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::StaticIdentifier(it) => {
                TSEnumMemberName::StaticIdentifier(it.clone_in(allocator))
            }
            Self::StaticStringLiteral(it) => {
                TSEnumMemberName::StaticStringLiteral(it.clone_in(allocator))
            }
            Self::StaticTemplateLiteral(it) => {
                TSEnumMemberName::StaticTemplateLiteral(it.clone_in(allocator))
            }
            Self::StaticNumericLiteral(it) => {
                TSEnumMemberName::StaticNumericLiteral(it.clone_in(allocator))
            }
            Self::BooleanLiteral(it) => TSEnumMemberName::BooleanLiteral(it.clone_in(allocator)),
            Self::NullLiteral(it) => TSEnumMemberName::NullLiteral(it.clone_in(allocator)),
            Self::NumericLiteral(it) => TSEnumMemberName::NumericLiteral(it.clone_in(allocator)),
            Self::BigIntLiteral(it) => TSEnumMemberName::BigIntLiteral(it.clone_in(allocator)),
            Self::RegExpLiteral(it) => TSEnumMemberName::RegExpLiteral(it.clone_in(allocator)),
            Self::StringLiteral(it) => TSEnumMemberName::StringLiteral(it.clone_in(allocator)),
            Self::TemplateLiteral(it) => TSEnumMemberName::TemplateLiteral(it.clone_in(allocator)),
            Self::Identifier(it) => TSEnumMemberName::Identifier(it.clone_in(allocator)),
            Self::MetaProperty(it) => TSEnumMemberName::MetaProperty(it.clone_in(allocator)),
            Self::Super(it) => TSEnumMemberName::Super(it.clone_in(allocator)),
            Self::ArrayExpression(it) => TSEnumMemberName::ArrayExpression(it.clone_in(allocator)),
            Self::ArrowFunctionExpression(it) => {
                TSEnumMemberName::ArrowFunctionExpression(it.clone_in(allocator))
            }
            Self::AssignmentExpression(it) => {
                TSEnumMemberName::AssignmentExpression(it.clone_in(allocator))
            }
            Self::AwaitExpression(it) => TSEnumMemberName::AwaitExpression(it.clone_in(allocator)),
            Self::BinaryExpression(it) => {
                TSEnumMemberName::BinaryExpression(it.clone_in(allocator))
            }
            Self::CallExpression(it) => TSEnumMemberName::CallExpression(it.clone_in(allocator)),
            Self::ChainExpression(it) => TSEnumMemberName::ChainExpression(it.clone_in(allocator)),
            Self::ClassExpression(it) => TSEnumMemberName::ClassExpression(it.clone_in(allocator)),
            Self::ConditionalExpression(it) => {
                TSEnumMemberName::ConditionalExpression(it.clone_in(allocator))
            }
            Self::FunctionExpression(it) => {
                TSEnumMemberName::FunctionExpression(it.clone_in(allocator))
            }
            Self::ImportExpression(it) => {
                TSEnumMemberName::ImportExpression(it.clone_in(allocator))
            }
            Self::LogicalExpression(it) => {
                TSEnumMemberName::LogicalExpression(it.clone_in(allocator))
            }
            Self::NewExpression(it) => TSEnumMemberName::NewExpression(it.clone_in(allocator)),
            Self::ObjectExpression(it) => {
                TSEnumMemberName::ObjectExpression(it.clone_in(allocator))
            }
            Self::ParenthesizedExpression(it) => {
                TSEnumMemberName::ParenthesizedExpression(it.clone_in(allocator))
            }
            Self::SequenceExpression(it) => {
                TSEnumMemberName::SequenceExpression(it.clone_in(allocator))
            }
            Self::TaggedTemplateExpression(it) => {
                TSEnumMemberName::TaggedTemplateExpression(it.clone_in(allocator))
            }
            Self::ThisExpression(it) => TSEnumMemberName::ThisExpression(it.clone_in(allocator)),
            Self::UnaryExpression(it) => TSEnumMemberName::UnaryExpression(it.clone_in(allocator)),
            Self::UpdateExpression(it) => {
                TSEnumMemberName::UpdateExpression(it.clone_in(allocator))
            }
            Self::YieldExpression(it) => TSEnumMemberName::YieldExpression(it.clone_in(allocator)),
            Self::PrivateInExpression(it) => {
                TSEnumMemberName::PrivateInExpression(it.clone_in(allocator))
            }
            Self::JSXElement(it) => TSEnumMemberName::JSXElement(it.clone_in(allocator)),
            Self::JSXFragment(it) => TSEnumMemberName::JSXFragment(it.clone_in(allocator)),
            Self::TSAsExpression(it) => TSEnumMemberName::TSAsExpression(it.clone_in(allocator)),
            Self::TSSatisfiesExpression(it) => {
                TSEnumMemberName::TSSatisfiesExpression(it.clone_in(allocator))
            }
            Self::TSTypeAssertion(it) => TSEnumMemberName::TSTypeAssertion(it.clone_in(allocator)),
            Self::TSNonNullExpression(it) => {
                TSEnumMemberName::TSNonNullExpression(it.clone_in(allocator))
            }
            Self::TSInstantiationExpression(it) => {
                TSEnumMemberName::TSInstantiationExpression(it.clone_in(allocator))
            }
            Self::ComputedMemberExpression(it) => {
                TSEnumMemberName::ComputedMemberExpression(it.clone_in(allocator))
            }
            Self::StaticMemberExpression(it) => {
                TSEnumMemberName::StaticMemberExpression(it.clone_in(allocator))
            }
            Self::PrivateFieldExpression(it) => {
                TSEnumMemberName::PrivateFieldExpression(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeAnnotation<'old_alloc> {
    type Cloned = TSTypeAnnotation<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeAnnotation {
            span: self.span.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSLiteralType<'old_alloc> {
    type Cloned = TSLiteralType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSLiteralType {
            span: self.span.clone_in(allocator),
            literal: self.literal.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSLiteral<'old_alloc> {
    type Cloned = TSLiteral<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::BooleanLiteral(it) => TSLiteral::BooleanLiteral(it.clone_in(allocator)),
            Self::NullLiteral(it) => TSLiteral::NullLiteral(it.clone_in(allocator)),
            Self::NumericLiteral(it) => TSLiteral::NumericLiteral(it.clone_in(allocator)),
            Self::BigIntLiteral(it) => TSLiteral::BigIntLiteral(it.clone_in(allocator)),
            Self::RegExpLiteral(it) => TSLiteral::RegExpLiteral(it.clone_in(allocator)),
            Self::StringLiteral(it) => TSLiteral::StringLiteral(it.clone_in(allocator)),
            Self::TemplateLiteral(it) => TSLiteral::TemplateLiteral(it.clone_in(allocator)),
            Self::UnaryExpression(it) => TSLiteral::UnaryExpression(it.clone_in(allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSType<'old_alloc> {
    type Cloned = TSType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::TSAnyKeyword(it) => TSType::TSAnyKeyword(it.clone_in(allocator)),
            Self::TSBigIntKeyword(it) => TSType::TSBigIntKeyword(it.clone_in(allocator)),
            Self::TSBooleanKeyword(it) => TSType::TSBooleanKeyword(it.clone_in(allocator)),
            Self::TSIntrinsicKeyword(it) => TSType::TSIntrinsicKeyword(it.clone_in(allocator)),
            Self::TSNeverKeyword(it) => TSType::TSNeverKeyword(it.clone_in(allocator)),
            Self::TSNullKeyword(it) => TSType::TSNullKeyword(it.clone_in(allocator)),
            Self::TSNumberKeyword(it) => TSType::TSNumberKeyword(it.clone_in(allocator)),
            Self::TSObjectKeyword(it) => TSType::TSObjectKeyword(it.clone_in(allocator)),
            Self::TSStringKeyword(it) => TSType::TSStringKeyword(it.clone_in(allocator)),
            Self::TSSymbolKeyword(it) => TSType::TSSymbolKeyword(it.clone_in(allocator)),
            Self::TSUndefinedKeyword(it) => TSType::TSUndefinedKeyword(it.clone_in(allocator)),
            Self::TSUnknownKeyword(it) => TSType::TSUnknownKeyword(it.clone_in(allocator)),
            Self::TSVoidKeyword(it) => TSType::TSVoidKeyword(it.clone_in(allocator)),
            Self::TSArrayType(it) => TSType::TSArrayType(it.clone_in(allocator)),
            Self::TSConditionalType(it) => TSType::TSConditionalType(it.clone_in(allocator)),
            Self::TSConstructorType(it) => TSType::TSConstructorType(it.clone_in(allocator)),
            Self::TSFunctionType(it) => TSType::TSFunctionType(it.clone_in(allocator)),
            Self::TSImportType(it) => TSType::TSImportType(it.clone_in(allocator)),
            Self::TSIndexedAccessType(it) => TSType::TSIndexedAccessType(it.clone_in(allocator)),
            Self::TSInferType(it) => TSType::TSInferType(it.clone_in(allocator)),
            Self::TSIntersectionType(it) => TSType::TSIntersectionType(it.clone_in(allocator)),
            Self::TSLiteralType(it) => TSType::TSLiteralType(it.clone_in(allocator)),
            Self::TSMappedType(it) => TSType::TSMappedType(it.clone_in(allocator)),
            Self::TSNamedTupleMember(it) => TSType::TSNamedTupleMember(it.clone_in(allocator)),
            Self::TSQualifiedName(it) => TSType::TSQualifiedName(it.clone_in(allocator)),
            Self::TSTemplateLiteralType(it) => {
                TSType::TSTemplateLiteralType(it.clone_in(allocator))
            }
            Self::TSThisType(it) => TSType::TSThisType(it.clone_in(allocator)),
            Self::TSTupleType(it) => TSType::TSTupleType(it.clone_in(allocator)),
            Self::TSTypeLiteral(it) => TSType::TSTypeLiteral(it.clone_in(allocator)),
            Self::TSTypeOperatorType(it) => TSType::TSTypeOperatorType(it.clone_in(allocator)),
            Self::TSTypePredicate(it) => TSType::TSTypePredicate(it.clone_in(allocator)),
            Self::TSTypeQuery(it) => TSType::TSTypeQuery(it.clone_in(allocator)),
            Self::TSTypeReference(it) => TSType::TSTypeReference(it.clone_in(allocator)),
            Self::TSUnionType(it) => TSType::TSUnionType(it.clone_in(allocator)),
            Self::TSParenthesizedType(it) => TSType::TSParenthesizedType(it.clone_in(allocator)),
            Self::JSDocNullableType(it) => TSType::JSDocNullableType(it.clone_in(allocator)),
            Self::JSDocNonNullableType(it) => TSType::JSDocNonNullableType(it.clone_in(allocator)),
            Self::JSDocUnknownType(it) => TSType::JSDocUnknownType(it.clone_in(allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSConditionalType<'old_alloc> {
    type Cloned = TSConditionalType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSConditionalType {
            span: self.span.clone_in(allocator),
            check_type: self.check_type.clone_in(allocator),
            extends_type: self.extends_type.clone_in(allocator),
            true_type: self.true_type.clone_in(allocator),
            false_type: self.false_type.clone_in(allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSUnionType<'old_alloc> {
    type Cloned = TSUnionType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSUnionType { span: self.span.clone_in(allocator), types: self.types.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSIntersectionType<'old_alloc> {
    type Cloned = TSIntersectionType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSIntersectionType {
            span: self.span.clone_in(allocator),
            types: self.types.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSParenthesizedType<'old_alloc> {
    type Cloned = TSParenthesizedType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSParenthesizedType {
            span: self.span.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeOperator<'old_alloc> {
    type Cloned = TSTypeOperator<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeOperator {
            span: self.span.clone_in(allocator),
            operator: self.operator.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
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
            span: self.span.clone_in(allocator),
            element_type: self.element_type.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSIndexedAccessType<'old_alloc> {
    type Cloned = TSIndexedAccessType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSIndexedAccessType {
            span: self.span.clone_in(allocator),
            object_type: self.object_type.clone_in(allocator),
            index_type: self.index_type.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTupleType<'old_alloc> {
    type Cloned = TSTupleType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTupleType {
            span: self.span.clone_in(allocator),
            element_types: self.element_types.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSNamedTupleMember<'old_alloc> {
    type Cloned = TSNamedTupleMember<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSNamedTupleMember {
            span: self.span.clone_in(allocator),
            element_type: self.element_type.clone_in(allocator),
            label: self.label.clone_in(allocator),
            optional: self.optional.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSOptionalType<'old_alloc> {
    type Cloned = TSOptionalType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSOptionalType {
            span: self.span.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSRestType<'old_alloc> {
    type Cloned = TSRestType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSRestType {
            span: self.span.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTupleElement<'old_alloc> {
    type Cloned = TSTupleElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::TSOptionalType(it) => TSTupleElement::TSOptionalType(it.clone_in(allocator)),
            Self::TSRestType(it) => TSTupleElement::TSRestType(it.clone_in(allocator)),
            Self::TSAnyKeyword(it) => TSTupleElement::TSAnyKeyword(it.clone_in(allocator)),
            Self::TSBigIntKeyword(it) => TSTupleElement::TSBigIntKeyword(it.clone_in(allocator)),
            Self::TSBooleanKeyword(it) => TSTupleElement::TSBooleanKeyword(it.clone_in(allocator)),
            Self::TSIntrinsicKeyword(it) => {
                TSTupleElement::TSIntrinsicKeyword(it.clone_in(allocator))
            }
            Self::TSNeverKeyword(it) => TSTupleElement::TSNeverKeyword(it.clone_in(allocator)),
            Self::TSNullKeyword(it) => TSTupleElement::TSNullKeyword(it.clone_in(allocator)),
            Self::TSNumberKeyword(it) => TSTupleElement::TSNumberKeyword(it.clone_in(allocator)),
            Self::TSObjectKeyword(it) => TSTupleElement::TSObjectKeyword(it.clone_in(allocator)),
            Self::TSStringKeyword(it) => TSTupleElement::TSStringKeyword(it.clone_in(allocator)),
            Self::TSSymbolKeyword(it) => TSTupleElement::TSSymbolKeyword(it.clone_in(allocator)),
            Self::TSUndefinedKeyword(it) => {
                TSTupleElement::TSUndefinedKeyword(it.clone_in(allocator))
            }
            Self::TSUnknownKeyword(it) => TSTupleElement::TSUnknownKeyword(it.clone_in(allocator)),
            Self::TSVoidKeyword(it) => TSTupleElement::TSVoidKeyword(it.clone_in(allocator)),
            Self::TSArrayType(it) => TSTupleElement::TSArrayType(it.clone_in(allocator)),
            Self::TSConditionalType(it) => {
                TSTupleElement::TSConditionalType(it.clone_in(allocator))
            }
            Self::TSConstructorType(it) => {
                TSTupleElement::TSConstructorType(it.clone_in(allocator))
            }
            Self::TSFunctionType(it) => TSTupleElement::TSFunctionType(it.clone_in(allocator)),
            Self::TSImportType(it) => TSTupleElement::TSImportType(it.clone_in(allocator)),
            Self::TSIndexedAccessType(it) => {
                TSTupleElement::TSIndexedAccessType(it.clone_in(allocator))
            }
            Self::TSInferType(it) => TSTupleElement::TSInferType(it.clone_in(allocator)),
            Self::TSIntersectionType(it) => {
                TSTupleElement::TSIntersectionType(it.clone_in(allocator))
            }
            Self::TSLiteralType(it) => TSTupleElement::TSLiteralType(it.clone_in(allocator)),
            Self::TSMappedType(it) => TSTupleElement::TSMappedType(it.clone_in(allocator)),
            Self::TSNamedTupleMember(it) => {
                TSTupleElement::TSNamedTupleMember(it.clone_in(allocator))
            }
            Self::TSQualifiedName(it) => TSTupleElement::TSQualifiedName(it.clone_in(allocator)),
            Self::TSTemplateLiteralType(it) => {
                TSTupleElement::TSTemplateLiteralType(it.clone_in(allocator))
            }
            Self::TSThisType(it) => TSTupleElement::TSThisType(it.clone_in(allocator)),
            Self::TSTupleType(it) => TSTupleElement::TSTupleType(it.clone_in(allocator)),
            Self::TSTypeLiteral(it) => TSTupleElement::TSTypeLiteral(it.clone_in(allocator)),
            Self::TSTypeOperatorType(it) => {
                TSTupleElement::TSTypeOperatorType(it.clone_in(allocator))
            }
            Self::TSTypePredicate(it) => TSTupleElement::TSTypePredicate(it.clone_in(allocator)),
            Self::TSTypeQuery(it) => TSTupleElement::TSTypeQuery(it.clone_in(allocator)),
            Self::TSTypeReference(it) => TSTupleElement::TSTypeReference(it.clone_in(allocator)),
            Self::TSUnionType(it) => TSTupleElement::TSUnionType(it.clone_in(allocator)),
            Self::TSParenthesizedType(it) => {
                TSTupleElement::TSParenthesizedType(it.clone_in(allocator))
            }
            Self::JSDocNullableType(it) => {
                TSTupleElement::JSDocNullableType(it.clone_in(allocator))
            }
            Self::JSDocNonNullableType(it) => {
                TSTupleElement::JSDocNonNullableType(it.clone_in(allocator))
            }
            Self::JSDocUnknownType(it) => TSTupleElement::JSDocUnknownType(it.clone_in(allocator)),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for TSAnyKeyword {
    type Cloned = TSAnyKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSAnyKeyword { span: self.span.clone_in(allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSStringKeyword {
    type Cloned = TSStringKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSStringKeyword { span: self.span.clone_in(allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSBooleanKeyword {
    type Cloned = TSBooleanKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSBooleanKeyword { span: self.span.clone_in(allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSNumberKeyword {
    type Cloned = TSNumberKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSNumberKeyword { span: self.span.clone_in(allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSNeverKeyword {
    type Cloned = TSNeverKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSNeverKeyword { span: self.span.clone_in(allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSIntrinsicKeyword {
    type Cloned = TSIntrinsicKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSIntrinsicKeyword { span: self.span.clone_in(allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSUnknownKeyword {
    type Cloned = TSUnknownKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSUnknownKeyword { span: self.span.clone_in(allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSNullKeyword {
    type Cloned = TSNullKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSNullKeyword { span: self.span.clone_in(allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSUndefinedKeyword {
    type Cloned = TSUndefinedKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSUndefinedKeyword { span: self.span.clone_in(allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSVoidKeyword {
    type Cloned = TSVoidKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSVoidKeyword { span: self.span.clone_in(allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSSymbolKeyword {
    type Cloned = TSSymbolKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSSymbolKeyword { span: self.span.clone_in(allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSThisType {
    type Cloned = TSThisType;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSThisType { span: self.span.clone_in(allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSObjectKeyword {
    type Cloned = TSObjectKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSObjectKeyword { span: self.span.clone_in(allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSBigIntKeyword {
    type Cloned = TSBigIntKeyword;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        TSBigIntKeyword { span: self.span.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeReference<'old_alloc> {
    type Cloned = TSTypeReference<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeReference {
            span: self.span.clone_in(allocator),
            type_name: self.type_name.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeName<'old_alloc> {
    type Cloned = TSTypeName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::IdentifierReference(it) => {
                TSTypeName::IdentifierReference(it.clone_in(allocator))
            }
            Self::QualifiedName(it) => TSTypeName::QualifiedName(it.clone_in(allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSQualifiedName<'old_alloc> {
    type Cloned = TSQualifiedName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSQualifiedName {
            span: self.span.clone_in(allocator),
            left: self.left.clone_in(allocator),
            right: self.right.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeParameterInstantiation<'old_alloc> {
    type Cloned = TSTypeParameterInstantiation<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeParameterInstantiation {
            span: self.span.clone_in(allocator),
            params: self.params.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeParameter<'old_alloc> {
    type Cloned = TSTypeParameter<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeParameter {
            span: self.span.clone_in(allocator),
            name: self.name.clone_in(allocator),
            constraint: self.constraint.clone_in(allocator),
            default: self.default.clone_in(allocator),
            r#in: self.r#in.clone_in(allocator),
            out: self.out.clone_in(allocator),
            r#const: self.r#const.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeParameterDeclaration<'old_alloc> {
    type Cloned = TSTypeParameterDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeParameterDeclaration {
            span: self.span.clone_in(allocator),
            params: self.params.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeAliasDeclaration<'old_alloc> {
    type Cloned = TSTypeAliasDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeAliasDeclaration {
            span: self.span.clone_in(allocator),
            id: self.id.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
            declare: self.declare.clone_in(allocator),
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
            span: self.span.clone_in(allocator),
            expression: self.expression.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSInterfaceDeclaration<'old_alloc> {
    type Cloned = TSInterfaceDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSInterfaceDeclaration {
            span: self.span.clone_in(allocator),
            id: self.id.clone_in(allocator),
            extends: self.extends.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
            body: self.body.clone_in(allocator),
            declare: self.declare.clone_in(allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSInterfaceBody<'old_alloc> {
    type Cloned = TSInterfaceBody<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSInterfaceBody { span: self.span.clone_in(allocator), body: self.body.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSPropertySignature<'old_alloc> {
    type Cloned = TSPropertySignature<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSPropertySignature {
            span: self.span.clone_in(allocator),
            computed: self.computed.clone_in(allocator),
            optional: self.optional.clone_in(allocator),
            readonly: self.readonly.clone_in(allocator),
            key: self.key.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSSignature<'old_alloc> {
    type Cloned = TSSignature<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::TSIndexSignature(it) => TSSignature::TSIndexSignature(it.clone_in(allocator)),
            Self::TSPropertySignature(it) => {
                TSSignature::TSPropertySignature(it.clone_in(allocator))
            }
            Self::TSCallSignatureDeclaration(it) => {
                TSSignature::TSCallSignatureDeclaration(it.clone_in(allocator))
            }
            Self::TSConstructSignatureDeclaration(it) => {
                TSSignature::TSConstructSignatureDeclaration(it.clone_in(allocator))
            }
            Self::TSMethodSignature(it) => TSSignature::TSMethodSignature(it.clone_in(allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSIndexSignature<'old_alloc> {
    type Cloned = TSIndexSignature<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSIndexSignature {
            span: self.span.clone_in(allocator),
            parameters: self.parameters.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
            readonly: self.readonly.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSCallSignatureDeclaration<'old_alloc> {
    type Cloned = TSCallSignatureDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSCallSignatureDeclaration {
            span: self.span.clone_in(allocator),
            this_param: self.this_param.clone_in(allocator),
            params: self.params.clone_in(allocator),
            return_type: self.return_type.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
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
            span: self.span.clone_in(allocator),
            key: self.key.clone_in(allocator),
            computed: self.computed.clone_in(allocator),
            optional: self.optional.clone_in(allocator),
            kind: self.kind.clone_in(allocator),
            this_param: self.this_param.clone_in(allocator),
            params: self.params.clone_in(allocator),
            return_type: self.return_type.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSConstructSignatureDeclaration<'old_alloc> {
    type Cloned = TSConstructSignatureDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSConstructSignatureDeclaration {
            span: self.span.clone_in(allocator),
            params: self.params.clone_in(allocator),
            return_type: self.return_type.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSIndexSignatureName<'old_alloc> {
    type Cloned = TSIndexSignatureName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSIndexSignatureName {
            span: self.span.clone_in(allocator),
            name: self.name.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSInterfaceHeritage<'old_alloc> {
    type Cloned = TSInterfaceHeritage<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSInterfaceHeritage {
            span: self.span.clone_in(allocator),
            expression: self.expression.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypePredicate<'old_alloc> {
    type Cloned = TSTypePredicate<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypePredicate {
            span: self.span.clone_in(allocator),
            parameter_name: self.parameter_name.clone_in(allocator),
            asserts: self.asserts.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypePredicateName<'old_alloc> {
    type Cloned = TSTypePredicateName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => TSTypePredicateName::Identifier(it.clone_in(allocator)),
            Self::This(it) => TSTypePredicateName::This(it.clone_in(allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSModuleDeclaration<'old_alloc> {
    type Cloned = TSModuleDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSModuleDeclaration {
            span: self.span.clone_in(allocator),
            id: self.id.clone_in(allocator),
            body: self.body.clone_in(allocator),
            kind: self.kind.clone_in(allocator),
            declare: self.declare.clone_in(allocator),
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
            Self::Identifier(it) => TSModuleDeclarationName::Identifier(it.clone_in(allocator)),
            Self::StringLiteral(it) => {
                TSModuleDeclarationName::StringLiteral(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSModuleDeclarationBody<'old_alloc> {
    type Cloned = TSModuleDeclarationBody<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::TSModuleDeclaration(it) => {
                TSModuleDeclarationBody::TSModuleDeclaration(it.clone_in(allocator))
            }
            Self::TSModuleBlock(it) => {
                TSModuleDeclarationBody::TSModuleBlock(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSModuleBlock<'old_alloc> {
    type Cloned = TSModuleBlock<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSModuleBlock {
            span: self.span.clone_in(allocator),
            directives: self.directives.clone_in(allocator),
            body: self.body.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeLiteral<'old_alloc> {
    type Cloned = TSTypeLiteral<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeLiteral {
            span: self.span.clone_in(allocator),
            members: self.members.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSInferType<'old_alloc> {
    type Cloned = TSInferType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSInferType {
            span: self.span.clone_in(allocator),
            type_parameter: self.type_parameter.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeQuery<'old_alloc> {
    type Cloned = TSTypeQuery<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeQuery {
            span: self.span.clone_in(allocator),
            expr_name: self.expr_name.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeQueryExprName<'old_alloc> {
    type Cloned = TSTypeQueryExprName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::TSImportType(it) => TSTypeQueryExprName::TSImportType(it.clone_in(allocator)),
            Self::IdentifierReference(it) => {
                TSTypeQueryExprName::IdentifierReference(it.clone_in(allocator))
            }
            Self::QualifiedName(it) => TSTypeQueryExprName::QualifiedName(it.clone_in(allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSImportType<'old_alloc> {
    type Cloned = TSImportType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSImportType {
            span: self.span.clone_in(allocator),
            is_type_of: self.is_type_of.clone_in(allocator),
            parameter: self.parameter.clone_in(allocator),
            qualifier: self.qualifier.clone_in(allocator),
            attributes: self.attributes.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSImportAttributes<'old_alloc> {
    type Cloned = TSImportAttributes<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSImportAttributes {
            span: self.span.clone_in(allocator),
            attributes_keyword: self.attributes_keyword.clone_in(allocator),
            elements: self.elements.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSImportAttribute<'old_alloc> {
    type Cloned = TSImportAttribute<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSImportAttribute {
            span: self.span.clone_in(allocator),
            name: self.name.clone_in(allocator),
            value: self.value.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSImportAttributeName<'old_alloc> {
    type Cloned = TSImportAttributeName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => TSImportAttributeName::Identifier(it.clone_in(allocator)),
            Self::StringLiteral(it) => TSImportAttributeName::StringLiteral(it.clone_in(allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSFunctionType<'old_alloc> {
    type Cloned = TSFunctionType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSFunctionType {
            span: self.span.clone_in(allocator),
            this_param: self.this_param.clone_in(allocator),
            params: self.params.clone_in(allocator),
            return_type: self.return_type.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSConstructorType<'old_alloc> {
    type Cloned = TSConstructorType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSConstructorType {
            span: self.span.clone_in(allocator),
            r#abstract: self.r#abstract.clone_in(allocator),
            params: self.params.clone_in(allocator),
            return_type: self.return_type.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSMappedType<'old_alloc> {
    type Cloned = TSMappedType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSMappedType {
            span: self.span.clone_in(allocator),
            type_parameter: self.type_parameter.clone_in(allocator),
            name_type: self.name_type.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
            optional: self.optional.clone_in(allocator),
            readonly: self.readonly.clone_in(allocator),
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
            span: self.span.clone_in(allocator),
            quasis: self.quasis.clone_in(allocator),
            types: self.types.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSAsExpression<'old_alloc> {
    type Cloned = TSAsExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSAsExpression {
            span: self.span.clone_in(allocator),
            expression: self.expression.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSSatisfiesExpression<'old_alloc> {
    type Cloned = TSSatisfiesExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSSatisfiesExpression {
            span: self.span.clone_in(allocator),
            expression: self.expression.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeAssertion<'old_alloc> {
    type Cloned = TSTypeAssertion<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeAssertion {
            span: self.span.clone_in(allocator),
            expression: self.expression.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSImportEqualsDeclaration<'old_alloc> {
    type Cloned = TSImportEqualsDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSImportEqualsDeclaration {
            span: self.span.clone_in(allocator),
            id: self.id.clone_in(allocator),
            module_reference: self.module_reference.clone_in(allocator),
            import_kind: self.import_kind.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSModuleReference<'old_alloc> {
    type Cloned = TSModuleReference<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::ExternalModuleReference(it) => {
                TSModuleReference::ExternalModuleReference(it.clone_in(allocator))
            }
            Self::IdentifierReference(it) => {
                TSModuleReference::IdentifierReference(it.clone_in(allocator))
            }
            Self::QualifiedName(it) => TSModuleReference::QualifiedName(it.clone_in(allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSExternalModuleReference<'old_alloc> {
    type Cloned = TSExternalModuleReference<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSExternalModuleReference {
            span: self.span.clone_in(allocator),
            expression: self.expression.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSNonNullExpression<'old_alloc> {
    type Cloned = TSNonNullExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSNonNullExpression {
            span: self.span.clone_in(allocator),
            expression: self.expression.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Decorator<'old_alloc> {
    type Cloned = Decorator<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Decorator {
            span: self.span.clone_in(allocator),
            expression: self.expression.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSExportAssignment<'old_alloc> {
    type Cloned = TSExportAssignment<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSExportAssignment {
            span: self.span.clone_in(allocator),
            expression: self.expression.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSNamespaceExportDeclaration<'old_alloc> {
    type Cloned = TSNamespaceExportDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSNamespaceExportDeclaration {
            span: self.span.clone_in(allocator),
            id: self.id.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSInstantiationExpression<'old_alloc> {
    type Cloned = TSInstantiationExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSInstantiationExpression {
            span: self.span.clone_in(allocator),
            expression: self.expression.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
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
            span: self.span.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
            postfix: self.postfix.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSDocNonNullableType<'old_alloc> {
    type Cloned = JSDocNonNullableType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSDocNonNullableType {
            span: self.span.clone_in(allocator),
            type_annotation: self.type_annotation.clone_in(allocator),
            postfix: self.postfix.clone_in(allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for JSDocUnknownType {
    type Cloned = JSDocUnknownType;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        JSDocUnknownType { span: self.span.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXElement<'old_alloc> {
    type Cloned = JSXElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXElement {
            span: self.span.clone_in(allocator),
            opening_element: self.opening_element.clone_in(allocator),
            closing_element: self.closing_element.clone_in(allocator),
            children: self.children.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXOpeningElement<'old_alloc> {
    type Cloned = JSXOpeningElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXOpeningElement {
            span: self.span.clone_in(allocator),
            self_closing: self.self_closing.clone_in(allocator),
            name: self.name.clone_in(allocator),
            attributes: self.attributes.clone_in(allocator),
            type_parameters: self.type_parameters.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXClosingElement<'old_alloc> {
    type Cloned = JSXClosingElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXClosingElement {
            span: self.span.clone_in(allocator),
            name: self.name.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXFragment<'old_alloc> {
    type Cloned = JSXFragment<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXFragment {
            span: self.span.clone_in(allocator),
            opening_fragment: self.opening_fragment.clone_in(allocator),
            closing_fragment: self.closing_fragment.clone_in(allocator),
            children: self.children.clone_in(allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for JSXOpeningFragment {
    type Cloned = JSXOpeningFragment;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        JSXOpeningFragment { span: self.span.clone_in(allocator) }
    }
}

impl<'alloc> CloneIn<'alloc> for JSXClosingFragment {
    type Cloned = JSXClosingFragment;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        JSXClosingFragment { span: self.span.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXElementName<'old_alloc> {
    type Cloned = JSXElementName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => JSXElementName::Identifier(it.clone_in(allocator)),
            Self::IdentifierReference(it) => {
                JSXElementName::IdentifierReference(it.clone_in(allocator))
            }
            Self::NamespacedName(it) => JSXElementName::NamespacedName(it.clone_in(allocator)),
            Self::MemberExpression(it) => JSXElementName::MemberExpression(it.clone_in(allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXNamespacedName<'old_alloc> {
    type Cloned = JSXNamespacedName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXNamespacedName {
            span: self.span.clone_in(allocator),
            namespace: self.namespace.clone_in(allocator),
            property: self.property.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXMemberExpression<'old_alloc> {
    type Cloned = JSXMemberExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXMemberExpression {
            span: self.span.clone_in(allocator),
            object: self.object.clone_in(allocator),
            property: self.property.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXMemberExpressionObject<'old_alloc> {
    type Cloned = JSXMemberExpressionObject<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::IdentifierReference(it) => {
                JSXMemberExpressionObject::IdentifierReference(it.clone_in(allocator))
            }
            Self::MemberExpression(it) => {
                JSXMemberExpressionObject::MemberExpression(it.clone_in(allocator))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXExpressionContainer<'old_alloc> {
    type Cloned = JSXExpressionContainer<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXExpressionContainer {
            span: self.span.clone_in(allocator),
            expression: self.expression.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXExpression<'old_alloc> {
    type Cloned = JSXExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::EmptyExpression(it) => JSXExpression::EmptyExpression(it.clone_in(allocator)),
            Self::BooleanLiteral(it) => JSXExpression::BooleanLiteral(it.clone_in(allocator)),
            Self::NullLiteral(it) => JSXExpression::NullLiteral(it.clone_in(allocator)),
            Self::NumericLiteral(it) => JSXExpression::NumericLiteral(it.clone_in(allocator)),
            Self::BigIntLiteral(it) => JSXExpression::BigIntLiteral(it.clone_in(allocator)),
            Self::RegExpLiteral(it) => JSXExpression::RegExpLiteral(it.clone_in(allocator)),
            Self::StringLiteral(it) => JSXExpression::StringLiteral(it.clone_in(allocator)),
            Self::TemplateLiteral(it) => JSXExpression::TemplateLiteral(it.clone_in(allocator)),
            Self::Identifier(it) => JSXExpression::Identifier(it.clone_in(allocator)),
            Self::MetaProperty(it) => JSXExpression::MetaProperty(it.clone_in(allocator)),
            Self::Super(it) => JSXExpression::Super(it.clone_in(allocator)),
            Self::ArrayExpression(it) => JSXExpression::ArrayExpression(it.clone_in(allocator)),
            Self::ArrowFunctionExpression(it) => {
                JSXExpression::ArrowFunctionExpression(it.clone_in(allocator))
            }
            Self::AssignmentExpression(it) => {
                JSXExpression::AssignmentExpression(it.clone_in(allocator))
            }
            Self::AwaitExpression(it) => JSXExpression::AwaitExpression(it.clone_in(allocator)),
            Self::BinaryExpression(it) => JSXExpression::BinaryExpression(it.clone_in(allocator)),
            Self::CallExpression(it) => JSXExpression::CallExpression(it.clone_in(allocator)),
            Self::ChainExpression(it) => JSXExpression::ChainExpression(it.clone_in(allocator)),
            Self::ClassExpression(it) => JSXExpression::ClassExpression(it.clone_in(allocator)),
            Self::ConditionalExpression(it) => {
                JSXExpression::ConditionalExpression(it.clone_in(allocator))
            }
            Self::FunctionExpression(it) => {
                JSXExpression::FunctionExpression(it.clone_in(allocator))
            }
            Self::ImportExpression(it) => JSXExpression::ImportExpression(it.clone_in(allocator)),
            Self::LogicalExpression(it) => JSXExpression::LogicalExpression(it.clone_in(allocator)),
            Self::NewExpression(it) => JSXExpression::NewExpression(it.clone_in(allocator)),
            Self::ObjectExpression(it) => JSXExpression::ObjectExpression(it.clone_in(allocator)),
            Self::ParenthesizedExpression(it) => {
                JSXExpression::ParenthesizedExpression(it.clone_in(allocator))
            }
            Self::SequenceExpression(it) => {
                JSXExpression::SequenceExpression(it.clone_in(allocator))
            }
            Self::TaggedTemplateExpression(it) => {
                JSXExpression::TaggedTemplateExpression(it.clone_in(allocator))
            }
            Self::ThisExpression(it) => JSXExpression::ThisExpression(it.clone_in(allocator)),
            Self::UnaryExpression(it) => JSXExpression::UnaryExpression(it.clone_in(allocator)),
            Self::UpdateExpression(it) => JSXExpression::UpdateExpression(it.clone_in(allocator)),
            Self::YieldExpression(it) => JSXExpression::YieldExpression(it.clone_in(allocator)),
            Self::PrivateInExpression(it) => {
                JSXExpression::PrivateInExpression(it.clone_in(allocator))
            }
            Self::JSXElement(it) => JSXExpression::JSXElement(it.clone_in(allocator)),
            Self::JSXFragment(it) => JSXExpression::JSXFragment(it.clone_in(allocator)),
            Self::TSAsExpression(it) => JSXExpression::TSAsExpression(it.clone_in(allocator)),
            Self::TSSatisfiesExpression(it) => {
                JSXExpression::TSSatisfiesExpression(it.clone_in(allocator))
            }
            Self::TSTypeAssertion(it) => JSXExpression::TSTypeAssertion(it.clone_in(allocator)),
            Self::TSNonNullExpression(it) => {
                JSXExpression::TSNonNullExpression(it.clone_in(allocator))
            }
            Self::TSInstantiationExpression(it) => {
                JSXExpression::TSInstantiationExpression(it.clone_in(allocator))
            }
            Self::ComputedMemberExpression(it) => {
                JSXExpression::ComputedMemberExpression(it.clone_in(allocator))
            }
            Self::StaticMemberExpression(it) => {
                JSXExpression::StaticMemberExpression(it.clone_in(allocator))
            }
            Self::PrivateFieldExpression(it) => {
                JSXExpression::PrivateFieldExpression(it.clone_in(allocator))
            }
        }
    }
}

impl<'alloc> CloneIn<'alloc> for JSXEmptyExpression {
    type Cloned = JSXEmptyExpression;
    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        JSXEmptyExpression { span: self.span.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXAttributeItem<'old_alloc> {
    type Cloned = JSXAttributeItem<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Attribute(it) => JSXAttributeItem::Attribute(it.clone_in(allocator)),
            Self::SpreadAttribute(it) => JSXAttributeItem::SpreadAttribute(it.clone_in(allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXAttribute<'old_alloc> {
    type Cloned = JSXAttribute<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXAttribute {
            span: self.span.clone_in(allocator),
            name: self.name.clone_in(allocator),
            value: self.value.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXSpreadAttribute<'old_alloc> {
    type Cloned = JSXSpreadAttribute<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXSpreadAttribute {
            span: self.span.clone_in(allocator),
            argument: self.argument.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXAttributeName<'old_alloc> {
    type Cloned = JSXAttributeName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => JSXAttributeName::Identifier(it.clone_in(allocator)),
            Self::NamespacedName(it) => JSXAttributeName::NamespacedName(it.clone_in(allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXAttributeValue<'old_alloc> {
    type Cloned = JSXAttributeValue<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::StringLiteral(it) => JSXAttributeValue::StringLiteral(it.clone_in(allocator)),
            Self::ExpressionContainer(it) => {
                JSXAttributeValue::ExpressionContainer(it.clone_in(allocator))
            }
            Self::Element(it) => JSXAttributeValue::Element(it.clone_in(allocator)),
            Self::Fragment(it) => JSXAttributeValue::Fragment(it.clone_in(allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXIdentifier<'old_alloc> {
    type Cloned = JSXIdentifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXIdentifier { span: self.span.clone_in(allocator), name: self.name.clone_in(allocator) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXChild<'old_alloc> {
    type Cloned = JSXChild<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Text(it) => JSXChild::Text(it.clone_in(allocator)),
            Self::Element(it) => JSXChild::Element(it.clone_in(allocator)),
            Self::Fragment(it) => JSXChild::Fragment(it.clone_in(allocator)),
            Self::ExpressionContainer(it) => JSXChild::ExpressionContainer(it.clone_in(allocator)),
            Self::Spread(it) => JSXChild::Spread(it.clone_in(allocator)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXSpreadChild<'old_alloc> {
    type Cloned = JSXSpreadChild<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXSpreadChild {
            span: self.span.clone_in(allocator),
            expression: self.expression.clone_in(allocator),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXText<'old_alloc> {
    type Cloned = JSXText<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXText { span: self.span.clone_in(allocator), value: self.value.clone_in(allocator) }
    }
}
