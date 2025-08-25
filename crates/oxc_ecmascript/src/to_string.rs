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
    fn to_js_string(
        &self,
        ctx: &impl GlobalContext<'a>,
    ) -> Option<(Cow<'a, str>, /* lone_surrogates */ bool)>;
}

impl<'a> ToJsString<'a> for Expression<'a> {
    fn to_js_string(
        &self,
        ctx: &impl GlobalContext<'a>,
    ) -> Option<(Cow<'a, str>, /* lone_surrogates */ bool)> {
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
    fn to_js_string(
        &self,
        ctx: &impl GlobalContext<'a>,
    ) -> Option<(Cow<'a, str>, /* lone_surrogates */ bool)> {
        match self {
            ArrayExpressionElement::SpreadElement(_) => None,
            ArrayExpressionElement::Elision(_) => Some((Cow::Borrowed(""), false)),
            expr @ match_expression!(ArrayExpressionElement) => {
                let expr = expr.as_expression()?;
                match expr.value_type(ctx) {
                    ValueType::Undefined | ValueType::Null => Some((Cow::Borrowed(""), false)),
                    ValueType::Undetermined => None,
                    _ => expr.to_js_string(ctx),
                }
            }
        }
    }
}

impl<'a> ToJsString<'a> for StringLiteral<'a> {
    fn to_js_string(
        &self,
        _ctx: &impl GlobalContext<'a>,
    ) -> Option<(Cow<'a, str>, /* lone_surrogates */ bool)> {
        Some((Cow::Borrowed(self.value.as_str()), self.lone_surrogates))
    }
}

impl<'a> ToJsString<'a> for TemplateLiteral<'a> {
    fn to_js_string(
        &self,
        ctx: &impl GlobalContext<'a>,
    ) -> Option<(Cow<'a, str>, /* lone_surrogates */ bool)> {
        let mut str = String::new();
        let mut lone_surrogates = false;
        for (i, quasi) in self.quasis.iter().enumerate() {
            str.push_str(quasi.value.cooked.as_ref()?);
            lone_surrogates |= quasi.lone_surrogates;

            if i < self.expressions.len() {
                let expr = &self.expressions[i];
                let (value, ls) = expr.to_js_string(ctx)?;
                lone_surrogates |= ls;
                str.push_str(&value);
            }
        }
        Some((Cow::Owned(str), lone_surrogates))
    }
}

impl<'a> ToJsString<'a> for IdentifierReference<'a> {
    fn to_js_string(
        &self,
        ctx: &impl GlobalContext<'a>,
    ) -> Option<(Cow<'a, str>, /* lone_surrogates */ bool)> {
        let name = self.name.as_str();
        (matches!(name, "undefined" | "Infinity" | "NaN") && ctx.is_global_reference(self))
            .then_some((Cow::Borrowed(name), false))
    }
}

impl<'a> ToJsString<'a> for NumericLiteral<'a> {
    fn to_js_string(
        &self,
        _ctx: &impl GlobalContext<'a>,
    ) -> Option<(Cow<'a, str>, /* lone_surrogates */ bool)> {
        use oxc_syntax::number::ToJsString;
        let value = self.value;
        Some(if value == 0.0 {
            (Cow::Borrowed("0"), false)
        } else {
            (Cow::Owned(value.to_js_string()), false)
        })
    }
}

/// <https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-bigint.prototype.tostring>
impl<'a> ToJsString<'a> for BigIntLiteral<'a> {
    fn to_js_string(
        &self,
        _ctx: &impl GlobalContext<'a>,
    ) -> Option<(Cow<'a, str>, /* lone_surrogates */ bool)> {
        Some((Cow::Borrowed(self.value.as_str()), false))
    }
}

impl<'a> ToJsString<'a> for BooleanLiteral {
    fn to_js_string(
        &self,
        _ctx: &impl GlobalContext<'a>,
    ) -> Option<(Cow<'a, str>, /* lone_surrogates */ bool)> {
        Some((Cow::Borrowed(if self.value { "true" } else { "false" }), false))
    }
}

impl<'a> ToJsString<'a> for NullLiteral {
    fn to_js_string(
        &self,
        _ctx: &impl GlobalContext<'a>,
    ) -> Option<(Cow<'a, str>, /* lone_surrogates */ bool)> {
        Some((Cow::Borrowed("null"), false))
    }
}

impl<'a> ToJsString<'a> for UnaryExpression<'a> {
    fn to_js_string(
        &self,
        ctx: &impl GlobalContext<'a>,
    ) -> Option<(Cow<'a, str>, /* lone_surrogates */ bool)> {
        match self.operator {
            UnaryOperator::Void => Some((Cow::Borrowed("undefined"), false)),
            UnaryOperator::LogicalNot => self
                .argument
                .to_boolean(ctx)
                .map(|boolean| (Cow::Borrowed(if boolean { "false" } else { "true" }), false)),
            _ => None,
        }
    }
}

impl<'a> ToJsString<'a> for ArrayExpression<'a> {
    fn to_js_string(
        &self,
        ctx: &impl GlobalContext<'a>,
    ) -> Option<(Cow<'a, str>, /* lone_surrogates */ bool)> {
        self.array_join(ctx, Some(",")).map(|value| (Cow::Owned(value), false))
    }
}

impl<'a> ToJsString<'a> for ObjectExpression<'a> {
    fn to_js_string(
        &self,
        _ctx: &impl GlobalContext<'a>,
    ) -> Option<(Cow<'a, str>, /* lone_surrogates */ bool)> {
        if maybe_object_with_to_primitive_related_properties_overridden(self) {
            None
        } else {
            Some((Cow::Borrowed("[object Object]"), false))
        }
    }
}

impl<'a> ToJsString<'a> for RegExpLiteral<'a> {
    fn to_js_string(
        &self,
        _ctx: &impl GlobalContext<'a>,
    ) -> Option<(Cow<'a, str>, /* lone_surrogates */ bool)> {
        if let Some(raw) = self.raw.as_ref() {
            Some((Cow::Borrowed(raw.as_str()), false))
        } else {
            Some((Cow::Owned(self.regex.to_string()), false))
        }
    }
}
