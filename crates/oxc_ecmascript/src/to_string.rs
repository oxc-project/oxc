use std::borrow::Cow;

use oxc_ast::ast::*;
use oxc_syntax::operator::UnaryOperator;

use crate::{
    GlobalContext, ToBoolean,
    array_join::ArrayJoin,
    constant_evaluation::{DetermineValueType, ValueType},
    to_primitive::maybe_object_with_to_primitive_related_properties_overridden,
};

/// `ToString`
///
/// <https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tostring>
pub trait ToJsString<'a> {
    fn to_js_string(&self, ctx: &impl GlobalContext<'a>) -> Option<Cow<'a, str>>;
}

impl<'a> ToJsString<'a> for Expression<'a> {
    fn to_js_string(&self, ctx: &impl GlobalContext<'a>) -> Option<Cow<'a, str>> {
        match self {
            Expression::StringLiteral(lit) => lit.to_js_string(ctx),
            Expression::TemplateLiteral(lit) => lit.to_js_string(ctx),
            Expression::Identifier(ident) => ident.to_js_string(ctx),
            Expression::NumericLiteral(lit) => lit.to_js_string(ctx),
            Expression::BigIntLiteral(lit) => lit.to_js_string(ctx),
            Expression::NullLiteral(lit) => lit.to_js_string(ctx),
            Expression::BooleanLiteral(lit) => lit.to_js_string(ctx),
            Expression::UnaryExpression(e) => e.to_js_string(ctx),
            Expression::ArrayExpression(e) => e.to_js_string(ctx),
            Expression::ObjectExpression(e) => e.to_js_string(ctx),
            Expression::RegExpLiteral(e) => e.to_js_string(ctx),
            _ => None,
        }
    }
}

impl<'a> ToJsString<'a> for ArrayExpressionElement<'a> {
    fn to_js_string(&self, ctx: &impl GlobalContext<'a>) -> Option<Cow<'a, str>> {
        match self {
            ArrayExpressionElement::SpreadElement(_) => None,
            ArrayExpressionElement::Elision(_) => Some(Cow::Borrowed("")),
            expr @ match_expression!(ArrayExpressionElement) => {
                let expr = expr.as_expression()?;
                match expr.value_type(ctx) {
                    ValueType::Undefined | ValueType::Null => Some(Cow::Borrowed("")),
                    ValueType::Undetermined => None,
                    _ => expr.to_js_string(ctx),
                }
            }
        }
    }
}

impl<'a> ToJsString<'a> for StringLiteral<'a> {
    fn to_js_string(&self, _ctx: &impl GlobalContext<'a>) -> Option<Cow<'a, str>> {
        Some(Cow::Borrowed(self.value.as_str()))
    }
}

impl<'a> ToJsString<'a> for TemplateLiteral<'a> {
    fn to_js_string(&self, ctx: &impl GlobalContext<'a>) -> Option<Cow<'a, str>> {
        let mut str = String::new();
        for (i, quasi) in self.quasis.iter().enumerate() {
            str.push_str(quasi.value.cooked.as_ref()?);

            if i < self.expressions.len() {
                let expr = &self.expressions[i];
                let value = expr.to_js_string(ctx)?;
                str.push_str(&value);
            }
        }
        Some(Cow::Owned(str))
    }
}

impl<'a> ToJsString<'a> for IdentifierReference<'a> {
    fn to_js_string(&self, ctx: &impl GlobalContext<'a>) -> Option<Cow<'a, str>> {
        let name = self.name.as_str();
        (matches!(name, "undefined" | "Infinity" | "NaN")
            && ctx.is_global_reference(self) == Some(true))
        .then_some(Cow::Borrowed(name))
    }
}

impl<'a> ToJsString<'a> for NumericLiteral<'a> {
    fn to_js_string(&self, _ctx: &impl GlobalContext<'a>) -> Option<Cow<'a, str>> {
        use oxc_syntax::number::ToJsString;
        let value = self.value;
        Some(if value == 0.0 { Cow::Borrowed("0") } else { Cow::Owned(value.to_js_string()) })
    }
}

/// <https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-bigint.prototype.tostring>
impl<'a> ToJsString<'a> for BigIntLiteral<'a> {
    fn to_js_string(&self, _ctx: &impl GlobalContext<'a>) -> Option<Cow<'a, str>> {
        Some(Cow::Borrowed(self.value.as_str()))
    }
}

impl<'a> ToJsString<'a> for BooleanLiteral {
    fn to_js_string(&self, _ctx: &impl GlobalContext<'a>) -> Option<Cow<'a, str>> {
        Some(Cow::Borrowed(if self.value { "true" } else { "false" }))
    }
}

impl<'a> ToJsString<'a> for NullLiteral {
    fn to_js_string(&self, _ctx: &impl GlobalContext<'a>) -> Option<Cow<'a, str>> {
        Some(Cow::Borrowed("null"))
    }
}

impl<'a> ToJsString<'a> for UnaryExpression<'a> {
    fn to_js_string(&self, ctx: &impl GlobalContext<'a>) -> Option<Cow<'a, str>> {
        match self.operator {
            UnaryOperator::Void => Some(Cow::Borrowed("undefined")),
            UnaryOperator::LogicalNot => self
                .argument
                .to_boolean(ctx)
                .map(|boolean| Cow::Borrowed(if boolean { "false" } else { "true" })),
            _ => None,
        }
    }
}

impl<'a> ToJsString<'a> for ArrayExpression<'a> {
    fn to_js_string(&self, ctx: &impl GlobalContext<'a>) -> Option<Cow<'a, str>> {
        self.array_join(ctx, Some(",")).map(Cow::Owned)
    }
}

impl<'a> ToJsString<'a> for ObjectExpression<'a> {
    fn to_js_string(&self, _ctx: &impl GlobalContext<'a>) -> Option<Cow<'a, str>> {
        if maybe_object_with_to_primitive_related_properties_overridden(self) {
            None
        } else {
            Some(Cow::Borrowed("[object Object]"))
        }
    }
}

impl<'a> ToJsString<'a> for RegExpLiteral<'a> {
    fn to_js_string(&self, _ctx: &impl GlobalContext<'a>) -> Option<Cow<'a, str>> {
        if let Some(raw) = self.raw.as_ref() {
            Some(Cow::Borrowed(raw.as_str()))
        } else {
            Some(Cow::Owned(self.regex.to_string()))
        }
    }
}
