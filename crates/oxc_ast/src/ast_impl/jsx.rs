//! [JSX](https://facebook.github.io/jsx)

use crate::ast::*;
use oxc_span::{Atom, Span};

// 1.2 JSX Elements

impl<'a> std::fmt::Display for JSXNamespacedName<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace.name, self.property.name)
    }
}

impl<'a> JSXMemberExpression<'a> {
    pub fn get_object_identifier(&self) -> &JSXIdentifier {
        let mut member_expr = self;
        loop {
            match &member_expr.object {
                JSXMemberExpressionObject::Identifier(ident) => {
                    break ident;
                }
                JSXMemberExpressionObject::MemberExpression(expr) => {
                    member_expr = expr;
                }
            }
        }
    }

    pub fn get_object_identifier_mut(&mut self) -> &mut JSXIdentifier<'a> {
        let mut member_expr = self;
        loop {
            match &mut member_expr.object {
                JSXMemberExpressionObject::Identifier(ident) => {
                    break &mut *ident;
                }
                JSXMemberExpressionObject::MemberExpression(expr) => {
                    member_expr = expr;
                }
            }
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

    pub fn is_key(&self) -> bool {
        self.is_identifier("key")
    }
}

impl<'a> JSXIdentifier<'a> {
    pub fn new(span: Span, name: Atom<'a>) -> Self {
        Self { span, name }
    }
}
