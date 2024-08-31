use oxc_span::Atom;
use oxc_syntax::symbol::SymbolId;

/// Trait for accessing a [`BindingIdentifier`](`crate::ast::BindingIdentifier`)
/// within an AST Node. Most often used on nodes that create symbol bindings.
pub trait WithBindingIdentifier<'a> {
    /// Get the [`SymbolId`] bound to this AST node.
    ///
    /// Will return [`None`] if:
    /// 1. The AST node does not create a symbol binding, such as anonymous
    ///    function expressions,
    /// 2. Semantic analysis has been skipped, or
    /// 3. The symbol binding is within a destructuring pattern and cannot be uniquely
    ///    identified.
    fn symbol_id(&self) -> Option<SymbolId>;

    /// Get the identifier name of the symbol this AST node binds.
    ///
    /// Will return [`None`] if:
    /// 1. The AST node does not create a symbol binding, such as anonymous
    ///   function expressions,
    /// 2. The symbol binding is within a destructuring pattern and cannot be uniquely
    ///    identified.
    ///
    /// Note that identifier names are determined at parse time, so this method
    /// is unaffected by semantic analysis.
    fn name(&self) -> Option<Atom<'a>>;
}
