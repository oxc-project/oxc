use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{
    array, dynamic_text,
    format::Format,
    group, hardline, indent,
    ir::{Doc, JoinSeparator},
    join, line, softline, text, wrap, Prettier,
};

impl<'a> Format<'a> for JSXIdentifier<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        dynamic_text!(p, self.name.as_str())
    }
}

impl<'a> Format<'a> for JSXMemberExpressionObject<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            JSXMemberExpressionObject::IdentifierReference(it) => it.format(p),
            JSXMemberExpressionObject::MemberExpression(it) => it.format(p),
            JSXMemberExpressionObject::ThisExpression(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for JSXMemberExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let object_doc = self.object.format(p);
        let property_doc = self.property.format(p);
        array!(p, [object_doc, text!("."), property_doc])
    }
}

impl<'a> Format<'a> for JSXElementName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            JSXElementName::Identifier(it) => it.format(p),
            JSXElementName::IdentifierReference(it) => it.format(p),
            JSXElementName::MemberExpression(it) => it.format(p),
            JSXElementName::NamespacedName(it) => it.format(p),
            JSXElementName::ThisExpression(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for JSXNamespacedName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let namespace_doc = self.namespace.format(p);
        let property_doc = self.property.format(p);
        array!(p, [namespace_doc, text!(":"), property_doc])
    }
}

impl<'a> Format<'a> for JSXAttributeName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            JSXAttributeName::Identifier(it) => it.format(p),
            JSXAttributeName::NamespacedName(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for JSXAttribute<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);
        parts.push(self.name.format(p));

        if let Some(value) = &self.value {
            parts.push(text!("="));
            parts.push(value.format(p));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for JSXEmptyExpression {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("")
    }
}

impl<'a> Format<'a> for JSXExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            JSXExpression::EmptyExpression(it) => it.format(p),
            match_member_expression!(Self) => self.to_member_expression().format(p),
            JSXExpression::BooleanLiteral(it) => it.format(p),
            JSXExpression::NullLiteral(it) => it.format(p),
            JSXExpression::NumericLiteral(it) => it.format(p),
            JSXExpression::BigIntLiteral(it) => it.format(p),
            JSXExpression::RegExpLiteral(it) => it.format(p),
            JSXExpression::StringLiteral(it) => it.format(p),
            JSXExpression::TemplateLiteral(it) => it.format(p),
            JSXExpression::Identifier(it) => it.format(p),
            JSXExpression::MetaProperty(it) => it.format(p),
            JSXExpression::Super(it) => it.format(p),
            JSXExpression::ArrayExpression(it) => it.format(p),
            JSXExpression::ArrowFunctionExpression(it) => it.format(p),
            JSXExpression::AssignmentExpression(it) => it.format(p),
            JSXExpression::AwaitExpression(it) => it.format(p),
            JSXExpression::BinaryExpression(it) => it.format(p),
            JSXExpression::CallExpression(it) => it.format(p),
            JSXExpression::ChainExpression(it) => it.format(p),
            JSXExpression::ClassExpression(it) => it.format(p),
            JSXExpression::ConditionalExpression(it) => it.format(p),
            JSXExpression::FunctionExpression(it) => it.format(p),
            JSXExpression::ImportExpression(it) => it.format(p),
            JSXExpression::LogicalExpression(it) => it.format(p),
            JSXExpression::NewExpression(it) => it.format(p),
            JSXExpression::ObjectExpression(it) => it.format(p),
            JSXExpression::ParenthesizedExpression(it) => it.format(p),
            JSXExpression::SequenceExpression(it) => it.format(p),
            JSXExpression::TaggedTemplateExpression(it) => it.format(p),
            JSXExpression::ThisExpression(it) => it.format(p),
            JSXExpression::UnaryExpression(it) => it.format(p),
            JSXExpression::UpdateExpression(it) => it.format(p),
            JSXExpression::YieldExpression(it) => it.format(p),
            JSXExpression::PrivateInExpression(it) => it.format(p),
            JSXExpression::JSXElement(it) => it.format(p),
            JSXExpression::JSXFragment(it) => it.format(p),
            JSXExpression::TSAsExpression(it) => it.format(p),
            JSXExpression::TSSatisfiesExpression(it) => it.format(p),
            JSXExpression::TSTypeAssertion(it) => it.format(p),
            JSXExpression::TSNonNullExpression(it) => it.format(p),
            JSXExpression::TSInstantiationExpression(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for JSXExpressionContainer<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let expression_doc = self.expression.format(p);
        array!(p, [text!("{"), expression_doc, text!("}")])
    }
}

impl<'a> Format<'a> for JSXAttributeValue<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            JSXAttributeValue::Element(it) => it.format(p),
            JSXAttributeValue::ExpressionContainer(it) => it.format(p),
            JSXAttributeValue::Fragment(it) => it.format(p),
            JSXAttributeValue::StringLiteral(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for JSXSpreadAttribute<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let argument_doc = self.argument.format(p);
        array!(p, [text!("..."), argument_doc])
    }
}

impl<'a> Format<'a> for JSXAttributeItem<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            JSXAttributeItem::Attribute(it) => it.format(p),
            JSXAttributeItem::SpreadAttribute(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for JSXOpeningElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        parts.push(text!("<"));
        parts.push(self.name.format(p));

        if let Some(type_parameters) = &self.type_parameters {
            parts.push(type_parameters.format(p));
        }

        for attribute in &self.attributes {
            parts.push(text!(" "));
            parts.push(attribute.format(p));
        }

        if self.self_closing {
            parts.push(text!(" "));
            parts.push(text!("/"));
        }

        parts.push(text!(">"));

        array!(p, parts)
    }
}

impl<'a> Format<'a> for JSXClosingElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let name_doc = self.name.format(p);
        array!(p, [text!("</"), name_doc, text!(">")])
    }
}

impl<'a> Format<'a> for JSXElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        parts.push(self.opening_element.format(p));

        for child in &self.children {
            parts.push(child.format(p));
        }

        if let Some(closing_element) = &self.closing_element {
            parts.push(closing_element.format(p));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for JSXOpeningFragment {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("<>")
    }
}

impl<'a> Format<'a> for JSXClosingFragment {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("</>")
    }
}

impl<'a> Format<'a> for JSXText<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        dynamic_text!(p, self.value.as_str())
    }
}

impl<'a> Format<'a> for JSXSpreadChild<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let expression_doc = self.expression.format(p);
        array!(p, [text!("..."), expression_doc])
    }
}

impl<'a> Format<'a> for JSXChild<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            JSXChild::Element(it) => it.format(p),
            JSXChild::ExpressionContainer(it) => it.format(p),
            JSXChild::Fragment(it) => it.format(p),
            JSXChild::Spread(it) => it.format(p),
            JSXChild::Text(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for JSXFragment<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        parts.push(self.opening_fragment.format(p));

        for child in &self.children {
            parts.push(child.format(p));
        }

        parts.push(self.closing_fragment.format(p));

        array!(p, parts)
    }
}
