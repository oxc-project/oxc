use smallvec::SmallVec;

use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_ast_visit::{VisitJs, walk_js};
use oxc_ecmascript::{
    constant_evaluation::{DetermineValueType, ValueType},
    side_effects::MayHaveSideEffects,
};
use oxc_span::ContentEq;
use oxc_syntax::{scope::ScopeFlags, symbol::SymbolId};

use crate::TraverseCtx;

use super::PeepholeOptimizations;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AssignmentGroup {
    Identifiers,
    StaticMembers(StableObject),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StableObject {
    Symbol(SymbolId),
    This,
}

struct StableIdentifierVisitor<'c, 'a> {
    ctx: &'c TraverseCtx<'a>,
    cache: &'c mut StableSymbolCache,
    stable: bool,
}

#[derive(Default)]
struct StableSymbolCache {
    entries: [Option<(SymbolId, bool)>; 8],
    next: usize,
}

impl<'a> VisitJs<'a> for StableIdentifierVisitor<'_, 'a> {
    fn visit_expression(&mut self, expr: &Expression<'a>) {
        if !self.stable {
            return;
        }

        match expr {
            Expression::Identifier(ident) => self.visit_identifier_reference(ident),
            Expression::ThisExpression(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::StringLiteral(_) => {}
            Expression::BinaryExpression(binary) => {
                let coerces_operands = matches!(
                    binary.operator,
                    BinaryOperator::Equality
                        | BinaryOperator::Inequality
                        | BinaryOperator::LessThan
                        | BinaryOperator::LessEqualThan
                        | BinaryOperator::GreaterThan
                        | BinaryOperator::GreaterEqualThan
                );
                if coerces_operands
                    && [&binary.left, &binary.right].iter().any(|operand| {
                        matches!(
                            operand.value_type(self.ctx),
                            ValueType::Object | ValueType::Undetermined
                        )
                    })
                {
                    self.stable = false;
                } else {
                    walk_js::walk_expression(self, expr);
                }
            }
            Expression::TemplateLiteral(_)
            | Expression::ConditionalExpression(_)
            | Expression::LogicalExpression(_)
            | Expression::ParenthesizedExpression(_)
            | Expression::SequenceExpression(_)
            | Expression::UnaryExpression(_)
            | Expression::PrivateInExpression(_) => walk_js::walk_expression(self, expr),
            _ => self.stable = false,
        }
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        self.stable = symbol_for_identifier(ident, self.ctx)
            .is_some_and(|symbol_id| symbol_is_stable(symbol_id, self.ctx, self.cache));
    }
}

impl<'a> PeepholeOptimizations {
    /// Statement fusion creates sequence expressions after their expression-exit hook has run.
    /// Finish assignment-specific sequence rewrites at the statement-list boundary so they share
    /// fusion's follow-up pass instead of forcing another whole-program iteration.
    pub(super) fn conflate_assignments_after_statement_fusion(
        statements: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.is_tree_shake_only() || ctx.current_scope_flags().contains(ScopeFlags::DirectEval) {
            return;
        }
        for statement in statements {
            if let Statement::ExpressionStatement(expr_stmt) = statement
                && let Expression::SequenceExpression(sequence) = &mut expr_stmt.expression
                && Self::conflate_assignments(sequence, ctx)
            {
                Self::remove_sequence_expression(&mut expr_stmt.expression, ctx);
            }
        }
    }

    /// `x = value, y = value` -> `y = x = value`
    ///
    /// The outer assignment target moves before the inner assignment, so this is restricted to
    /// bound identifiers or static properties on the same stable object. The duplicated value
    /// must also be repeatable: evaluating it once must produce the same value as evaluating it
    /// twice, even when an intervening property setter can run arbitrary code.
    pub(super) fn conflate_assignments(
        sequence: &mut SequenceExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> bool {
        if sequence.expressions.len() < 2
            || ctx.current_scope_flags().contains(ScopeFlags::DirectEval)
        {
            return false;
        }

        // `symbol_is_mutated` scans a symbol's references when no `SymbolValue` cache exists
        // (notably for parameters). Memoize those queries so long assignment runs stay linear.
        let mut stable_symbol_cache = StableSymbolCache::default();

        // Keep short run lists on the stack. Large sequence expressions with more than eight
        // disjoint conflation runs spill once, instead of allocating once per transformed run.
        let mut ranges: SmallVec<[std::ops::Range<usize>; 8]> = SmallVec::new();
        let mut start = 0;
        while start + 1 < sequence.expressions.len() {
            let Some((group, rhs)) = can_start_run(
                &sequence.expressions[start],
                &sequence.expressions[start + 1],
                ctx,
                &mut stable_symbol_cache,
            ) else {
                start += 1;
                continue;
            };
            let mut end = start + 2;
            while end < sequence.expressions.len()
                && assignment_chain(&sequence.expressions[end], ctx).is_some_and(
                    |(next_group, next_rhs)| next_group == group && rhs.content_eq(next_rhs),
                )
            {
                end += 1;
            }
            ranges.push(start..end);
            start = end;
        }
        if ranges.is_empty() {
            return false;
        }

        // Move each preceding assignment into the next assignment's RHS with a swap. The duplicate
        // RHS moves into the now-dead preceding slot, so no dummy AST nodes or arena allocations are
        // needed. All dead slots are compacted out of the existing vector in one retain pass below.
        for range in &ranges {
            for index in range.start + 1..range.end {
                let (before, current_and_after) = sequence.expressions.split_at_mut(index);
                let previous = &mut before[index - 1];
                let current_rhs = assignment_chain_rhs_mut(&mut current_and_after[0]);
                std::mem::swap(previous, current_rhs);
                ctx.drop_expression(previous);
            }
        }

        let mut index = 0;
        let mut ranges = ranges.into_iter().peekable();
        sequence.expressions.retain(|_| {
            let keep =
                ranges.peek().is_none_or(|range| index < range.start || index + 1 == range.end);
            index += 1;
            if ranges.peek().is_some_and(|range| index == range.end) {
                ranges.next();
            }
            keep
        });
        true
    }

    /// Merge the equal assignments at the boundary of two sequences without flattening either
    /// arena vector. Codegen prints nested sequences flat, so leaving an eligible boundary for a
    /// reparse would make compression non-idempotent.
    pub(super) fn conflate_assignment_sequence_boundary(
        previous: &mut SequenceExpression<'a>,
        current: &mut SequenceExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> bool {
        if ctx.current_scope_flags().contains(ScopeFlags::DirectEval) {
            return false;
        }
        let Some(previous_expression) = previous.expressions.last() else { return false };
        let Some(current_expression) = current.expressions.first() else { return false };
        let mut stable_symbol_cache = StableSymbolCache::default();
        if can_start_run(previous_expression, current_expression, ctx, &mut stable_symbol_cache)
            .is_none()
        {
            return false;
        }

        let previous_expression = previous.expressions.last_mut().unwrap();
        let current_rhs = assignment_chain_rhs_mut(current.expressions.first_mut().unwrap());
        std::mem::swap(previous_expression, current_rhs);
        ctx.drop_expression(previous_expression);
        previous.expressions.pop();
        true
    }
}

fn can_start_run<'b, 'a>(
    previous: &'b Expression<'a>,
    current: &Expression<'a>,
    ctx: &TraverseCtx<'a>,
    stable_symbol_cache: &mut StableSymbolCache,
) -> Option<(AssignmentGroup, &'b Expression<'a>)> {
    let (group, rhs) = assignment_chain(previous, ctx)?;
    let (next_group, next_rhs) = assignment_chain(current, ctx)?;
    (group == next_group
        && rhs.content_eq(next_rhs)
        && assignment_group_is_stable(group, ctx, stable_symbol_cache)
        && rhs_is_repeatable(rhs, ctx, stable_symbol_cache))
    .then_some((group, rhs))
}

fn assignment_chain<'b, 'a>(
    expression: &'b Expression<'a>,
    ctx: &TraverseCtx<'a>,
) -> Option<(AssignmentGroup, &'b Expression<'a>)> {
    let Expression::AssignmentExpression(assignment) = expression else { return None };
    if assignment.operator != AssignmentOperator::Assign {
        return None;
    }
    let group = assignment_group(&assignment.left, ctx)?;
    let mut rhs = &assignment.right;
    while let Expression::AssignmentExpression(assignment) = rhs {
        if assignment.operator != AssignmentOperator::Assign {
            return None;
        }
        let next_group = assignment_group(&assignment.left, ctx)?;
        if group != next_group {
            return None;
        }
        rhs = &assignment.right;
    }
    Some((group, rhs))
}

fn assignment_chain_rhs_mut<'b, 'a>(expression: &'b mut Expression<'a>) -> &'b mut Expression<'a> {
    if matches!(expression, Expression::AssignmentExpression(assignment) if assignment.operator == AssignmentOperator::Assign)
    {
        let Expression::AssignmentExpression(assignment) = expression else { unreachable!() };
        return assignment_chain_rhs_mut(&mut assignment.right);
    }
    expression
}

fn assignment_group<'a>(
    target: &AssignmentTarget<'a>,
    ctx: &TraverseCtx<'a>,
) -> Option<AssignmentGroup> {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(ident) => {
            symbol_for_identifier(ident, ctx).map(|_| AssignmentGroup::Identifiers)
        }
        AssignmentTarget::StaticMemberExpression(member) => {
            stable_object(&member.object, ctx).map(AssignmentGroup::StaticMembers)
        }
        _ => None,
    }
}

fn stable_object<'a>(object: &Expression<'a>, ctx: &TraverseCtx<'a>) -> Option<StableObject> {
    match object {
        Expression::Identifier(ident) => {
            symbol_for_identifier(ident, ctx).map(StableObject::Symbol)
        }
        Expression::ThisExpression(_)
            if !PeepholeOptimizations::member_part_blocks_reorder(object, ctx) =>
        {
            Some(StableObject::This)
        }
        _ => None,
    }
}

fn assignment_group_is_stable(
    group: AssignmentGroup,
    ctx: &TraverseCtx<'_>,
    stable_symbol_cache: &mut StableSymbolCache,
) -> bool {
    match group {
        AssignmentGroup::Identifiers | AssignmentGroup::StaticMembers(StableObject::This) => true,
        AssignmentGroup::StaticMembers(StableObject::Symbol(symbol_id)) => {
            symbol_is_stable(symbol_id, ctx, stable_symbol_cache)
        }
    }
}

fn rhs_is_repeatable<'a>(
    expression: &Expression<'a>,
    ctx: &TraverseCtx<'a>,
    stable_symbol_cache: &mut StableSymbolCache,
) -> bool {
    if expression.may_have_side_effects(ctx) {
        return false;
    }

    match expression {
        Expression::Identifier(ident) => symbol_for_identifier(ident, ctx)
            .is_some_and(|symbol_id| symbol_is_stable(symbol_id, ctx, stable_symbol_cache)),
        Expression::ThisExpression(_) => true,
        _ => {
            if matches!(expression.value_type(ctx), ValueType::Object | ValueType::Undetermined) {
                return false;
            }

            let mut visitor =
                StableIdentifierVisitor { ctx, cache: stable_symbol_cache, stable: true };
            visitor.visit_expression(expression);
            visitor.stable
        }
    }
}

fn symbol_for_identifier(
    ident: &IdentifierReference<'_>,
    ctx: &TraverseCtx<'_>,
) -> Option<SymbolId> {
    ctx.scoping().get_reference(ident.reference_id()).symbol_id()
}

fn symbol_is_stable(
    symbol_id: SymbolId,
    ctx: &TraverseCtx<'_>,
    cache: &mut StableSymbolCache,
) -> bool {
    if let Some(stable) = cache
        .entries
        .iter()
        .flatten()
        .find_map(|&(cached_id, stable)| (cached_id == symbol_id).then_some(stable))
    {
        return stable;
    }
    let scoping = ctx.scoping();
    let symbol_flags = scoping.symbol_flags(symbol_id);
    let symbol_scope_id = scoping.symbol_scope_id(symbol_id);
    let scope_flags = scoping.scope_flags(symbol_scope_id);
    // In sloppy functions, parameters may be rebound through the mapped `arguments` object.
    // Symbols do not currently distinguish parameters from `var` declarations, so reject all
    // function-scoped variables conservatively.
    let may_be_mapped_parameter = symbol_flags.is_function_scoped_declaration()
        && !symbol_flags.is_catch_variable()
        && !scope_flags.is_strict_mode();
    // Even a sole `var` initializer can run after a closure's first read: a
    // setter may resume the enclosing generator while it is suspended before
    // that initializer. Write references do not account for initialization.
    let may_have_delayed_initializer = symbol_flags.is_function_scoped_declaration()
        && !symbol_flags.is_catch_variable()
        && PeepholeOptimizations::read_crosses_function_boundary(
            ctx.current_scope_id(),
            symbol_scope_id,
            ctx,
        );
    // Later declaration initializers are not write references. A setter can
    // resume a generator and execute one between the original evaluations.
    let has_multiple_value_declarations = scoping
        .symbol_redeclarations(symbol_id)
        .iter()
        .filter(|declaration| declaration.flags.is_value())
        .nth(1)
        .is_some();
    let stable = !(scope_flags.contains(ScopeFlags::DirectEval)
        || may_be_mapped_parameter
        || may_have_delayed_initializer
        || has_multiple_value_declarations
        || PeepholeOptimizations::symbol_value_may_change(symbol_id, ctx));
    cache.entries[cache.next] = Some((symbol_id, stable));
    cache.next = (cache.next + 1) % cache.entries.len();
    stable
}
