//! JavaScript Parsing Functions

mod grammar;

mod arrow;
mod binding;
mod class;
mod declaration;
mod expression;
mod function;
mod module;
mod object;
mod operator;
mod statement;

/// Classification of a `(`/`<`/`async (` head for parenthesized-arrow disambiguation.
///
/// Refines the old `Tristate`: the `Maybe` cases split into `Cover` (expression-shaped — parse
/// `(...)` once as an expression then refine to params) and `Speculate` (no expression form, or
/// async/return-type/context-sensitive — keep the speculate-and-rewind path).
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ArrowKind {
    /// Not an arrow (was `Tristate::False`). Fall through to expression parse.
    No,
    /// Definitely an arrow (was `Tristate::True`). Parse params directly.
    Yes,
    /// Expression-shaped ambiguous: `([`, `({`, `(a)`, `(a =` (non-async). Parse `(...)` once as
    /// an expression, then refine to `FormalParameters` iff `=>` follows.
    Cover,
    /// Ambiguous with no expression form / context-sensitive: `():T`, `(...[`, generic/JSX `<...>`,
    /// any comma-shape (`(a,` — a later `...rest` is invisible to the worker), and all `async (`.
    /// Keep the speculate-and-rewind path.
    Speculate,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FunctionKind {
    Constructor,
    ClassMethod,
    ObjectMethod,
    Declaration,
    Expression,
    DefaultExport,
    TSDeclaration,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VariableDeclarationParent {
    For,
    Statement,
}
