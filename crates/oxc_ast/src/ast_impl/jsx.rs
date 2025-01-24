//! [JSX](https://facebook.github.io/jsx)
use std::fmt;

use oxc_span::Atom;

use crate::ast::*;

// 1.2 JSX Elements

impl fmt::Display for JSXIdentifier<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)
    }
}

impl fmt::Display for JSXNamespacedName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace.name, self.property.name)
    }
}

impl<'a> JSXElementName<'a> {
    /// Get this name's contained identifier reference, returning [`None`] if it
    /// is some other variant. Note that [namespaced
    /// identifiers](JSXElementName::NamespacedName) are not included.
    pub fn get_identifier(&self) -> Option<&IdentifierReference<'a>> {
        match self {
            JSXElementName::Identifier(_)
            | JSXElementName::NamespacedName(_)
            | JSXElementName::ThisExpression(_) => None,
            JSXElementName::IdentifierReference(ident) => Some(ident),
            JSXElementName::MemberExpression(member_expr) => member_expr.get_identifier(),
        }
    }

    #[allow(missing_docs)]
    pub fn get_identifier_name(&self) -> Option<Atom<'a>> {
        match self {
            Self::Identifier(id) => Some(id.as_ref().name),
            Self::IdentifierReference(id) => Some(id.as_ref().name),
            _ => None,
        }
    }
}

impl<'a> JSXMemberExpression<'a> {
    /// Get the identifier being referenced, if there is one. Will return
    /// [`None`] for `this` expressions or if semantic analysis was skipped.
    pub fn get_identifier(&self) -> Option<&IdentifierReference<'a>> {
        self.object.get_identifier()
    }
}

impl<'a> JSXMemberExpressionObject<'a> {
    /// Get the identifier being referenced, if there is one. Will return
    /// [`None`] for [`this`](JSXMemberExpressionObject::ThisExpression)
    /// expressions or if semantic analysis was skipped.
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

impl fmt::Display for JSXMemberExpression<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.object, self.property)
    }
}

impl fmt::Display for JSXMemberExpressionObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdentifierReference(id) => id.fmt(f),
            Self::MemberExpression(expr) => expr.fmt(f),
            Self::ThisExpression(_) => "this".fmt(f),
        }
    }
}

impl fmt::Display for JSXElementName<'_> {
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

impl JSXExpression<'_> {
    /// Determines whether the given expr is a `undefined` literal
    pub fn is_undefined(&self) -> bool {
        matches!(self, Self::Identifier(ident) if ident.name == "undefined")
    }
}

impl JSXAttribute<'_> {
    /// Returns `true` if this attribute's name is the expected `name`.
    ///
    /// Use [`JSXAttribute::is_identifier_ignore_case`] if you want to ignore
    /// upper/lower case differences.
    pub fn is_identifier(&self, name: &str) -> bool {
        matches!(&self.name, JSXAttributeName::Identifier(ident) if ident.name == name)
    }

    /// Returns `true` if this attribute's name is the expected `name`, ignoring
    /// casing.
    pub fn is_identifier_ignore_case(&self, name: &str) -> bool {
        matches!(&self.name, JSXAttributeName::Identifier(ident) if ident.name.eq_ignore_ascii_case(name))
    }

    /// Returns `true` if this is a React `key`.
    ///
    /// ## Example
    /// ```tsx
    /// <Foo key="value" /> // -> `true`
    /// <Foo bar="value" /> // -> `false`
    /// ```
    pub fn is_key(&self) -> bool {
        self.is_identifier("key")
    }
}

impl<'a> JSXAttributeName<'a> {
    /// Try to convert this attribute name into an [identifier](JSXIdentifier).
    /// Returns [`None`] for [namespaced names](JSXAttributeName::NamespacedName).
    pub fn as_identifier(&self) -> Option<&JSXIdentifier<'a>> {
        match self {
            Self::Identifier(ident) => Some(ident.as_ref()),
            Self::NamespacedName(_) => None,
        }
    }

    /// Get the rightmost identifier in the attribute name.
    ///
    /// ## Example
    /// ```tsx
    /// <Foo /> // -> `Foo`
    /// <Foo.Bar /> // -> `Bar`
    /// <Foo.Bar.Baz /> // -> `Baz`
    /// ```
    pub fn get_identifier(&self) -> &JSXIdentifier<'a> {
        match self {
            Self::Identifier(ident) => ident.as_ref(),
            Self::NamespacedName(namespaced) => &namespaced.property,
        }
    }
}
impl<'a> JSXAttributeValue<'a> {
    /// Get the contained [`StringLiteral`], or [`None`] if this is some other
    /// kind of value.
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

impl JSXChild<'_> {
    /// Returns `true` if this an [expression container](JSXChild::ExpressionContainer).
    pub const fn is_expression_container(&self) -> bool {
        matches!(self, Self::ExpressionContainer(_))
    }
}
