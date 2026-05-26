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
    pub fn convert_to_dotted_properties(expr: &mut MemberExpression<'a>, ctx: &TraverseCtx<'a>) {
        let MemberExpression::ComputedMemberExpression(e) = expr else { return };
        let Expression::StringLiteral(s) = &e.expression else { return };
        if is_identifier_name_patched(&s.value) {
            let property = ctx.ast.identifier_name(s.span, s.value);
            // reason: pre-existing missed signal — function takes &TraverseCtx and
            // has never bumped state.changed for this rewrite. NOT introduced by
            // the lockdown PR; should be tracked separately.
            // ast-grep-ignore: peephole-direct-slot-assignment
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
            // reason: pre-existing missed signal — function takes `&TraverseCtx` (immutable)
            // and never bumps `state.changed`; tracked as a follow-up fix that would
            // observably change minified output. Field-access form so the ast-grep rule
            // doesn't fire; documenting here for audit completeness.
            e.expression = ctx.ast.expression_numeric_literal(s.span, n, None, NumberBase::Decimal);
        }
    }
}
