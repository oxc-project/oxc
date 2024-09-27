//! [JSX](https://facebook.github.io/jsx)

use std::fmt;

use oxc_span::{Atom, Span};

use crate::ast::*;

#[cfg(feature = "serialize")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type JSXElementName = JSXIdentifier | JSXNamespacedName | JSXMemberExpression;
export type JSXMemberExpressionObject = JSXIdentifier | JSXMemberExpression;
"#;

// 1.2 JSX Elements

impl<'a> JSXIdentifier<'a> {
    pub fn new(span: Span, name: Atom<'a>) -> Self {
        Self { span, name }
    }
}

impl<'a> fmt::Display for JSXIdentifier<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)
    }
}

impl<'a> fmt::Display for JSXNamespacedName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace.name, self.property.name)
    }
}

impl<'a> JSXElementName<'a> {
    pub fn get_identifier(&self) -> Option<&IdentifierReference<'a>> {
        match self {
            JSXElementName::Identifier(_)
            | JSXElementName::NamespacedName(_)
            | JSXElementName::ThisExpression(_) => None,
            JSXElementName::IdentifierReference(ident) => Some(ident),
            JSXElementName::MemberExpression(member_expr) => member_expr.get_identifier(),
        }
    }

    pub fn get_identifier_name(&self) -> Option<Atom<'a>> {
        match self {
            Self::Identifier(id) => Some(id.as_ref().name.clone()),
            Self::IdentifierReference(id) => Some(id.as_ref().name.clone()),
            _ => None,
        }
    }
}

impl<'a> JSXMemberExpression<'a> {
    pub fn get_identifier(&self) -> Option<&IdentifierReference<'a>> {
        self.object.get_identifier()
    }
}

impl<'a> JSXMemberExpressionObject<'a> {
    pub fn get_identifier(&self) -> Option<&IdentifierReference<'a>> {
        let mut object = self;
        loop {
            match object {
                JSXMemberExpressionObject::IdentifierReference(ident) => return Some(ident),
                JSXMemberExpressionObject::MemberExpression(member_expr) => {
                    object = &member_expr.object;
                }
                JSXMemberExpressionObject::ThisExpression(_) => return None,
            }
        }
    }
}

impl<'a> fmt::Display for JSXMemberExpression<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.object, self.property)
    }
}

impl<'a> fmt::Display for JSXMemberExpressionObject<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdentifierReference(id) => id.fmt(f),
            Self::MemberExpression(expr) => expr.fmt(f),
            Self::ThisExpression(_) => "this".fmt(f),
        }
    }
}

impl<'a> fmt::Display for JSXElementName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identifier(ident) => ident.fmt(f),
            Self::IdentifierReference(ident) => ident.fmt(f),
            Self::NamespacedName(namespaced) => namespaced.fmt(f),
            Self::MemberExpression(member_expr) => member_expr.fmt(f),
            Self::ThisExpression(_) => "this".fmt(f),
        }
    }
}

impl<'a> JSXExpression<'a> {
    /// Determines whether the given expr is a `undefined` literal
    pub fn is_undefined(&self) -> bool {
        matches!(self, Self::Identifier(ident) if ident.name == "undefined")
    }
}

impl<'a> JSXAttribute<'a> {
    pub fn is_identifier(&self, name: &str) -> bool {
        matches!(&self.name, JSXAttributeName::Identifier(ident) if ident.name == name)
    }

    pub fn is_identifier_ignore_case(&self, name: &str) -> bool {
        matches!(&self.name, JSXAttributeName::Identifier(ident) if ident.name.eq_ignore_ascii_case(name))
    }

    pub fn is_key(&self) -> bool {
        self.is_identifier("key")
    }
}

impl<'a> JSXAttributeName<'a> {
    pub fn as_identifier(&self) -> Option<&JSXIdentifier<'a>> {
        match self {
            Self::Identifier(ident) => Some(ident.as_ref()),
            Self::NamespacedName(_) => None,
        }
    }

    pub fn get_identifier(&self) -> &JSXIdentifier<'a> {
        match self {
            Self::Identifier(ident) => ident.as_ref(),
            Self::NamespacedName(namespaced) => &namespaced.property,
        }
    }
}
impl<'a> JSXAttributeValue<'a> {
    pub fn as_string_literal(&self) -> Option<&StringLiteral<'a>> {
        match self {
            Self::StringLiteral(lit) => Some(lit.as_ref()),
            _ => None,
        }
    }
}

impl<'a> JSXAttributeItem<'a> {
    /// Get the contained [`JSXAttribute`] if it is an attribute item, otherwise
    /// returns [`None`].
    ///
    /// This is the inverse of [`JSXAttributeItem::as_spread`].
    pub fn as_attribute(&self) -> Option<&JSXAttribute<'a>> {
        match self {
            Self::Attribute(attr) => Some(attr),
            Self::SpreadAttribute(_) => None,
        }
    }

    /// Get the contained [`JSXSpreadAttribute`] if it is a spread attribute item,
    /// otherwise returns [`None`].
    ///
    /// This is the inverse of [`JSXAttributeItem::as_attribute`].
    pub fn as_spread(&self) -> Option<&JSXSpreadAttribute<'a>> {
        match self {
            Self::Attribute(_) => None,
            Self::SpreadAttribute(spread) => Some(spread),
        }
    }
}

impl<'a> JSXChild<'a> {
    pub const fn is_expression_container(&self) -> bool {
        matches!(self, Self::ExpressionContainer(_))
    }
}
