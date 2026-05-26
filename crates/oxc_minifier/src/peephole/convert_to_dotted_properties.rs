use oxc_allocator::TakeIn;
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
            let property = ctx.ast.identifier_name(s.span, s.value);
            let new_member = ctx.ast.alloc_static_member_expression(
                e.span,
                e.object.take_in(ctx.ast),
                property,
                e.optional,
            );
            // No typed helper exists for the `MemberExpression` enum slot;
            // the direct write is followed by `notice_change()` so the
            // mutation signal is preserved.
            // reason: no typed helper for `MemberExpression` enum slot; sibling `notice_change()` preserves signal
            // ast-grep-ignore: peephole-direct-slot-assignment
            *expr = MemberExpression::StaticMemberExpression(new_member);
            ctx.notice_change();
            return;
        }
        let v = s.value.as_str();
        if e.optional {
            return;
        }
        if let Some(n) = TraverseCtx::string_to_equivalent_number_value(v) {
            let new_expr = ctx.ast.expression_numeric_literal(s.span, n, None, NumberBase::Decimal);
            ctx.replace_expression(&mut e.expression, new_expr);
        }
    }
}
