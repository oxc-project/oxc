use syn::Ident;

use crate::util::ToIdent;

use super::defs::{EnumDef, StructDef, TypeDef};

pub trait GetIdent {
    fn ident(&self) -> Ident;
}

impl GetIdent for TypeDef {
    fn ident(&self) -> Ident {
        match self {
            TypeDef::Struct(def) => def.ident(),
            TypeDef::Enum(def) => def.ident(),
        }
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
