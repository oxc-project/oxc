use syn::Ident;

use super::{
    defs::{EnumDef, StructDef, TypeDef},
    with_either,
};
use crate::util::ToIdent;

pub trait GetIdent {
    fn ident(&self) -> Ident;
}

impl GetIdent for TypeDef {
    fn ident(&self) -> Ident {
        with_either!(self, it => it.ident())
    }
}

impl GetIdent for StructDef {
    fn ident(&self) -> Ident {
        self.name.to_ident()
    }
}

impl GetIdent for EnumDef {
    fn ident(&self) -> Ident {
        self.name.to_ident()
    }
}
