// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/clone_in.rs`

#![allow(clippy::default_trait_access)]

use oxc_allocator::{Allocator, CloneIn};

use crate::ast::comment::*;
use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
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

impl<'new_alloc> CloneIn<'new_alloc> for NumericLiteral<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for StringLiteral<'_> {
    type Cloned = StringLiteral<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        StringLiteral {
            span: CloneIn::clone_in(&self.span, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
            raw: CloneIn::clone_in(&self.raw, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BigIntLiteral<'_> {
    type Cloned = BigIntLiteral<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BigIntLiteral {
            span: CloneIn::clone_in(&self.span, allocator),
            raw: CloneIn::clone_in(&self.raw, allocator),
            base: CloneIn::clone_in(&self.base, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for RegExpLiteral<'_> {
    type Cloned = RegExpLiteral<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        RegExpLiteral {
            span: CloneIn::clone_in(&self.span, allocator),
            regex: CloneIn::clone_in(&self.regex, allocator),
            raw: CloneIn::clone_in(&self.raw, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for RegExp<'_> {
    type Cloned = RegExp<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        RegExp {
            pattern: CloneIn::clone_in(&self.pattern, allocator),
            flags: CloneIn::clone_in(&self.flags, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for RegExpPattern<'_> {
    type Cloned = RegExpPattern<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Raw(it) => RegExpPattern::Raw(CloneIn::clone_in(it, allocator)),
            Self::Invalid(it) => RegExpPattern::Invalid(CloneIn::clone_in(it, allocator)),
            Self::Pattern(it) => RegExpPattern::Pattern(CloneIn::clone_in(it, allocator)),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Program<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for Expression<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for IdentifierName<'_> {
    type Cloned = IdentifierName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        IdentifierName {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for IdentifierReference<'_> {
    type Cloned = IdentifierReference<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        IdentifierReference {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
            reference_id: Default::default(),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BindingIdentifier<'_> {
    type Cloned = BindingIdentifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BindingIdentifier {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
            symbol_id: Default::default(),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for LabelIdentifier<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ArrayExpression<'_> {
    type Cloned = ArrayExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ArrayExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            elements: CloneIn::clone_in(&self.elements, allocator),
            trailing_comma: CloneIn::clone_in(&self.trailing_comma, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ArrayExpressionElement<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ObjectExpression<'_> {
    type Cloned = ObjectExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ObjectExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            properties: CloneIn::clone_in(&self.properties, allocator),
            trailing_comma: CloneIn::clone_in(&self.trailing_comma, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ObjectPropertyKind<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ObjectProperty<'_> {
    type Cloned = ObjectProperty<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ObjectProperty {
            span: CloneIn::clone_in(&self.span, allocator),
            kind: CloneIn::clone_in(&self.kind, allocator),
            key: CloneIn::clone_in(&self.key, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
            method: CloneIn::clone_in(&self.method, allocator),
            shorthand: CloneIn::clone_in(&self.shorthand, allocator),
            computed: CloneIn::clone_in(&self.computed, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for PropertyKey<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TemplateLiteral<'_> {
    type Cloned = TemplateLiteral<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TemplateLiteral {
            span: CloneIn::clone_in(&self.span, allocator),
            quasis: CloneIn::clone_in(&self.quasis, allocator),
            expressions: CloneIn::clone_in(&self.expressions, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TaggedTemplateExpression<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TemplateElement<'_> {
    type Cloned = TemplateElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TemplateElement {
            span: CloneIn::clone_in(&self.span, allocator),
            tail: CloneIn::clone_in(&self.tail, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TemplateElementValue<'_> {
    type Cloned = TemplateElementValue<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TemplateElementValue {
            raw: CloneIn::clone_in(&self.raw, allocator),
            cooked: CloneIn::clone_in(&self.cooked, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for MemberExpression<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ComputedMemberExpression<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for StaticMemberExpression<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for PrivateFieldExpression<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for CallExpression<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for NewExpression<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for MetaProperty<'_> {
    type Cloned = MetaProperty<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        MetaProperty {
            span: CloneIn::clone_in(&self.span, allocator),
            meta: CloneIn::clone_in(&self.meta, allocator),
            property: CloneIn::clone_in(&self.property, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for SpreadElement<'_> {
    type Cloned = SpreadElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        SpreadElement {
            span: CloneIn::clone_in(&self.span, allocator),
            argument: CloneIn::clone_in(&self.argument, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Argument<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for UpdateExpression<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for UnaryExpression<'_> {
    type Cloned = UnaryExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        UnaryExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            operator: CloneIn::clone_in(&self.operator, allocator),
            argument: CloneIn::clone_in(&self.argument, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BinaryExpression<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for PrivateInExpression<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for LogicalExpression<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ConditionalExpression<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentExpression<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentTarget<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for SimpleAssignmentTarget<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentTargetPattern<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ArrayAssignmentTarget<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ObjectAssignmentTarget<'_> {
    type Cloned = ObjectAssignmentTarget<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ObjectAssignmentTarget {
            span: CloneIn::clone_in(&self.span, allocator),
            properties: CloneIn::clone_in(&self.properties, allocator),
            rest: CloneIn::clone_in(&self.rest, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentTargetRest<'_> {
    type Cloned = AssignmentTargetRest<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentTargetRest {
            span: CloneIn::clone_in(&self.span, allocator),
            target: CloneIn::clone_in(&self.target, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentTargetMaybeDefault<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentTargetWithDefault<'_> {
    type Cloned = AssignmentTargetWithDefault<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentTargetWithDefault {
            span: CloneIn::clone_in(&self.span, allocator),
            binding: CloneIn::clone_in(&self.binding, allocator),
            init: CloneIn::clone_in(&self.init, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentTargetProperty<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentTargetPropertyIdentifier<'_> {
    type Cloned = AssignmentTargetPropertyIdentifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentTargetPropertyIdentifier {
            span: CloneIn::clone_in(&self.span, allocator),
            binding: CloneIn::clone_in(&self.binding, allocator),
            init: CloneIn::clone_in(&self.init, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentTargetPropertyProperty<'_> {
    type Cloned = AssignmentTargetPropertyProperty<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentTargetPropertyProperty {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
            binding: CloneIn::clone_in(&self.binding, allocator),
            computed: CloneIn::clone_in(&self.computed, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for SequenceExpression<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for AwaitExpression<'_> {
    type Cloned = AwaitExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AwaitExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            argument: CloneIn::clone_in(&self.argument, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ChainExpression<'_> {
    type Cloned = ChainExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ChainExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ChainElement<'_> {
    type Cloned = ChainElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::CallExpression(it) => {
                ChainElement::CallExpression(CloneIn::clone_in(it, allocator))
            }
            Self::TSNonNullExpression(it) => {
                ChainElement::TSNonNullExpression(CloneIn::clone_in(it, allocator))
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

impl<'new_alloc> CloneIn<'new_alloc> for ParenthesizedExpression<'_> {
    type Cloned = ParenthesizedExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ParenthesizedExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Statement<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for Directive<'_> {
    type Cloned = Directive<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Directive {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
            directive: CloneIn::clone_in(&self.directive, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Hashbang<'_> {
    type Cloned = Hashbang<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Hashbang {
            span: CloneIn::clone_in(&self.span, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BlockStatement<'_> {
    type Cloned = BlockStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BlockStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Declaration<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for VariableDeclaration<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for VariableDeclarator<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ExpressionStatement<'_> {
    type Cloned = ExpressionStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ExpressionStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for IfStatement<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for DoWhileStatement<'_> {
    type Cloned = DoWhileStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        DoWhileStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
            test: CloneIn::clone_in(&self.test, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for WhileStatement<'_> {
    type Cloned = WhileStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        WhileStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            test: CloneIn::clone_in(&self.test, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ForStatement<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ForStatementInit<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ForInStatement<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ForStatementLeft<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ForOfStatement<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ContinueStatement<'_> {
    type Cloned = ContinueStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ContinueStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            label: CloneIn::clone_in(&self.label, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BreakStatement<'_> {
    type Cloned = BreakStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BreakStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            label: CloneIn::clone_in(&self.label, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ReturnStatement<'_> {
    type Cloned = ReturnStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ReturnStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            argument: CloneIn::clone_in(&self.argument, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for WithStatement<'_> {
    type Cloned = WithStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        WithStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            object: CloneIn::clone_in(&self.object, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for SwitchStatement<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for SwitchCase<'_> {
    type Cloned = SwitchCase<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        SwitchCase {
            span: CloneIn::clone_in(&self.span, allocator),
            test: CloneIn::clone_in(&self.test, allocator),
            consequent: CloneIn::clone_in(&self.consequent, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for LabeledStatement<'_> {
    type Cloned = LabeledStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        LabeledStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            label: CloneIn::clone_in(&self.label, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ThrowStatement<'_> {
    type Cloned = ThrowStatement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ThrowStatement {
            span: CloneIn::clone_in(&self.span, allocator),
            argument: CloneIn::clone_in(&self.argument, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TryStatement<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for CatchClause<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for CatchParameter<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for BindingPattern<'_> {
    type Cloned = BindingPattern<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BindingPattern {
            kind: CloneIn::clone_in(&self.kind, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
            optional: CloneIn::clone_in(&self.optional, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BindingPatternKind<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentPattern<'_> {
    type Cloned = AssignmentPattern<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentPattern {
            span: CloneIn::clone_in(&self.span, allocator),
            left: CloneIn::clone_in(&self.left, allocator),
            right: CloneIn::clone_in(&self.right, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ObjectPattern<'_> {
    type Cloned = ObjectPattern<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ObjectPattern {
            span: CloneIn::clone_in(&self.span, allocator),
            properties: CloneIn::clone_in(&self.properties, allocator),
            rest: CloneIn::clone_in(&self.rest, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BindingProperty<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ArrayPattern<'_> {
    type Cloned = ArrayPattern<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ArrayPattern {
            span: CloneIn::clone_in(&self.span, allocator),
            elements: CloneIn::clone_in(&self.elements, allocator),
            rest: CloneIn::clone_in(&self.rest, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BindingRestElement<'_> {
    type Cloned = BindingRestElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        BindingRestElement {
            span: CloneIn::clone_in(&self.span, allocator),
            argument: CloneIn::clone_in(&self.argument, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Function<'_> {
    type Cloned = Function<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Function {
            span: CloneIn::clone_in(&self.span, allocator),
            r#type: CloneIn::clone_in(&self.r#type, allocator),
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

impl<'new_alloc> CloneIn<'new_alloc> for FormalParameters<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for FormalParameter<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for FunctionBody<'_> {
    type Cloned = FunctionBody<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        FunctionBody {
            span: CloneIn::clone_in(&self.span, allocator),
            directives: CloneIn::clone_in(&self.directives, allocator),
            statements: CloneIn::clone_in(&self.statements, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ArrowFunctionExpression<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for YieldExpression<'_> {
    type Cloned = YieldExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        YieldExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            delegate: CloneIn::clone_in(&self.delegate, allocator),
            argument: CloneIn::clone_in(&self.argument, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Class<'_> {
    type Cloned = Class<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Class {
            span: CloneIn::clone_in(&self.span, allocator),
            r#type: CloneIn::clone_in(&self.r#type, allocator),
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

impl<'new_alloc> CloneIn<'new_alloc> for ClassBody<'_> {
    type Cloned = ClassBody<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ClassBody {
            span: CloneIn::clone_in(&self.span, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ClassElement<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for MethodDefinition<'_> {
    type Cloned = MethodDefinition<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        MethodDefinition {
            span: CloneIn::clone_in(&self.span, allocator),
            r#type: CloneIn::clone_in(&self.r#type, allocator),
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

impl<'new_alloc> CloneIn<'new_alloc> for PropertyDefinition<'_> {
    type Cloned = PropertyDefinition<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        PropertyDefinition {
            span: CloneIn::clone_in(&self.span, allocator),
            r#type: CloneIn::clone_in(&self.r#type, allocator),
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

impl<'new_alloc> CloneIn<'new_alloc> for PrivateIdentifier<'_> {
    type Cloned = PrivateIdentifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        PrivateIdentifier {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for StaticBlock<'_> {
    type Cloned = StaticBlock<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        StaticBlock {
            span: CloneIn::clone_in(&self.span, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
            scope_id: Default::default(),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ModuleDeclaration<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for AccessorProperty<'_> {
    type Cloned = AccessorProperty<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        AccessorProperty {
            span: CloneIn::clone_in(&self.span, allocator),
            r#type: CloneIn::clone_in(&self.r#type, allocator),
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

impl<'new_alloc> CloneIn<'new_alloc> for ImportExpression<'_> {
    type Cloned = ImportExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ImportExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            source: CloneIn::clone_in(&self.source, allocator),
            arguments: CloneIn::clone_in(&self.arguments, allocator),
            phase: CloneIn::clone_in(&self.phase, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ImportDeclaration<'_> {
    type Cloned = ImportDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ImportDeclaration {
            span: CloneIn::clone_in(&self.span, allocator),
            specifiers: CloneIn::clone_in(&self.specifiers, allocator),
            source: CloneIn::clone_in(&self.source, allocator),
            phase: CloneIn::clone_in(&self.phase, allocator),
            with_clause: CloneIn::clone_in(&self.with_clause, allocator),
            import_kind: CloneIn::clone_in(&self.import_kind, allocator),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for ImportPhase {
    type Cloned = ImportPhase;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Source => ImportPhase::Source,
            Self::Defer => ImportPhase::Defer,
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ImportDeclarationSpecifier<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ImportSpecifier<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ImportDefaultSpecifier<'_> {
    type Cloned = ImportDefaultSpecifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ImportDefaultSpecifier {
            span: CloneIn::clone_in(&self.span, allocator),
            local: CloneIn::clone_in(&self.local, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ImportNamespaceSpecifier<'_> {
    type Cloned = ImportNamespaceSpecifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ImportNamespaceSpecifier {
            span: CloneIn::clone_in(&self.span, allocator),
            local: CloneIn::clone_in(&self.local, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for WithClause<'_> {
    type Cloned = WithClause<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        WithClause {
            span: CloneIn::clone_in(&self.span, allocator),
            attributes_keyword: CloneIn::clone_in(&self.attributes_keyword, allocator),
            with_entries: CloneIn::clone_in(&self.with_entries, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ImportAttribute<'_> {
    type Cloned = ImportAttribute<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ImportAttribute {
            span: CloneIn::clone_in(&self.span, allocator),
            key: CloneIn::clone_in(&self.key, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ImportAttributeKey<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ExportNamedDeclaration<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ExportDefaultDeclaration<'_> {
    type Cloned = ExportDefaultDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        ExportDefaultDeclaration {
            span: CloneIn::clone_in(&self.span, allocator),
            declaration: CloneIn::clone_in(&self.declaration, allocator),
            exported: CloneIn::clone_in(&self.exported, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ExportAllDeclaration<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ExportSpecifier<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ExportDefaultDeclarationKind<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for ModuleExportName<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSThisParameter<'_> {
    type Cloned = TSThisParameter<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSThisParameter {
            span: CloneIn::clone_in(&self.span, allocator),
            this_span: CloneIn::clone_in(&self.this_span, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSEnumDeclaration<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSEnumMember<'_> {
    type Cloned = TSEnumMember<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSEnumMember {
            span: CloneIn::clone_in(&self.span, allocator),
            id: CloneIn::clone_in(&self.id, allocator),
            initializer: CloneIn::clone_in(&self.initializer, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSEnumMemberName<'_> {
    type Cloned = TSEnumMemberName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => TSEnumMemberName::Identifier(CloneIn::clone_in(it, allocator)),
            Self::String(it) => TSEnumMemberName::String(CloneIn::clone_in(it, allocator)),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeAnnotation<'_> {
    type Cloned = TSTypeAnnotation<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeAnnotation {
            span: CloneIn::clone_in(&self.span, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSLiteralType<'_> {
    type Cloned = TSLiteralType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSLiteralType {
            span: CloneIn::clone_in(&self.span, allocator),
            literal: CloneIn::clone_in(&self.literal, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSLiteral<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSType<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSConditionalType<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSUnionType<'_> {
    type Cloned = TSUnionType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSUnionType {
            span: CloneIn::clone_in(&self.span, allocator),
            types: CloneIn::clone_in(&self.types, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSIntersectionType<'_> {
    type Cloned = TSIntersectionType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSIntersectionType {
            span: CloneIn::clone_in(&self.span, allocator),
            types: CloneIn::clone_in(&self.types, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSParenthesizedType<'_> {
    type Cloned = TSParenthesizedType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSParenthesizedType {
            span: CloneIn::clone_in(&self.span, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeOperator<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSArrayType<'_> {
    type Cloned = TSArrayType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSArrayType {
            span: CloneIn::clone_in(&self.span, allocator),
            element_type: CloneIn::clone_in(&self.element_type, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSIndexedAccessType<'_> {
    type Cloned = TSIndexedAccessType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSIndexedAccessType {
            span: CloneIn::clone_in(&self.span, allocator),
            object_type: CloneIn::clone_in(&self.object_type, allocator),
            index_type: CloneIn::clone_in(&self.index_type, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTupleType<'_> {
    type Cloned = TSTupleType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTupleType {
            span: CloneIn::clone_in(&self.span, allocator),
            element_types: CloneIn::clone_in(&self.element_types, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSNamedTupleMember<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSOptionalType<'_> {
    type Cloned = TSOptionalType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSOptionalType {
            span: CloneIn::clone_in(&self.span, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSRestType<'_> {
    type Cloned = TSRestType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSRestType {
            span: CloneIn::clone_in(&self.span, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTupleElement<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeReference<'_> {
    type Cloned = TSTypeReference<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeReference {
            span: CloneIn::clone_in(&self.span, allocator),
            type_name: CloneIn::clone_in(&self.type_name, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeName<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSQualifiedName<'_> {
    type Cloned = TSQualifiedName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSQualifiedName {
            span: CloneIn::clone_in(&self.span, allocator),
            left: CloneIn::clone_in(&self.left, allocator),
            right: CloneIn::clone_in(&self.right, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeParameterInstantiation<'_> {
    type Cloned = TSTypeParameterInstantiation<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeParameterInstantiation {
            span: CloneIn::clone_in(&self.span, allocator),
            params: CloneIn::clone_in(&self.params, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeParameter<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeParameterDeclaration<'_> {
    type Cloned = TSTypeParameterDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeParameterDeclaration {
            span: CloneIn::clone_in(&self.span, allocator),
            params: CloneIn::clone_in(&self.params, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeAliasDeclaration<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSClassImplements<'_> {
    type Cloned = TSClassImplements<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSClassImplements {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSInterfaceDeclaration<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSInterfaceBody<'_> {
    type Cloned = TSInterfaceBody<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSInterfaceBody {
            span: CloneIn::clone_in(&self.span, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSPropertySignature<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSSignature<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSIndexSignature<'_> {
    type Cloned = TSIndexSignature<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSIndexSignature {
            span: CloneIn::clone_in(&self.span, allocator),
            parameters: CloneIn::clone_in(&self.parameters, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
            readonly: CloneIn::clone_in(&self.readonly, allocator),
            r#static: CloneIn::clone_in(&self.r#static, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSCallSignatureDeclaration<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSMethodSignature<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSConstructSignatureDeclaration<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSIndexSignatureName<'_> {
    type Cloned = TSIndexSignatureName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSIndexSignatureName {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSInterfaceHeritage<'_> {
    type Cloned = TSInterfaceHeritage<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSInterfaceHeritage {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypePredicate<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSTypePredicateName<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSModuleDeclaration<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSModuleDeclarationName<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSModuleDeclarationBody<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSModuleBlock<'_> {
    type Cloned = TSModuleBlock<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSModuleBlock {
            span: CloneIn::clone_in(&self.span, allocator),
            directives: CloneIn::clone_in(&self.directives, allocator),
            body: CloneIn::clone_in(&self.body, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeLiteral<'_> {
    type Cloned = TSTypeLiteral<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeLiteral {
            span: CloneIn::clone_in(&self.span, allocator),
            members: CloneIn::clone_in(&self.members, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSInferType<'_> {
    type Cloned = TSInferType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSInferType {
            span: CloneIn::clone_in(&self.span, allocator),
            type_parameter: CloneIn::clone_in(&self.type_parameter, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeQuery<'_> {
    type Cloned = TSTypeQuery<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeQuery {
            span: CloneIn::clone_in(&self.span, allocator),
            expr_name: CloneIn::clone_in(&self.expr_name, allocator),
            type_parameters: CloneIn::clone_in(&self.type_parameters, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeQueryExprName<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSImportType<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSImportAttributes<'_> {
    type Cloned = TSImportAttributes<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSImportAttributes {
            span: CloneIn::clone_in(&self.span, allocator),
            attributes_keyword: CloneIn::clone_in(&self.attributes_keyword, allocator),
            elements: CloneIn::clone_in(&self.elements, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSImportAttribute<'_> {
    type Cloned = TSImportAttribute<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSImportAttribute {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSImportAttributeName<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSFunctionType<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSConstructorType<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSMappedType<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSTemplateLiteralType<'_> {
    type Cloned = TSTemplateLiteralType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTemplateLiteralType {
            span: CloneIn::clone_in(&self.span, allocator),
            quasis: CloneIn::clone_in(&self.quasis, allocator),
            types: CloneIn::clone_in(&self.types, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSAsExpression<'_> {
    type Cloned = TSAsExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSAsExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSSatisfiesExpression<'_> {
    type Cloned = TSSatisfiesExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSSatisfiesExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeAssertion<'_> {
    type Cloned = TSTypeAssertion<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeAssertion {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSImportEqualsDeclaration<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSModuleReference<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for TSExternalModuleReference<'_> {
    type Cloned = TSExternalModuleReference<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSExternalModuleReference {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSNonNullExpression<'_> {
    type Cloned = TSNonNullExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSNonNullExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Decorator<'_> {
    type Cloned = Decorator<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Decorator {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSExportAssignment<'_> {
    type Cloned = TSExportAssignment<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSExportAssignment {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSNamespaceExportDeclaration<'_> {
    type Cloned = TSNamespaceExportDeclaration<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        TSNamespaceExportDeclaration {
            span: CloneIn::clone_in(&self.span, allocator),
            id: CloneIn::clone_in(&self.id, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSInstantiationExpression<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for JSDocNullableType<'_> {
    type Cloned = JSDocNullableType<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSDocNullableType {
            span: CloneIn::clone_in(&self.span, allocator),
            type_annotation: CloneIn::clone_in(&self.type_annotation, allocator),
            postfix: CloneIn::clone_in(&self.postfix, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSDocNonNullableType<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for JSXElement<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for JSXOpeningElement<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for JSXClosingElement<'_> {
    type Cloned = JSXClosingElement<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXClosingElement {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXFragment<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for JSXElementName<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for JSXNamespacedName<'_> {
    type Cloned = JSXNamespacedName<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXNamespacedName {
            span: CloneIn::clone_in(&self.span, allocator),
            namespace: CloneIn::clone_in(&self.namespace, allocator),
            property: CloneIn::clone_in(&self.property, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXMemberExpression<'_> {
    type Cloned = JSXMemberExpression<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXMemberExpression {
            span: CloneIn::clone_in(&self.span, allocator),
            object: CloneIn::clone_in(&self.object, allocator),
            property: CloneIn::clone_in(&self.property, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXMemberExpressionObject<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for JSXExpressionContainer<'_> {
    type Cloned = JSXExpressionContainer<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXExpressionContainer {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXExpression<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for JSXAttributeItem<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for JSXAttribute<'_> {
    type Cloned = JSXAttribute<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXAttribute {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
            value: CloneIn::clone_in(&self.value, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXSpreadAttribute<'_> {
    type Cloned = JSXSpreadAttribute<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXSpreadAttribute {
            span: CloneIn::clone_in(&self.span, allocator),
            argument: CloneIn::clone_in(&self.argument, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXAttributeName<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for JSXAttributeValue<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for JSXIdentifier<'_> {
    type Cloned = JSXIdentifier<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXIdentifier {
            span: CloneIn::clone_in(&self.span, allocator),
            name: CloneIn::clone_in(&self.name, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXChild<'_> {
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

impl<'new_alloc> CloneIn<'new_alloc> for JSXSpreadChild<'_> {
    type Cloned = JSXSpreadChild<'new_alloc>;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        JSXSpreadChild {
            span: CloneIn::clone_in(&self.span, allocator),
            expression: CloneIn::clone_in(&self.expression, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXText<'_> {
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
            attached_to: CloneIn::clone_in(&self.attached_to, allocator),
            kind: CloneIn::clone_in(&self.kind, allocator),
            position: CloneIn::clone_in(&self.position, allocator),
            preceded_by_newline: CloneIn::clone_in(&self.preceded_by_newline, allocator),
            followed_by_newline: CloneIn::clone_in(&self.followed_by_newline, allocator),
        }
    }
}
