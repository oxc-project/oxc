use oxc_allocator::TakeIn;
use oxc_ast::ast::*;

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
    pub fn convert_to_dotted_properties(expr: &mut MemberExpression<'a>, ctx: &TraverseCtx<'a>) {
        let MemberExpression::ComputedMemberExpression(e) = expr else { return };
        let Expression::StringLiteral(s) = &e.expression else { return };
        if TraverseCtx::is_identifier_name_patched(&s.value) {
            let property = ctx.ast.identifier_name(s.span, s.value);
            *expr =
                MemberExpression::StaticMemberExpression(ctx.ast.alloc_static_member_expression(
                    e.span,
                    e.object.take_in(ctx.ast),
                    property,
                    e.optional,
                ));
            return;
        }
        let v = s.value.as_str();
        if e.optional {
            return;
        }
        if let Some(n) = TraverseCtx::string_to_equivalent_number_value(v) {
            e.expression = ctx.ast.expression_numeric_literal(s.span, n, None, NumberBase::Decimal);
        }
    }
}
