use super::{
    defs::{EnumDef, StructDef, TypeDef},
    with_either,
};
use crate::util::ToIdent;

pub trait GetIdent {
    fn ident(&self) -> syn::Ident;
}

impl GetIdent for TypeDef {
    fn ident(&self) -> syn::Ident {
        with_either!(self, it => it.ident())
    }
}

impl GetIdent for StructDef {
    fn ident(&self) -> syn::Ident {
        self.name.to_ident()
    }
}

impl GetIdent for EnumDef {
    fn ident(&self) -> syn::Ident {
        self.name.to_ident()
    }
}
