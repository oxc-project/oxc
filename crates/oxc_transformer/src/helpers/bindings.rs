use oxc_ast::ast::{BindingIdentifier, IdentifierReference};
use oxc_span::{Atom, Span, SPAN};
use oxc_syntax::{
    reference::ReferenceFlags,
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};
use oxc_traverse::TraverseCtx;

/// Info about a binding, from which one can create a `BindingIdentifier` or `IdentifierReference`s.
///
/// Typical usage:
///
/// ```rs
/// // Generate a UID for a top-level var
/// let binding = BoundIdentifier::new_root_uid("foo", SymbolFlags::FunctionScopedVariable, ctx);
///
/// // Generate an `IdentifierReference`s and insert them into AST
/// some_node.id = binding.create_read_reference(ctx);
/// some_other_node.id = binding.create_read_reference(ctx);
///
/// // Store details of the binding for later
/// self.foo_binding = binding;
///
/// // Later on in `exit_program`
/// let id = binding.create_binding_identifier();
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
#[derive(Clone)]
pub struct BoundIdentifier<'a> {
    pub name: Atom<'a>,
    pub symbol_id: SymbolId,
}

impl<'a> BoundIdentifier<'a> {
    /// Create `BoundIdentifier` for `name` and `symbol_id`
    pub fn new(name: Atom<'a>, symbol_id: SymbolId) -> Self {
        Self { name, symbol_id }
    }

    /// Create `BoundIdentifier` for new binding in specified scope
    pub fn new_uid(
        name: &str,
        scope_id: ScopeId,
        flags: SymbolFlags,
        ctx: &mut TraverseCtx<'a>,
    ) -> Self {
        let symbol_id = ctx.generate_uid(name, scope_id, flags);
        let name = ctx.ast.atom(&ctx.symbols().names[symbol_id]);
        Self { name, symbol_id }
    }

    /// Create `BoundIdentifier` for new binding in root scope
    pub fn new_uid_in_root_scope(
        name: &str,
        flags: SymbolFlags,
        ctx: &mut TraverseCtx<'a>,
    ) -> Self {
        let scope_id = ctx.scopes().root_scope_id();
        Self::new_uid(name, scope_id, flags, ctx)
    }

    /// Create `BoundIdentifier` for new binding in current scope
    #[allow(unused)]
    pub fn new_uid_in_current_scope(
        name: &str,
        flags: SymbolFlags,
        ctx: &mut TraverseCtx<'a>,
    ) -> Self {
        let scope_id = ctx.current_scope_id();
        Self::new_uid(name, scope_id, flags, ctx)
    }

    /// Create `BindingIdentifier` for this binding
    pub fn create_binding_identifier(&self) -> BindingIdentifier<'a> {
        BindingIdentifier::new_with_symbol_id(SPAN, self.name.clone(), self.symbol_id)
    }

    /// Create `IdentifierReference` referencing this binding, which is read from, with dummy `Span`
    pub fn create_read_reference(&self, ctx: &mut TraverseCtx<'a>) -> IdentifierReference<'a> {
        self.create_spanned_read_reference(SPAN, ctx)
    }

    /// Create `IdentifierReference` referencing this binding, which is read from, with specified `Span`
    pub fn create_spanned_read_reference(
        &self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> IdentifierReference<'a> {
        self.create_spanned_reference(span, ReferenceFlags::Read, ctx)
    }

    /// Create `IdentifierReference` referencing this binding, which is written to, with dummy `Span`
    #[allow(unused)]
    pub fn create_write_reference(&self, ctx: &mut TraverseCtx<'a>) -> IdentifierReference<'a> {
        self.create_spanned_write_reference(SPAN, ctx)
    }

    /// Create `IdentifierReference` referencing this binding, which is written to, with specified `Span`
    pub fn create_spanned_write_reference(
        &self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> IdentifierReference<'a> {
        self.create_spanned_reference(span, ReferenceFlags::Write, ctx)
    }

    /// Create `IdentifierReference` referencing this binding, which is read from + written to,
    /// with dummy `Span`
    #[allow(unused)]
    pub fn create_read_write_reference(
        &self,
        ctx: &mut TraverseCtx<'a>,
    ) -> IdentifierReference<'a> {
        self.create_spanned_read_write_reference(SPAN, ctx)
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

    /// Create `IdentifierReference` referencing this binding, with specified `Span` and `ReferenceFlags`
    pub fn create_spanned_reference(
        &self,
        span: Span,
        flags: ReferenceFlags,
        ctx: &mut TraverseCtx<'a>,
    ) -> IdentifierReference<'a> {
        ctx.create_bound_reference_id(span, self.name.clone(), self.symbol_id, flags)
    }
}
