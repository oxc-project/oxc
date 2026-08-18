use oxc_allocator::ReplaceWith;
use oxc_ast::ast::*;
use oxc_syntax::identifier::is_identifier_name_patched;

use crate::TraverseCtx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// Converts property accesses from quoted string or bracket access syntax to dot or unquoted string
    /// syntax, where possible. Dot syntax is more compact.
    ///
    /// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/ConvertToDottedProperties.java>
    ///
    /// `foo['bar']` -> `foo.bar`
    /// `foo?.['bar']` -> `foo?.bar`
    pub fn convert_to_dotted_properties(
        expr: &mut MemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let MemberExpression::ComputedMemberExpression(e) = expr else { return };
        let Expression::StringLiteral(s) = &e.expression else { return };
        if is_identifier_name_patched(&s.value) {
            let property = IdentifierName::new(s.span, s.value, ctx);
            expr.replace_with(|expr| {
                let MemberExpression::ComputedMemberExpression(e) = expr else { unreachable!() };
                let ComputedMemberExpression { span, object, optional, .. } = e.unbox();
                MemberExpression::StaticMemberExpression(StaticMemberExpression::boxed(
                    span, object, property, optional, ctx,
                ))
            });
            ctx.notice_change();
            return;
        }
        let v = s.value.as_str();
        if e.optional {
            return;
        }
        if let Some(n) = TraverseCtx::string_to_equivalent_number_value(v) {
            let new_expr =
                Expression::new_numeric_literal(s.span, n, None, NumberBase::Decimal, ctx);
            ctx.replace_expression(&mut e.expression, new_expr);
        }
    }
}
