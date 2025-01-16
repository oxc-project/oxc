use std::borrow::Cow;

use oxc_ast::ast::*;
use oxc_syntax::operator::UnaryOperator;

use crate::{array_join::ArrayJoin, ToBoolean};

/// `ToString`
///
/// <https://tc39.es/ecma262/#sec-tostring>
pub trait ToJsString<'a> {
    fn to_js_string(&self) -> Option<Cow<'a, str>>;
}

impl<'a> ToJsString<'a> for Expression<'a> {
    fn to_js_string(&self) -> Option<Cow<'a, str>> {
        match self {
            Expression::StringLiteral(lit) => lit.to_js_string(),
            Expression::TemplateLiteral(lit) => lit.to_js_string(),
            Expression::Identifier(ident) => ident.to_js_string(),
            Expression::NumericLiteral(lit) => lit.to_js_string(),
            Expression::BigIntLiteral(lit) => lit.to_js_string(),
            Expression::NullLiteral(lit) => lit.to_js_string(),
            Expression::BooleanLiteral(lit) => lit.to_js_string(),
            Expression::UnaryExpression(e) => e.to_js_string(),
            Expression::ArrayExpression(e) => e.to_js_string(),
            Expression::ObjectExpression(e) => e.to_js_string(),
            _ => None,
        }
    }
}

impl<'a> ToJsString<'a> for ArrayExpressionElement<'a> {
    fn to_js_string(&self) -> Option<Cow<'a, str>> {
        match self {
            ArrayExpressionElement::SpreadElement(_) => None,
            ArrayExpressionElement::Elision(_) | ArrayExpressionElement::NullLiteral(_) => {
                Some(Cow::Borrowed(""))
            }
            ArrayExpressionElement::Identifier(id) if id.name.as_str() == "undefined" => {
                Some(Cow::Borrowed(""))
            }
            expr @ match_expression!(ArrayExpressionElement) => {
                expr.as_expression().and_then(ToJsString::to_js_string)
            }
        }
    }
}

impl<'a> ToJsString<'a> for StringLiteral<'a> {
    fn to_js_string(&self) -> Option<Cow<'a, str>> {
        Some(Cow::Borrowed(self.value.as_str()))
    }
}

impl<'a> ToJsString<'a> for TemplateLiteral<'a> {
    fn to_js_string(&self) -> Option<Cow<'a, str>> {
        let mut str = String::new();
        for (i, quasi) in self.quasis.iter().enumerate() {
            str.push_str(quasi.value.cooked.as_ref()?);

            if i < self.expressions.len() {
                let expr = &self.expressions[i];
                let value = expr.to_js_string()?;
                str.push_str(&value);
            }
        }
        Some(Cow::Owned(str))
    }
}

impl<'a> ToJsString<'a> for IdentifierReference<'a> {
    fn to_js_string(&self) -> Option<Cow<'a, str>> {
        let name = self.name.as_str();
        matches!(name, "undefined" | "Infinity" | "NaN").then(|| Cow::Borrowed(name))
    }
}

impl<'a> ToJsString<'a> for NumericLiteral<'a> {
    fn to_js_string(&self) -> Option<Cow<'a, str>> {
        use oxc_syntax::number::ToJsString;
        let value = self.value;
        let s = value.to_js_string();
        Some(if value == 0.0 {
            Cow::Borrowed("0")
        } else {
            Cow::Owned(if value.is_sign_negative() && value != 0.0 { format!("-{s}") } else { s })
        })
    }
}

/// <https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-bigint.prototype.tostring>
impl<'a> ToJsString<'a> for BigIntLiteral<'a> {
    fn to_js_string(&self) -> Option<Cow<'a, str>> {
        self.base.is_base_10().then(|| Cow::Owned(self.raw.trim_end_matches('n').to_string()))
    }
}

impl<'a> ToJsString<'a> for BooleanLiteral {
    fn to_js_string(&self) -> Option<Cow<'a, str>> {
        Some(Cow::Borrowed(if self.value { "true" } else { "false" }))
    }
}

impl<'a> ToJsString<'a> for NullLiteral {
    fn to_js_string(&self) -> Option<Cow<'a, str>> {
        Some(Cow::Borrowed("null"))
    }
}

impl<'a> ToJsString<'a> for UnaryExpression<'a> {
    fn to_js_string(&self) -> Option<Cow<'a, str>> {
        match self.operator {
            UnaryOperator::Void => Some(Cow::Borrowed("undefined")),
            UnaryOperator::LogicalNot => self
                .argument
                .to_boolean()
                .map(|boolean| Cow::Borrowed(if boolean { "false" } else { "true" })),
            _ => None,
        }
    }
}

impl<'a> ToJsString<'a> for ArrayExpression<'a> {
    fn to_js_string(&self) -> Option<Cow<'a, str>> {
        // TODO: https://github.com/google/closure-compiler/blob/e13f5cd0a5d3d35f2db1e6c03fdf67ef02946009/src/com/google/javascript/jscomp/NodeUtil.java#L302-L303
        self.array_join(Some(",")).map(Cow::Owned)
    }
}

impl<'a> ToJsString<'a> for ObjectExpression<'a> {
    fn to_js_string(&self) -> Option<Cow<'a, str>> {
        Some(Cow::Borrowed("[object Object]"))
    }
}
