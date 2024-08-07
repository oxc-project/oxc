// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_codegen/src/generators/derive_clone_in.rs`

use oxc_allocator::{Allocator, CloneIn};

use crate::ast::*;

impl<'alloc> CloneIn<'alloc> for BooleanLiteral {
    type Cloned = BooleanLiteral;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        BooleanLiteral { span: self.span.clone_in(alloc), value: self.value.clone_in(alloc) }
    }
}

impl<'alloc> CloneIn<'alloc> for NullLiteral {
    type Cloned = NullLiteral;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        NullLiteral { span: self.span.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for NumericLiteral<'old_alloc> {
    type Cloned = NumericLiteral<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        NumericLiteral {
            span: self.span.clone_in(alloc),
            value: self.value.clone_in(alloc),
            raw: self.raw.clone_in(alloc),
            base: self.base.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BigIntLiteral<'old_alloc> {
    type Cloned = BigIntLiteral<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        BigIntLiteral {
            span: self.span.clone_in(alloc),
            raw: self.raw.clone_in(alloc),
            base: self.base.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for RegExpLiteral<'old_alloc> {
    type Cloned = RegExpLiteral<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        RegExpLiteral {
            span: self.span.clone_in(alloc),
            value: self.value.clone_in(alloc),
            regex: self.regex.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for RegExp<'old_alloc> {
    type Cloned = RegExp<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        RegExp { pattern: self.pattern.clone_in(alloc), flags: self.flags.clone_in(alloc) }
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
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        StringLiteral { span: self.span.clone_in(alloc), value: self.value.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Program<'old_alloc> {
    type Cloned = Program<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        Program {
            span: self.span.clone_in(alloc),
            source_type: self.source_type.clone_in(alloc),
            hashbang: self.hashbang.clone_in(alloc),
            directives: self.directives.clone_in(alloc),
            body: self.body.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Expression<'old_alloc> {
    type Cloned = Expression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::BooleanLiteral(it) => Self::Cloned::BooleanLiteral(it.clone_in(alloc)),
            Self::NullLiteral(it) => Self::Cloned::NullLiteral(it.clone_in(alloc)),
            Self::NumericLiteral(it) => Self::Cloned::NumericLiteral(it.clone_in(alloc)),
            Self::BigIntLiteral(it) => Self::Cloned::BigIntLiteral(it.clone_in(alloc)),
            Self::RegExpLiteral(it) => Self::Cloned::RegExpLiteral(it.clone_in(alloc)),
            Self::StringLiteral(it) => Self::Cloned::StringLiteral(it.clone_in(alloc)),
            Self::TemplateLiteral(it) => Self::Cloned::TemplateLiteral(it.clone_in(alloc)),
            Self::Identifier(it) => Self::Cloned::Identifier(it.clone_in(alloc)),
            Self::MetaProperty(it) => Self::Cloned::MetaProperty(it.clone_in(alloc)),
            Self::Super(it) => Self::Cloned::Super(it.clone_in(alloc)),
            Self::ArrayExpression(it) => Self::Cloned::ArrayExpression(it.clone_in(alloc)),
            Self::ArrowFunctionExpression(it) => {
                Self::Cloned::ArrowFunctionExpression(it.clone_in(alloc))
            }
            Self::AssignmentExpression(it) => {
                Self::Cloned::AssignmentExpression(it.clone_in(alloc))
            }
            Self::AwaitExpression(it) => Self::Cloned::AwaitExpression(it.clone_in(alloc)),
            Self::BinaryExpression(it) => Self::Cloned::BinaryExpression(it.clone_in(alloc)),
            Self::CallExpression(it) => Self::Cloned::CallExpression(it.clone_in(alloc)),
            Self::ChainExpression(it) => Self::Cloned::ChainExpression(it.clone_in(alloc)),
            Self::ClassExpression(it) => Self::Cloned::ClassExpression(it.clone_in(alloc)),
            Self::ConditionalExpression(it) => {
                Self::Cloned::ConditionalExpression(it.clone_in(alloc))
            }
            Self::FunctionExpression(it) => Self::Cloned::FunctionExpression(it.clone_in(alloc)),
            Self::ImportExpression(it) => Self::Cloned::ImportExpression(it.clone_in(alloc)),
            Self::LogicalExpression(it) => Self::Cloned::LogicalExpression(it.clone_in(alloc)),
            Self::NewExpression(it) => Self::Cloned::NewExpression(it.clone_in(alloc)),
            Self::ObjectExpression(it) => Self::Cloned::ObjectExpression(it.clone_in(alloc)),
            Self::ParenthesizedExpression(it) => {
                Self::Cloned::ParenthesizedExpression(it.clone_in(alloc))
            }
            Self::SequenceExpression(it) => Self::Cloned::SequenceExpression(it.clone_in(alloc)),
            Self::TaggedTemplateExpression(it) => {
                Self::Cloned::TaggedTemplateExpression(it.clone_in(alloc))
            }
            Self::ThisExpression(it) => Self::Cloned::ThisExpression(it.clone_in(alloc)),
            Self::UnaryExpression(it) => Self::Cloned::UnaryExpression(it.clone_in(alloc)),
            Self::UpdateExpression(it) => Self::Cloned::UpdateExpression(it.clone_in(alloc)),
            Self::YieldExpression(it) => Self::Cloned::YieldExpression(it.clone_in(alloc)),
            Self::PrivateInExpression(it) => Self::Cloned::PrivateInExpression(it.clone_in(alloc)),
            Self::JSXElement(it) => Self::Cloned::JSXElement(it.clone_in(alloc)),
            Self::JSXFragment(it) => Self::Cloned::JSXFragment(it.clone_in(alloc)),
            Self::TSAsExpression(it) => Self::Cloned::TSAsExpression(it.clone_in(alloc)),
            Self::TSSatisfiesExpression(it) => {
                Self::Cloned::TSSatisfiesExpression(it.clone_in(alloc))
            }
            Self::TSTypeAssertion(it) => Self::Cloned::TSTypeAssertion(it.clone_in(alloc)),
            Self::TSNonNullExpression(it) => Self::Cloned::TSNonNullExpression(it.clone_in(alloc)),
            Self::TSInstantiationExpression(it) => {
                Self::Cloned::TSInstantiationExpression(it.clone_in(alloc))
            }
            Self::ComputedMemberExpression(it) => {
                Self::Cloned::ComputedMemberExpression(it.clone_in(alloc))
            }
            Self::StaticMemberExpression(it) => {
                Self::Cloned::StaticMemberExpression(it.clone_in(alloc))
            }
            Self::PrivateFieldExpression(it) => {
                Self::Cloned::PrivateFieldExpression(it.clone_in(alloc))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for IdentifierName<'old_alloc> {
    type Cloned = IdentifierName<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        IdentifierName { span: self.span.clone_in(alloc), name: self.name.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for IdentifierReference<'old_alloc> {
    type Cloned = IdentifierReference<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        IdentifierReference {
            span: self.span.clone_in(alloc),
            name: self.name.clone_in(alloc),
            reference_id: self.reference_id.clone_in(alloc),
            reference_flag: self.reference_flag.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BindingIdentifier<'old_alloc> {
    type Cloned = BindingIdentifier<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        BindingIdentifier {
            span: self.span.clone_in(alloc),
            name: self.name.clone_in(alloc),
            symbol_id: self.symbol_id.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for LabelIdentifier<'old_alloc> {
    type Cloned = LabelIdentifier<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        LabelIdentifier { span: self.span.clone_in(alloc), name: self.name.clone_in(alloc) }
    }
}

impl<'alloc> CloneIn<'alloc> for ThisExpression {
    type Cloned = ThisExpression;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        ThisExpression { span: self.span.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ArrayExpression<'old_alloc> {
    type Cloned = ArrayExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ArrayExpression {
            span: self.span.clone_in(alloc),
            elements: self.elements.clone_in(alloc),
            trailing_comma: self.trailing_comma.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ArrayExpressionElement<'old_alloc> {
    type Cloned = ArrayExpressionElement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::SpreadElement(it) => Self::Cloned::SpreadElement(it.clone_in(alloc)),
            Self::Elision(it) => Self::Cloned::Elision(it.clone_in(alloc)),
            Self::BooleanLiteral(it) => Self::Cloned::BooleanLiteral(it.clone_in(alloc)),
            Self::NullLiteral(it) => Self::Cloned::NullLiteral(it.clone_in(alloc)),
            Self::NumericLiteral(it) => Self::Cloned::NumericLiteral(it.clone_in(alloc)),
            Self::BigIntLiteral(it) => Self::Cloned::BigIntLiteral(it.clone_in(alloc)),
            Self::RegExpLiteral(it) => Self::Cloned::RegExpLiteral(it.clone_in(alloc)),
            Self::StringLiteral(it) => Self::Cloned::StringLiteral(it.clone_in(alloc)),
            Self::TemplateLiteral(it) => Self::Cloned::TemplateLiteral(it.clone_in(alloc)),
            Self::Identifier(it) => Self::Cloned::Identifier(it.clone_in(alloc)),
            Self::MetaProperty(it) => Self::Cloned::MetaProperty(it.clone_in(alloc)),
            Self::Super(it) => Self::Cloned::Super(it.clone_in(alloc)),
            Self::ArrayExpression(it) => Self::Cloned::ArrayExpression(it.clone_in(alloc)),
            Self::ArrowFunctionExpression(it) => {
                Self::Cloned::ArrowFunctionExpression(it.clone_in(alloc))
            }
            Self::AssignmentExpression(it) => {
                Self::Cloned::AssignmentExpression(it.clone_in(alloc))
            }
            Self::AwaitExpression(it) => Self::Cloned::AwaitExpression(it.clone_in(alloc)),
            Self::BinaryExpression(it) => Self::Cloned::BinaryExpression(it.clone_in(alloc)),
            Self::CallExpression(it) => Self::Cloned::CallExpression(it.clone_in(alloc)),
            Self::ChainExpression(it) => Self::Cloned::ChainExpression(it.clone_in(alloc)),
            Self::ClassExpression(it) => Self::Cloned::ClassExpression(it.clone_in(alloc)),
            Self::ConditionalExpression(it) => {
                Self::Cloned::ConditionalExpression(it.clone_in(alloc))
            }
            Self::FunctionExpression(it) => Self::Cloned::FunctionExpression(it.clone_in(alloc)),
            Self::ImportExpression(it) => Self::Cloned::ImportExpression(it.clone_in(alloc)),
            Self::LogicalExpression(it) => Self::Cloned::LogicalExpression(it.clone_in(alloc)),
            Self::NewExpression(it) => Self::Cloned::NewExpression(it.clone_in(alloc)),
            Self::ObjectExpression(it) => Self::Cloned::ObjectExpression(it.clone_in(alloc)),
            Self::ParenthesizedExpression(it) => {
                Self::Cloned::ParenthesizedExpression(it.clone_in(alloc))
            }
            Self::SequenceExpression(it) => Self::Cloned::SequenceExpression(it.clone_in(alloc)),
            Self::TaggedTemplateExpression(it) => {
                Self::Cloned::TaggedTemplateExpression(it.clone_in(alloc))
            }
            Self::ThisExpression(it) => Self::Cloned::ThisExpression(it.clone_in(alloc)),
            Self::UnaryExpression(it) => Self::Cloned::UnaryExpression(it.clone_in(alloc)),
            Self::UpdateExpression(it) => Self::Cloned::UpdateExpression(it.clone_in(alloc)),
            Self::YieldExpression(it) => Self::Cloned::YieldExpression(it.clone_in(alloc)),
            Self::PrivateInExpression(it) => Self::Cloned::PrivateInExpression(it.clone_in(alloc)),
            Self::JSXElement(it) => Self::Cloned::JSXElement(it.clone_in(alloc)),
            Self::JSXFragment(it) => Self::Cloned::JSXFragment(it.clone_in(alloc)),
            Self::TSAsExpression(it) => Self::Cloned::TSAsExpression(it.clone_in(alloc)),
            Self::TSSatisfiesExpression(it) => {
                Self::Cloned::TSSatisfiesExpression(it.clone_in(alloc))
            }
            Self::TSTypeAssertion(it) => Self::Cloned::TSTypeAssertion(it.clone_in(alloc)),
            Self::TSNonNullExpression(it) => Self::Cloned::TSNonNullExpression(it.clone_in(alloc)),
            Self::TSInstantiationExpression(it) => {
                Self::Cloned::TSInstantiationExpression(it.clone_in(alloc))
            }
            Self::ComputedMemberExpression(it) => {
                Self::Cloned::ComputedMemberExpression(it.clone_in(alloc))
            }
            Self::StaticMemberExpression(it) => {
                Self::Cloned::StaticMemberExpression(it.clone_in(alloc))
            }
            Self::PrivateFieldExpression(it) => {
                Self::Cloned::PrivateFieldExpression(it.clone_in(alloc))
            }
        }
    }
}

impl<'alloc> CloneIn<'alloc> for Elision {
    type Cloned = Elision;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        Elision { span: self.span.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ObjectExpression<'old_alloc> {
    type Cloned = ObjectExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ObjectExpression {
            span: self.span.clone_in(alloc),
            properties: self.properties.clone_in(alloc),
            trailing_comma: self.trailing_comma.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ObjectPropertyKind<'old_alloc> {
    type Cloned = ObjectPropertyKind<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::ObjectProperty(it) => Self::Cloned::ObjectProperty(it.clone_in(alloc)),
            Self::SpreadProperty(it) => Self::Cloned::SpreadProperty(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ObjectProperty<'old_alloc> {
    type Cloned = ObjectProperty<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ObjectProperty {
            span: self.span.clone_in(alloc),
            kind: self.kind.clone_in(alloc),
            key: self.key.clone_in(alloc),
            value: self.value.clone_in(alloc),
            init: self.init.clone_in(alloc),
            method: self.method.clone_in(alloc),
            shorthand: self.shorthand.clone_in(alloc),
            computed: self.computed.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for PropertyKey<'old_alloc> {
    type Cloned = PropertyKey<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::StaticIdentifier(it) => Self::Cloned::StaticIdentifier(it.clone_in(alloc)),
            Self::PrivateIdentifier(it) => Self::Cloned::PrivateIdentifier(it.clone_in(alloc)),
            Self::BooleanLiteral(it) => Self::Cloned::BooleanLiteral(it.clone_in(alloc)),
            Self::NullLiteral(it) => Self::Cloned::NullLiteral(it.clone_in(alloc)),
            Self::NumericLiteral(it) => Self::Cloned::NumericLiteral(it.clone_in(alloc)),
            Self::BigIntLiteral(it) => Self::Cloned::BigIntLiteral(it.clone_in(alloc)),
            Self::RegExpLiteral(it) => Self::Cloned::RegExpLiteral(it.clone_in(alloc)),
            Self::StringLiteral(it) => Self::Cloned::StringLiteral(it.clone_in(alloc)),
            Self::TemplateLiteral(it) => Self::Cloned::TemplateLiteral(it.clone_in(alloc)),
            Self::Identifier(it) => Self::Cloned::Identifier(it.clone_in(alloc)),
            Self::MetaProperty(it) => Self::Cloned::MetaProperty(it.clone_in(alloc)),
            Self::Super(it) => Self::Cloned::Super(it.clone_in(alloc)),
            Self::ArrayExpression(it) => Self::Cloned::ArrayExpression(it.clone_in(alloc)),
            Self::ArrowFunctionExpression(it) => {
                Self::Cloned::ArrowFunctionExpression(it.clone_in(alloc))
            }
            Self::AssignmentExpression(it) => {
                Self::Cloned::AssignmentExpression(it.clone_in(alloc))
            }
            Self::AwaitExpression(it) => Self::Cloned::AwaitExpression(it.clone_in(alloc)),
            Self::BinaryExpression(it) => Self::Cloned::BinaryExpression(it.clone_in(alloc)),
            Self::CallExpression(it) => Self::Cloned::CallExpression(it.clone_in(alloc)),
            Self::ChainExpression(it) => Self::Cloned::ChainExpression(it.clone_in(alloc)),
            Self::ClassExpression(it) => Self::Cloned::ClassExpression(it.clone_in(alloc)),
            Self::ConditionalExpression(it) => {
                Self::Cloned::ConditionalExpression(it.clone_in(alloc))
            }
            Self::FunctionExpression(it) => Self::Cloned::FunctionExpression(it.clone_in(alloc)),
            Self::ImportExpression(it) => Self::Cloned::ImportExpression(it.clone_in(alloc)),
            Self::LogicalExpression(it) => Self::Cloned::LogicalExpression(it.clone_in(alloc)),
            Self::NewExpression(it) => Self::Cloned::NewExpression(it.clone_in(alloc)),
            Self::ObjectExpression(it) => Self::Cloned::ObjectExpression(it.clone_in(alloc)),
            Self::ParenthesizedExpression(it) => {
                Self::Cloned::ParenthesizedExpression(it.clone_in(alloc))
            }
            Self::SequenceExpression(it) => Self::Cloned::SequenceExpression(it.clone_in(alloc)),
            Self::TaggedTemplateExpression(it) => {
                Self::Cloned::TaggedTemplateExpression(it.clone_in(alloc))
            }
            Self::ThisExpression(it) => Self::Cloned::ThisExpression(it.clone_in(alloc)),
            Self::UnaryExpression(it) => Self::Cloned::UnaryExpression(it.clone_in(alloc)),
            Self::UpdateExpression(it) => Self::Cloned::UpdateExpression(it.clone_in(alloc)),
            Self::YieldExpression(it) => Self::Cloned::YieldExpression(it.clone_in(alloc)),
            Self::PrivateInExpression(it) => Self::Cloned::PrivateInExpression(it.clone_in(alloc)),
            Self::JSXElement(it) => Self::Cloned::JSXElement(it.clone_in(alloc)),
            Self::JSXFragment(it) => Self::Cloned::JSXFragment(it.clone_in(alloc)),
            Self::TSAsExpression(it) => Self::Cloned::TSAsExpression(it.clone_in(alloc)),
            Self::TSSatisfiesExpression(it) => {
                Self::Cloned::TSSatisfiesExpression(it.clone_in(alloc))
            }
            Self::TSTypeAssertion(it) => Self::Cloned::TSTypeAssertion(it.clone_in(alloc)),
            Self::TSNonNullExpression(it) => Self::Cloned::TSNonNullExpression(it.clone_in(alloc)),
            Self::TSInstantiationExpression(it) => {
                Self::Cloned::TSInstantiationExpression(it.clone_in(alloc))
            }
            Self::ComputedMemberExpression(it) => {
                Self::Cloned::ComputedMemberExpression(it.clone_in(alloc))
            }
            Self::StaticMemberExpression(it) => {
                Self::Cloned::StaticMemberExpression(it.clone_in(alloc))
            }
            Self::PrivateFieldExpression(it) => {
                Self::Cloned::PrivateFieldExpression(it.clone_in(alloc))
            }
        }
    }
}

impl<'alloc> CloneIn<'alloc> for PropertyKind {
    type Cloned = PropertyKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Init => Self::Cloned::Init,
            Self::Get => Self::Cloned::Get,
            Self::Set => Self::Cloned::Set,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TemplateLiteral<'old_alloc> {
    type Cloned = TemplateLiteral<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TemplateLiteral {
            span: self.span.clone_in(alloc),
            quasis: self.quasis.clone_in(alloc),
            expressions: self.expressions.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TaggedTemplateExpression<'old_alloc> {
    type Cloned = TaggedTemplateExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TaggedTemplateExpression {
            span: self.span.clone_in(alloc),
            tag: self.tag.clone_in(alloc),
            quasi: self.quasi.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TemplateElement<'old_alloc> {
    type Cloned = TemplateElement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TemplateElement {
            span: self.span.clone_in(alloc),
            tail: self.tail.clone_in(alloc),
            value: self.value.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TemplateElementValue<'old_alloc> {
    type Cloned = TemplateElementValue<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TemplateElementValue { raw: self.raw.clone_in(alloc), cooked: self.cooked.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for MemberExpression<'old_alloc> {
    type Cloned = MemberExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::ComputedMemberExpression(it) => {
                Self::Cloned::ComputedMemberExpression(it.clone_in(alloc))
            }
            Self::StaticMemberExpression(it) => {
                Self::Cloned::StaticMemberExpression(it.clone_in(alloc))
            }
            Self::PrivateFieldExpression(it) => {
                Self::Cloned::PrivateFieldExpression(it.clone_in(alloc))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ComputedMemberExpression<'old_alloc> {
    type Cloned = ComputedMemberExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ComputedMemberExpression {
            span: self.span.clone_in(alloc),
            object: self.object.clone_in(alloc),
            expression: self.expression.clone_in(alloc),
            optional: self.optional.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for StaticMemberExpression<'old_alloc> {
    type Cloned = StaticMemberExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        StaticMemberExpression {
            span: self.span.clone_in(alloc),
            object: self.object.clone_in(alloc),
            property: self.property.clone_in(alloc),
            optional: self.optional.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for PrivateFieldExpression<'old_alloc> {
    type Cloned = PrivateFieldExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        PrivateFieldExpression {
            span: self.span.clone_in(alloc),
            object: self.object.clone_in(alloc),
            field: self.field.clone_in(alloc),
            optional: self.optional.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for CallExpression<'old_alloc> {
    type Cloned = CallExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        CallExpression {
            span: self.span.clone_in(alloc),
            arguments: self.arguments.clone_in(alloc),
            callee: self.callee.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
            optional: self.optional.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for NewExpression<'old_alloc> {
    type Cloned = NewExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        NewExpression {
            span: self.span.clone_in(alloc),
            callee: self.callee.clone_in(alloc),
            arguments: self.arguments.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for MetaProperty<'old_alloc> {
    type Cloned = MetaProperty<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        MetaProperty {
            span: self.span.clone_in(alloc),
            meta: self.meta.clone_in(alloc),
            property: self.property.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for SpreadElement<'old_alloc> {
    type Cloned = SpreadElement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        SpreadElement { span: self.span.clone_in(alloc), argument: self.argument.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Argument<'old_alloc> {
    type Cloned = Argument<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::SpreadElement(it) => Self::Cloned::SpreadElement(it.clone_in(alloc)),
            Self::BooleanLiteral(it) => Self::Cloned::BooleanLiteral(it.clone_in(alloc)),
            Self::NullLiteral(it) => Self::Cloned::NullLiteral(it.clone_in(alloc)),
            Self::NumericLiteral(it) => Self::Cloned::NumericLiteral(it.clone_in(alloc)),
            Self::BigIntLiteral(it) => Self::Cloned::BigIntLiteral(it.clone_in(alloc)),
            Self::RegExpLiteral(it) => Self::Cloned::RegExpLiteral(it.clone_in(alloc)),
            Self::StringLiteral(it) => Self::Cloned::StringLiteral(it.clone_in(alloc)),
            Self::TemplateLiteral(it) => Self::Cloned::TemplateLiteral(it.clone_in(alloc)),
            Self::Identifier(it) => Self::Cloned::Identifier(it.clone_in(alloc)),
            Self::MetaProperty(it) => Self::Cloned::MetaProperty(it.clone_in(alloc)),
            Self::Super(it) => Self::Cloned::Super(it.clone_in(alloc)),
            Self::ArrayExpression(it) => Self::Cloned::ArrayExpression(it.clone_in(alloc)),
            Self::ArrowFunctionExpression(it) => {
                Self::Cloned::ArrowFunctionExpression(it.clone_in(alloc))
            }
            Self::AssignmentExpression(it) => {
                Self::Cloned::AssignmentExpression(it.clone_in(alloc))
            }
            Self::AwaitExpression(it) => Self::Cloned::AwaitExpression(it.clone_in(alloc)),
            Self::BinaryExpression(it) => Self::Cloned::BinaryExpression(it.clone_in(alloc)),
            Self::CallExpression(it) => Self::Cloned::CallExpression(it.clone_in(alloc)),
            Self::ChainExpression(it) => Self::Cloned::ChainExpression(it.clone_in(alloc)),
            Self::ClassExpression(it) => Self::Cloned::ClassExpression(it.clone_in(alloc)),
            Self::ConditionalExpression(it) => {
                Self::Cloned::ConditionalExpression(it.clone_in(alloc))
            }
            Self::FunctionExpression(it) => Self::Cloned::FunctionExpression(it.clone_in(alloc)),
            Self::ImportExpression(it) => Self::Cloned::ImportExpression(it.clone_in(alloc)),
            Self::LogicalExpression(it) => Self::Cloned::LogicalExpression(it.clone_in(alloc)),
            Self::NewExpression(it) => Self::Cloned::NewExpression(it.clone_in(alloc)),
            Self::ObjectExpression(it) => Self::Cloned::ObjectExpression(it.clone_in(alloc)),
            Self::ParenthesizedExpression(it) => {
                Self::Cloned::ParenthesizedExpression(it.clone_in(alloc))
            }
            Self::SequenceExpression(it) => Self::Cloned::SequenceExpression(it.clone_in(alloc)),
            Self::TaggedTemplateExpression(it) => {
                Self::Cloned::TaggedTemplateExpression(it.clone_in(alloc))
            }
            Self::ThisExpression(it) => Self::Cloned::ThisExpression(it.clone_in(alloc)),
            Self::UnaryExpression(it) => Self::Cloned::UnaryExpression(it.clone_in(alloc)),
            Self::UpdateExpression(it) => Self::Cloned::UpdateExpression(it.clone_in(alloc)),
            Self::YieldExpression(it) => Self::Cloned::YieldExpression(it.clone_in(alloc)),
            Self::PrivateInExpression(it) => Self::Cloned::PrivateInExpression(it.clone_in(alloc)),
            Self::JSXElement(it) => Self::Cloned::JSXElement(it.clone_in(alloc)),
            Self::JSXFragment(it) => Self::Cloned::JSXFragment(it.clone_in(alloc)),
            Self::TSAsExpression(it) => Self::Cloned::TSAsExpression(it.clone_in(alloc)),
            Self::TSSatisfiesExpression(it) => {
                Self::Cloned::TSSatisfiesExpression(it.clone_in(alloc))
            }
            Self::TSTypeAssertion(it) => Self::Cloned::TSTypeAssertion(it.clone_in(alloc)),
            Self::TSNonNullExpression(it) => Self::Cloned::TSNonNullExpression(it.clone_in(alloc)),
            Self::TSInstantiationExpression(it) => {
                Self::Cloned::TSInstantiationExpression(it.clone_in(alloc))
            }
            Self::ComputedMemberExpression(it) => {
                Self::Cloned::ComputedMemberExpression(it.clone_in(alloc))
            }
            Self::StaticMemberExpression(it) => {
                Self::Cloned::StaticMemberExpression(it.clone_in(alloc))
            }
            Self::PrivateFieldExpression(it) => {
                Self::Cloned::PrivateFieldExpression(it.clone_in(alloc))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for UpdateExpression<'old_alloc> {
    type Cloned = UpdateExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        UpdateExpression {
            span: self.span.clone_in(alloc),
            operator: self.operator.clone_in(alloc),
            prefix: self.prefix.clone_in(alloc),
            argument: self.argument.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for UnaryExpression<'old_alloc> {
    type Cloned = UnaryExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        UnaryExpression {
            span: self.span.clone_in(alloc),
            operator: self.operator.clone_in(alloc),
            argument: self.argument.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BinaryExpression<'old_alloc> {
    type Cloned = BinaryExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        BinaryExpression {
            span: self.span.clone_in(alloc),
            left: self.left.clone_in(alloc),
            operator: self.operator.clone_in(alloc),
            right: self.right.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for PrivateInExpression<'old_alloc> {
    type Cloned = PrivateInExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        PrivateInExpression {
            span: self.span.clone_in(alloc),
            left: self.left.clone_in(alloc),
            operator: self.operator.clone_in(alloc),
            right: self.right.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for LogicalExpression<'old_alloc> {
    type Cloned = LogicalExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        LogicalExpression {
            span: self.span.clone_in(alloc),
            left: self.left.clone_in(alloc),
            operator: self.operator.clone_in(alloc),
            right: self.right.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ConditionalExpression<'old_alloc> {
    type Cloned = ConditionalExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ConditionalExpression {
            span: self.span.clone_in(alloc),
            test: self.test.clone_in(alloc),
            consequent: self.consequent.clone_in(alloc),
            alternate: self.alternate.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentExpression<'old_alloc> {
    type Cloned = AssignmentExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentExpression {
            span: self.span.clone_in(alloc),
            operator: self.operator.clone_in(alloc),
            left: self.left.clone_in(alloc),
            right: self.right.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTarget<'old_alloc> {
    type Cloned = AssignmentTarget<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::AssignmentTargetIdentifier(it) => {
                Self::Cloned::AssignmentTargetIdentifier(it.clone_in(alloc))
            }
            Self::TSAsExpression(it) => Self::Cloned::TSAsExpression(it.clone_in(alloc)),
            Self::TSSatisfiesExpression(it) => {
                Self::Cloned::TSSatisfiesExpression(it.clone_in(alloc))
            }
            Self::TSNonNullExpression(it) => Self::Cloned::TSNonNullExpression(it.clone_in(alloc)),
            Self::TSTypeAssertion(it) => Self::Cloned::TSTypeAssertion(it.clone_in(alloc)),
            Self::TSInstantiationExpression(it) => {
                Self::Cloned::TSInstantiationExpression(it.clone_in(alloc))
            }
            Self::ComputedMemberExpression(it) => {
                Self::Cloned::ComputedMemberExpression(it.clone_in(alloc))
            }
            Self::StaticMemberExpression(it) => {
                Self::Cloned::StaticMemberExpression(it.clone_in(alloc))
            }
            Self::PrivateFieldExpression(it) => {
                Self::Cloned::PrivateFieldExpression(it.clone_in(alloc))
            }
            Self::ArrayAssignmentTarget(it) => {
                Self::Cloned::ArrayAssignmentTarget(it.clone_in(alloc))
            }
            Self::ObjectAssignmentTarget(it) => {
                Self::Cloned::ObjectAssignmentTarget(it.clone_in(alloc))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for SimpleAssignmentTarget<'old_alloc> {
    type Cloned = SimpleAssignmentTarget<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::AssignmentTargetIdentifier(it) => {
                Self::Cloned::AssignmentTargetIdentifier(it.clone_in(alloc))
            }
            Self::TSAsExpression(it) => Self::Cloned::TSAsExpression(it.clone_in(alloc)),
            Self::TSSatisfiesExpression(it) => {
                Self::Cloned::TSSatisfiesExpression(it.clone_in(alloc))
            }
            Self::TSNonNullExpression(it) => Self::Cloned::TSNonNullExpression(it.clone_in(alloc)),
            Self::TSTypeAssertion(it) => Self::Cloned::TSTypeAssertion(it.clone_in(alloc)),
            Self::TSInstantiationExpression(it) => {
                Self::Cloned::TSInstantiationExpression(it.clone_in(alloc))
            }
            Self::ComputedMemberExpression(it) => {
                Self::Cloned::ComputedMemberExpression(it.clone_in(alloc))
            }
            Self::StaticMemberExpression(it) => {
                Self::Cloned::StaticMemberExpression(it.clone_in(alloc))
            }
            Self::PrivateFieldExpression(it) => {
                Self::Cloned::PrivateFieldExpression(it.clone_in(alloc))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTargetPattern<'old_alloc> {
    type Cloned = AssignmentTargetPattern<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::ArrayAssignmentTarget(it) => {
                Self::Cloned::ArrayAssignmentTarget(it.clone_in(alloc))
            }
            Self::ObjectAssignmentTarget(it) => {
                Self::Cloned::ObjectAssignmentTarget(it.clone_in(alloc))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ArrayAssignmentTarget<'old_alloc> {
    type Cloned = ArrayAssignmentTarget<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ArrayAssignmentTarget {
            span: self.span.clone_in(alloc),
            elements: self.elements.clone_in(alloc),
            rest: self.rest.clone_in(alloc),
            trailing_comma: self.trailing_comma.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ObjectAssignmentTarget<'old_alloc> {
    type Cloned = ObjectAssignmentTarget<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ObjectAssignmentTarget {
            span: self.span.clone_in(alloc),
            properties: self.properties.clone_in(alloc),
            rest: self.rest.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTargetRest<'old_alloc> {
    type Cloned = AssignmentTargetRest<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentTargetRest {
            span: self.span.clone_in(alloc),
            target: self.target.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTargetMaybeDefault<'old_alloc> {
    type Cloned = AssignmentTargetMaybeDefault<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::AssignmentTargetWithDefault(it) => {
                Self::Cloned::AssignmentTargetWithDefault(it.clone_in(alloc))
            }
            Self::AssignmentTargetIdentifier(it) => {
                Self::Cloned::AssignmentTargetIdentifier(it.clone_in(alloc))
            }
            Self::TSAsExpression(it) => Self::Cloned::TSAsExpression(it.clone_in(alloc)),
            Self::TSSatisfiesExpression(it) => {
                Self::Cloned::TSSatisfiesExpression(it.clone_in(alloc))
            }
            Self::TSNonNullExpression(it) => Self::Cloned::TSNonNullExpression(it.clone_in(alloc)),
            Self::TSTypeAssertion(it) => Self::Cloned::TSTypeAssertion(it.clone_in(alloc)),
            Self::TSInstantiationExpression(it) => {
                Self::Cloned::TSInstantiationExpression(it.clone_in(alloc))
            }
            Self::ComputedMemberExpression(it) => {
                Self::Cloned::ComputedMemberExpression(it.clone_in(alloc))
            }
            Self::StaticMemberExpression(it) => {
                Self::Cloned::StaticMemberExpression(it.clone_in(alloc))
            }
            Self::PrivateFieldExpression(it) => {
                Self::Cloned::PrivateFieldExpression(it.clone_in(alloc))
            }
            Self::ArrayAssignmentTarget(it) => {
                Self::Cloned::ArrayAssignmentTarget(it.clone_in(alloc))
            }
            Self::ObjectAssignmentTarget(it) => {
                Self::Cloned::ObjectAssignmentTarget(it.clone_in(alloc))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTargetWithDefault<'old_alloc> {
    type Cloned = AssignmentTargetWithDefault<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentTargetWithDefault {
            span: self.span.clone_in(alloc),
            binding: self.binding.clone_in(alloc),
            init: self.init.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTargetProperty<'old_alloc> {
    type Cloned = AssignmentTargetProperty<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => {
                Self::Cloned::AssignmentTargetPropertyIdentifier(it.clone_in(alloc))
            }
            Self::AssignmentTargetPropertyProperty(it) => {
                Self::Cloned::AssignmentTargetPropertyProperty(it.clone_in(alloc))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc>
    for AssignmentTargetPropertyIdentifier<'old_alloc>
{
    type Cloned = AssignmentTargetPropertyIdentifier<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentTargetPropertyIdentifier {
            span: self.span.clone_in(alloc),
            binding: self.binding.clone_in(alloc),
            init: self.init.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentTargetPropertyProperty<'old_alloc> {
    type Cloned = AssignmentTargetPropertyProperty<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentTargetPropertyProperty {
            span: self.span.clone_in(alloc),
            name: self.name.clone_in(alloc),
            binding: self.binding.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for SequenceExpression<'old_alloc> {
    type Cloned = SequenceExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        SequenceExpression {
            span: self.span.clone_in(alloc),
            expressions: self.expressions.clone_in(alloc),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for Super {
    type Cloned = Super;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        Super { span: self.span.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AwaitExpression<'old_alloc> {
    type Cloned = AwaitExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        AwaitExpression { span: self.span.clone_in(alloc), argument: self.argument.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ChainExpression<'old_alloc> {
    type Cloned = ChainExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ChainExpression {
            span: self.span.clone_in(alloc),
            expression: self.expression.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ChainElement<'old_alloc> {
    type Cloned = ChainElement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::CallExpression(it) => Self::Cloned::CallExpression(it.clone_in(alloc)),
            Self::ComputedMemberExpression(it) => {
                Self::Cloned::ComputedMemberExpression(it.clone_in(alloc))
            }
            Self::StaticMemberExpression(it) => {
                Self::Cloned::StaticMemberExpression(it.clone_in(alloc))
            }
            Self::PrivateFieldExpression(it) => {
                Self::Cloned::PrivateFieldExpression(it.clone_in(alloc))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ParenthesizedExpression<'old_alloc> {
    type Cloned = ParenthesizedExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ParenthesizedExpression {
            span: self.span.clone_in(alloc),
            expression: self.expression.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Statement<'old_alloc> {
    type Cloned = Statement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::BlockStatement(it) => Self::Cloned::BlockStatement(it.clone_in(alloc)),
            Self::BreakStatement(it) => Self::Cloned::BreakStatement(it.clone_in(alloc)),
            Self::ContinueStatement(it) => Self::Cloned::ContinueStatement(it.clone_in(alloc)),
            Self::DebuggerStatement(it) => Self::Cloned::DebuggerStatement(it.clone_in(alloc)),
            Self::DoWhileStatement(it) => Self::Cloned::DoWhileStatement(it.clone_in(alloc)),
            Self::EmptyStatement(it) => Self::Cloned::EmptyStatement(it.clone_in(alloc)),
            Self::ExpressionStatement(it) => Self::Cloned::ExpressionStatement(it.clone_in(alloc)),
            Self::ForInStatement(it) => Self::Cloned::ForInStatement(it.clone_in(alloc)),
            Self::ForOfStatement(it) => Self::Cloned::ForOfStatement(it.clone_in(alloc)),
            Self::ForStatement(it) => Self::Cloned::ForStatement(it.clone_in(alloc)),
            Self::IfStatement(it) => Self::Cloned::IfStatement(it.clone_in(alloc)),
            Self::LabeledStatement(it) => Self::Cloned::LabeledStatement(it.clone_in(alloc)),
            Self::ReturnStatement(it) => Self::Cloned::ReturnStatement(it.clone_in(alloc)),
            Self::SwitchStatement(it) => Self::Cloned::SwitchStatement(it.clone_in(alloc)),
            Self::ThrowStatement(it) => Self::Cloned::ThrowStatement(it.clone_in(alloc)),
            Self::TryStatement(it) => Self::Cloned::TryStatement(it.clone_in(alloc)),
            Self::WhileStatement(it) => Self::Cloned::WhileStatement(it.clone_in(alloc)),
            Self::WithStatement(it) => Self::Cloned::WithStatement(it.clone_in(alloc)),
            Self::VariableDeclaration(it) => Self::Cloned::VariableDeclaration(it.clone_in(alloc)),
            Self::FunctionDeclaration(it) => Self::Cloned::FunctionDeclaration(it.clone_in(alloc)),
            Self::ClassDeclaration(it) => Self::Cloned::ClassDeclaration(it.clone_in(alloc)),
            Self::UsingDeclaration(it) => Self::Cloned::UsingDeclaration(it.clone_in(alloc)),
            Self::TSTypeAliasDeclaration(it) => {
                Self::Cloned::TSTypeAliasDeclaration(it.clone_in(alloc))
            }
            Self::TSInterfaceDeclaration(it) => {
                Self::Cloned::TSInterfaceDeclaration(it.clone_in(alloc))
            }
            Self::TSEnumDeclaration(it) => Self::Cloned::TSEnumDeclaration(it.clone_in(alloc)),
            Self::TSModuleDeclaration(it) => Self::Cloned::TSModuleDeclaration(it.clone_in(alloc)),
            Self::TSImportEqualsDeclaration(it) => {
                Self::Cloned::TSImportEqualsDeclaration(it.clone_in(alloc))
            }
            Self::ImportDeclaration(it) => Self::Cloned::ImportDeclaration(it.clone_in(alloc)),
            Self::ExportAllDeclaration(it) => {
                Self::Cloned::ExportAllDeclaration(it.clone_in(alloc))
            }
            Self::ExportDefaultDeclaration(it) => {
                Self::Cloned::ExportDefaultDeclaration(it.clone_in(alloc))
            }
            Self::ExportNamedDeclaration(it) => {
                Self::Cloned::ExportNamedDeclaration(it.clone_in(alloc))
            }
            Self::TSExportAssignment(it) => Self::Cloned::TSExportAssignment(it.clone_in(alloc)),
            Self::TSNamespaceExportDeclaration(it) => {
                Self::Cloned::TSNamespaceExportDeclaration(it.clone_in(alloc))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Directive<'old_alloc> {
    type Cloned = Directive<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        Directive {
            span: self.span.clone_in(alloc),
            expression: self.expression.clone_in(alloc),
            directive: self.directive.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Hashbang<'old_alloc> {
    type Cloned = Hashbang<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        Hashbang { span: self.span.clone_in(alloc), value: self.value.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BlockStatement<'old_alloc> {
    type Cloned = BlockStatement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        BlockStatement {
            span: self.span.clone_in(alloc),
            body: self.body.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Declaration<'old_alloc> {
    type Cloned = Declaration<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::VariableDeclaration(it) => Self::Cloned::VariableDeclaration(it.clone_in(alloc)),
            Self::FunctionDeclaration(it) => Self::Cloned::FunctionDeclaration(it.clone_in(alloc)),
            Self::ClassDeclaration(it) => Self::Cloned::ClassDeclaration(it.clone_in(alloc)),
            Self::UsingDeclaration(it) => Self::Cloned::UsingDeclaration(it.clone_in(alloc)),
            Self::TSTypeAliasDeclaration(it) => {
                Self::Cloned::TSTypeAliasDeclaration(it.clone_in(alloc))
            }
            Self::TSInterfaceDeclaration(it) => {
                Self::Cloned::TSInterfaceDeclaration(it.clone_in(alloc))
            }
            Self::TSEnumDeclaration(it) => Self::Cloned::TSEnumDeclaration(it.clone_in(alloc)),
            Self::TSModuleDeclaration(it) => Self::Cloned::TSModuleDeclaration(it.clone_in(alloc)),
            Self::TSImportEqualsDeclaration(it) => {
                Self::Cloned::TSImportEqualsDeclaration(it.clone_in(alloc))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for VariableDeclaration<'old_alloc> {
    type Cloned = VariableDeclaration<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        VariableDeclaration {
            span: self.span.clone_in(alloc),
            kind: self.kind.clone_in(alloc),
            declarations: self.declarations.clone_in(alloc),
            declare: self.declare.clone_in(alloc),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for VariableDeclarationKind {
    type Cloned = VariableDeclarationKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Var => Self::Cloned::Var,
            Self::Const => Self::Cloned::Const,
            Self::Let => Self::Cloned::Let,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for VariableDeclarator<'old_alloc> {
    type Cloned = VariableDeclarator<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        VariableDeclarator {
            span: self.span.clone_in(alloc),
            kind: self.kind.clone_in(alloc),
            id: self.id.clone_in(alloc),
            init: self.init.clone_in(alloc),
            definite: self.definite.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for UsingDeclaration<'old_alloc> {
    type Cloned = UsingDeclaration<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        UsingDeclaration {
            span: self.span.clone_in(alloc),
            is_await: self.is_await.clone_in(alloc),
            declarations: self.declarations.clone_in(alloc),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for EmptyStatement {
    type Cloned = EmptyStatement;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        EmptyStatement { span: self.span.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ExpressionStatement<'old_alloc> {
    type Cloned = ExpressionStatement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ExpressionStatement {
            span: self.span.clone_in(alloc),
            expression: self.expression.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for IfStatement<'old_alloc> {
    type Cloned = IfStatement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        IfStatement {
            span: self.span.clone_in(alloc),
            test: self.test.clone_in(alloc),
            consequent: self.consequent.clone_in(alloc),
            alternate: self.alternate.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for DoWhileStatement<'old_alloc> {
    type Cloned = DoWhileStatement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        DoWhileStatement {
            span: self.span.clone_in(alloc),
            body: self.body.clone_in(alloc),
            test: self.test.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for WhileStatement<'old_alloc> {
    type Cloned = WhileStatement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        WhileStatement {
            span: self.span.clone_in(alloc),
            test: self.test.clone_in(alloc),
            body: self.body.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ForStatement<'old_alloc> {
    type Cloned = ForStatement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ForStatement {
            span: self.span.clone_in(alloc),
            init: self.init.clone_in(alloc),
            test: self.test.clone_in(alloc),
            update: self.update.clone_in(alloc),
            body: self.body.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ForStatementInit<'old_alloc> {
    type Cloned = ForStatementInit<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::VariableDeclaration(it) => Self::Cloned::VariableDeclaration(it.clone_in(alloc)),
            Self::UsingDeclaration(it) => Self::Cloned::UsingDeclaration(it.clone_in(alloc)),
            Self::BooleanLiteral(it) => Self::Cloned::BooleanLiteral(it.clone_in(alloc)),
            Self::NullLiteral(it) => Self::Cloned::NullLiteral(it.clone_in(alloc)),
            Self::NumericLiteral(it) => Self::Cloned::NumericLiteral(it.clone_in(alloc)),
            Self::BigIntLiteral(it) => Self::Cloned::BigIntLiteral(it.clone_in(alloc)),
            Self::RegExpLiteral(it) => Self::Cloned::RegExpLiteral(it.clone_in(alloc)),
            Self::StringLiteral(it) => Self::Cloned::StringLiteral(it.clone_in(alloc)),
            Self::TemplateLiteral(it) => Self::Cloned::TemplateLiteral(it.clone_in(alloc)),
            Self::Identifier(it) => Self::Cloned::Identifier(it.clone_in(alloc)),
            Self::MetaProperty(it) => Self::Cloned::MetaProperty(it.clone_in(alloc)),
            Self::Super(it) => Self::Cloned::Super(it.clone_in(alloc)),
            Self::ArrayExpression(it) => Self::Cloned::ArrayExpression(it.clone_in(alloc)),
            Self::ArrowFunctionExpression(it) => {
                Self::Cloned::ArrowFunctionExpression(it.clone_in(alloc))
            }
            Self::AssignmentExpression(it) => {
                Self::Cloned::AssignmentExpression(it.clone_in(alloc))
            }
            Self::AwaitExpression(it) => Self::Cloned::AwaitExpression(it.clone_in(alloc)),
            Self::BinaryExpression(it) => Self::Cloned::BinaryExpression(it.clone_in(alloc)),
            Self::CallExpression(it) => Self::Cloned::CallExpression(it.clone_in(alloc)),
            Self::ChainExpression(it) => Self::Cloned::ChainExpression(it.clone_in(alloc)),
            Self::ClassExpression(it) => Self::Cloned::ClassExpression(it.clone_in(alloc)),
            Self::ConditionalExpression(it) => {
                Self::Cloned::ConditionalExpression(it.clone_in(alloc))
            }
            Self::FunctionExpression(it) => Self::Cloned::FunctionExpression(it.clone_in(alloc)),
            Self::ImportExpression(it) => Self::Cloned::ImportExpression(it.clone_in(alloc)),
            Self::LogicalExpression(it) => Self::Cloned::LogicalExpression(it.clone_in(alloc)),
            Self::NewExpression(it) => Self::Cloned::NewExpression(it.clone_in(alloc)),
            Self::ObjectExpression(it) => Self::Cloned::ObjectExpression(it.clone_in(alloc)),
            Self::ParenthesizedExpression(it) => {
                Self::Cloned::ParenthesizedExpression(it.clone_in(alloc))
            }
            Self::SequenceExpression(it) => Self::Cloned::SequenceExpression(it.clone_in(alloc)),
            Self::TaggedTemplateExpression(it) => {
                Self::Cloned::TaggedTemplateExpression(it.clone_in(alloc))
            }
            Self::ThisExpression(it) => Self::Cloned::ThisExpression(it.clone_in(alloc)),
            Self::UnaryExpression(it) => Self::Cloned::UnaryExpression(it.clone_in(alloc)),
            Self::UpdateExpression(it) => Self::Cloned::UpdateExpression(it.clone_in(alloc)),
            Self::YieldExpression(it) => Self::Cloned::YieldExpression(it.clone_in(alloc)),
            Self::PrivateInExpression(it) => Self::Cloned::PrivateInExpression(it.clone_in(alloc)),
            Self::JSXElement(it) => Self::Cloned::JSXElement(it.clone_in(alloc)),
            Self::JSXFragment(it) => Self::Cloned::JSXFragment(it.clone_in(alloc)),
            Self::TSAsExpression(it) => Self::Cloned::TSAsExpression(it.clone_in(alloc)),
            Self::TSSatisfiesExpression(it) => {
                Self::Cloned::TSSatisfiesExpression(it.clone_in(alloc))
            }
            Self::TSTypeAssertion(it) => Self::Cloned::TSTypeAssertion(it.clone_in(alloc)),
            Self::TSNonNullExpression(it) => Self::Cloned::TSNonNullExpression(it.clone_in(alloc)),
            Self::TSInstantiationExpression(it) => {
                Self::Cloned::TSInstantiationExpression(it.clone_in(alloc))
            }
            Self::ComputedMemberExpression(it) => {
                Self::Cloned::ComputedMemberExpression(it.clone_in(alloc))
            }
            Self::StaticMemberExpression(it) => {
                Self::Cloned::StaticMemberExpression(it.clone_in(alloc))
            }
            Self::PrivateFieldExpression(it) => {
                Self::Cloned::PrivateFieldExpression(it.clone_in(alloc))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ForInStatement<'old_alloc> {
    type Cloned = ForInStatement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ForInStatement {
            span: self.span.clone_in(alloc),
            left: self.left.clone_in(alloc),
            right: self.right.clone_in(alloc),
            body: self.body.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ForStatementLeft<'old_alloc> {
    type Cloned = ForStatementLeft<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::VariableDeclaration(it) => Self::Cloned::VariableDeclaration(it.clone_in(alloc)),
            Self::UsingDeclaration(it) => Self::Cloned::UsingDeclaration(it.clone_in(alloc)),
            Self::AssignmentTargetIdentifier(it) => {
                Self::Cloned::AssignmentTargetIdentifier(it.clone_in(alloc))
            }
            Self::TSAsExpression(it) => Self::Cloned::TSAsExpression(it.clone_in(alloc)),
            Self::TSSatisfiesExpression(it) => {
                Self::Cloned::TSSatisfiesExpression(it.clone_in(alloc))
            }
            Self::TSNonNullExpression(it) => Self::Cloned::TSNonNullExpression(it.clone_in(alloc)),
            Self::TSTypeAssertion(it) => Self::Cloned::TSTypeAssertion(it.clone_in(alloc)),
            Self::TSInstantiationExpression(it) => {
                Self::Cloned::TSInstantiationExpression(it.clone_in(alloc))
            }
            Self::ComputedMemberExpression(it) => {
                Self::Cloned::ComputedMemberExpression(it.clone_in(alloc))
            }
            Self::StaticMemberExpression(it) => {
                Self::Cloned::StaticMemberExpression(it.clone_in(alloc))
            }
            Self::PrivateFieldExpression(it) => {
                Self::Cloned::PrivateFieldExpression(it.clone_in(alloc))
            }
            Self::ArrayAssignmentTarget(it) => {
                Self::Cloned::ArrayAssignmentTarget(it.clone_in(alloc))
            }
            Self::ObjectAssignmentTarget(it) => {
                Self::Cloned::ObjectAssignmentTarget(it.clone_in(alloc))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ForOfStatement<'old_alloc> {
    type Cloned = ForOfStatement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ForOfStatement {
            span: self.span.clone_in(alloc),
            r#await: self.r#await.clone_in(alloc),
            left: self.left.clone_in(alloc),
            right: self.right.clone_in(alloc),
            body: self.body.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ContinueStatement<'old_alloc> {
    type Cloned = ContinueStatement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ContinueStatement { span: self.span.clone_in(alloc), label: self.label.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BreakStatement<'old_alloc> {
    type Cloned = BreakStatement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        BreakStatement { span: self.span.clone_in(alloc), label: self.label.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ReturnStatement<'old_alloc> {
    type Cloned = ReturnStatement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ReturnStatement { span: self.span.clone_in(alloc), argument: self.argument.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for WithStatement<'old_alloc> {
    type Cloned = WithStatement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        WithStatement {
            span: self.span.clone_in(alloc),
            object: self.object.clone_in(alloc),
            body: self.body.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for SwitchStatement<'old_alloc> {
    type Cloned = SwitchStatement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        SwitchStatement {
            span: self.span.clone_in(alloc),
            discriminant: self.discriminant.clone_in(alloc),
            cases: self.cases.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for SwitchCase<'old_alloc> {
    type Cloned = SwitchCase<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        SwitchCase {
            span: self.span.clone_in(alloc),
            test: self.test.clone_in(alloc),
            consequent: self.consequent.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for LabeledStatement<'old_alloc> {
    type Cloned = LabeledStatement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        LabeledStatement {
            span: self.span.clone_in(alloc),
            label: self.label.clone_in(alloc),
            body: self.body.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ThrowStatement<'old_alloc> {
    type Cloned = ThrowStatement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ThrowStatement { span: self.span.clone_in(alloc), argument: self.argument.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TryStatement<'old_alloc> {
    type Cloned = TryStatement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TryStatement {
            span: self.span.clone_in(alloc),
            block: self.block.clone_in(alloc),
            handler: self.handler.clone_in(alloc),
            finalizer: self.finalizer.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for CatchClause<'old_alloc> {
    type Cloned = CatchClause<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        CatchClause {
            span: self.span.clone_in(alloc),
            param: self.param.clone_in(alloc),
            body: self.body.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for CatchParameter<'old_alloc> {
    type Cloned = CatchParameter<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        CatchParameter { span: self.span.clone_in(alloc), pattern: self.pattern.clone_in(alloc) }
    }
}

impl<'alloc> CloneIn<'alloc> for DebuggerStatement {
    type Cloned = DebuggerStatement;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        DebuggerStatement { span: self.span.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BindingPattern<'old_alloc> {
    type Cloned = BindingPattern<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        BindingPattern {
            kind: self.kind.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
            optional: self.optional.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BindingPatternKind<'old_alloc> {
    type Cloned = BindingPatternKind<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::BindingIdentifier(it) => Self::Cloned::BindingIdentifier(it.clone_in(alloc)),
            Self::ObjectPattern(it) => Self::Cloned::ObjectPattern(it.clone_in(alloc)),
            Self::ArrayPattern(it) => Self::Cloned::ArrayPattern(it.clone_in(alloc)),
            Self::AssignmentPattern(it) => Self::Cloned::AssignmentPattern(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AssignmentPattern<'old_alloc> {
    type Cloned = AssignmentPattern<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        AssignmentPattern {
            span: self.span.clone_in(alloc),
            left: self.left.clone_in(alloc),
            right: self.right.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ObjectPattern<'old_alloc> {
    type Cloned = ObjectPattern<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ObjectPattern {
            span: self.span.clone_in(alloc),
            properties: self.properties.clone_in(alloc),
            rest: self.rest.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BindingProperty<'old_alloc> {
    type Cloned = BindingProperty<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        BindingProperty {
            span: self.span.clone_in(alloc),
            key: self.key.clone_in(alloc),
            value: self.value.clone_in(alloc),
            shorthand: self.shorthand.clone_in(alloc),
            computed: self.computed.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ArrayPattern<'old_alloc> {
    type Cloned = ArrayPattern<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ArrayPattern {
            span: self.span.clone_in(alloc),
            elements: self.elements.clone_in(alloc),
            rest: self.rest.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for BindingRestElement<'old_alloc> {
    type Cloned = BindingRestElement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        BindingRestElement {
            span: self.span.clone_in(alloc),
            argument: self.argument.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Function<'old_alloc> {
    type Cloned = Function<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        Function {
            r#type: self.r#type.clone_in(alloc),
            span: self.span.clone_in(alloc),
            id: self.id.clone_in(alloc),
            generator: self.generator.clone_in(alloc),
            r#async: self.r#async.clone_in(alloc),
            declare: self.declare.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
            this_param: self.this_param.clone_in(alloc),
            params: self.params.clone_in(alloc),
            return_type: self.return_type.clone_in(alloc),
            body: self.body.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for FunctionType {
    type Cloned = FunctionType;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::FunctionDeclaration => Self::Cloned::FunctionDeclaration,
            Self::FunctionExpression => Self::Cloned::FunctionExpression,
            Self::TSDeclareFunction => Self::Cloned::TSDeclareFunction,
            Self::TSEmptyBodyFunctionExpression => Self::Cloned::TSEmptyBodyFunctionExpression,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for FormalParameters<'old_alloc> {
    type Cloned = FormalParameters<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        FormalParameters {
            span: self.span.clone_in(alloc),
            kind: self.kind.clone_in(alloc),
            items: self.items.clone_in(alloc),
            rest: self.rest.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for FormalParameter<'old_alloc> {
    type Cloned = FormalParameter<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        FormalParameter {
            span: self.span.clone_in(alloc),
            decorators: self.decorators.clone_in(alloc),
            pattern: self.pattern.clone_in(alloc),
            accessibility: self.accessibility.clone_in(alloc),
            readonly: self.readonly.clone_in(alloc),
            r#override: self.r#override.clone_in(alloc),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for FormalParameterKind {
    type Cloned = FormalParameterKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::FormalParameter => Self::Cloned::FormalParameter,
            Self::UniqueFormalParameters => Self::Cloned::UniqueFormalParameters,
            Self::ArrowFormalParameters => Self::Cloned::ArrowFormalParameters,
            Self::Signature => Self::Cloned::Signature,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for FunctionBody<'old_alloc> {
    type Cloned = FunctionBody<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        FunctionBody {
            span: self.span.clone_in(alloc),
            directives: self.directives.clone_in(alloc),
            statements: self.statements.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ArrowFunctionExpression<'old_alloc> {
    type Cloned = ArrowFunctionExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ArrowFunctionExpression {
            span: self.span.clone_in(alloc),
            expression: self.expression.clone_in(alloc),
            r#async: self.r#async.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
            params: self.params.clone_in(alloc),
            return_type: self.return_type.clone_in(alloc),
            body: self.body.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for YieldExpression<'old_alloc> {
    type Cloned = YieldExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        YieldExpression {
            span: self.span.clone_in(alloc),
            delegate: self.delegate.clone_in(alloc),
            argument: self.argument.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Class<'old_alloc> {
    type Cloned = Class<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        Class {
            r#type: self.r#type.clone_in(alloc),
            span: self.span.clone_in(alloc),
            decorators: self.decorators.clone_in(alloc),
            id: self.id.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
            super_class: self.super_class.clone_in(alloc),
            super_type_parameters: self.super_type_parameters.clone_in(alloc),
            implements: self.implements.clone_in(alloc),
            body: self.body.clone_in(alloc),
            r#abstract: self.r#abstract.clone_in(alloc),
            declare: self.declare.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for ClassType {
    type Cloned = ClassType;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::ClassDeclaration => Self::Cloned::ClassDeclaration,
            Self::ClassExpression => Self::Cloned::ClassExpression,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ClassBody<'old_alloc> {
    type Cloned = ClassBody<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ClassBody { span: self.span.clone_in(alloc), body: self.body.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ClassElement<'old_alloc> {
    type Cloned = ClassElement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::StaticBlock(it) => Self::Cloned::StaticBlock(it.clone_in(alloc)),
            Self::MethodDefinition(it) => Self::Cloned::MethodDefinition(it.clone_in(alloc)),
            Self::PropertyDefinition(it) => Self::Cloned::PropertyDefinition(it.clone_in(alloc)),
            Self::AccessorProperty(it) => Self::Cloned::AccessorProperty(it.clone_in(alloc)),
            Self::TSIndexSignature(it) => Self::Cloned::TSIndexSignature(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for MethodDefinition<'old_alloc> {
    type Cloned = MethodDefinition<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        MethodDefinition {
            r#type: self.r#type.clone_in(alloc),
            span: self.span.clone_in(alloc),
            decorators: self.decorators.clone_in(alloc),
            key: self.key.clone_in(alloc),
            value: self.value.clone_in(alloc),
            kind: self.kind.clone_in(alloc),
            computed: self.computed.clone_in(alloc),
            r#static: self.r#static.clone_in(alloc),
            r#override: self.r#override.clone_in(alloc),
            optional: self.optional.clone_in(alloc),
            accessibility: self.accessibility.clone_in(alloc),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for MethodDefinitionType {
    type Cloned = MethodDefinitionType;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::MethodDefinition => Self::Cloned::MethodDefinition,
            Self::TSAbstractMethodDefinition => Self::Cloned::TSAbstractMethodDefinition,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for PropertyDefinition<'old_alloc> {
    type Cloned = PropertyDefinition<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        PropertyDefinition {
            r#type: self.r#type.clone_in(alloc),
            span: self.span.clone_in(alloc),
            decorators: self.decorators.clone_in(alloc),
            key: self.key.clone_in(alloc),
            value: self.value.clone_in(alloc),
            computed: self.computed.clone_in(alloc),
            r#static: self.r#static.clone_in(alloc),
            declare: self.declare.clone_in(alloc),
            r#override: self.r#override.clone_in(alloc),
            optional: self.optional.clone_in(alloc),
            definite: self.definite.clone_in(alloc),
            readonly: self.readonly.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
            accessibility: self.accessibility.clone_in(alloc),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for PropertyDefinitionType {
    type Cloned = PropertyDefinitionType;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::PropertyDefinition => Self::Cloned::PropertyDefinition,
            Self::TSAbstractPropertyDefinition => Self::Cloned::TSAbstractPropertyDefinition,
        }
    }
}

impl<'alloc> CloneIn<'alloc> for MethodDefinitionKind {
    type Cloned = MethodDefinitionKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Constructor => Self::Cloned::Constructor,
            Self::Method => Self::Cloned::Method,
            Self::Get => Self::Cloned::Get,
            Self::Set => Self::Cloned::Set,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for PrivateIdentifier<'old_alloc> {
    type Cloned = PrivateIdentifier<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        PrivateIdentifier { span: self.span.clone_in(alloc), name: self.name.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for StaticBlock<'old_alloc> {
    type Cloned = StaticBlock<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        StaticBlock {
            span: self.span.clone_in(alloc),
            body: self.body.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ModuleDeclaration<'old_alloc> {
    type Cloned = ModuleDeclaration<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::ImportDeclaration(it) => Self::Cloned::ImportDeclaration(it.clone_in(alloc)),
            Self::ExportAllDeclaration(it) => {
                Self::Cloned::ExportAllDeclaration(it.clone_in(alloc))
            }
            Self::ExportDefaultDeclaration(it) => {
                Self::Cloned::ExportDefaultDeclaration(it.clone_in(alloc))
            }
            Self::ExportNamedDeclaration(it) => {
                Self::Cloned::ExportNamedDeclaration(it.clone_in(alloc))
            }
            Self::TSExportAssignment(it) => Self::Cloned::TSExportAssignment(it.clone_in(alloc)),
            Self::TSNamespaceExportDeclaration(it) => {
                Self::Cloned::TSNamespaceExportDeclaration(it.clone_in(alloc))
            }
        }
    }
}

impl<'alloc> CloneIn<'alloc> for AccessorPropertyType {
    type Cloned = AccessorPropertyType;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::AccessorProperty => Self::Cloned::AccessorProperty,
            Self::TSAbstractAccessorProperty => Self::Cloned::TSAbstractAccessorProperty,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for AccessorProperty<'old_alloc> {
    type Cloned = AccessorProperty<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        AccessorProperty {
            r#type: self.r#type.clone_in(alloc),
            span: self.span.clone_in(alloc),
            decorators: self.decorators.clone_in(alloc),
            key: self.key.clone_in(alloc),
            value: self.value.clone_in(alloc),
            computed: self.computed.clone_in(alloc),
            r#static: self.r#static.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportExpression<'old_alloc> {
    type Cloned = ImportExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ImportExpression {
            span: self.span.clone_in(alloc),
            source: self.source.clone_in(alloc),
            arguments: self.arguments.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportDeclaration<'old_alloc> {
    type Cloned = ImportDeclaration<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ImportDeclaration {
            span: self.span.clone_in(alloc),
            specifiers: self.specifiers.clone_in(alloc),
            source: self.source.clone_in(alloc),
            with_clause: self.with_clause.clone_in(alloc),
            import_kind: self.import_kind.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportDeclarationSpecifier<'old_alloc> {
    type Cloned = ImportDeclarationSpecifier<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::ImportSpecifier(it) => Self::Cloned::ImportSpecifier(it.clone_in(alloc)),
            Self::ImportDefaultSpecifier(it) => {
                Self::Cloned::ImportDefaultSpecifier(it.clone_in(alloc))
            }
            Self::ImportNamespaceSpecifier(it) => {
                Self::Cloned::ImportNamespaceSpecifier(it.clone_in(alloc))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportSpecifier<'old_alloc> {
    type Cloned = ImportSpecifier<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ImportSpecifier {
            span: self.span.clone_in(alloc),
            imported: self.imported.clone_in(alloc),
            local: self.local.clone_in(alloc),
            import_kind: self.import_kind.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportDefaultSpecifier<'old_alloc> {
    type Cloned = ImportDefaultSpecifier<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ImportDefaultSpecifier {
            span: self.span.clone_in(alloc),
            local: self.local.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportNamespaceSpecifier<'old_alloc> {
    type Cloned = ImportNamespaceSpecifier<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ImportNamespaceSpecifier {
            span: self.span.clone_in(alloc),
            local: self.local.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for WithClause<'old_alloc> {
    type Cloned = WithClause<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        WithClause {
            span: self.span.clone_in(alloc),
            attributes_keyword: self.attributes_keyword.clone_in(alloc),
            with_entries: self.with_entries.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportAttribute<'old_alloc> {
    type Cloned = ImportAttribute<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ImportAttribute {
            span: self.span.clone_in(alloc),
            key: self.key.clone_in(alloc),
            value: self.value.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ImportAttributeKey<'old_alloc> {
    type Cloned = ImportAttributeKey<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => Self::Cloned::Identifier(it.clone_in(alloc)),
            Self::StringLiteral(it) => Self::Cloned::StringLiteral(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ExportNamedDeclaration<'old_alloc> {
    type Cloned = ExportNamedDeclaration<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ExportNamedDeclaration {
            span: self.span.clone_in(alloc),
            declaration: self.declaration.clone_in(alloc),
            specifiers: self.specifiers.clone_in(alloc),
            source: self.source.clone_in(alloc),
            export_kind: self.export_kind.clone_in(alloc),
            with_clause: self.with_clause.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ExportDefaultDeclaration<'old_alloc> {
    type Cloned = ExportDefaultDeclaration<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ExportDefaultDeclaration {
            span: self.span.clone_in(alloc),
            declaration: self.declaration.clone_in(alloc),
            exported: self.exported.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ExportAllDeclaration<'old_alloc> {
    type Cloned = ExportAllDeclaration<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ExportAllDeclaration {
            span: self.span.clone_in(alloc),
            exported: self.exported.clone_in(alloc),
            source: self.source.clone_in(alloc),
            with_clause: self.with_clause.clone_in(alloc),
            export_kind: self.export_kind.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ExportSpecifier<'old_alloc> {
    type Cloned = ExportSpecifier<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        ExportSpecifier {
            span: self.span.clone_in(alloc),
            local: self.local.clone_in(alloc),
            exported: self.exported.clone_in(alloc),
            export_kind: self.export_kind.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ExportDefaultDeclarationKind<'old_alloc> {
    type Cloned = ExportDefaultDeclarationKind<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::FunctionDeclaration(it) => Self::Cloned::FunctionDeclaration(it.clone_in(alloc)),
            Self::ClassDeclaration(it) => Self::Cloned::ClassDeclaration(it.clone_in(alloc)),
            Self::TSInterfaceDeclaration(it) => {
                Self::Cloned::TSInterfaceDeclaration(it.clone_in(alloc))
            }
            Self::BooleanLiteral(it) => Self::Cloned::BooleanLiteral(it.clone_in(alloc)),
            Self::NullLiteral(it) => Self::Cloned::NullLiteral(it.clone_in(alloc)),
            Self::NumericLiteral(it) => Self::Cloned::NumericLiteral(it.clone_in(alloc)),
            Self::BigIntLiteral(it) => Self::Cloned::BigIntLiteral(it.clone_in(alloc)),
            Self::RegExpLiteral(it) => Self::Cloned::RegExpLiteral(it.clone_in(alloc)),
            Self::StringLiteral(it) => Self::Cloned::StringLiteral(it.clone_in(alloc)),
            Self::TemplateLiteral(it) => Self::Cloned::TemplateLiteral(it.clone_in(alloc)),
            Self::Identifier(it) => Self::Cloned::Identifier(it.clone_in(alloc)),
            Self::MetaProperty(it) => Self::Cloned::MetaProperty(it.clone_in(alloc)),
            Self::Super(it) => Self::Cloned::Super(it.clone_in(alloc)),
            Self::ArrayExpression(it) => Self::Cloned::ArrayExpression(it.clone_in(alloc)),
            Self::ArrowFunctionExpression(it) => {
                Self::Cloned::ArrowFunctionExpression(it.clone_in(alloc))
            }
            Self::AssignmentExpression(it) => {
                Self::Cloned::AssignmentExpression(it.clone_in(alloc))
            }
            Self::AwaitExpression(it) => Self::Cloned::AwaitExpression(it.clone_in(alloc)),
            Self::BinaryExpression(it) => Self::Cloned::BinaryExpression(it.clone_in(alloc)),
            Self::CallExpression(it) => Self::Cloned::CallExpression(it.clone_in(alloc)),
            Self::ChainExpression(it) => Self::Cloned::ChainExpression(it.clone_in(alloc)),
            Self::ClassExpression(it) => Self::Cloned::ClassExpression(it.clone_in(alloc)),
            Self::ConditionalExpression(it) => {
                Self::Cloned::ConditionalExpression(it.clone_in(alloc))
            }
            Self::FunctionExpression(it) => Self::Cloned::FunctionExpression(it.clone_in(alloc)),
            Self::ImportExpression(it) => Self::Cloned::ImportExpression(it.clone_in(alloc)),
            Self::LogicalExpression(it) => Self::Cloned::LogicalExpression(it.clone_in(alloc)),
            Self::NewExpression(it) => Self::Cloned::NewExpression(it.clone_in(alloc)),
            Self::ObjectExpression(it) => Self::Cloned::ObjectExpression(it.clone_in(alloc)),
            Self::ParenthesizedExpression(it) => {
                Self::Cloned::ParenthesizedExpression(it.clone_in(alloc))
            }
            Self::SequenceExpression(it) => Self::Cloned::SequenceExpression(it.clone_in(alloc)),
            Self::TaggedTemplateExpression(it) => {
                Self::Cloned::TaggedTemplateExpression(it.clone_in(alloc))
            }
            Self::ThisExpression(it) => Self::Cloned::ThisExpression(it.clone_in(alloc)),
            Self::UnaryExpression(it) => Self::Cloned::UnaryExpression(it.clone_in(alloc)),
            Self::UpdateExpression(it) => Self::Cloned::UpdateExpression(it.clone_in(alloc)),
            Self::YieldExpression(it) => Self::Cloned::YieldExpression(it.clone_in(alloc)),
            Self::PrivateInExpression(it) => Self::Cloned::PrivateInExpression(it.clone_in(alloc)),
            Self::JSXElement(it) => Self::Cloned::JSXElement(it.clone_in(alloc)),
            Self::JSXFragment(it) => Self::Cloned::JSXFragment(it.clone_in(alloc)),
            Self::TSAsExpression(it) => Self::Cloned::TSAsExpression(it.clone_in(alloc)),
            Self::TSSatisfiesExpression(it) => {
                Self::Cloned::TSSatisfiesExpression(it.clone_in(alloc))
            }
            Self::TSTypeAssertion(it) => Self::Cloned::TSTypeAssertion(it.clone_in(alloc)),
            Self::TSNonNullExpression(it) => Self::Cloned::TSNonNullExpression(it.clone_in(alloc)),
            Self::TSInstantiationExpression(it) => {
                Self::Cloned::TSInstantiationExpression(it.clone_in(alloc))
            }
            Self::ComputedMemberExpression(it) => {
                Self::Cloned::ComputedMemberExpression(it.clone_in(alloc))
            }
            Self::StaticMemberExpression(it) => {
                Self::Cloned::StaticMemberExpression(it.clone_in(alloc))
            }
            Self::PrivateFieldExpression(it) => {
                Self::Cloned::PrivateFieldExpression(it.clone_in(alloc))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for ModuleExportName<'old_alloc> {
    type Cloned = ModuleExportName<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::IdentifierName(it) => Self::Cloned::IdentifierName(it.clone_in(alloc)),
            Self::IdentifierReference(it) => Self::Cloned::IdentifierReference(it.clone_in(alloc)),
            Self::StringLiteral(it) => Self::Cloned::StringLiteral(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSThisParameter<'old_alloc> {
    type Cloned = TSThisParameter<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSThisParameter {
            span: self.span.clone_in(alloc),
            this: self.this.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSEnumDeclaration<'old_alloc> {
    type Cloned = TSEnumDeclaration<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSEnumDeclaration {
            span: self.span.clone_in(alloc),
            id: self.id.clone_in(alloc),
            members: self.members.clone_in(alloc),
            r#const: self.r#const.clone_in(alloc),
            declare: self.declare.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSEnumMember<'old_alloc> {
    type Cloned = TSEnumMember<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSEnumMember {
            span: self.span.clone_in(alloc),
            id: self.id.clone_in(alloc),
            initializer: self.initializer.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSEnumMemberName<'old_alloc> {
    type Cloned = TSEnumMemberName<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::StaticIdentifier(it) => Self::Cloned::StaticIdentifier(it.clone_in(alloc)),
            Self::StaticStringLiteral(it) => Self::Cloned::StaticStringLiteral(it.clone_in(alloc)),
            Self::StaticTemplateLiteral(it) => {
                Self::Cloned::StaticTemplateLiteral(it.clone_in(alloc))
            }
            Self::StaticNumericLiteral(it) => {
                Self::Cloned::StaticNumericLiteral(it.clone_in(alloc))
            }
            Self::BooleanLiteral(it) => Self::Cloned::BooleanLiteral(it.clone_in(alloc)),
            Self::NullLiteral(it) => Self::Cloned::NullLiteral(it.clone_in(alloc)),
            Self::NumericLiteral(it) => Self::Cloned::NumericLiteral(it.clone_in(alloc)),
            Self::BigIntLiteral(it) => Self::Cloned::BigIntLiteral(it.clone_in(alloc)),
            Self::RegExpLiteral(it) => Self::Cloned::RegExpLiteral(it.clone_in(alloc)),
            Self::StringLiteral(it) => Self::Cloned::StringLiteral(it.clone_in(alloc)),
            Self::TemplateLiteral(it) => Self::Cloned::TemplateLiteral(it.clone_in(alloc)),
            Self::Identifier(it) => Self::Cloned::Identifier(it.clone_in(alloc)),
            Self::MetaProperty(it) => Self::Cloned::MetaProperty(it.clone_in(alloc)),
            Self::Super(it) => Self::Cloned::Super(it.clone_in(alloc)),
            Self::ArrayExpression(it) => Self::Cloned::ArrayExpression(it.clone_in(alloc)),
            Self::ArrowFunctionExpression(it) => {
                Self::Cloned::ArrowFunctionExpression(it.clone_in(alloc))
            }
            Self::AssignmentExpression(it) => {
                Self::Cloned::AssignmentExpression(it.clone_in(alloc))
            }
            Self::AwaitExpression(it) => Self::Cloned::AwaitExpression(it.clone_in(alloc)),
            Self::BinaryExpression(it) => Self::Cloned::BinaryExpression(it.clone_in(alloc)),
            Self::CallExpression(it) => Self::Cloned::CallExpression(it.clone_in(alloc)),
            Self::ChainExpression(it) => Self::Cloned::ChainExpression(it.clone_in(alloc)),
            Self::ClassExpression(it) => Self::Cloned::ClassExpression(it.clone_in(alloc)),
            Self::ConditionalExpression(it) => {
                Self::Cloned::ConditionalExpression(it.clone_in(alloc))
            }
            Self::FunctionExpression(it) => Self::Cloned::FunctionExpression(it.clone_in(alloc)),
            Self::ImportExpression(it) => Self::Cloned::ImportExpression(it.clone_in(alloc)),
            Self::LogicalExpression(it) => Self::Cloned::LogicalExpression(it.clone_in(alloc)),
            Self::NewExpression(it) => Self::Cloned::NewExpression(it.clone_in(alloc)),
            Self::ObjectExpression(it) => Self::Cloned::ObjectExpression(it.clone_in(alloc)),
            Self::ParenthesizedExpression(it) => {
                Self::Cloned::ParenthesizedExpression(it.clone_in(alloc))
            }
            Self::SequenceExpression(it) => Self::Cloned::SequenceExpression(it.clone_in(alloc)),
            Self::TaggedTemplateExpression(it) => {
                Self::Cloned::TaggedTemplateExpression(it.clone_in(alloc))
            }
            Self::ThisExpression(it) => Self::Cloned::ThisExpression(it.clone_in(alloc)),
            Self::UnaryExpression(it) => Self::Cloned::UnaryExpression(it.clone_in(alloc)),
            Self::UpdateExpression(it) => Self::Cloned::UpdateExpression(it.clone_in(alloc)),
            Self::YieldExpression(it) => Self::Cloned::YieldExpression(it.clone_in(alloc)),
            Self::PrivateInExpression(it) => Self::Cloned::PrivateInExpression(it.clone_in(alloc)),
            Self::JSXElement(it) => Self::Cloned::JSXElement(it.clone_in(alloc)),
            Self::JSXFragment(it) => Self::Cloned::JSXFragment(it.clone_in(alloc)),
            Self::TSAsExpression(it) => Self::Cloned::TSAsExpression(it.clone_in(alloc)),
            Self::TSSatisfiesExpression(it) => {
                Self::Cloned::TSSatisfiesExpression(it.clone_in(alloc))
            }
            Self::TSTypeAssertion(it) => Self::Cloned::TSTypeAssertion(it.clone_in(alloc)),
            Self::TSNonNullExpression(it) => Self::Cloned::TSNonNullExpression(it.clone_in(alloc)),
            Self::TSInstantiationExpression(it) => {
                Self::Cloned::TSInstantiationExpression(it.clone_in(alloc))
            }
            Self::ComputedMemberExpression(it) => {
                Self::Cloned::ComputedMemberExpression(it.clone_in(alloc))
            }
            Self::StaticMemberExpression(it) => {
                Self::Cloned::StaticMemberExpression(it.clone_in(alloc))
            }
            Self::PrivateFieldExpression(it) => {
                Self::Cloned::PrivateFieldExpression(it.clone_in(alloc))
            }
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeAnnotation<'old_alloc> {
    type Cloned = TSTypeAnnotation<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeAnnotation {
            span: self.span.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSLiteralType<'old_alloc> {
    type Cloned = TSLiteralType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSLiteralType { span: self.span.clone_in(alloc), literal: self.literal.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSLiteral<'old_alloc> {
    type Cloned = TSLiteral<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::BooleanLiteral(it) => Self::Cloned::BooleanLiteral(it.clone_in(alloc)),
            Self::NullLiteral(it) => Self::Cloned::NullLiteral(it.clone_in(alloc)),
            Self::NumericLiteral(it) => Self::Cloned::NumericLiteral(it.clone_in(alloc)),
            Self::BigIntLiteral(it) => Self::Cloned::BigIntLiteral(it.clone_in(alloc)),
            Self::RegExpLiteral(it) => Self::Cloned::RegExpLiteral(it.clone_in(alloc)),
            Self::StringLiteral(it) => Self::Cloned::StringLiteral(it.clone_in(alloc)),
            Self::TemplateLiteral(it) => Self::Cloned::TemplateLiteral(it.clone_in(alloc)),
            Self::UnaryExpression(it) => Self::Cloned::UnaryExpression(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSType<'old_alloc> {
    type Cloned = TSType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::TSAnyKeyword(it) => Self::Cloned::TSAnyKeyword(it.clone_in(alloc)),
            Self::TSBigIntKeyword(it) => Self::Cloned::TSBigIntKeyword(it.clone_in(alloc)),
            Self::TSBooleanKeyword(it) => Self::Cloned::TSBooleanKeyword(it.clone_in(alloc)),
            Self::TSIntrinsicKeyword(it) => Self::Cloned::TSIntrinsicKeyword(it.clone_in(alloc)),
            Self::TSNeverKeyword(it) => Self::Cloned::TSNeverKeyword(it.clone_in(alloc)),
            Self::TSNullKeyword(it) => Self::Cloned::TSNullKeyword(it.clone_in(alloc)),
            Self::TSNumberKeyword(it) => Self::Cloned::TSNumberKeyword(it.clone_in(alloc)),
            Self::TSObjectKeyword(it) => Self::Cloned::TSObjectKeyword(it.clone_in(alloc)),
            Self::TSStringKeyword(it) => Self::Cloned::TSStringKeyword(it.clone_in(alloc)),
            Self::TSSymbolKeyword(it) => Self::Cloned::TSSymbolKeyword(it.clone_in(alloc)),
            Self::TSUndefinedKeyword(it) => Self::Cloned::TSUndefinedKeyword(it.clone_in(alloc)),
            Self::TSUnknownKeyword(it) => Self::Cloned::TSUnknownKeyword(it.clone_in(alloc)),
            Self::TSVoidKeyword(it) => Self::Cloned::TSVoidKeyword(it.clone_in(alloc)),
            Self::TSArrayType(it) => Self::Cloned::TSArrayType(it.clone_in(alloc)),
            Self::TSConditionalType(it) => Self::Cloned::TSConditionalType(it.clone_in(alloc)),
            Self::TSConstructorType(it) => Self::Cloned::TSConstructorType(it.clone_in(alloc)),
            Self::TSFunctionType(it) => Self::Cloned::TSFunctionType(it.clone_in(alloc)),
            Self::TSImportType(it) => Self::Cloned::TSImportType(it.clone_in(alloc)),
            Self::TSIndexedAccessType(it) => Self::Cloned::TSIndexedAccessType(it.clone_in(alloc)),
            Self::TSInferType(it) => Self::Cloned::TSInferType(it.clone_in(alloc)),
            Self::TSIntersectionType(it) => Self::Cloned::TSIntersectionType(it.clone_in(alloc)),
            Self::TSLiteralType(it) => Self::Cloned::TSLiteralType(it.clone_in(alloc)),
            Self::TSMappedType(it) => Self::Cloned::TSMappedType(it.clone_in(alloc)),
            Self::TSNamedTupleMember(it) => Self::Cloned::TSNamedTupleMember(it.clone_in(alloc)),
            Self::TSQualifiedName(it) => Self::Cloned::TSQualifiedName(it.clone_in(alloc)),
            Self::TSTemplateLiteralType(it) => {
                Self::Cloned::TSTemplateLiteralType(it.clone_in(alloc))
            }
            Self::TSThisType(it) => Self::Cloned::TSThisType(it.clone_in(alloc)),
            Self::TSTupleType(it) => Self::Cloned::TSTupleType(it.clone_in(alloc)),
            Self::TSTypeLiteral(it) => Self::Cloned::TSTypeLiteral(it.clone_in(alloc)),
            Self::TSTypeOperatorType(it) => Self::Cloned::TSTypeOperatorType(it.clone_in(alloc)),
            Self::TSTypePredicate(it) => Self::Cloned::TSTypePredicate(it.clone_in(alloc)),
            Self::TSTypeQuery(it) => Self::Cloned::TSTypeQuery(it.clone_in(alloc)),
            Self::TSTypeReference(it) => Self::Cloned::TSTypeReference(it.clone_in(alloc)),
            Self::TSUnionType(it) => Self::Cloned::TSUnionType(it.clone_in(alloc)),
            Self::TSParenthesizedType(it) => Self::Cloned::TSParenthesizedType(it.clone_in(alloc)),
            Self::JSDocNullableType(it) => Self::Cloned::JSDocNullableType(it.clone_in(alloc)),
            Self::JSDocNonNullableType(it) => {
                Self::Cloned::JSDocNonNullableType(it.clone_in(alloc))
            }
            Self::JSDocUnknownType(it) => Self::Cloned::JSDocUnknownType(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSConditionalType<'old_alloc> {
    type Cloned = TSConditionalType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSConditionalType {
            span: self.span.clone_in(alloc),
            check_type: self.check_type.clone_in(alloc),
            extends_type: self.extends_type.clone_in(alloc),
            true_type: self.true_type.clone_in(alloc),
            false_type: self.false_type.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSUnionType<'old_alloc> {
    type Cloned = TSUnionType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSUnionType { span: self.span.clone_in(alloc), types: self.types.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSIntersectionType<'old_alloc> {
    type Cloned = TSIntersectionType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSIntersectionType { span: self.span.clone_in(alloc), types: self.types.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSParenthesizedType<'old_alloc> {
    type Cloned = TSParenthesizedType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSParenthesizedType {
            span: self.span.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeOperator<'old_alloc> {
    type Cloned = TSTypeOperator<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeOperator {
            span: self.span.clone_in(alloc),
            operator: self.operator.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for TSTypeOperatorOperator {
    type Cloned = TSTypeOperatorOperator;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Keyof => Self::Cloned::Keyof,
            Self::Unique => Self::Cloned::Unique,
            Self::Readonly => Self::Cloned::Readonly,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSArrayType<'old_alloc> {
    type Cloned = TSArrayType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSArrayType {
            span: self.span.clone_in(alloc),
            element_type: self.element_type.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSIndexedAccessType<'old_alloc> {
    type Cloned = TSIndexedAccessType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSIndexedAccessType {
            span: self.span.clone_in(alloc),
            object_type: self.object_type.clone_in(alloc),
            index_type: self.index_type.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTupleType<'old_alloc> {
    type Cloned = TSTupleType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSTupleType {
            span: self.span.clone_in(alloc),
            element_types: self.element_types.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSNamedTupleMember<'old_alloc> {
    type Cloned = TSNamedTupleMember<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSNamedTupleMember {
            span: self.span.clone_in(alloc),
            element_type: self.element_type.clone_in(alloc),
            label: self.label.clone_in(alloc),
            optional: self.optional.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSOptionalType<'old_alloc> {
    type Cloned = TSOptionalType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSOptionalType {
            span: self.span.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSRestType<'old_alloc> {
    type Cloned = TSRestType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSRestType {
            span: self.span.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTupleElement<'old_alloc> {
    type Cloned = TSTupleElement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::TSOptionalType(it) => Self::Cloned::TSOptionalType(it.clone_in(alloc)),
            Self::TSRestType(it) => Self::Cloned::TSRestType(it.clone_in(alloc)),
            Self::TSAnyKeyword(it) => Self::Cloned::TSAnyKeyword(it.clone_in(alloc)),
            Self::TSBigIntKeyword(it) => Self::Cloned::TSBigIntKeyword(it.clone_in(alloc)),
            Self::TSBooleanKeyword(it) => Self::Cloned::TSBooleanKeyword(it.clone_in(alloc)),
            Self::TSIntrinsicKeyword(it) => Self::Cloned::TSIntrinsicKeyword(it.clone_in(alloc)),
            Self::TSNeverKeyword(it) => Self::Cloned::TSNeverKeyword(it.clone_in(alloc)),
            Self::TSNullKeyword(it) => Self::Cloned::TSNullKeyword(it.clone_in(alloc)),
            Self::TSNumberKeyword(it) => Self::Cloned::TSNumberKeyword(it.clone_in(alloc)),
            Self::TSObjectKeyword(it) => Self::Cloned::TSObjectKeyword(it.clone_in(alloc)),
            Self::TSStringKeyword(it) => Self::Cloned::TSStringKeyword(it.clone_in(alloc)),
            Self::TSSymbolKeyword(it) => Self::Cloned::TSSymbolKeyword(it.clone_in(alloc)),
            Self::TSUndefinedKeyword(it) => Self::Cloned::TSUndefinedKeyword(it.clone_in(alloc)),
            Self::TSUnknownKeyword(it) => Self::Cloned::TSUnknownKeyword(it.clone_in(alloc)),
            Self::TSVoidKeyword(it) => Self::Cloned::TSVoidKeyword(it.clone_in(alloc)),
            Self::TSArrayType(it) => Self::Cloned::TSArrayType(it.clone_in(alloc)),
            Self::TSConditionalType(it) => Self::Cloned::TSConditionalType(it.clone_in(alloc)),
            Self::TSConstructorType(it) => Self::Cloned::TSConstructorType(it.clone_in(alloc)),
            Self::TSFunctionType(it) => Self::Cloned::TSFunctionType(it.clone_in(alloc)),
            Self::TSImportType(it) => Self::Cloned::TSImportType(it.clone_in(alloc)),
            Self::TSIndexedAccessType(it) => Self::Cloned::TSIndexedAccessType(it.clone_in(alloc)),
            Self::TSInferType(it) => Self::Cloned::TSInferType(it.clone_in(alloc)),
            Self::TSIntersectionType(it) => Self::Cloned::TSIntersectionType(it.clone_in(alloc)),
            Self::TSLiteralType(it) => Self::Cloned::TSLiteralType(it.clone_in(alloc)),
            Self::TSMappedType(it) => Self::Cloned::TSMappedType(it.clone_in(alloc)),
            Self::TSNamedTupleMember(it) => Self::Cloned::TSNamedTupleMember(it.clone_in(alloc)),
            Self::TSQualifiedName(it) => Self::Cloned::TSQualifiedName(it.clone_in(alloc)),
            Self::TSTemplateLiteralType(it) => {
                Self::Cloned::TSTemplateLiteralType(it.clone_in(alloc))
            }
            Self::TSThisType(it) => Self::Cloned::TSThisType(it.clone_in(alloc)),
            Self::TSTupleType(it) => Self::Cloned::TSTupleType(it.clone_in(alloc)),
            Self::TSTypeLiteral(it) => Self::Cloned::TSTypeLiteral(it.clone_in(alloc)),
            Self::TSTypeOperatorType(it) => Self::Cloned::TSTypeOperatorType(it.clone_in(alloc)),
            Self::TSTypePredicate(it) => Self::Cloned::TSTypePredicate(it.clone_in(alloc)),
            Self::TSTypeQuery(it) => Self::Cloned::TSTypeQuery(it.clone_in(alloc)),
            Self::TSTypeReference(it) => Self::Cloned::TSTypeReference(it.clone_in(alloc)),
            Self::TSUnionType(it) => Self::Cloned::TSUnionType(it.clone_in(alloc)),
            Self::TSParenthesizedType(it) => Self::Cloned::TSParenthesizedType(it.clone_in(alloc)),
            Self::JSDocNullableType(it) => Self::Cloned::JSDocNullableType(it.clone_in(alloc)),
            Self::JSDocNonNullableType(it) => {
                Self::Cloned::JSDocNonNullableType(it.clone_in(alloc))
            }
            Self::JSDocUnknownType(it) => Self::Cloned::JSDocUnknownType(it.clone_in(alloc)),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for TSAnyKeyword {
    type Cloned = TSAnyKeyword;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        TSAnyKeyword { span: self.span.clone_in(alloc) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSStringKeyword {
    type Cloned = TSStringKeyword;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        TSStringKeyword { span: self.span.clone_in(alloc) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSBooleanKeyword {
    type Cloned = TSBooleanKeyword;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        TSBooleanKeyword { span: self.span.clone_in(alloc) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSNumberKeyword {
    type Cloned = TSNumberKeyword;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        TSNumberKeyword { span: self.span.clone_in(alloc) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSNeverKeyword {
    type Cloned = TSNeverKeyword;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        TSNeverKeyword { span: self.span.clone_in(alloc) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSIntrinsicKeyword {
    type Cloned = TSIntrinsicKeyword;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        TSIntrinsicKeyword { span: self.span.clone_in(alloc) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSUnknownKeyword {
    type Cloned = TSUnknownKeyword;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        TSUnknownKeyword { span: self.span.clone_in(alloc) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSNullKeyword {
    type Cloned = TSNullKeyword;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        TSNullKeyword { span: self.span.clone_in(alloc) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSUndefinedKeyword {
    type Cloned = TSUndefinedKeyword;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        TSUndefinedKeyword { span: self.span.clone_in(alloc) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSVoidKeyword {
    type Cloned = TSVoidKeyword;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        TSVoidKeyword { span: self.span.clone_in(alloc) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSSymbolKeyword {
    type Cloned = TSSymbolKeyword;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        TSSymbolKeyword { span: self.span.clone_in(alloc) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSThisType {
    type Cloned = TSThisType;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        TSThisType { span: self.span.clone_in(alloc) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSObjectKeyword {
    type Cloned = TSObjectKeyword;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        TSObjectKeyword { span: self.span.clone_in(alloc) }
    }
}

impl<'alloc> CloneIn<'alloc> for TSBigIntKeyword {
    type Cloned = TSBigIntKeyword;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        TSBigIntKeyword { span: self.span.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeReference<'old_alloc> {
    type Cloned = TSTypeReference<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeReference {
            span: self.span.clone_in(alloc),
            type_name: self.type_name.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeName<'old_alloc> {
    type Cloned = TSTypeName<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::IdentifierReference(it) => Self::Cloned::IdentifierReference(it.clone_in(alloc)),
            Self::QualifiedName(it) => Self::Cloned::QualifiedName(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSQualifiedName<'old_alloc> {
    type Cloned = TSQualifiedName<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSQualifiedName {
            span: self.span.clone_in(alloc),
            left: self.left.clone_in(alloc),
            right: self.right.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeParameterInstantiation<'old_alloc> {
    type Cloned = TSTypeParameterInstantiation<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeParameterInstantiation {
            span: self.span.clone_in(alloc),
            params: self.params.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeParameter<'old_alloc> {
    type Cloned = TSTypeParameter<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeParameter {
            span: self.span.clone_in(alloc),
            name: self.name.clone_in(alloc),
            constraint: self.constraint.clone_in(alloc),
            default: self.default.clone_in(alloc),
            r#in: self.r#in.clone_in(alloc),
            out: self.out.clone_in(alloc),
            r#const: self.r#const.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeParameterDeclaration<'old_alloc> {
    type Cloned = TSTypeParameterDeclaration<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeParameterDeclaration {
            span: self.span.clone_in(alloc),
            params: self.params.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeAliasDeclaration<'old_alloc> {
    type Cloned = TSTypeAliasDeclaration<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeAliasDeclaration {
            span: self.span.clone_in(alloc),
            id: self.id.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
            declare: self.declare.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for TSAccessibility {
    type Cloned = TSAccessibility;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Private => Self::Cloned::Private,
            Self::Protected => Self::Cloned::Protected,
            Self::Public => Self::Cloned::Public,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSClassImplements<'old_alloc> {
    type Cloned = TSClassImplements<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSClassImplements {
            span: self.span.clone_in(alloc),
            expression: self.expression.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSInterfaceDeclaration<'old_alloc> {
    type Cloned = TSInterfaceDeclaration<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSInterfaceDeclaration {
            span: self.span.clone_in(alloc),
            id: self.id.clone_in(alloc),
            extends: self.extends.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
            body: self.body.clone_in(alloc),
            declare: self.declare.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSInterfaceBody<'old_alloc> {
    type Cloned = TSInterfaceBody<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSInterfaceBody { span: self.span.clone_in(alloc), body: self.body.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSPropertySignature<'old_alloc> {
    type Cloned = TSPropertySignature<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSPropertySignature {
            span: self.span.clone_in(alloc),
            computed: self.computed.clone_in(alloc),
            optional: self.optional.clone_in(alloc),
            readonly: self.readonly.clone_in(alloc),
            key: self.key.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSSignature<'old_alloc> {
    type Cloned = TSSignature<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::TSIndexSignature(it) => Self::Cloned::TSIndexSignature(it.clone_in(alloc)),
            Self::TSPropertySignature(it) => Self::Cloned::TSPropertySignature(it.clone_in(alloc)),
            Self::TSCallSignatureDeclaration(it) => {
                Self::Cloned::TSCallSignatureDeclaration(it.clone_in(alloc))
            }
            Self::TSConstructSignatureDeclaration(it) => {
                Self::Cloned::TSConstructSignatureDeclaration(it.clone_in(alloc))
            }
            Self::TSMethodSignature(it) => Self::Cloned::TSMethodSignature(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSIndexSignature<'old_alloc> {
    type Cloned = TSIndexSignature<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSIndexSignature {
            span: self.span.clone_in(alloc),
            parameters: self.parameters.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
            readonly: self.readonly.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSCallSignatureDeclaration<'old_alloc> {
    type Cloned = TSCallSignatureDeclaration<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSCallSignatureDeclaration {
            span: self.span.clone_in(alloc),
            this_param: self.this_param.clone_in(alloc),
            params: self.params.clone_in(alloc),
            return_type: self.return_type.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for TSMethodSignatureKind {
    type Cloned = TSMethodSignatureKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Method => Self::Cloned::Method,
            Self::Get => Self::Cloned::Get,
            Self::Set => Self::Cloned::Set,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSMethodSignature<'old_alloc> {
    type Cloned = TSMethodSignature<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSMethodSignature {
            span: self.span.clone_in(alloc),
            key: self.key.clone_in(alloc),
            computed: self.computed.clone_in(alloc),
            optional: self.optional.clone_in(alloc),
            kind: self.kind.clone_in(alloc),
            this_param: self.this_param.clone_in(alloc),
            params: self.params.clone_in(alloc),
            return_type: self.return_type.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSConstructSignatureDeclaration<'old_alloc> {
    type Cloned = TSConstructSignatureDeclaration<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSConstructSignatureDeclaration {
            span: self.span.clone_in(alloc),
            params: self.params.clone_in(alloc),
            return_type: self.return_type.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSIndexSignatureName<'old_alloc> {
    type Cloned = TSIndexSignatureName<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSIndexSignatureName {
            span: self.span.clone_in(alloc),
            name: self.name.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSInterfaceHeritage<'old_alloc> {
    type Cloned = TSInterfaceHeritage<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSInterfaceHeritage {
            span: self.span.clone_in(alloc),
            expression: self.expression.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypePredicate<'old_alloc> {
    type Cloned = TSTypePredicate<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSTypePredicate {
            span: self.span.clone_in(alloc),
            parameter_name: self.parameter_name.clone_in(alloc),
            asserts: self.asserts.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypePredicateName<'old_alloc> {
    type Cloned = TSTypePredicateName<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => Self::Cloned::Identifier(it.clone_in(alloc)),
            Self::This(it) => Self::Cloned::This(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSModuleDeclaration<'old_alloc> {
    type Cloned = TSModuleDeclaration<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSModuleDeclaration {
            span: self.span.clone_in(alloc),
            id: self.id.clone_in(alloc),
            body: self.body.clone_in(alloc),
            kind: self.kind.clone_in(alloc),
            declare: self.declare.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for TSModuleDeclarationKind {
    type Cloned = TSModuleDeclarationKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Global => Self::Cloned::Global,
            Self::Module => Self::Cloned::Module,
            Self::Namespace => Self::Cloned::Namespace,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSModuleDeclarationName<'old_alloc> {
    type Cloned = TSModuleDeclarationName<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => Self::Cloned::Identifier(it.clone_in(alloc)),
            Self::StringLiteral(it) => Self::Cloned::StringLiteral(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSModuleDeclarationBody<'old_alloc> {
    type Cloned = TSModuleDeclarationBody<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::TSModuleDeclaration(it) => Self::Cloned::TSModuleDeclaration(it.clone_in(alloc)),
            Self::TSModuleBlock(it) => Self::Cloned::TSModuleBlock(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSModuleBlock<'old_alloc> {
    type Cloned = TSModuleBlock<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSModuleBlock {
            span: self.span.clone_in(alloc),
            directives: self.directives.clone_in(alloc),
            body: self.body.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeLiteral<'old_alloc> {
    type Cloned = TSTypeLiteral<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeLiteral { span: self.span.clone_in(alloc), members: self.members.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSInferType<'old_alloc> {
    type Cloned = TSInferType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSInferType {
            span: self.span.clone_in(alloc),
            type_parameter: self.type_parameter.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeQuery<'old_alloc> {
    type Cloned = TSTypeQuery<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeQuery {
            span: self.span.clone_in(alloc),
            expr_name: self.expr_name.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeQueryExprName<'old_alloc> {
    type Cloned = TSTypeQueryExprName<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::TSImportType(it) => Self::Cloned::TSImportType(it.clone_in(alloc)),
            Self::IdentifierReference(it) => Self::Cloned::IdentifierReference(it.clone_in(alloc)),
            Self::QualifiedName(it) => Self::Cloned::QualifiedName(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSImportType<'old_alloc> {
    type Cloned = TSImportType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSImportType {
            span: self.span.clone_in(alloc),
            is_type_of: self.is_type_of.clone_in(alloc),
            parameter: self.parameter.clone_in(alloc),
            qualifier: self.qualifier.clone_in(alloc),
            attributes: self.attributes.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSImportAttributes<'old_alloc> {
    type Cloned = TSImportAttributes<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSImportAttributes {
            span: self.span.clone_in(alloc),
            attributes_keyword: self.attributes_keyword.clone_in(alloc),
            elements: self.elements.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSImportAttribute<'old_alloc> {
    type Cloned = TSImportAttribute<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSImportAttribute {
            span: self.span.clone_in(alloc),
            name: self.name.clone_in(alloc),
            value: self.value.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSImportAttributeName<'old_alloc> {
    type Cloned = TSImportAttributeName<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => Self::Cloned::Identifier(it.clone_in(alloc)),
            Self::StringLiteral(it) => Self::Cloned::StringLiteral(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSFunctionType<'old_alloc> {
    type Cloned = TSFunctionType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSFunctionType {
            span: self.span.clone_in(alloc),
            this_param: self.this_param.clone_in(alloc),
            params: self.params.clone_in(alloc),
            return_type: self.return_type.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSConstructorType<'old_alloc> {
    type Cloned = TSConstructorType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSConstructorType {
            span: self.span.clone_in(alloc),
            r#abstract: self.r#abstract.clone_in(alloc),
            params: self.params.clone_in(alloc),
            return_type: self.return_type.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSMappedType<'old_alloc> {
    type Cloned = TSMappedType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSMappedType {
            span: self.span.clone_in(alloc),
            type_parameter: self.type_parameter.clone_in(alloc),
            name_type: self.name_type.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
            optional: self.optional.clone_in(alloc),
            readonly: self.readonly.clone_in(alloc),
            scope_id: self.scope_id.clone_in(alloc),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for TSMappedTypeModifierOperator {
    type Cloned = TSMappedTypeModifierOperator;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::True => Self::Cloned::True,
            Self::Plus => Self::Cloned::Plus,
            Self::Minus => Self::Cloned::Minus,
            Self::None => Self::Cloned::None,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTemplateLiteralType<'old_alloc> {
    type Cloned = TSTemplateLiteralType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSTemplateLiteralType {
            span: self.span.clone_in(alloc),
            quasis: self.quasis.clone_in(alloc),
            types: self.types.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSAsExpression<'old_alloc> {
    type Cloned = TSAsExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSAsExpression {
            span: self.span.clone_in(alloc),
            expression: self.expression.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSSatisfiesExpression<'old_alloc> {
    type Cloned = TSSatisfiesExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSSatisfiesExpression {
            span: self.span.clone_in(alloc),
            expression: self.expression.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSTypeAssertion<'old_alloc> {
    type Cloned = TSTypeAssertion<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSTypeAssertion {
            span: self.span.clone_in(alloc),
            expression: self.expression.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSImportEqualsDeclaration<'old_alloc> {
    type Cloned = TSImportEqualsDeclaration<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSImportEqualsDeclaration {
            span: self.span.clone_in(alloc),
            id: self.id.clone_in(alloc),
            module_reference: self.module_reference.clone_in(alloc),
            import_kind: self.import_kind.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSModuleReference<'old_alloc> {
    type Cloned = TSModuleReference<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::ExternalModuleReference(it) => {
                Self::Cloned::ExternalModuleReference(it.clone_in(alloc))
            }
            Self::IdentifierReference(it) => Self::Cloned::IdentifierReference(it.clone_in(alloc)),
            Self::QualifiedName(it) => Self::Cloned::QualifiedName(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSExternalModuleReference<'old_alloc> {
    type Cloned = TSExternalModuleReference<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSExternalModuleReference {
            span: self.span.clone_in(alloc),
            expression: self.expression.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSNonNullExpression<'old_alloc> {
    type Cloned = TSNonNullExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSNonNullExpression {
            span: self.span.clone_in(alloc),
            expression: self.expression.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Decorator<'old_alloc> {
    type Cloned = Decorator<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        Decorator { span: self.span.clone_in(alloc), expression: self.expression.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSExportAssignment<'old_alloc> {
    type Cloned = TSExportAssignment<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSExportAssignment {
            span: self.span.clone_in(alloc),
            expression: self.expression.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSNamespaceExportDeclaration<'old_alloc> {
    type Cloned = TSNamespaceExportDeclaration<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSNamespaceExportDeclaration {
            span: self.span.clone_in(alloc),
            id: self.id.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for TSInstantiationExpression<'old_alloc> {
    type Cloned = TSInstantiationExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        TSInstantiationExpression {
            span: self.span.clone_in(alloc),
            expression: self.expression.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for ImportOrExportKind {
    type Cloned = ImportOrExportKind;
    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        match self {
            Self::Value => Self::Cloned::Value,
            Self::Type => Self::Cloned::Type,
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSDocNullableType<'old_alloc> {
    type Cloned = JSDocNullableType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        JSDocNullableType {
            span: self.span.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
            postfix: self.postfix.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSDocNonNullableType<'old_alloc> {
    type Cloned = JSDocNonNullableType<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        JSDocNonNullableType {
            span: self.span.clone_in(alloc),
            type_annotation: self.type_annotation.clone_in(alloc),
            postfix: self.postfix.clone_in(alloc),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for JSDocUnknownType {
    type Cloned = JSDocUnknownType;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        JSDocUnknownType { span: self.span.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXElement<'old_alloc> {
    type Cloned = JSXElement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        JSXElement {
            span: self.span.clone_in(alloc),
            opening_element: self.opening_element.clone_in(alloc),
            closing_element: self.closing_element.clone_in(alloc),
            children: self.children.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXOpeningElement<'old_alloc> {
    type Cloned = JSXOpeningElement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        JSXOpeningElement {
            span: self.span.clone_in(alloc),
            self_closing: self.self_closing.clone_in(alloc),
            name: self.name.clone_in(alloc),
            attributes: self.attributes.clone_in(alloc),
            type_parameters: self.type_parameters.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXClosingElement<'old_alloc> {
    type Cloned = JSXClosingElement<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        JSXClosingElement { span: self.span.clone_in(alloc), name: self.name.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXFragment<'old_alloc> {
    type Cloned = JSXFragment<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        JSXFragment {
            span: self.span.clone_in(alloc),
            opening_fragment: self.opening_fragment.clone_in(alloc),
            closing_fragment: self.closing_fragment.clone_in(alloc),
            children: self.children.clone_in(alloc),
        }
    }
}

impl<'alloc> CloneIn<'alloc> for JSXOpeningFragment {
    type Cloned = JSXOpeningFragment;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        JSXOpeningFragment { span: self.span.clone_in(alloc) }
    }
}

impl<'alloc> CloneIn<'alloc> for JSXClosingFragment {
    type Cloned = JSXClosingFragment;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        JSXClosingFragment { span: self.span.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXElementName<'old_alloc> {
    type Cloned = JSXElementName<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => Self::Cloned::Identifier(it.clone_in(alloc)),
            Self::NamespacedName(it) => Self::Cloned::NamespacedName(it.clone_in(alloc)),
            Self::MemberExpression(it) => Self::Cloned::MemberExpression(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXNamespacedName<'old_alloc> {
    type Cloned = JSXNamespacedName<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        JSXNamespacedName {
            span: self.span.clone_in(alloc),
            namespace: self.namespace.clone_in(alloc),
            property: self.property.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXMemberExpression<'old_alloc> {
    type Cloned = JSXMemberExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        JSXMemberExpression {
            span: self.span.clone_in(alloc),
            object: self.object.clone_in(alloc),
            property: self.property.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXMemberExpressionObject<'old_alloc> {
    type Cloned = JSXMemberExpressionObject<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => Self::Cloned::Identifier(it.clone_in(alloc)),
            Self::MemberExpression(it) => Self::Cloned::MemberExpression(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXExpressionContainer<'old_alloc> {
    type Cloned = JSXExpressionContainer<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        JSXExpressionContainer {
            span: self.span.clone_in(alloc),
            expression: self.expression.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXExpression<'old_alloc> {
    type Cloned = JSXExpression<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::EmptyExpression(it) => Self::Cloned::EmptyExpression(it.clone_in(alloc)),
            Self::BooleanLiteral(it) => Self::Cloned::BooleanLiteral(it.clone_in(alloc)),
            Self::NullLiteral(it) => Self::Cloned::NullLiteral(it.clone_in(alloc)),
            Self::NumericLiteral(it) => Self::Cloned::NumericLiteral(it.clone_in(alloc)),
            Self::BigIntLiteral(it) => Self::Cloned::BigIntLiteral(it.clone_in(alloc)),
            Self::RegExpLiteral(it) => Self::Cloned::RegExpLiteral(it.clone_in(alloc)),
            Self::StringLiteral(it) => Self::Cloned::StringLiteral(it.clone_in(alloc)),
            Self::TemplateLiteral(it) => Self::Cloned::TemplateLiteral(it.clone_in(alloc)),
            Self::Identifier(it) => Self::Cloned::Identifier(it.clone_in(alloc)),
            Self::MetaProperty(it) => Self::Cloned::MetaProperty(it.clone_in(alloc)),
            Self::Super(it) => Self::Cloned::Super(it.clone_in(alloc)),
            Self::ArrayExpression(it) => Self::Cloned::ArrayExpression(it.clone_in(alloc)),
            Self::ArrowFunctionExpression(it) => {
                Self::Cloned::ArrowFunctionExpression(it.clone_in(alloc))
            }
            Self::AssignmentExpression(it) => {
                Self::Cloned::AssignmentExpression(it.clone_in(alloc))
            }
            Self::AwaitExpression(it) => Self::Cloned::AwaitExpression(it.clone_in(alloc)),
            Self::BinaryExpression(it) => Self::Cloned::BinaryExpression(it.clone_in(alloc)),
            Self::CallExpression(it) => Self::Cloned::CallExpression(it.clone_in(alloc)),
            Self::ChainExpression(it) => Self::Cloned::ChainExpression(it.clone_in(alloc)),
            Self::ClassExpression(it) => Self::Cloned::ClassExpression(it.clone_in(alloc)),
            Self::ConditionalExpression(it) => {
                Self::Cloned::ConditionalExpression(it.clone_in(alloc))
            }
            Self::FunctionExpression(it) => Self::Cloned::FunctionExpression(it.clone_in(alloc)),
            Self::ImportExpression(it) => Self::Cloned::ImportExpression(it.clone_in(alloc)),
            Self::LogicalExpression(it) => Self::Cloned::LogicalExpression(it.clone_in(alloc)),
            Self::NewExpression(it) => Self::Cloned::NewExpression(it.clone_in(alloc)),
            Self::ObjectExpression(it) => Self::Cloned::ObjectExpression(it.clone_in(alloc)),
            Self::ParenthesizedExpression(it) => {
                Self::Cloned::ParenthesizedExpression(it.clone_in(alloc))
            }
            Self::SequenceExpression(it) => Self::Cloned::SequenceExpression(it.clone_in(alloc)),
            Self::TaggedTemplateExpression(it) => {
                Self::Cloned::TaggedTemplateExpression(it.clone_in(alloc))
            }
            Self::ThisExpression(it) => Self::Cloned::ThisExpression(it.clone_in(alloc)),
            Self::UnaryExpression(it) => Self::Cloned::UnaryExpression(it.clone_in(alloc)),
            Self::UpdateExpression(it) => Self::Cloned::UpdateExpression(it.clone_in(alloc)),
            Self::YieldExpression(it) => Self::Cloned::YieldExpression(it.clone_in(alloc)),
            Self::PrivateInExpression(it) => Self::Cloned::PrivateInExpression(it.clone_in(alloc)),
            Self::JSXElement(it) => Self::Cloned::JSXElement(it.clone_in(alloc)),
            Self::JSXFragment(it) => Self::Cloned::JSXFragment(it.clone_in(alloc)),
            Self::TSAsExpression(it) => Self::Cloned::TSAsExpression(it.clone_in(alloc)),
            Self::TSSatisfiesExpression(it) => {
                Self::Cloned::TSSatisfiesExpression(it.clone_in(alloc))
            }
            Self::TSTypeAssertion(it) => Self::Cloned::TSTypeAssertion(it.clone_in(alloc)),
            Self::TSNonNullExpression(it) => Self::Cloned::TSNonNullExpression(it.clone_in(alloc)),
            Self::TSInstantiationExpression(it) => {
                Self::Cloned::TSInstantiationExpression(it.clone_in(alloc))
            }
            Self::ComputedMemberExpression(it) => {
                Self::Cloned::ComputedMemberExpression(it.clone_in(alloc))
            }
            Self::StaticMemberExpression(it) => {
                Self::Cloned::StaticMemberExpression(it.clone_in(alloc))
            }
            Self::PrivateFieldExpression(it) => {
                Self::Cloned::PrivateFieldExpression(it.clone_in(alloc))
            }
        }
    }
}

impl<'alloc> CloneIn<'alloc> for JSXEmptyExpression {
    type Cloned = JSXEmptyExpression;
    fn clone_in(&self, alloc: &'alloc Allocator) -> Self::Cloned {
        JSXEmptyExpression { span: self.span.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXAttributeItem<'old_alloc> {
    type Cloned = JSXAttributeItem<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Attribute(it) => Self::Cloned::Attribute(it.clone_in(alloc)),
            Self::SpreadAttribute(it) => Self::Cloned::SpreadAttribute(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXAttribute<'old_alloc> {
    type Cloned = JSXAttribute<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        JSXAttribute {
            span: self.span.clone_in(alloc),
            name: self.name.clone_in(alloc),
            value: self.value.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXSpreadAttribute<'old_alloc> {
    type Cloned = JSXSpreadAttribute<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        JSXSpreadAttribute {
            span: self.span.clone_in(alloc),
            argument: self.argument.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXAttributeName<'old_alloc> {
    type Cloned = JSXAttributeName<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Identifier(it) => Self::Cloned::Identifier(it.clone_in(alloc)),
            Self::NamespacedName(it) => Self::Cloned::NamespacedName(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXAttributeValue<'old_alloc> {
    type Cloned = JSXAttributeValue<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::StringLiteral(it) => Self::Cloned::StringLiteral(it.clone_in(alloc)),
            Self::ExpressionContainer(it) => Self::Cloned::ExpressionContainer(it.clone_in(alloc)),
            Self::Element(it) => Self::Cloned::Element(it.clone_in(alloc)),
            Self::Fragment(it) => Self::Cloned::Fragment(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXIdentifier<'old_alloc> {
    type Cloned = JSXIdentifier<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        JSXIdentifier { span: self.span.clone_in(alloc), name: self.name.clone_in(alloc) }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXChild<'old_alloc> {
    type Cloned = JSXChild<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        match self {
            Self::Text(it) => Self::Cloned::Text(it.clone_in(alloc)),
            Self::Element(it) => Self::Cloned::Element(it.clone_in(alloc)),
            Self::Fragment(it) => Self::Cloned::Fragment(it.clone_in(alloc)),
            Self::ExpressionContainer(it) => Self::Cloned::ExpressionContainer(it.clone_in(alloc)),
            Self::Spread(it) => Self::Cloned::Spread(it.clone_in(alloc)),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXSpreadChild<'old_alloc> {
    type Cloned = JSXSpreadChild<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        JSXSpreadChild {
            span: self.span.clone_in(alloc),
            expression: self.expression.clone_in(alloc),
        }
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for JSXText<'old_alloc> {
    type Cloned = JSXText<'new_alloc>;
    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        JSXText { span: self.span.clone_in(alloc), value: self.value.clone_in(alloc) }
    }
}
