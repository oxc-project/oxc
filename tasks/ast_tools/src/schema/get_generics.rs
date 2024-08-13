use syn::parse_quote;

use super::{
    defs::{EnumDef, StructDef, TypeDef},
    with_either,
};

pub trait GetGenerics {
    fn has_lifetime(&self) -> bool {
        false
    }

    fn generics(&self) -> Option<syn::Generics> {
        if self.has_lifetime() {
            Some(parse_quote!(<'a>))
        } else {
            None
        }
    }
}

impl GetGenerics for TypeDef {
    fn has_lifetime(&self) -> bool {
        with_either!(self, it => it.has_lifetime())
    }
}

impl GetGenerics for StructDef {
    fn has_lifetime(&self) -> bool {
        self.has_lifetime
    }
}

impl GetGenerics for EnumDef {
    fn has_lifetime(&self) -> bool {
        self.has_lifetime
    }
}
