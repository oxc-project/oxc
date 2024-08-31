use oxc_span::Atom;
use oxc_syntax::symbol::SymbolId;

pub trait WithBindingIdentifier<'a> {
    fn symbol_id(&self) -> Option<SymbolId>;

    fn name(&self) -> Option<Atom<'a>>;
}
