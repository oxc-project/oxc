use oxc_ast::ast::MemberExpression;

/// Trait for JSX state.
///
/// Used in TS style for tracking when to emit JSX identifiers.
///
/// Implemented by [`JSXStateJS`] and [`JSXStateTS`].
/// In JS style, there is no state to track, so [`JSXStateJS`] is a ZST which implements all methods as no-ops.
/// In TS style, [`JSXStateTS`] implements all methods, because state is required.
pub trait JSXState: Default {
    /// Call when entering a JSX expression.
    fn enter_jsx_expression(&mut self);

    /// Call when exiting a JSX expression.
    fn exit_jsx_expression(&mut self);

    /// Call when entering a `MemberExpression`.
    fn enter_member_expression(&mut self, member_expr: &MemberExpression<'_>);

    /// Call when exiting a `MemberExpression`.
    fn exit_member_expression(&mut self, member_expr: &MemberExpression<'_>);

    /// Returns `true` if current `Identifier` token should be emitted as a `JSXIdentifier` token.
    fn should_emit_jsx_identifier(&self) -> bool;
}

/// No-op JSX state for JS-style tokens.
#[derive(Default)]
pub struct JSXStateJS;

impl JSXState for JSXStateJS {
    #[inline(always)]
    fn enter_jsx_expression(&mut self) {}

    #[inline(always)]
    fn exit_jsx_expression(&mut self) {}

    #[inline(always)]
    fn enter_member_expression(&mut self, _member_expr: &MemberExpression<'_>) {}

    #[inline(always)]
    fn exit_member_expression(&mut self, _member_expr: &MemberExpression<'_>) {}

    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn should_emit_jsx_identifier(&self) -> bool {
        false
    }
}

/// JSX state for TS-style tokens.
///
/// Used in TS style for tracking when to emit JSX identifiers.
#[derive(Default)]
#[expect(clippy::struct_field_names)]
pub struct JSXStateTS {
    jsx_expression_depth: u32,
    member_expression_depth: u32,
    computed_member_depth: u32,
}

impl JSXState for JSXStateTS {
    #[inline]
    fn enter_jsx_expression(&mut self) {
        self.jsx_expression_depth += 1;
    }

    #[inline]
    fn exit_jsx_expression(&mut self) {
        self.jsx_expression_depth -= 1;
    }

    #[inline]
    fn enter_member_expression(&mut self, member_expr: &MemberExpression<'_>) {
        if self.jsx_expression_depth > 0 {
            self.member_expression_depth += 1;
            if matches!(member_expr, MemberExpression::ComputedMemberExpression(_)) {
                self.computed_member_depth += 1;
            }
        }
    }

    #[inline]
    fn exit_member_expression(&mut self, member_expr: &MemberExpression<'_>) {
        if self.jsx_expression_depth > 0 {
            self.member_expression_depth -= 1;
            if matches!(member_expr, MemberExpression::ComputedMemberExpression(_)) {
                self.computed_member_depth -= 1;
            }
        }
    }

    fn should_emit_jsx_identifier(&self) -> bool {
        self.jsx_expression_depth > 0
            && self.member_expression_depth > 0
            && self.computed_member_depth == 0
    }
}
