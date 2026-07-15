// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/clone_in.rs`.

#![allow(unused_imports, unused_variables, clippy::default_trait_access, clippy::inline_always)]

use std::cell::Cell;

use oxc_allocator::{Allocator, CloneIn};

use crate::ast::comment::*;
use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
use crate::ast::ts::*;

impl<'new_alloc> CloneIn<'new_alloc> for Program<'_> {
    type Cloned = Program<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Program {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            source_type: CloneIn::clone_in_impl(&self.source_type, with_semantic_ids, allocator),
            source_text: CloneIn::clone_in_impl(&self.source_text, with_semantic_ids, allocator),
            comments: CloneIn::clone_in_impl(&self.comments, with_semantic_ids, allocator),
            hashbang: CloneIn::clone_in_impl(&self.hashbang, with_semantic_ids, allocator),
            directives: CloneIn::clone_in_impl(&self.directives, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Expression<'_> {
    type Cloned = Expression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::BooleanLiteral(it) => {
                Expression::BooleanLiteral(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::NullLiteral(it) => {
                Expression::NullLiteral(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::NumericLiteral(it) => {
                Expression::NumericLiteral(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::BigIntLiteral(it) => {
                Expression::BigIntLiteral(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::RegExpLiteral(it) => {
                Expression::RegExpLiteral(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::StringLiteral(it) => {
                Expression::StringLiteral(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TemplateLiteral(it) => Expression::TemplateLiteral(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::Identifier(it) => {
                Expression::Identifier(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::Super(it) => {
                Expression::Super(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::ArrayExpression(it) => Expression::ArrayExpression(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::ArrowFunctionExpression(it) => Expression::ArrowFunctionExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::AssignmentExpression(it) => Expression::AssignmentExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::AwaitExpression(it) => Expression::AwaitExpression(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::BinaryExpression(it) => Expression::BinaryExpression(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::CallExpression(it) => {
                Expression::CallExpression(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::ChainExpression(it) => Expression::ChainExpression(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::ClassExpression(it) => Expression::ClassExpression(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::ConditionalExpression(it) => Expression::ConditionalExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::FunctionExpression(it) => Expression::FunctionExpression(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::ImportExpression(it) => Expression::ImportExpression(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::LogicalExpression(it) => Expression::LogicalExpression(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::NewExpression(it) => {
                Expression::NewExpression(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::ObjectExpression(it) => Expression::ObjectExpression(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::ParenthesizedExpression(it) => Expression::ParenthesizedExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::SequenceExpression(it) => Expression::SequenceExpression(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::TaggedTemplateExpression(it) => Expression::TaggedTemplateExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::ThisExpression(it) => {
                Expression::ThisExpression(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::UnaryExpression(it) => Expression::UnaryExpression(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::UpdateExpression(it) => Expression::UpdateExpression(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::YieldExpression(it) => Expression::YieldExpression(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::PrivateInExpression(it) => Expression::PrivateInExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::ImportMeta(it) => {
                Expression::ImportMeta(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::NewTarget(it) => {
                Expression::NewTarget(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::JSXElement(it) => {
                Expression::JSXElement(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::JSXFragment(it) => {
                Expression::JSXFragment(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSAsExpression(it) => {
                Expression::TSAsExpression(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSSatisfiesExpression(it) => Expression::TSSatisfiesExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::TSTypeAssertion(it) => Expression::TSTypeAssertion(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::TSNonNullExpression(it) => Expression::TSNonNullExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::TSInstantiationExpression(it) => Expression::TSInstantiationExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::V8IntrinsicExpression(it) => Expression::V8IntrinsicExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::ComputedMemberExpression(_)
            | Self::StaticMemberExpression(_)
            | Self::PrivateFieldExpression(_) => Expression::from(CloneIn::clone_in_impl(
                self.to_member_expression(),
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for IdentifierName<'_> {
    type Cloned = IdentifierName<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        IdentifierName {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            name: CloneIn::clone_in_impl(&self.name, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for IdentifierReference<'_> {
    type Cloned = IdentifierReference<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        IdentifierReference {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            name: CloneIn::clone_in_impl(&self.name, with_semantic_ids, allocator),
            reference_id: CloneIn::clone_in_impl(&self.reference_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BindingIdentifier<'_> {
    type Cloned = BindingIdentifier<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        BindingIdentifier {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            name: CloneIn::clone_in_impl(&self.name, with_semantic_ids, allocator),
            symbol_id: CloneIn::clone_in_impl(&self.symbol_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for LabelIdentifier<'_> {
    type Cloned = LabelIdentifier<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        LabelIdentifier {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            name: CloneIn::clone_in_impl(&self.name, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ThisExpression {
    type Cloned = ThisExpression;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ThisExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ArrayExpression<'_> {
    type Cloned = ArrayExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ArrayExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            elements: CloneIn::clone_in_impl(&self.elements, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ArrayExpressionElement<'_> {
    type Cloned = ArrayExpressionElement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::SpreadElement(it) => ArrayExpressionElement::SpreadElement(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::Elision(it) => ArrayExpressionElement::Elision(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::BooleanLiteral(_)
            | Self::NullLiteral(_)
            | Self::NumericLiteral(_)
            | Self::BigIntLiteral(_)
            | Self::RegExpLiteral(_)
            | Self::StringLiteral(_)
            | Self::TemplateLiteral(_)
            | Self::Identifier(_)
            | Self::Super(_)
            | Self::ArrayExpression(_)
            | Self::ArrowFunctionExpression(_)
            | Self::AssignmentExpression(_)
            | Self::AwaitExpression(_)
            | Self::BinaryExpression(_)
            | Self::CallExpression(_)
            | Self::ChainExpression(_)
            | Self::ClassExpression(_)
            | Self::ConditionalExpression(_)
            | Self::FunctionExpression(_)
            | Self::ImportExpression(_)
            | Self::LogicalExpression(_)
            | Self::NewExpression(_)
            | Self::ObjectExpression(_)
            | Self::ParenthesizedExpression(_)
            | Self::SequenceExpression(_)
            | Self::TaggedTemplateExpression(_)
            | Self::ThisExpression(_)
            | Self::UnaryExpression(_)
            | Self::UpdateExpression(_)
            | Self::YieldExpression(_)
            | Self::PrivateInExpression(_)
            | Self::ImportMeta(_)
            | Self::NewTarget(_)
            | Self::JSXElement(_)
            | Self::JSXFragment(_)
            | Self::TSAsExpression(_)
            | Self::TSSatisfiesExpression(_)
            | Self::TSTypeAssertion(_)
            | Self::TSNonNullExpression(_)
            | Self::TSInstantiationExpression(_)
            | Self::V8IntrinsicExpression(_)
            | Self::ComputedMemberExpression(_)
            | Self::StaticMemberExpression(_)
            | Self::PrivateFieldExpression(_) => ArrayExpressionElement::from(
                CloneIn::clone_in_impl(self.to_expression(), with_semantic_ids, allocator),
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Elision {
    type Cloned = Elision;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Elision {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ObjectExpression<'_> {
    type Cloned = ObjectExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ObjectExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            properties: CloneIn::clone_in_impl(&self.properties, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ObjectPropertyKind<'_> {
    type Cloned = ObjectPropertyKind<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::ObjectProperty(it) => ObjectPropertyKind::ObjectProperty(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::SpreadProperty(it) => ObjectPropertyKind::SpreadProperty(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ObjectProperty<'_> {
    type Cloned = ObjectProperty<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ObjectProperty {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            kind: CloneIn::clone_in_impl(&self.kind, with_semantic_ids, allocator),
            key: CloneIn::clone_in_impl(&self.key, with_semantic_ids, allocator),
            value: CloneIn::clone_in_impl(&self.value, with_semantic_ids, allocator),
            method: CloneIn::clone_in_impl(&self.method, with_semantic_ids, allocator),
            shorthand: CloneIn::clone_in_impl(&self.shorthand, with_semantic_ids, allocator),
            computed: CloneIn::clone_in_impl(&self.computed, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for PropertyKey<'_> {
    type Cloned = PropertyKey<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::StaticIdentifier(it) => PropertyKey::StaticIdentifier(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::PrivateIdentifier(it) => PropertyKey::PrivateIdentifier(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::BooleanLiteral(_)
            | Self::NullLiteral(_)
            | Self::NumericLiteral(_)
            | Self::BigIntLiteral(_)
            | Self::RegExpLiteral(_)
            | Self::StringLiteral(_)
            | Self::TemplateLiteral(_)
            | Self::Identifier(_)
            | Self::Super(_)
            | Self::ArrayExpression(_)
            | Self::ArrowFunctionExpression(_)
            | Self::AssignmentExpression(_)
            | Self::AwaitExpression(_)
            | Self::BinaryExpression(_)
            | Self::CallExpression(_)
            | Self::ChainExpression(_)
            | Self::ClassExpression(_)
            | Self::ConditionalExpression(_)
            | Self::FunctionExpression(_)
            | Self::ImportExpression(_)
            | Self::LogicalExpression(_)
            | Self::NewExpression(_)
            | Self::ObjectExpression(_)
            | Self::ParenthesizedExpression(_)
            | Self::SequenceExpression(_)
            | Self::TaggedTemplateExpression(_)
            | Self::ThisExpression(_)
            | Self::UnaryExpression(_)
            | Self::UpdateExpression(_)
            | Self::YieldExpression(_)
            | Self::PrivateInExpression(_)
            | Self::ImportMeta(_)
            | Self::NewTarget(_)
            | Self::JSXElement(_)
            | Self::JSXFragment(_)
            | Self::TSAsExpression(_)
            | Self::TSSatisfiesExpression(_)
            | Self::TSTypeAssertion(_)
            | Self::TSNonNullExpression(_)
            | Self::TSInstantiationExpression(_)
            | Self::V8IntrinsicExpression(_)
            | Self::ComputedMemberExpression(_)
            | Self::StaticMemberExpression(_)
            | Self::PrivateFieldExpression(_) => PropertyKey::from(CloneIn::clone_in_impl(
                self.to_expression(),
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for PropertyKind {
    type Cloned = PropertyKind;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TemplateLiteral<'_> {
    type Cloned = TemplateLiteral<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TemplateLiteral {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            quasis: CloneIn::clone_in_impl(&self.quasis, with_semantic_ids, allocator),
            expressions: CloneIn::clone_in_impl(&self.expressions, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TaggedTemplateExpression<'_> {
    type Cloned = TaggedTemplateExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TaggedTemplateExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            tag: CloneIn::clone_in_impl(&self.tag, with_semantic_ids, allocator),
            type_arguments: CloneIn::clone_in_impl(
                &self.type_arguments,
                with_semantic_ids,
                allocator,
            ),
            quasi: CloneIn::clone_in_impl(&self.quasi, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TemplateElement<'_> {
    type Cloned = TemplateElement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TemplateElement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            value: CloneIn::clone_in_impl(&self.value, with_semantic_ids, allocator),
            tail: CloneIn::clone_in_impl(&self.tail, with_semantic_ids, allocator),
            lone_surrogates: CloneIn::clone_in_impl(
                &self.lone_surrogates,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TemplateElementValue<'_> {
    type Cloned = TemplateElementValue<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TemplateElementValue {
            raw: CloneIn::clone_in_impl(&self.raw, with_semantic_ids, allocator),
            cooked: CloneIn::clone_in_impl(&self.cooked, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for MemberExpression<'_> {
    type Cloned = MemberExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::ComputedMemberExpression(it) => MemberExpression::ComputedMemberExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::StaticMemberExpression(it) => MemberExpression::StaticMemberExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::PrivateFieldExpression(it) => MemberExpression::PrivateFieldExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ComputedMemberExpression<'_> {
    type Cloned = ComputedMemberExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ComputedMemberExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            object: CloneIn::clone_in_impl(&self.object, with_semantic_ids, allocator),
            expression: CloneIn::clone_in_impl(&self.expression, with_semantic_ids, allocator),
            optional: CloneIn::clone_in_impl(&self.optional, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for StaticMemberExpression<'_> {
    type Cloned = StaticMemberExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        StaticMemberExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            object: CloneIn::clone_in_impl(&self.object, with_semantic_ids, allocator),
            property: CloneIn::clone_in_impl(&self.property, with_semantic_ids, allocator),
            optional: CloneIn::clone_in_impl(&self.optional, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for PrivateFieldExpression<'_> {
    type Cloned = PrivateFieldExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        PrivateFieldExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            object: CloneIn::clone_in_impl(&self.object, with_semantic_ids, allocator),
            field: CloneIn::clone_in_impl(&self.field, with_semantic_ids, allocator),
            optional: CloneIn::clone_in_impl(&self.optional, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for CallExpression<'_> {
    type Cloned = CallExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        CallExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            callee: CloneIn::clone_in_impl(&self.callee, with_semantic_ids, allocator),
            type_arguments: CloneIn::clone_in_impl(
                &self.type_arguments,
                with_semantic_ids,
                allocator,
            ),
            arguments: CloneIn::clone_in_impl(&self.arguments, with_semantic_ids, allocator),
            optional: CloneIn::clone_in_impl(&self.optional, with_semantic_ids, allocator),
            pure: CloneIn::clone_in_impl(&self.pure, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for NewExpression<'_> {
    type Cloned = NewExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        NewExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            callee: CloneIn::clone_in_impl(&self.callee, with_semantic_ids, allocator),
            type_arguments: CloneIn::clone_in_impl(
                &self.type_arguments,
                with_semantic_ids,
                allocator,
            ),
            arguments: CloneIn::clone_in_impl(&self.arguments, with_semantic_ids, allocator),
            pure: CloneIn::clone_in_impl(&self.pure, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ImportMeta {
    type Cloned = ImportMeta;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ImportMeta {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for NewTarget {
    type Cloned = NewTarget;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        NewTarget {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for SpreadElement<'_> {
    type Cloned = SpreadElement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        SpreadElement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            argument: CloneIn::clone_in_impl(&self.argument, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Argument<'_> {
    type Cloned = Argument<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::SpreadElement(it) => {
                Argument::SpreadElement(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::BooleanLiteral(_)
            | Self::NullLiteral(_)
            | Self::NumericLiteral(_)
            | Self::BigIntLiteral(_)
            | Self::RegExpLiteral(_)
            | Self::StringLiteral(_)
            | Self::TemplateLiteral(_)
            | Self::Identifier(_)
            | Self::Super(_)
            | Self::ArrayExpression(_)
            | Self::ArrowFunctionExpression(_)
            | Self::AssignmentExpression(_)
            | Self::AwaitExpression(_)
            | Self::BinaryExpression(_)
            | Self::CallExpression(_)
            | Self::ChainExpression(_)
            | Self::ClassExpression(_)
            | Self::ConditionalExpression(_)
            | Self::FunctionExpression(_)
            | Self::ImportExpression(_)
            | Self::LogicalExpression(_)
            | Self::NewExpression(_)
            | Self::ObjectExpression(_)
            | Self::ParenthesizedExpression(_)
            | Self::SequenceExpression(_)
            | Self::TaggedTemplateExpression(_)
            | Self::ThisExpression(_)
            | Self::UnaryExpression(_)
            | Self::UpdateExpression(_)
            | Self::YieldExpression(_)
            | Self::PrivateInExpression(_)
            | Self::ImportMeta(_)
            | Self::NewTarget(_)
            | Self::JSXElement(_)
            | Self::JSXFragment(_)
            | Self::TSAsExpression(_)
            | Self::TSSatisfiesExpression(_)
            | Self::TSTypeAssertion(_)
            | Self::TSNonNullExpression(_)
            | Self::TSInstantiationExpression(_)
            | Self::V8IntrinsicExpression(_)
            | Self::ComputedMemberExpression(_)
            | Self::StaticMemberExpression(_)
            | Self::PrivateFieldExpression(_) => Argument::from(CloneIn::clone_in_impl(
                self.to_expression(),
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for UpdateExpression<'_> {
    type Cloned = UpdateExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        UpdateExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            operator: CloneIn::clone_in_impl(&self.operator, with_semantic_ids, allocator),
            prefix: CloneIn::clone_in_impl(&self.prefix, with_semantic_ids, allocator),
            argument: CloneIn::clone_in_impl(&self.argument, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for UnaryExpression<'_> {
    type Cloned = UnaryExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        UnaryExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            operator: CloneIn::clone_in_impl(&self.operator, with_semantic_ids, allocator),
            argument: CloneIn::clone_in_impl(&self.argument, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BinaryExpression<'_> {
    type Cloned = BinaryExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        BinaryExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            left: CloneIn::clone_in_impl(&self.left, with_semantic_ids, allocator),
            operator: CloneIn::clone_in_impl(&self.operator, with_semantic_ids, allocator),
            right: CloneIn::clone_in_impl(&self.right, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for PrivateInExpression<'_> {
    type Cloned = PrivateInExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        PrivateInExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            left: CloneIn::clone_in_impl(&self.left, with_semantic_ids, allocator),
            right: CloneIn::clone_in_impl(&self.right, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for LogicalExpression<'_> {
    type Cloned = LogicalExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        LogicalExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            left: CloneIn::clone_in_impl(&self.left, with_semantic_ids, allocator),
            operator: CloneIn::clone_in_impl(&self.operator, with_semantic_ids, allocator),
            right: CloneIn::clone_in_impl(&self.right, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ConditionalExpression<'_> {
    type Cloned = ConditionalExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ConditionalExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            test: CloneIn::clone_in_impl(&self.test, with_semantic_ids, allocator),
            consequent: CloneIn::clone_in_impl(&self.consequent, with_semantic_ids, allocator),
            alternate: CloneIn::clone_in_impl(&self.alternate, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentExpression<'_> {
    type Cloned = AssignmentExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        AssignmentExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            operator: CloneIn::clone_in_impl(&self.operator, with_semantic_ids, allocator),
            left: CloneIn::clone_in_impl(&self.left, with_semantic_ids, allocator),
            right: CloneIn::clone_in_impl(&self.right, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentTarget<'_> {
    type Cloned = AssignmentTarget<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::AssignmentTargetIdentifier(_)
            | Self::TSAsExpression(_)
            | Self::TSSatisfiesExpression(_)
            | Self::TSNonNullExpression(_)
            | Self::TSTypeAssertion(_)
            | Self::ComputedMemberExpression(_)
            | Self::StaticMemberExpression(_)
            | Self::PrivateFieldExpression(_) => AssignmentTarget::from(CloneIn::clone_in_impl(
                self.to_simple_assignment_target(),
                with_semantic_ids,
                allocator,
            )),
            Self::ArrayAssignmentTarget(_) | Self::ObjectAssignmentTarget(_) => {
                AssignmentTarget::from(CloneIn::clone_in_impl(
                    self.to_assignment_target_pattern(),
                    with_semantic_ids,
                    allocator,
                ))
            }
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for SimpleAssignmentTarget<'_> {
    type Cloned = SimpleAssignmentTarget<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::AssignmentTargetIdentifier(it) => {
                SimpleAssignmentTarget::AssignmentTargetIdentifier(CloneIn::clone_in_impl(
                    it,
                    with_semantic_ids,
                    allocator,
                ))
            }
            Self::TSAsExpression(it) => SimpleAssignmentTarget::TSAsExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::TSSatisfiesExpression(it) => SimpleAssignmentTarget::TSSatisfiesExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::TSNonNullExpression(it) => SimpleAssignmentTarget::TSNonNullExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::TSTypeAssertion(it) => SimpleAssignmentTarget::TSTypeAssertion(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::ComputedMemberExpression(_)
            | Self::StaticMemberExpression(_)
            | Self::PrivateFieldExpression(_) => SimpleAssignmentTarget::from(
                CloneIn::clone_in_impl(self.to_member_expression(), with_semantic_ids, allocator),
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentTargetPattern<'_> {
    type Cloned = AssignmentTargetPattern<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::ArrayAssignmentTarget(it) => AssignmentTargetPattern::ArrayAssignmentTarget(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::ObjectAssignmentTarget(it) => AssignmentTargetPattern::ObjectAssignmentTarget(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ArrayAssignmentTarget<'_> {
    type Cloned = ArrayAssignmentTarget<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ArrayAssignmentTarget {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            elements: CloneIn::clone_in_impl(&self.elements, with_semantic_ids, allocator),
            rest: CloneIn::clone_in_impl(&self.rest, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ObjectAssignmentTarget<'_> {
    type Cloned = ObjectAssignmentTarget<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ObjectAssignmentTarget {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            properties: CloneIn::clone_in_impl(&self.properties, with_semantic_ids, allocator),
            rest: CloneIn::clone_in_impl(&self.rest, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentTargetRest<'_> {
    type Cloned = AssignmentTargetRest<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        AssignmentTargetRest {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            target: CloneIn::clone_in_impl(&self.target, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentTargetMaybeDefault<'_> {
    type Cloned = AssignmentTargetMaybeDefault<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::AssignmentTargetWithDefault(it) => {
                AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(CloneIn::clone_in_impl(
                    it,
                    with_semantic_ids,
                    allocator,
                ))
            }
            Self::AssignmentTargetIdentifier(_)
            | Self::TSAsExpression(_)
            | Self::TSSatisfiesExpression(_)
            | Self::TSNonNullExpression(_)
            | Self::TSTypeAssertion(_)
            | Self::ComputedMemberExpression(_)
            | Self::StaticMemberExpression(_)
            | Self::PrivateFieldExpression(_)
            | Self::ArrayAssignmentTarget(_)
            | Self::ObjectAssignmentTarget(_) => AssignmentTargetMaybeDefault::from(
                CloneIn::clone_in_impl(self.to_assignment_target(), with_semantic_ids, allocator),
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentTargetWithDefault<'_> {
    type Cloned = AssignmentTargetWithDefault<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        AssignmentTargetWithDefault {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            binding: CloneIn::clone_in_impl(&self.binding, with_semantic_ids, allocator),
            init: CloneIn::clone_in_impl(&self.init, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentTargetProperty<'_> {
    type Cloned = AssignmentTargetProperty<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => {
                AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(
                    CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
                )
            }
            Self::AssignmentTargetPropertyProperty(it) => {
                AssignmentTargetProperty::AssignmentTargetPropertyProperty(CloneIn::clone_in_impl(
                    it,
                    with_semantic_ids,
                    allocator,
                ))
            }
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentTargetPropertyIdentifier<'_> {
    type Cloned = AssignmentTargetPropertyIdentifier<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        AssignmentTargetPropertyIdentifier {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            binding: CloneIn::clone_in_impl(&self.binding, with_semantic_ids, allocator),
            init: CloneIn::clone_in_impl(&self.init, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentTargetPropertyProperty<'_> {
    type Cloned = AssignmentTargetPropertyProperty<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        AssignmentTargetPropertyProperty {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            name: CloneIn::clone_in_impl(&self.name, with_semantic_ids, allocator),
            binding: CloneIn::clone_in_impl(&self.binding, with_semantic_ids, allocator),
            computed: CloneIn::clone_in_impl(&self.computed, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for SequenceExpression<'_> {
    type Cloned = SequenceExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        SequenceExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            expressions: CloneIn::clone_in_impl(&self.expressions, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Super {
    type Cloned = Super;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Super {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for AwaitExpression<'_> {
    type Cloned = AwaitExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        AwaitExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            argument: CloneIn::clone_in_impl(&self.argument, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ChainExpression<'_> {
    type Cloned = ChainExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ChainExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            expression: CloneIn::clone_in_impl(&self.expression, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ChainElement<'_> {
    type Cloned = ChainElement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::CallExpression(it) => ChainElement::CallExpression(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::TSNonNullExpression(it) => ChainElement::TSNonNullExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::ComputedMemberExpression(_)
            | Self::StaticMemberExpression(_)
            | Self::PrivateFieldExpression(_) => ChainElement::from(CloneIn::clone_in_impl(
                self.to_member_expression(),
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ParenthesizedExpression<'_> {
    type Cloned = ParenthesizedExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ParenthesizedExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            expression: CloneIn::clone_in_impl(&self.expression, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Statement<'_> {
    type Cloned = Statement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::BlockStatement(it) => {
                Statement::BlockStatement(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::BreakStatement(it) => {
                Statement::BreakStatement(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::ContinueStatement(it) => Statement::ContinueStatement(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::DebuggerStatement(it) => Statement::DebuggerStatement(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::DoWhileStatement(it) => Statement::DoWhileStatement(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::EmptyStatement(it) => {
                Statement::EmptyStatement(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::ExpressionStatement(it) => Statement::ExpressionStatement(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::ForInStatement(it) => {
                Statement::ForInStatement(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::ForOfStatement(it) => {
                Statement::ForOfStatement(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::ForStatement(it) => {
                Statement::ForStatement(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::IfStatement(it) => {
                Statement::IfStatement(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::LabeledStatement(it) => Statement::LabeledStatement(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::ReturnStatement(it) => {
                Statement::ReturnStatement(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::SwitchStatement(it) => {
                Statement::SwitchStatement(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::ThrowStatement(it) => {
                Statement::ThrowStatement(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TryStatement(it) => {
                Statement::TryStatement(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::WhileStatement(it) => {
                Statement::WhileStatement(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::WithStatement(it) => {
                Statement::WithStatement(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::VariableDeclaration(_)
            | Self::FunctionDeclaration(_)
            | Self::ClassDeclaration(_)
            | Self::TSTypeAliasDeclaration(_)
            | Self::TSInterfaceDeclaration(_)
            | Self::TSEnumDeclaration(_)
            | Self::TSModuleDeclaration(_)
            | Self::TSGlobalDeclaration(_)
            | Self::TSImportEqualsDeclaration(_) => Statement::from(CloneIn::clone_in_impl(
                self.to_declaration(),
                with_semantic_ids,
                allocator,
            )),
            Self::ImportDeclaration(_)
            | Self::ExportAllDeclaration(_)
            | Self::ExportDefaultDeclaration(_)
            | Self::ExportNamedDeclaration(_)
            | Self::TSExportAssignment(_)
            | Self::TSNamespaceExportDeclaration(_) => Statement::from(CloneIn::clone_in_impl(
                self.to_module_declaration(),
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Directive<'_> {
    type Cloned = Directive<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Directive {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            expression: CloneIn::clone_in_impl(&self.expression, with_semantic_ids, allocator),
            directive: CloneIn::clone_in_impl(&self.directive, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Hashbang<'_> {
    type Cloned = Hashbang<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Hashbang {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            value: CloneIn::clone_in_impl(&self.value, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BlockStatement<'_> {
    type Cloned = BlockStatement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        BlockStatement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Declaration<'_> {
    type Cloned = Declaration<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::VariableDeclaration(it) => Declaration::VariableDeclaration(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::FunctionDeclaration(it) => Declaration::FunctionDeclaration(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::ClassDeclaration(it) => Declaration::ClassDeclaration(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::TSTypeAliasDeclaration(it) => Declaration::TSTypeAliasDeclaration(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::TSInterfaceDeclaration(it) => Declaration::TSInterfaceDeclaration(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::TSEnumDeclaration(it) => Declaration::TSEnumDeclaration(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::TSModuleDeclaration(it) => Declaration::TSModuleDeclaration(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::TSGlobalDeclaration(it) => Declaration::TSGlobalDeclaration(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::TSImportEqualsDeclaration(it) => Declaration::TSImportEqualsDeclaration(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for VariableDeclaration<'_> {
    type Cloned = VariableDeclaration<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        VariableDeclaration {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            kind: CloneIn::clone_in_impl(&self.kind, with_semantic_ids, allocator),
            declarations: CloneIn::clone_in_impl(&self.declarations, with_semantic_ids, allocator),
            declare: CloneIn::clone_in_impl(&self.declare, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for VariableDeclarationKind {
    type Cloned = VariableDeclarationKind;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for VariableDeclarator<'_> {
    type Cloned = VariableDeclarator<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        VariableDeclarator {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            kind: CloneIn::clone_in_impl(&self.kind, with_semantic_ids, allocator),
            id: CloneIn::clone_in_impl(&self.id, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
            init: CloneIn::clone_in_impl(&self.init, with_semantic_ids, allocator),
            definite: CloneIn::clone_in_impl(&self.definite, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for EmptyStatement {
    type Cloned = EmptyStatement;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        EmptyStatement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ExpressionStatement<'_> {
    type Cloned = ExpressionStatement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ExpressionStatement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            expression: CloneIn::clone_in_impl(&self.expression, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for IfStatement<'_> {
    type Cloned = IfStatement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        IfStatement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            test: CloneIn::clone_in_impl(&self.test, with_semantic_ids, allocator),
            consequent: CloneIn::clone_in_impl(&self.consequent, with_semantic_ids, allocator),
            alternate: CloneIn::clone_in_impl(&self.alternate, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for DoWhileStatement<'_> {
    type Cloned = DoWhileStatement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        DoWhileStatement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
            test: CloneIn::clone_in_impl(&self.test, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for WhileStatement<'_> {
    type Cloned = WhileStatement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        WhileStatement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            test: CloneIn::clone_in_impl(&self.test, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ForStatement<'_> {
    type Cloned = ForStatement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ForStatement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            init: CloneIn::clone_in_impl(&self.init, with_semantic_ids, allocator),
            test: CloneIn::clone_in_impl(&self.test, with_semantic_ids, allocator),
            update: CloneIn::clone_in_impl(&self.update, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ForStatementInit<'_> {
    type Cloned = ForStatementInit<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::VariableDeclaration(it) => ForStatementInit::VariableDeclaration(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::BooleanLiteral(_)
            | Self::NullLiteral(_)
            | Self::NumericLiteral(_)
            | Self::BigIntLiteral(_)
            | Self::RegExpLiteral(_)
            | Self::StringLiteral(_)
            | Self::TemplateLiteral(_)
            | Self::Identifier(_)
            | Self::Super(_)
            | Self::ArrayExpression(_)
            | Self::ArrowFunctionExpression(_)
            | Self::AssignmentExpression(_)
            | Self::AwaitExpression(_)
            | Self::BinaryExpression(_)
            | Self::CallExpression(_)
            | Self::ChainExpression(_)
            | Self::ClassExpression(_)
            | Self::ConditionalExpression(_)
            | Self::FunctionExpression(_)
            | Self::ImportExpression(_)
            | Self::LogicalExpression(_)
            | Self::NewExpression(_)
            | Self::ObjectExpression(_)
            | Self::ParenthesizedExpression(_)
            | Self::SequenceExpression(_)
            | Self::TaggedTemplateExpression(_)
            | Self::ThisExpression(_)
            | Self::UnaryExpression(_)
            | Self::UpdateExpression(_)
            | Self::YieldExpression(_)
            | Self::PrivateInExpression(_)
            | Self::ImportMeta(_)
            | Self::NewTarget(_)
            | Self::JSXElement(_)
            | Self::JSXFragment(_)
            | Self::TSAsExpression(_)
            | Self::TSSatisfiesExpression(_)
            | Self::TSTypeAssertion(_)
            | Self::TSNonNullExpression(_)
            | Self::TSInstantiationExpression(_)
            | Self::V8IntrinsicExpression(_)
            | Self::ComputedMemberExpression(_)
            | Self::StaticMemberExpression(_)
            | Self::PrivateFieldExpression(_) => ForStatementInit::from(CloneIn::clone_in_impl(
                self.to_expression(),
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ForInStatement<'_> {
    type Cloned = ForInStatement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ForInStatement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            left: CloneIn::clone_in_impl(&self.left, with_semantic_ids, allocator),
            right: CloneIn::clone_in_impl(&self.right, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ForStatementLeft<'_> {
    type Cloned = ForStatementLeft<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::VariableDeclaration(it) => ForStatementLeft::VariableDeclaration(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::AssignmentTargetIdentifier(_)
            | Self::TSAsExpression(_)
            | Self::TSSatisfiesExpression(_)
            | Self::TSNonNullExpression(_)
            | Self::TSTypeAssertion(_)
            | Self::ComputedMemberExpression(_)
            | Self::StaticMemberExpression(_)
            | Self::PrivateFieldExpression(_)
            | Self::ArrayAssignmentTarget(_)
            | Self::ObjectAssignmentTarget(_) => ForStatementLeft::from(CloneIn::clone_in_impl(
                self.to_assignment_target(),
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ForOfStatement<'_> {
    type Cloned = ForOfStatement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ForOfStatement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            r#await: CloneIn::clone_in_impl(&self.r#await, with_semantic_ids, allocator),
            left: CloneIn::clone_in_impl(&self.left, with_semantic_ids, allocator),
            right: CloneIn::clone_in_impl(&self.right, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ContinueStatement<'_> {
    type Cloned = ContinueStatement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ContinueStatement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            label: CloneIn::clone_in_impl(&self.label, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BreakStatement<'_> {
    type Cloned = BreakStatement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        BreakStatement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            label: CloneIn::clone_in_impl(&self.label, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ReturnStatement<'_> {
    type Cloned = ReturnStatement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ReturnStatement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            argument: CloneIn::clone_in_impl(&self.argument, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for WithStatement<'_> {
    type Cloned = WithStatement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        WithStatement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            object: CloneIn::clone_in_impl(&self.object, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for SwitchStatement<'_> {
    type Cloned = SwitchStatement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        SwitchStatement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            discriminant: CloneIn::clone_in_impl(&self.discriminant, with_semantic_ids, allocator),
            cases: CloneIn::clone_in_impl(&self.cases, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for SwitchCase<'_> {
    type Cloned = SwitchCase<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        SwitchCase {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            test: CloneIn::clone_in_impl(&self.test, with_semantic_ids, allocator),
            consequent: CloneIn::clone_in_impl(&self.consequent, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for LabeledStatement<'_> {
    type Cloned = LabeledStatement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        LabeledStatement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            label: CloneIn::clone_in_impl(&self.label, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ThrowStatement<'_> {
    type Cloned = ThrowStatement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ThrowStatement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            argument: CloneIn::clone_in_impl(&self.argument, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TryStatement<'_> {
    type Cloned = TryStatement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TryStatement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            block: CloneIn::clone_in_impl(&self.block, with_semantic_ids, allocator),
            handler: CloneIn::clone_in_impl(&self.handler, with_semantic_ids, allocator),
            finalizer: CloneIn::clone_in_impl(&self.finalizer, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for CatchClause<'_> {
    type Cloned = CatchClause<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        CatchClause {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            param: CloneIn::clone_in_impl(&self.param, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for CatchParameter<'_> {
    type Cloned = CatchParameter<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        CatchParameter {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            pattern: CloneIn::clone_in_impl(&self.pattern, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for DebuggerStatement {
    type Cloned = DebuggerStatement;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        DebuggerStatement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BindingPattern<'_> {
    type Cloned = BindingPattern<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::BindingIdentifier(it) => BindingPattern::BindingIdentifier(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::ObjectPattern(it) => BindingPattern::ObjectPattern(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::ArrayPattern(it) => BindingPattern::ArrayPattern(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::AssignmentPattern(it) => BindingPattern::AssignmentPattern(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for AssignmentPattern<'_> {
    type Cloned = AssignmentPattern<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        AssignmentPattern {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            left: CloneIn::clone_in_impl(&self.left, with_semantic_ids, allocator),
            right: CloneIn::clone_in_impl(&self.right, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ObjectPattern<'_> {
    type Cloned = ObjectPattern<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ObjectPattern {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            properties: CloneIn::clone_in_impl(&self.properties, with_semantic_ids, allocator),
            rest: CloneIn::clone_in_impl(&self.rest, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BindingProperty<'_> {
    type Cloned = BindingProperty<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        BindingProperty {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            key: CloneIn::clone_in_impl(&self.key, with_semantic_ids, allocator),
            value: CloneIn::clone_in_impl(&self.value, with_semantic_ids, allocator),
            shorthand: CloneIn::clone_in_impl(&self.shorthand, with_semantic_ids, allocator),
            computed: CloneIn::clone_in_impl(&self.computed, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ArrayPattern<'_> {
    type Cloned = ArrayPattern<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ArrayPattern {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            elements: CloneIn::clone_in_impl(&self.elements, with_semantic_ids, allocator),
            rest: CloneIn::clone_in_impl(&self.rest, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BindingRestElement<'_> {
    type Cloned = BindingRestElement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        BindingRestElement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            argument: CloneIn::clone_in_impl(&self.argument, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Function<'_> {
    type Cloned = Function<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Function {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            r#type: CloneIn::clone_in_impl(&self.r#type, with_semantic_ids, allocator),
            id: CloneIn::clone_in_impl(&self.id, with_semantic_ids, allocator),
            generator: CloneIn::clone_in_impl(&self.generator, with_semantic_ids, allocator),
            r#async: CloneIn::clone_in_impl(&self.r#async, with_semantic_ids, allocator),
            declare: CloneIn::clone_in_impl(&self.declare, with_semantic_ids, allocator),
            type_parameters: CloneIn::clone_in_impl(
                &self.type_parameters,
                with_semantic_ids,
                allocator,
            ),
            this_param: CloneIn::clone_in_impl(&self.this_param, with_semantic_ids, allocator),
            params: CloneIn::clone_in_impl(&self.params, with_semantic_ids, allocator),
            return_type: CloneIn::clone_in_impl(&self.return_type, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
            pure: CloneIn::clone_in_impl(&self.pure, with_semantic_ids, allocator),
            pife: CloneIn::clone_in_impl(&self.pife, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for FunctionType {
    type Cloned = FunctionType;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for FormalParameters<'_> {
    type Cloned = FormalParameters<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        FormalParameters {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            kind: CloneIn::clone_in_impl(&self.kind, with_semantic_ids, allocator),
            items: CloneIn::clone_in_impl(&self.items, with_semantic_ids, allocator),
            rest: CloneIn::clone_in_impl(&self.rest, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for FormalParameter<'_> {
    type Cloned = FormalParameter<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        FormalParameter {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            decorators: CloneIn::clone_in_impl(&self.decorators, with_semantic_ids, allocator),
            pattern: CloneIn::clone_in_impl(&self.pattern, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
            initializer: CloneIn::clone_in_impl(&self.initializer, with_semantic_ids, allocator),
            optional: CloneIn::clone_in_impl(&self.optional, with_semantic_ids, allocator),
            accessibility: CloneIn::clone_in_impl(
                &self.accessibility,
                with_semantic_ids,
                allocator,
            ),
            readonly: CloneIn::clone_in_impl(&self.readonly, with_semantic_ids, allocator),
            r#override: CloneIn::clone_in_impl(&self.r#override, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for FormalParameterKind {
    type Cloned = FormalParameterKind;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for FormalParameterRest<'_> {
    type Cloned = FormalParameterRest<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        FormalParameterRest {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            decorators: CloneIn::clone_in_impl(&self.decorators, with_semantic_ids, allocator),
            rest: CloneIn::clone_in_impl(&self.rest, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for FunctionBody<'_> {
    type Cloned = FunctionBody<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        FunctionBody {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            directives: CloneIn::clone_in_impl(&self.directives, with_semantic_ids, allocator),
            statements: CloneIn::clone_in_impl(&self.statements, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ArrowFunctionExpression<'_> {
    type Cloned = ArrowFunctionExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ArrowFunctionExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            expression: CloneIn::clone_in_impl(&self.expression, with_semantic_ids, allocator),
            r#async: CloneIn::clone_in_impl(&self.r#async, with_semantic_ids, allocator),
            type_parameters: CloneIn::clone_in_impl(
                &self.type_parameters,
                with_semantic_ids,
                allocator,
            ),
            params: CloneIn::clone_in_impl(&self.params, with_semantic_ids, allocator),
            return_type: CloneIn::clone_in_impl(&self.return_type, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
            pure: CloneIn::clone_in_impl(&self.pure, with_semantic_ids, allocator),
            pife: CloneIn::clone_in_impl(&self.pife, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for YieldExpression<'_> {
    type Cloned = YieldExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        YieldExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            delegate: CloneIn::clone_in_impl(&self.delegate, with_semantic_ids, allocator),
            argument: CloneIn::clone_in_impl(&self.argument, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Class<'_> {
    type Cloned = Class<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Class {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            r#type: CloneIn::clone_in_impl(&self.r#type, with_semantic_ids, allocator),
            decorators: CloneIn::clone_in_impl(&self.decorators, with_semantic_ids, allocator),
            id: CloneIn::clone_in_impl(&self.id, with_semantic_ids, allocator),
            type_parameters: CloneIn::clone_in_impl(
                &self.type_parameters,
                with_semantic_ids,
                allocator,
            ),
            super_class: CloneIn::clone_in_impl(&self.super_class, with_semantic_ids, allocator),
            super_type_arguments: CloneIn::clone_in_impl(
                &self.super_type_arguments,
                with_semantic_ids,
                allocator,
            ),
            implements: CloneIn::clone_in_impl(&self.implements, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
            r#abstract: CloneIn::clone_in_impl(&self.r#abstract, with_semantic_ids, allocator),
            declare: CloneIn::clone_in_impl(&self.declare, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ClassType {
    type Cloned = ClassType;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ClassBody<'_> {
    type Cloned = ClassBody<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ClassBody {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ClassElement<'_> {
    type Cloned = ClassElement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::StaticBlock(it) => {
                ClassElement::StaticBlock(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::MethodDefinition(it) => ClassElement::MethodDefinition(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::PropertyDefinition(it) => ClassElement::PropertyDefinition(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::AccessorProperty(it) => ClassElement::AccessorProperty(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::TSIndexSignature(it) => ClassElement::TSIndexSignature(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for MethodDefinition<'_> {
    type Cloned = MethodDefinition<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        MethodDefinition {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            r#type: CloneIn::clone_in_impl(&self.r#type, with_semantic_ids, allocator),
            decorators: CloneIn::clone_in_impl(&self.decorators, with_semantic_ids, allocator),
            key: CloneIn::clone_in_impl(&self.key, with_semantic_ids, allocator),
            value: CloneIn::clone_in_impl(&self.value, with_semantic_ids, allocator),
            kind: CloneIn::clone_in_impl(&self.kind, with_semantic_ids, allocator),
            computed: CloneIn::clone_in_impl(&self.computed, with_semantic_ids, allocator),
            r#static: CloneIn::clone_in_impl(&self.r#static, with_semantic_ids, allocator),
            r#override: CloneIn::clone_in_impl(&self.r#override, with_semantic_ids, allocator),
            optional: CloneIn::clone_in_impl(&self.optional, with_semantic_ids, allocator),
            accessibility: CloneIn::clone_in_impl(
                &self.accessibility,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for MethodDefinitionType {
    type Cloned = MethodDefinitionType;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for PropertyDefinition<'_> {
    type Cloned = PropertyDefinition<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        PropertyDefinition {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            r#type: CloneIn::clone_in_impl(&self.r#type, with_semantic_ids, allocator),
            decorators: CloneIn::clone_in_impl(&self.decorators, with_semantic_ids, allocator),
            key: CloneIn::clone_in_impl(&self.key, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
            value: CloneIn::clone_in_impl(&self.value, with_semantic_ids, allocator),
            computed: CloneIn::clone_in_impl(&self.computed, with_semantic_ids, allocator),
            r#static: CloneIn::clone_in_impl(&self.r#static, with_semantic_ids, allocator),
            declare: CloneIn::clone_in_impl(&self.declare, with_semantic_ids, allocator),
            r#override: CloneIn::clone_in_impl(&self.r#override, with_semantic_ids, allocator),
            optional: CloneIn::clone_in_impl(&self.optional, with_semantic_ids, allocator),
            definite: CloneIn::clone_in_impl(&self.definite, with_semantic_ids, allocator),
            readonly: CloneIn::clone_in_impl(&self.readonly, with_semantic_ids, allocator),
            accessibility: CloneIn::clone_in_impl(
                &self.accessibility,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for PropertyDefinitionType {
    type Cloned = PropertyDefinitionType;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for MethodDefinitionKind {
    type Cloned = MethodDefinitionKind;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for PrivateIdentifier<'_> {
    type Cloned = PrivateIdentifier<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        PrivateIdentifier {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            name: CloneIn::clone_in_impl(&self.name, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for StaticBlock<'_> {
    type Cloned = StaticBlock<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        StaticBlock {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ModuleDeclaration<'_> {
    type Cloned = ModuleDeclaration<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::ImportDeclaration(it) => ModuleDeclaration::ImportDeclaration(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::ExportAllDeclaration(it) => ModuleDeclaration::ExportAllDeclaration(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::ExportDefaultDeclaration(it) => ModuleDeclaration::ExportDefaultDeclaration(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::ExportNamedDeclaration(it) => ModuleDeclaration::ExportNamedDeclaration(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::TSExportAssignment(it) => ModuleDeclaration::TSExportAssignment(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::TSNamespaceExportDeclaration(it) => {
                ModuleDeclaration::TSNamespaceExportDeclaration(CloneIn::clone_in_impl(
                    it,
                    with_semantic_ids,
                    allocator,
                ))
            }
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for AccessorPropertyType {
    type Cloned = AccessorPropertyType;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for AccessorProperty<'_> {
    type Cloned = AccessorProperty<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        AccessorProperty {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            r#type: CloneIn::clone_in_impl(&self.r#type, with_semantic_ids, allocator),
            decorators: CloneIn::clone_in_impl(&self.decorators, with_semantic_ids, allocator),
            key: CloneIn::clone_in_impl(&self.key, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
            value: CloneIn::clone_in_impl(&self.value, with_semantic_ids, allocator),
            computed: CloneIn::clone_in_impl(&self.computed, with_semantic_ids, allocator),
            r#static: CloneIn::clone_in_impl(&self.r#static, with_semantic_ids, allocator),
            r#override: CloneIn::clone_in_impl(&self.r#override, with_semantic_ids, allocator),
            definite: CloneIn::clone_in_impl(&self.definite, with_semantic_ids, allocator),
            accessibility: CloneIn::clone_in_impl(
                &self.accessibility,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ImportExpression<'_> {
    type Cloned = ImportExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ImportExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            source: CloneIn::clone_in_impl(&self.source, with_semantic_ids, allocator),
            options: CloneIn::clone_in_impl(&self.options, with_semantic_ids, allocator),
            phase: CloneIn::clone_in_impl(&self.phase, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ImportDeclaration<'_> {
    type Cloned = ImportDeclaration<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ImportDeclaration {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            specifiers: CloneIn::clone_in_impl(&self.specifiers, with_semantic_ids, allocator),
            source: CloneIn::clone_in_impl(&self.source, with_semantic_ids, allocator),
            phase: CloneIn::clone_in_impl(&self.phase, with_semantic_ids, allocator),
            with_clause: CloneIn::clone_in_impl(&self.with_clause, with_semantic_ids, allocator),
            import_kind: CloneIn::clone_in_impl(&self.import_kind, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ImportPhase {
    type Cloned = ImportPhase;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ImportDeclarationSpecifier<'_> {
    type Cloned = ImportDeclarationSpecifier<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::ImportSpecifier(it) => ImportDeclarationSpecifier::ImportSpecifier(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::ImportDefaultSpecifier(it) => ImportDeclarationSpecifier::ImportDefaultSpecifier(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::ImportNamespaceSpecifier(it) => {
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(CloneIn::clone_in_impl(
                    it,
                    with_semantic_ids,
                    allocator,
                ))
            }
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ImportSpecifier<'_> {
    type Cloned = ImportSpecifier<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ImportSpecifier {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            imported: CloneIn::clone_in_impl(&self.imported, with_semantic_ids, allocator),
            local: CloneIn::clone_in_impl(&self.local, with_semantic_ids, allocator),
            import_kind: CloneIn::clone_in_impl(&self.import_kind, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ImportDefaultSpecifier<'_> {
    type Cloned = ImportDefaultSpecifier<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ImportDefaultSpecifier {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            local: CloneIn::clone_in_impl(&self.local, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ImportNamespaceSpecifier<'_> {
    type Cloned = ImportNamespaceSpecifier<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ImportNamespaceSpecifier {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            local: CloneIn::clone_in_impl(&self.local, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for WithClause<'_> {
    type Cloned = WithClause<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        WithClause {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            keyword: CloneIn::clone_in_impl(&self.keyword, with_semantic_ids, allocator),
            with_entries: CloneIn::clone_in_impl(&self.with_entries, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for WithClauseKeyword {
    type Cloned = WithClauseKeyword;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ImportAttribute<'_> {
    type Cloned = ImportAttribute<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ImportAttribute {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            key: CloneIn::clone_in_impl(&self.key, with_semantic_ids, allocator),
            value: CloneIn::clone_in_impl(&self.value, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ImportAttributeKey<'_> {
    type Cloned = ImportAttributeKey<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::Identifier(it) => ImportAttributeKey::Identifier(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::StringLiteral(it) => ImportAttributeKey::StringLiteral(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ExportNamedDeclaration<'_> {
    type Cloned = ExportNamedDeclaration<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ExportNamedDeclaration {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            declaration: CloneIn::clone_in_impl(&self.declaration, with_semantic_ids, allocator),
            specifiers: CloneIn::clone_in_impl(&self.specifiers, with_semantic_ids, allocator),
            source: CloneIn::clone_in_impl(&self.source, with_semantic_ids, allocator),
            export_kind: CloneIn::clone_in_impl(&self.export_kind, with_semantic_ids, allocator),
            with_clause: CloneIn::clone_in_impl(&self.with_clause, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ExportDefaultDeclaration<'_> {
    type Cloned = ExportDefaultDeclaration<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ExportDefaultDeclaration {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            declaration: CloneIn::clone_in_impl(&self.declaration, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ExportAllDeclaration<'_> {
    type Cloned = ExportAllDeclaration<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ExportAllDeclaration {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            exported: CloneIn::clone_in_impl(&self.exported, with_semantic_ids, allocator),
            source: CloneIn::clone_in_impl(&self.source, with_semantic_ids, allocator),
            with_clause: CloneIn::clone_in_impl(&self.with_clause, with_semantic_ids, allocator),
            export_kind: CloneIn::clone_in_impl(&self.export_kind, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ExportSpecifier<'_> {
    type Cloned = ExportSpecifier<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        ExportSpecifier {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            local: CloneIn::clone_in_impl(&self.local, with_semantic_ids, allocator),
            exported: CloneIn::clone_in_impl(&self.exported, with_semantic_ids, allocator),
            export_kind: CloneIn::clone_in_impl(&self.export_kind, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ExportDefaultDeclarationKind<'_> {
    type Cloned = ExportDefaultDeclarationKind<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::FunctionDeclaration(it) => ExportDefaultDeclarationKind::FunctionDeclaration(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::ClassDeclaration(it) => ExportDefaultDeclarationKind::ClassDeclaration(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::TSInterfaceDeclaration(it) => {
                ExportDefaultDeclarationKind::TSInterfaceDeclaration(CloneIn::clone_in_impl(
                    it,
                    with_semantic_ids,
                    allocator,
                ))
            }
            Self::BooleanLiteral(_)
            | Self::NullLiteral(_)
            | Self::NumericLiteral(_)
            | Self::BigIntLiteral(_)
            | Self::RegExpLiteral(_)
            | Self::StringLiteral(_)
            | Self::TemplateLiteral(_)
            | Self::Identifier(_)
            | Self::Super(_)
            | Self::ArrayExpression(_)
            | Self::ArrowFunctionExpression(_)
            | Self::AssignmentExpression(_)
            | Self::AwaitExpression(_)
            | Self::BinaryExpression(_)
            | Self::CallExpression(_)
            | Self::ChainExpression(_)
            | Self::ClassExpression(_)
            | Self::ConditionalExpression(_)
            | Self::FunctionExpression(_)
            | Self::ImportExpression(_)
            | Self::LogicalExpression(_)
            | Self::NewExpression(_)
            | Self::ObjectExpression(_)
            | Self::ParenthesizedExpression(_)
            | Self::SequenceExpression(_)
            | Self::TaggedTemplateExpression(_)
            | Self::ThisExpression(_)
            | Self::UnaryExpression(_)
            | Self::UpdateExpression(_)
            | Self::YieldExpression(_)
            | Self::PrivateInExpression(_)
            | Self::ImportMeta(_)
            | Self::NewTarget(_)
            | Self::JSXElement(_)
            | Self::JSXFragment(_)
            | Self::TSAsExpression(_)
            | Self::TSSatisfiesExpression(_)
            | Self::TSTypeAssertion(_)
            | Self::TSNonNullExpression(_)
            | Self::TSInstantiationExpression(_)
            | Self::V8IntrinsicExpression(_)
            | Self::ComputedMemberExpression(_)
            | Self::StaticMemberExpression(_)
            | Self::PrivateFieldExpression(_) => ExportDefaultDeclarationKind::from(
                CloneIn::clone_in_impl(self.to_expression(), with_semantic_ids, allocator),
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ModuleExportName<'_> {
    type Cloned = ModuleExportName<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::IdentifierName(it) => ModuleExportName::IdentifierName(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::IdentifierReference(it) => ModuleExportName::IdentifierReference(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::StringLiteral(it) => ModuleExportName::StringLiteral(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for V8IntrinsicExpression<'_> {
    type Cloned = V8IntrinsicExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        V8IntrinsicExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            name: CloneIn::clone_in_impl(&self.name, with_semantic_ids, allocator),
            arguments: CloneIn::clone_in_impl(&self.arguments, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BooleanLiteral {
    type Cloned = BooleanLiteral;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        BooleanLiteral {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            value: CloneIn::clone_in_impl(&self.value, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for NullLiteral {
    type Cloned = NullLiteral;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        NullLiteral {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for NumericLiteral<'_> {
    type Cloned = NumericLiteral<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        NumericLiteral {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            value: CloneIn::clone_in_impl(&self.value, with_semantic_ids, allocator),
            raw: CloneIn::clone_in_impl(&self.raw, with_semantic_ids, allocator),
            base: CloneIn::clone_in_impl(&self.base, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for StringLiteral<'_> {
    type Cloned = StringLiteral<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        StringLiteral {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            value: CloneIn::clone_in_impl(&self.value, with_semantic_ids, allocator),
            raw: CloneIn::clone_in_impl(&self.raw, with_semantic_ids, allocator),
            lone_surrogates: CloneIn::clone_in_impl(
                &self.lone_surrogates,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for BigIntLiteral<'_> {
    type Cloned = BigIntLiteral<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        BigIntLiteral {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            value: CloneIn::clone_in_impl(&self.value, with_semantic_ids, allocator),
            raw: CloneIn::clone_in_impl(&self.raw, with_semantic_ids, allocator),
            base: CloneIn::clone_in_impl(&self.base, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for RegExpLiteral<'_> {
    type Cloned = RegExpLiteral<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        RegExpLiteral {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            regex: CloneIn::clone_in_impl(&self.regex, with_semantic_ids, allocator),
            raw: CloneIn::clone_in_impl(&self.raw, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for RegExp<'_> {
    type Cloned = RegExp<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        RegExp {
            pattern: CloneIn::clone_in_impl(&self.pattern, with_semantic_ids, allocator),
            flags: CloneIn::clone_in_impl(&self.flags, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for RegExpPattern<'_> {
    type Cloned = RegExpPattern<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        RegExpPattern {
            text: CloneIn::clone_in_impl(&self.text, with_semantic_ids, allocator),
            pattern: CloneIn::clone_in_impl(&self.pattern, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXElement<'_> {
    type Cloned = JSXElement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSXElement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            opening_element: CloneIn::clone_in_impl(
                &self.opening_element,
                with_semantic_ids,
                allocator,
            ),
            children: CloneIn::clone_in_impl(&self.children, with_semantic_ids, allocator),
            closing_element: CloneIn::clone_in_impl(
                &self.closing_element,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXOpeningElement<'_> {
    type Cloned = JSXOpeningElement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSXOpeningElement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            name: CloneIn::clone_in_impl(&self.name, with_semantic_ids, allocator),
            type_arguments: CloneIn::clone_in_impl(
                &self.type_arguments,
                with_semantic_ids,
                allocator,
            ),
            attributes: CloneIn::clone_in_impl(&self.attributes, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXClosingElement<'_> {
    type Cloned = JSXClosingElement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSXClosingElement {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            name: CloneIn::clone_in_impl(&self.name, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXFragment<'_> {
    type Cloned = JSXFragment<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSXFragment {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            opening_fragment: CloneIn::clone_in_impl(
                &self.opening_fragment,
                with_semantic_ids,
                allocator,
            ),
            children: CloneIn::clone_in_impl(&self.children, with_semantic_ids, allocator),
            closing_fragment: CloneIn::clone_in_impl(
                &self.closing_fragment,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXOpeningFragment {
    type Cloned = JSXOpeningFragment;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSXOpeningFragment {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXClosingFragment {
    type Cloned = JSXClosingFragment;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSXClosingFragment {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXElementName<'_> {
    type Cloned = JSXElementName<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::Identifier(it) => {
                JSXElementName::Identifier(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::IdentifierReference(it) => JSXElementName::IdentifierReference(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::NamespacedName(it) => JSXElementName::NamespacedName(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::MemberExpression(it) => JSXElementName::MemberExpression(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::ThisExpression(it) => JSXElementName::ThisExpression(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXNamespacedName<'_> {
    type Cloned = JSXNamespacedName<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSXNamespacedName {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            namespace: CloneIn::clone_in_impl(&self.namespace, with_semantic_ids, allocator),
            name: CloneIn::clone_in_impl(&self.name, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXMemberExpression<'_> {
    type Cloned = JSXMemberExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSXMemberExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            object: CloneIn::clone_in_impl(&self.object, with_semantic_ids, allocator),
            property: CloneIn::clone_in_impl(&self.property, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXMemberExpressionObject<'_> {
    type Cloned = JSXMemberExpressionObject<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::IdentifierReference(it) => JSXMemberExpressionObject::IdentifierReference(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::MemberExpression(it) => JSXMemberExpressionObject::MemberExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::ThisExpression(it) => JSXMemberExpressionObject::ThisExpression(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXExpressionContainer<'_> {
    type Cloned = JSXExpressionContainer<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSXExpressionContainer {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            expression: CloneIn::clone_in_impl(&self.expression, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXExpression<'_> {
    type Cloned = JSXExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::EmptyExpression(it) => JSXExpression::EmptyExpression(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::BooleanLiteral(_)
            | Self::NullLiteral(_)
            | Self::NumericLiteral(_)
            | Self::BigIntLiteral(_)
            | Self::RegExpLiteral(_)
            | Self::StringLiteral(_)
            | Self::TemplateLiteral(_)
            | Self::Identifier(_)
            | Self::Super(_)
            | Self::ArrayExpression(_)
            | Self::ArrowFunctionExpression(_)
            | Self::AssignmentExpression(_)
            | Self::AwaitExpression(_)
            | Self::BinaryExpression(_)
            | Self::CallExpression(_)
            | Self::ChainExpression(_)
            | Self::ClassExpression(_)
            | Self::ConditionalExpression(_)
            | Self::FunctionExpression(_)
            | Self::ImportExpression(_)
            | Self::LogicalExpression(_)
            | Self::NewExpression(_)
            | Self::ObjectExpression(_)
            | Self::ParenthesizedExpression(_)
            | Self::SequenceExpression(_)
            | Self::TaggedTemplateExpression(_)
            | Self::ThisExpression(_)
            | Self::UnaryExpression(_)
            | Self::UpdateExpression(_)
            | Self::YieldExpression(_)
            | Self::PrivateInExpression(_)
            | Self::ImportMeta(_)
            | Self::NewTarget(_)
            | Self::JSXElement(_)
            | Self::JSXFragment(_)
            | Self::TSAsExpression(_)
            | Self::TSSatisfiesExpression(_)
            | Self::TSTypeAssertion(_)
            | Self::TSNonNullExpression(_)
            | Self::TSInstantiationExpression(_)
            | Self::V8IntrinsicExpression(_)
            | Self::ComputedMemberExpression(_)
            | Self::StaticMemberExpression(_)
            | Self::PrivateFieldExpression(_) => JSXExpression::from(CloneIn::clone_in_impl(
                self.to_expression(),
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXEmptyExpression {
    type Cloned = JSXEmptyExpression;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSXEmptyExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXAttributeItem<'_> {
    type Cloned = JSXAttributeItem<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::Attribute(it) => JSXAttributeItem::Attribute(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::SpreadAttribute(it) => JSXAttributeItem::SpreadAttribute(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXAttribute<'_> {
    type Cloned = JSXAttribute<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSXAttribute {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            name: CloneIn::clone_in_impl(&self.name, with_semantic_ids, allocator),
            value: CloneIn::clone_in_impl(&self.value, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXSpreadAttribute<'_> {
    type Cloned = JSXSpreadAttribute<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSXSpreadAttribute {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            argument: CloneIn::clone_in_impl(&self.argument, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXAttributeName<'_> {
    type Cloned = JSXAttributeName<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::Identifier(it) => JSXAttributeName::Identifier(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::NamespacedName(it) => JSXAttributeName::NamespacedName(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXAttributeValue<'_> {
    type Cloned = JSXAttributeValue<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::StringLiteral(it) => JSXAttributeValue::StringLiteral(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::ExpressionContainer(it) => JSXAttributeValue::ExpressionContainer(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::Element(it) => {
                JSXAttributeValue::Element(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::Fragment(it) => JSXAttributeValue::Fragment(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXIdentifier<'_> {
    type Cloned = JSXIdentifier<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSXIdentifier {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            name: CloneIn::clone_in_impl(&self.name, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXChild<'_> {
    type Cloned = JSXChild<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::Text(it) => {
                JSXChild::Text(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::Element(it) => {
                JSXChild::Element(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::Fragment(it) => {
                JSXChild::Fragment(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::ExpressionContainer(it) => JSXChild::ExpressionContainer(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::Spread(it) => {
                JSXChild::Spread(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXSpreadChild<'_> {
    type Cloned = JSXSpreadChild<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSXSpreadChild {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            expression: CloneIn::clone_in_impl(&self.expression, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSXText<'_> {
    type Cloned = JSXText<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSXText {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            value: CloneIn::clone_in_impl(&self.value, with_semantic_ids, allocator),
            raw: CloneIn::clone_in_impl(&self.raw, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSThisParameter<'_> {
    type Cloned = TSThisParameter<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSThisParameter {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            this_span: CloneIn::clone_in_impl(&self.this_span, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSEnumDeclaration<'_> {
    type Cloned = TSEnumDeclaration<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSEnumDeclaration {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            id: CloneIn::clone_in_impl(&self.id, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
            r#const: CloneIn::clone_in_impl(&self.r#const, with_semantic_ids, allocator),
            declare: CloneIn::clone_in_impl(&self.declare, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSEnumBody<'_> {
    type Cloned = TSEnumBody<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSEnumBody {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            members: CloneIn::clone_in_impl(&self.members, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSEnumMember<'_> {
    type Cloned = TSEnumMember<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSEnumMember {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            id: CloneIn::clone_in_impl(&self.id, with_semantic_ids, allocator),
            initializer: CloneIn::clone_in_impl(&self.initializer, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSEnumMemberName<'_> {
    type Cloned = TSEnumMemberName<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::Identifier(it) => TSEnumMemberName::Identifier(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::String(it) => {
                TSEnumMemberName::String(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::ComputedString(it) => TSEnumMemberName::ComputedString(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::ComputedTemplateString(it) => TSEnumMemberName::ComputedTemplateString(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeAnnotation<'_> {
    type Cloned = TSTypeAnnotation<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSTypeAnnotation {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSLiteralType<'_> {
    type Cloned = TSLiteralType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSLiteralType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            literal: CloneIn::clone_in_impl(&self.literal, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSLiteral<'_> {
    type Cloned = TSLiteral<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::BooleanLiteral(it) => {
                TSLiteral::BooleanLiteral(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::NumericLiteral(it) => {
                TSLiteral::NumericLiteral(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::BigIntLiteral(it) => {
                TSLiteral::BigIntLiteral(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::StringLiteral(it) => {
                TSLiteral::StringLiteral(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TemplateLiteral(it) => {
                TSLiteral::TemplateLiteral(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::UnaryExpression(it) => {
                TSLiteral::UnaryExpression(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSType<'_> {
    type Cloned = TSType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::TSAnyKeyword(it) => {
                TSType::TSAnyKeyword(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSBigIntKeyword(it) => {
                TSType::TSBigIntKeyword(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSBooleanKeyword(it) => {
                TSType::TSBooleanKeyword(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSIntrinsicKeyword(it) => {
                TSType::TSIntrinsicKeyword(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSNeverKeyword(it) => {
                TSType::TSNeverKeyword(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSNullKeyword(it) => {
                TSType::TSNullKeyword(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSNumberKeyword(it) => {
                TSType::TSNumberKeyword(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSObjectKeyword(it) => {
                TSType::TSObjectKeyword(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSStringKeyword(it) => {
                TSType::TSStringKeyword(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSSymbolKeyword(it) => {
                TSType::TSSymbolKeyword(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSUndefinedKeyword(it) => {
                TSType::TSUndefinedKeyword(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSUnknownKeyword(it) => {
                TSType::TSUnknownKeyword(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSVoidKeyword(it) => {
                TSType::TSVoidKeyword(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSArrayType(it) => {
                TSType::TSArrayType(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSConditionalType(it) => {
                TSType::TSConditionalType(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSConstructorType(it) => {
                TSType::TSConstructorType(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSFunctionType(it) => {
                TSType::TSFunctionType(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSImportType(it) => {
                TSType::TSImportType(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSIndexedAccessType(it) => TSType::TSIndexedAccessType(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::TSInferType(it) => {
                TSType::TSInferType(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSIntersectionType(it) => {
                TSType::TSIntersectionType(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSLiteralType(it) => {
                TSType::TSLiteralType(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSMappedType(it) => {
                TSType::TSMappedType(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSNamedTupleMember(it) => {
                TSType::TSNamedTupleMember(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSTemplateLiteralType(it) => TSType::TSTemplateLiteralType(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::TSThisType(it) => {
                TSType::TSThisType(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSTupleType(it) => {
                TSType::TSTupleType(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSTypeLiteral(it) => {
                TSType::TSTypeLiteral(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSTypeOperatorType(it) => {
                TSType::TSTypeOperatorType(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSTypePredicate(it) => {
                TSType::TSTypePredicate(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSTypeQuery(it) => {
                TSType::TSTypeQuery(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSTypeReference(it) => {
                TSType::TSTypeReference(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSUnionType(it) => {
                TSType::TSUnionType(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSParenthesizedType(it) => TSType::TSParenthesizedType(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::JSDocNullableType(it) => {
                TSType::JSDocNullableType(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::JSDocNonNullableType(it) => TSType::JSDocNonNullableType(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::JSDocUnknownType(it) => {
                TSType::JSDocUnknownType(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSConditionalType<'_> {
    type Cloned = TSConditionalType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSConditionalType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            check_type: CloneIn::clone_in_impl(&self.check_type, with_semantic_ids, allocator),
            extends_type: CloneIn::clone_in_impl(&self.extends_type, with_semantic_ids, allocator),
            true_type: CloneIn::clone_in_impl(&self.true_type, with_semantic_ids, allocator),
            false_type: CloneIn::clone_in_impl(&self.false_type, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSUnionType<'_> {
    type Cloned = TSUnionType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSUnionType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            types: CloneIn::clone_in_impl(&self.types, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSIntersectionType<'_> {
    type Cloned = TSIntersectionType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSIntersectionType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            types: CloneIn::clone_in_impl(&self.types, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSParenthesizedType<'_> {
    type Cloned = TSParenthesizedType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSParenthesizedType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeOperator<'_> {
    type Cloned = TSTypeOperator<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSTypeOperator {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            operator: CloneIn::clone_in_impl(&self.operator, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeOperatorOperator {
    type Cloned = TSTypeOperatorOperator;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSArrayType<'_> {
    type Cloned = TSArrayType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSArrayType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            element_type: CloneIn::clone_in_impl(&self.element_type, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSIndexedAccessType<'_> {
    type Cloned = TSIndexedAccessType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSIndexedAccessType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            object_type: CloneIn::clone_in_impl(&self.object_type, with_semantic_ids, allocator),
            index_type: CloneIn::clone_in_impl(&self.index_type, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTupleType<'_> {
    type Cloned = TSTupleType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSTupleType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            element_types: CloneIn::clone_in_impl(
                &self.element_types,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSNamedTupleMember<'_> {
    type Cloned = TSNamedTupleMember<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSNamedTupleMember {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            label: CloneIn::clone_in_impl(&self.label, with_semantic_ids, allocator),
            element_type: CloneIn::clone_in_impl(&self.element_type, with_semantic_ids, allocator),
            optional: CloneIn::clone_in_impl(&self.optional, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSOptionalType<'_> {
    type Cloned = TSOptionalType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSOptionalType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSRestType<'_> {
    type Cloned = TSRestType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSRestType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTupleElement<'_> {
    type Cloned = TSTupleElement<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::TSOptionalType(it) => TSTupleElement::TSOptionalType(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::TSRestType(it) => {
                TSTupleElement::TSRestType(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::TSAnyKeyword(_)
            | Self::TSBigIntKeyword(_)
            | Self::TSBooleanKeyword(_)
            | Self::TSIntrinsicKeyword(_)
            | Self::TSNeverKeyword(_)
            | Self::TSNullKeyword(_)
            | Self::TSNumberKeyword(_)
            | Self::TSObjectKeyword(_)
            | Self::TSStringKeyword(_)
            | Self::TSSymbolKeyword(_)
            | Self::TSUndefinedKeyword(_)
            | Self::TSUnknownKeyword(_)
            | Self::TSVoidKeyword(_)
            | Self::TSArrayType(_)
            | Self::TSConditionalType(_)
            | Self::TSConstructorType(_)
            | Self::TSFunctionType(_)
            | Self::TSImportType(_)
            | Self::TSIndexedAccessType(_)
            | Self::TSInferType(_)
            | Self::TSIntersectionType(_)
            | Self::TSLiteralType(_)
            | Self::TSMappedType(_)
            | Self::TSNamedTupleMember(_)
            | Self::TSTemplateLiteralType(_)
            | Self::TSThisType(_)
            | Self::TSTupleType(_)
            | Self::TSTypeLiteral(_)
            | Self::TSTypeOperatorType(_)
            | Self::TSTypePredicate(_)
            | Self::TSTypeQuery(_)
            | Self::TSTypeReference(_)
            | Self::TSUnionType(_)
            | Self::TSParenthesizedType(_)
            | Self::JSDocNullableType(_)
            | Self::JSDocNonNullableType(_)
            | Self::JSDocUnknownType(_) => TSTupleElement::from(CloneIn::clone_in_impl(
                self.to_ts_type(),
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSAnyKeyword {
    type Cloned = TSAnyKeyword;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSAnyKeyword {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSStringKeyword {
    type Cloned = TSStringKeyword;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSStringKeyword {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSBooleanKeyword {
    type Cloned = TSBooleanKeyword;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSBooleanKeyword {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSNumberKeyword {
    type Cloned = TSNumberKeyword;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSNumberKeyword {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSNeverKeyword {
    type Cloned = TSNeverKeyword;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSNeverKeyword {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSIntrinsicKeyword {
    type Cloned = TSIntrinsicKeyword;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSIntrinsicKeyword {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSUnknownKeyword {
    type Cloned = TSUnknownKeyword;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSUnknownKeyword {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSNullKeyword {
    type Cloned = TSNullKeyword;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSNullKeyword {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSUndefinedKeyword {
    type Cloned = TSUndefinedKeyword;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSUndefinedKeyword {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSVoidKeyword {
    type Cloned = TSVoidKeyword;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSVoidKeyword {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSSymbolKeyword {
    type Cloned = TSSymbolKeyword;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSSymbolKeyword {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSThisType {
    type Cloned = TSThisType;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSThisType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSObjectKeyword {
    type Cloned = TSObjectKeyword;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSObjectKeyword {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSBigIntKeyword {
    type Cloned = TSBigIntKeyword;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSBigIntKeyword {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeReference<'_> {
    type Cloned = TSTypeReference<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSTypeReference {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            type_name: CloneIn::clone_in_impl(&self.type_name, with_semantic_ids, allocator),
            type_arguments: CloneIn::clone_in_impl(
                &self.type_arguments,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeName<'_> {
    type Cloned = TSTypeName<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::IdentifierReference(it) => TSTypeName::IdentifierReference(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::QualifiedName(it) => {
                TSTypeName::QualifiedName(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
            Self::ThisExpression(it) => {
                TSTypeName::ThisExpression(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSQualifiedName<'_> {
    type Cloned = TSQualifiedName<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSQualifiedName {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            left: CloneIn::clone_in_impl(&self.left, with_semantic_ids, allocator),
            right: CloneIn::clone_in_impl(&self.right, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeParameterInstantiation<'_> {
    type Cloned = TSTypeParameterInstantiation<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSTypeParameterInstantiation {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            params: CloneIn::clone_in_impl(&self.params, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeParameter<'_> {
    type Cloned = TSTypeParameter<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSTypeParameter {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            name: CloneIn::clone_in_impl(&self.name, with_semantic_ids, allocator),
            constraint: CloneIn::clone_in_impl(&self.constraint, with_semantic_ids, allocator),
            default: CloneIn::clone_in_impl(&self.default, with_semantic_ids, allocator),
            r#in: CloneIn::clone_in_impl(&self.r#in, with_semantic_ids, allocator),
            out: CloneIn::clone_in_impl(&self.out, with_semantic_ids, allocator),
            r#const: CloneIn::clone_in_impl(&self.r#const, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeParameterDeclaration<'_> {
    type Cloned = TSTypeParameterDeclaration<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSTypeParameterDeclaration {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            params: CloneIn::clone_in_impl(&self.params, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeAliasDeclaration<'_> {
    type Cloned = TSTypeAliasDeclaration<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSTypeAliasDeclaration {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            id: CloneIn::clone_in_impl(&self.id, with_semantic_ids, allocator),
            type_parameters: CloneIn::clone_in_impl(
                &self.type_parameters,
                with_semantic_ids,
                allocator,
            ),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
            declare: CloneIn::clone_in_impl(&self.declare, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSAccessibility {
    type Cloned = TSAccessibility;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSClassImplements<'_> {
    type Cloned = TSClassImplements<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSClassImplements {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            expression: CloneIn::clone_in_impl(&self.expression, with_semantic_ids, allocator),
            type_arguments: CloneIn::clone_in_impl(
                &self.type_arguments,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSInterfaceDeclaration<'_> {
    type Cloned = TSInterfaceDeclaration<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSInterfaceDeclaration {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            id: CloneIn::clone_in_impl(&self.id, with_semantic_ids, allocator),
            type_parameters: CloneIn::clone_in_impl(
                &self.type_parameters,
                with_semantic_ids,
                allocator,
            ),
            extends: CloneIn::clone_in_impl(&self.extends, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
            declare: CloneIn::clone_in_impl(&self.declare, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSInterfaceBody<'_> {
    type Cloned = TSInterfaceBody<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSInterfaceBody {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSPropertySignature<'_> {
    type Cloned = TSPropertySignature<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSPropertySignature {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            computed: CloneIn::clone_in_impl(&self.computed, with_semantic_ids, allocator),
            optional: CloneIn::clone_in_impl(&self.optional, with_semantic_ids, allocator),
            readonly: CloneIn::clone_in_impl(&self.readonly, with_semantic_ids, allocator),
            key: CloneIn::clone_in_impl(&self.key, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSSignature<'_> {
    type Cloned = TSSignature<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::TSIndexSignature(it) => TSSignature::TSIndexSignature(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::TSPropertySignature(it) => TSSignature::TSPropertySignature(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::TSCallSignatureDeclaration(it) => TSSignature::TSCallSignatureDeclaration(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::TSConstructSignatureDeclaration(it) => {
                TSSignature::TSConstructSignatureDeclaration(CloneIn::clone_in_impl(
                    it,
                    with_semantic_ids,
                    allocator,
                ))
            }
            Self::TSMethodSignature(it) => TSSignature::TSMethodSignature(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSIndexSignature<'_> {
    type Cloned = TSIndexSignature<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSIndexSignature {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            parameters: CloneIn::clone_in_impl(&self.parameters, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
            readonly: CloneIn::clone_in_impl(&self.readonly, with_semantic_ids, allocator),
            r#static: CloneIn::clone_in_impl(&self.r#static, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSCallSignatureDeclaration<'_> {
    type Cloned = TSCallSignatureDeclaration<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSCallSignatureDeclaration {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            type_parameters: CloneIn::clone_in_impl(
                &self.type_parameters,
                with_semantic_ids,
                allocator,
            ),
            this_param: CloneIn::clone_in_impl(&self.this_param, with_semantic_ids, allocator),
            params: CloneIn::clone_in_impl(&self.params, with_semantic_ids, allocator),
            return_type: CloneIn::clone_in_impl(&self.return_type, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSMethodSignatureKind {
    type Cloned = TSMethodSignatureKind;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSMethodSignature<'_> {
    type Cloned = TSMethodSignature<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSMethodSignature {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            key: CloneIn::clone_in_impl(&self.key, with_semantic_ids, allocator),
            computed: CloneIn::clone_in_impl(&self.computed, with_semantic_ids, allocator),
            optional: CloneIn::clone_in_impl(&self.optional, with_semantic_ids, allocator),
            kind: CloneIn::clone_in_impl(&self.kind, with_semantic_ids, allocator),
            type_parameters: CloneIn::clone_in_impl(
                &self.type_parameters,
                with_semantic_ids,
                allocator,
            ),
            this_param: CloneIn::clone_in_impl(&self.this_param, with_semantic_ids, allocator),
            params: CloneIn::clone_in_impl(&self.params, with_semantic_ids, allocator),
            return_type: CloneIn::clone_in_impl(&self.return_type, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSConstructSignatureDeclaration<'_> {
    type Cloned = TSConstructSignatureDeclaration<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSConstructSignatureDeclaration {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            type_parameters: CloneIn::clone_in_impl(
                &self.type_parameters,
                with_semantic_ids,
                allocator,
            ),
            params: CloneIn::clone_in_impl(&self.params, with_semantic_ids, allocator),
            return_type: CloneIn::clone_in_impl(&self.return_type, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSIndexSignatureName<'_> {
    type Cloned = TSIndexSignatureName<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSIndexSignatureName {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            name: CloneIn::clone_in_impl(&self.name, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSInterfaceHeritage<'_> {
    type Cloned = TSInterfaceHeritage<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSInterfaceHeritage {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            expression: CloneIn::clone_in_impl(&self.expression, with_semantic_ids, allocator),
            type_arguments: CloneIn::clone_in_impl(
                &self.type_arguments,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypePredicate<'_> {
    type Cloned = TSTypePredicate<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSTypePredicate {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            parameter_name: CloneIn::clone_in_impl(
                &self.parameter_name,
                with_semantic_ids,
                allocator,
            ),
            asserts: CloneIn::clone_in_impl(&self.asserts, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypePredicateName<'_> {
    type Cloned = TSTypePredicateName<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::Identifier(it) => TSTypePredicateName::Identifier(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::This(it) => {
                TSTypePredicateName::This(CloneIn::clone_in_impl(it, with_semantic_ids, allocator))
            }
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSModuleDeclaration<'_> {
    type Cloned = TSModuleDeclaration<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSModuleDeclaration {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            id: CloneIn::clone_in_impl(&self.id, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
            kind: CloneIn::clone_in_impl(&self.kind, with_semantic_ids, allocator),
            declare: CloneIn::clone_in_impl(&self.declare, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSModuleDeclarationKind {
    type Cloned = TSModuleDeclarationKind;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSModuleDeclarationName<'_> {
    type Cloned = TSModuleDeclarationName<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::Identifier(it) => TSModuleDeclarationName::Identifier(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::StringLiteral(it) => TSModuleDeclarationName::StringLiteral(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSModuleDeclarationBody<'_> {
    type Cloned = TSModuleDeclarationBody<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::TSModuleDeclaration(it) => TSModuleDeclarationBody::TSModuleDeclaration(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::TSModuleBlock(it) => TSModuleDeclarationBody::TSModuleBlock(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSGlobalDeclaration<'_> {
    type Cloned = TSGlobalDeclaration<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSGlobalDeclaration {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            global_span: CloneIn::clone_in_impl(&self.global_span, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
            declare: CloneIn::clone_in_impl(&self.declare, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSModuleBlock<'_> {
    type Cloned = TSModuleBlock<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSModuleBlock {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            directives: CloneIn::clone_in_impl(&self.directives, with_semantic_ids, allocator),
            body: CloneIn::clone_in_impl(&self.body, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeLiteral<'_> {
    type Cloned = TSTypeLiteral<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSTypeLiteral {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            members: CloneIn::clone_in_impl(&self.members, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSInferType<'_> {
    type Cloned = TSInferType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSInferType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            type_parameter: CloneIn::clone_in_impl(
                &self.type_parameter,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeQuery<'_> {
    type Cloned = TSTypeQuery<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSTypeQuery {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            expr_name: CloneIn::clone_in_impl(&self.expr_name, with_semantic_ids, allocator),
            type_arguments: CloneIn::clone_in_impl(
                &self.type_arguments,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeQueryExprName<'_> {
    type Cloned = TSTypeQueryExprName<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::TSImportType(it) => TSTypeQueryExprName::TSImportType(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::IdentifierReference(_) | Self::QualifiedName(_) | Self::ThisExpression(_) => {
                TSTypeQueryExprName::from(CloneIn::clone_in_impl(
                    self.to_ts_type_name(),
                    with_semantic_ids,
                    allocator,
                ))
            }
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSImportType<'_> {
    type Cloned = TSImportType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSImportType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            source: CloneIn::clone_in_impl(&self.source, with_semantic_ids, allocator),
            options: CloneIn::clone_in_impl(&self.options, with_semantic_ids, allocator),
            qualifier: CloneIn::clone_in_impl(&self.qualifier, with_semantic_ids, allocator),
            type_arguments: CloneIn::clone_in_impl(
                &self.type_arguments,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSImportTypeQualifier<'_> {
    type Cloned = TSImportTypeQualifier<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::Identifier(it) => TSImportTypeQualifier::Identifier(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
            Self::QualifiedName(it) => TSImportTypeQualifier::QualifiedName(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSImportTypeQualifiedName<'_> {
    type Cloned = TSImportTypeQualifiedName<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSImportTypeQualifiedName {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            left: CloneIn::clone_in_impl(&self.left, with_semantic_ids, allocator),
            right: CloneIn::clone_in_impl(&self.right, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSFunctionType<'_> {
    type Cloned = TSFunctionType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSFunctionType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            type_parameters: CloneIn::clone_in_impl(
                &self.type_parameters,
                with_semantic_ids,
                allocator,
            ),
            this_param: CloneIn::clone_in_impl(&self.this_param, with_semantic_ids, allocator),
            params: CloneIn::clone_in_impl(&self.params, with_semantic_ids, allocator),
            return_type: CloneIn::clone_in_impl(&self.return_type, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSConstructorType<'_> {
    type Cloned = TSConstructorType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSConstructorType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            r#abstract: CloneIn::clone_in_impl(&self.r#abstract, with_semantic_ids, allocator),
            type_parameters: CloneIn::clone_in_impl(
                &self.type_parameters,
                with_semantic_ids,
                allocator,
            ),
            params: CloneIn::clone_in_impl(&self.params, with_semantic_ids, allocator),
            return_type: CloneIn::clone_in_impl(&self.return_type, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSMappedType<'_> {
    type Cloned = TSMappedType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSMappedType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            key: CloneIn::clone_in_impl(&self.key, with_semantic_ids, allocator),
            constraint: CloneIn::clone_in_impl(&self.constraint, with_semantic_ids, allocator),
            name_type: CloneIn::clone_in_impl(&self.name_type, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
            optional: CloneIn::clone_in_impl(&self.optional, with_semantic_ids, allocator),
            readonly: CloneIn::clone_in_impl(&self.readonly, with_semantic_ids, allocator),
            scope_id: CloneIn::clone_in_impl(&self.scope_id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSMappedTypeModifierOperator {
    type Cloned = TSMappedTypeModifierOperator;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTemplateLiteralType<'_> {
    type Cloned = TSTemplateLiteralType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSTemplateLiteralType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            quasis: CloneIn::clone_in_impl(&self.quasis, with_semantic_ids, allocator),
            types: CloneIn::clone_in_impl(&self.types, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSAsExpression<'_> {
    type Cloned = TSAsExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSAsExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            expression: CloneIn::clone_in_impl(&self.expression, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSSatisfiesExpression<'_> {
    type Cloned = TSSatisfiesExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSSatisfiesExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            expression: CloneIn::clone_in_impl(&self.expression, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSTypeAssertion<'_> {
    type Cloned = TSTypeAssertion<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSTypeAssertion {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
            expression: CloneIn::clone_in_impl(&self.expression, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSImportEqualsDeclaration<'_> {
    type Cloned = TSImportEqualsDeclaration<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSImportEqualsDeclaration {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            id: CloneIn::clone_in_impl(&self.id, with_semantic_ids, allocator),
            module_reference: CloneIn::clone_in_impl(
                &self.module_reference,
                with_semantic_ids,
                allocator,
            ),
            import_kind: CloneIn::clone_in_impl(&self.import_kind, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSModuleReference<'_> {
    type Cloned = TSModuleReference<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        match self {
            Self::ExternalModuleReference(it) => TSModuleReference::ExternalModuleReference(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::IdentifierReference(it) => TSModuleReference::IdentifierReference(
                CloneIn::clone_in_impl(it, with_semantic_ids, allocator),
            ),
            Self::QualifiedName(it) => TSModuleReference::QualifiedName(CloneIn::clone_in_impl(
                it,
                with_semantic_ids,
                allocator,
            )),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSExternalModuleReference<'_> {
    type Cloned = TSExternalModuleReference<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSExternalModuleReference {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            expression: CloneIn::clone_in_impl(&self.expression, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSNonNullExpression<'_> {
    type Cloned = TSNonNullExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSNonNullExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            expression: CloneIn::clone_in_impl(&self.expression, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Decorator<'_> {
    type Cloned = Decorator<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Decorator {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            expression: CloneIn::clone_in_impl(&self.expression, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSExportAssignment<'_> {
    type Cloned = TSExportAssignment<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSExportAssignment {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            expression: CloneIn::clone_in_impl(&self.expression, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSNamespaceExportDeclaration<'_> {
    type Cloned = TSNamespaceExportDeclaration<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSNamespaceExportDeclaration {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            id: CloneIn::clone_in_impl(&self.id, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for TSInstantiationExpression<'_> {
    type Cloned = TSInstantiationExpression<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        TSInstantiationExpression {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            expression: CloneIn::clone_in_impl(&self.expression, with_semantic_ids, allocator),
            type_arguments: CloneIn::clone_in_impl(
                &self.type_arguments,
                with_semantic_ids,
                allocator,
            ),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for ImportOrExportKind {
    type Cloned = ImportOrExportKind;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSDocNullableType<'_> {
    type Cloned = JSDocNullableType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSDocNullableType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
            postfix: CloneIn::clone_in_impl(&self.postfix, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSDocNonNullableType<'_> {
    type Cloned = JSDocNonNullableType<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSDocNonNullableType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            type_annotation: CloneIn::clone_in_impl(
                &self.type_annotation,
                with_semantic_ids,
                allocator,
            ),
            postfix: CloneIn::clone_in_impl(&self.postfix, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSDocUnknownType {
    type Cloned = JSDocUnknownType;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSDocUnknownType {
            node_id: CloneIn::clone_in_impl(&self.node_id, with_semantic_ids, allocator),
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for CommentKind {
    type Cloned = CommentKind;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for CommentPosition {
    type Cloned = CommentPosition;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for CommentContent {
    type Cloned = CommentContent;

    #[inline(always)]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Comment {
    type Cloned = Comment;

    fn clone_in_impl(
        &self,
        with_semantic_ids: bool,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Comment {
            span: CloneIn::clone_in_impl(&self.span, with_semantic_ids, allocator),
            attached_to: CloneIn::clone_in_impl(&self.attached_to, with_semantic_ids, allocator),
            kind: CloneIn::clone_in_impl(&self.kind, with_semantic_ids, allocator),
            position: CloneIn::clone_in_impl(&self.position, with_semantic_ids, allocator),
            newlines: CloneIn::clone_in_impl(&self.newlines, with_semantic_ids, allocator),
            content: CloneIn::clone_in_impl(&self.content, with_semantic_ids, allocator),
        }
    }
}
