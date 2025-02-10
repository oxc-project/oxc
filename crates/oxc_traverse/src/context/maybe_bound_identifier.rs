use oxc_ast::ast::{AssignmentTarget, Expression, IdentifierReference, SimpleAssignmentTarget};
use oxc_span::{Atom, Span, SPAN};
use oxc_syntax::{reference::ReferenceFlags, symbol::SymbolId};

use crate::TraverseCtx;

use super::BoundIdentifier;

/// A factory for generating `IdentifierReference`s.
///
/// Typical usage:
///
/// ```rs
/// // Create `MaybeBoundIdentifier` from an existing `IdentifierReference`
/// let binding = MaybeBoundIdentifier::from_identifier_reference(ident, ctx);
///
/// // Generate `IdentifierReference`s and insert them into AST
/// assign_expr.left = binding.create_write_target(ctx);
/// assign_expr.right = binding.create_read_expression(ctx);
/// ```
///
/// Notes:
///
/// * The original `IdentifierReference` must also be used in the AST, or it'll be a dangling reference.
/// * `MaybeBoundIdentifier` is smaller than `IdentifierReference`, so takes less memory when you store
///   it for later use.
/// * `MaybeBoundIdentifier` re-uses the same `Atom` for all `BindingIdentifier` / `IdentifierReference`s
///   created from it.
/// * `MaybeBoundIdentifier` looks up the `SymbolId` for the reference only once.
#[derive(Debug, Clone)]
pub struct MaybeBoundIdentifier<'a> {
    pub name: Atom<'a>,
    pub symbol_id: Option<SymbolId>,
}

impl<'a> MaybeBoundIdentifier<'a> {
    /// Create `MaybeBoundIdentifier` for `name` and `Option<SymbolId>`
    pub fn new(name: Atom<'a>, symbol_id: Option<SymbolId>) -> Self {
        Self { name, symbol_id }
    }

    /// Create `MaybeBoundIdentifier` from an `IdentifierReference`
    pub fn from_identifier_reference(
        ident: &IdentifierReference<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Self {
        let symbol_id = ctx.symbols().get_reference(ident.reference_id()).symbol_id();
        Self { name: ident.name, symbol_id }
    }

    /// Convert `MaybeBoundIdentifier` to `BoundIdentifier`.
    ///
    /// Returns `None` if symbol is not bound.
    pub fn to_bound_identifier(&self) -> Option<BoundIdentifier<'a>> {
        self.symbol_id.map(|symbol_id| BoundIdentifier::new(self.name, symbol_id))
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
        ctx.create_ident_reference(span, self.name, self.symbol_id, flags)
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
