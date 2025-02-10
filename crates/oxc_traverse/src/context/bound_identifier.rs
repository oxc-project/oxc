use oxc_ast::{
    ast::{
        AssignmentTarget, BindingIdentifier, BindingPattern, BindingPatternKind, Expression,
        IdentifierReference, SimpleAssignmentTarget,
    },
    NONE,
};
use oxc_span::{Atom, Span, SPAN};
use oxc_syntax::{reference::ReferenceFlags, symbol::SymbolId};

use crate::TraverseCtx;

use super::MaybeBoundIdentifier;

/// Info about a binding, from which one can create a `BindingIdentifier` or `IdentifierReference`s.
///
/// Typical usage:
///
/// ```rs
/// // Generate a UID for a top-level var
/// let binding = ctx.generate_uid_in_current_scope("foo", SymbolFlags::FunctionScopedVariable);
///
/// // Generate `IdentifierReference`s and insert them into AST
/// some_node.id = binding.create_read_reference(ctx);
/// some_other_node.id = binding.create_read_reference(ctx);
///
/// // Store details of the binding for later
/// self.foo_binding = binding;
///
/// // Later on in `exit_program`
/// let id = self.foo_binding.create_binding_identifier(ctx);
/// // Insert `var <id> = something;` into `program.body`
/// ```
///
/// Notes:
///
/// * `BoundIdentifier` is smaller than `BindingIdentifier`, so takes less memory when you store
///   it for later use.
/// * `BoundIdentifier` is `Clone` (unlike `BindingIdentifier`).
/// * `BoundIdentifier` re-uses the same `Atom` for all `BindingIdentifier` / `IdentifierReference`s
///   created from it.
#[derive(Debug, Clone)]
pub struct BoundIdentifier<'a> {
    pub name: Atom<'a>,
    pub symbol_id: SymbolId,
}

impl<'a> BoundIdentifier<'a> {
    /// Create `BoundIdentifier` for `name` and `symbol_id`
    pub fn new(name: Atom<'a>, symbol_id: SymbolId) -> Self {
        Self { name, symbol_id }
    }

    /// Create `BoundIdentifier` from a `BindingIdentifier`
    pub fn from_binding_ident(ident: &BindingIdentifier<'a>) -> Self {
        Self { name: ident.name, symbol_id: ident.symbol_id() }
    }

    /// Convert `BoundIdentifier` to `MaybeBoundIdentifier`
    pub fn to_maybe_bound_identifier(&self) -> MaybeBoundIdentifier<'a> {
        MaybeBoundIdentifier::new(self.name, Some(self.symbol_id))
    }

    /// Create `BindingIdentifier` for this binding
    pub fn create_binding_identifier(&self, ctx: &TraverseCtx<'a>) -> BindingIdentifier<'a> {
        ctx.ast.binding_identifier_with_symbol_id(SPAN, self.name, self.symbol_id)
    }

    /// Create `BindingPattern` for this binding
    pub fn create_binding_pattern(&self, ctx: &TraverseCtx<'a>) -> BindingPattern<'a> {
        let ident = self.create_binding_identifier(ctx);
        let binding_pattern_kind = BindingPatternKind::BindingIdentifier(ctx.alloc(ident));
        ctx.ast.binding_pattern(binding_pattern_kind, NONE, false)
    }

    // --- Read only ---

    /// Create `IdentifierReference` referencing this binding, which is read from, with dummy `Span`
    pub fn create_read_reference(&self, ctx: &mut TraverseCtx<'a>) -> IdentifierReference<'a> {
        self.create_spanned_read_reference(SPAN, ctx)
    }

    /// Create `Expression::Identifier` referencing this binding, which is read from, with dummy `Span`
    pub fn create_read_expression(&self, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        self.create_spanned_read_expression(SPAN, ctx)
    }

    /// Create `IdentifierReference` referencing this binding, which is read from, with specified `Span`
    pub fn create_spanned_read_reference(
        &self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> IdentifierReference<'a> {
        self.create_spanned_reference(span, ReferenceFlags::Read, ctx)
    }

    /// Create `Expression::Identifier` referencing this binding, which is read from, with specified `Span`
    pub fn create_spanned_read_expression(
        &self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        self.create_spanned_expression(span, ReferenceFlags::Read, ctx)
    }

    // --- Write only ---

    /// Create `IdentifierReference` referencing this binding, which is written to, with dummy `Span`
    pub fn create_write_reference(&self, ctx: &mut TraverseCtx<'a>) -> IdentifierReference<'a> {
        self.create_spanned_write_reference(SPAN, ctx)
    }

    /// Create `Expression::Identifier` referencing this binding, which is written to, with dummy `Span`
    pub fn create_write_expression(&self, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        self.create_spanned_write_expression(SPAN, ctx)
    }

    /// Create `AssignmentTarget` referencing this binding, which is written to, with dummy `Span`
    pub fn create_write_target(&self, ctx: &mut TraverseCtx<'a>) -> AssignmentTarget<'a> {
        self.create_spanned_write_target(SPAN, ctx)
    }

    /// Create `SimpleAssignmentTarget` referencing this binding, which is written to, with dummy `Span`
    pub fn create_write_simple_target(
        &self,
        ctx: &mut TraverseCtx<'a>,
    ) -> SimpleAssignmentTarget<'a> {
        self.create_spanned_write_simple_target(SPAN, ctx)
    }

    /// Create `IdentifierReference` referencing this binding, which is written to, with specified `Span`
    pub fn create_spanned_write_reference(
        &self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> IdentifierReference<'a> {
        self.create_spanned_reference(span, ReferenceFlags::Write, ctx)
    }

    /// Create `Expression::Identifier` referencing this binding, which is written to, with specified `Span`
    pub fn create_spanned_write_expression(
        &self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        self.create_spanned_expression(span, ReferenceFlags::Write, ctx)
    }

    /// Create `AssignmentTarget` referencing this binding, which is written to, with specified `Span`
    pub fn create_spanned_write_target(
        &self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> AssignmentTarget<'a> {
        self.create_spanned_target(span, ReferenceFlags::Write, ctx)
    }

    /// Create `SimpleAssignmentTarget` referencing this binding, which is written to, with specified `Span`
    pub fn create_spanned_write_simple_target(
        &self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> SimpleAssignmentTarget<'a> {
        self.create_spanned_simple_target(span, ReferenceFlags::Write, ctx)
    }

    // --- Read and write ---

    /// Create `IdentifierReference` referencing this binding, which is read from + written to,
    /// with dummy `Span`
    pub fn create_read_write_reference(
        &self,
        ctx: &mut TraverseCtx<'a>,
    ) -> IdentifierReference<'a> {
        self.create_spanned_read_write_reference(SPAN, ctx)
    }

    /// Create `Expression::Identifier` referencing this binding, which is read from + written to,
    /// with dummy `Span`
    pub fn create_read_write_expression(&self, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        self.create_spanned_read_write_expression(SPAN, ctx)
    }

    /// Create `AssignmentTarget` referencing this binding, which is read from + written to,
    /// with dummy `Span`
    pub fn create_read_write_target(&self, ctx: &mut TraverseCtx<'a>) -> AssignmentTarget<'a> {
        self.create_spanned_read_write_target(SPAN, ctx)
    }

    /// Create `SimpleAssignmentTarget` referencing this binding, which is read from + written to,
    /// with dummy `Span`
    pub fn create_read_write_simple_target(
        &self,
        ctx: &mut TraverseCtx<'a>,
    ) -> SimpleAssignmentTarget<'a> {
        self.create_spanned_read_write_simple_target(SPAN, ctx)
    }

    /// Create `IdentifierReference` referencing this binding, which is read from + written to,
    /// with specified `Span`
    pub fn create_spanned_read_write_reference(
        &self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> IdentifierReference<'a> {
        self.create_spanned_reference(span, ReferenceFlags::Read | ReferenceFlags::Write, ctx)
    }

    /// Create `Expression::Identifier` referencing this binding, which is read from + written to,
    /// with specified `Span`
    pub fn create_spanned_read_write_expression(
        &self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        self.create_spanned_expression(span, ReferenceFlags::Read | ReferenceFlags::Write, ctx)
    }

    /// Create `AssignmentTarget` referencing this binding, which is read from + written to,
    /// with specified `Span`
    pub fn create_spanned_read_write_target(
        &self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> AssignmentTarget<'a> {
        self.create_spanned_target(span, ReferenceFlags::Read | ReferenceFlags::Write, ctx)
    }

    /// Create `SimpleAssignmentTarget` referencing this binding, which is read from + written to,
    /// with specified `Span`
    pub fn create_spanned_read_write_simple_target(
        &self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> SimpleAssignmentTarget<'a> {
        self.create_spanned_simple_target(span, ReferenceFlags::Read | ReferenceFlags::Write, ctx)
    }

    // --- Specified ReferenceFlags ---

    /// Create `IdentifierReference` referencing this binding, with specified `ReferenceFlags`
    pub fn create_reference(
        &self,
        flags: ReferenceFlags,
        ctx: &mut TraverseCtx<'a>,
    ) -> IdentifierReference<'a> {
        self.create_spanned_reference(SPAN, flags, ctx)
    }

    /// Create `Expression::Identifier` referencing this binding, with specified `ReferenceFlags`
    pub fn create_expression(
        &self,
        flags: ReferenceFlags,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        self.create_spanned_expression(SPAN, flags, ctx)
    }

    /// Create `AssignmentTarget` referencing this binding, with specified `ReferenceFlags`
    pub fn create_target(
        &self,
        flags: ReferenceFlags,
        ctx: &mut TraverseCtx<'a>,
    ) -> AssignmentTarget<'a> {
        self.create_spanned_target(SPAN, flags, ctx)
    }

    /// Create `SimpleAssignmentTarget` referencing this binding, with specified `ReferenceFlags`
    pub fn create_simple_target(
        &self,
        flags: ReferenceFlags,
        ctx: &mut TraverseCtx<'a>,
    ) -> SimpleAssignmentTarget<'a> {
        self.create_spanned_simple_target(SPAN, flags, ctx)
    }

    /// Create `IdentifierReference` referencing this binding, with specified `Span` and `ReferenceFlags`
    pub fn create_spanned_reference(
        &self,
        span: Span,
        flags: ReferenceFlags,
        ctx: &mut TraverseCtx<'a>,
    ) -> IdentifierReference<'a> {
        ctx.create_bound_ident_reference(span, self.name, self.symbol_id, flags)
    }

    /// Create `Expression::Identifier` referencing this binding, with specified `Span` and `ReferenceFlags`
    pub fn create_spanned_expression(
        &self,
        span: Span,
        flags: ReferenceFlags,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let ident = self.create_spanned_reference(span, flags, ctx);
        Expression::Identifier(ctx.alloc(ident))
    }

    /// Create `AssignmentTarget::AssignmentTargetIdentifier` referencing this binding,
    /// with specified `Span` and `ReferenceFlags`
    pub fn create_spanned_target(
        &self,
        span: Span,
        flags: ReferenceFlags,
        ctx: &mut TraverseCtx<'a>,
    ) -> AssignmentTarget<'a> {
        let ident = self.create_spanned_reference(span, flags, ctx);
        AssignmentTarget::AssignmentTargetIdentifier(ctx.alloc(ident))
    }

    /// Create `SimpleAssignmentTarget::AssignmentTargetIdentifier` referencing this binding,
    /// with specified `Span` and `ReferenceFlags`
    pub fn create_spanned_simple_target(
        &self,
        span: Span,
        flags: ReferenceFlags,
        ctx: &mut TraverseCtx<'a>,
    ) -> SimpleAssignmentTarget<'a> {
        let ident = self.create_spanned_reference(span, flags, ctx);
        SimpleAssignmentTarget::AssignmentTargetIdentifier(ctx.alloc(ident))
    }
}
