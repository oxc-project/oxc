use syn::{parse_quote, Generics};

use super::defs::{EnumDef, StructDef, TypeDef};

pub trait GetGenerics {
    fn has_lifetime(&self) -> bool {
        false
    }

    fn generics(&self) -> Option<Generics> {
        if self.has_lifetime() {
            Some(parse_quote!(<'a>))
        } else {
            None
        }
    }
}

impl GetGenerics for TypeDef {
    fn has_lifetime(&self) -> bool {
        match self {
            TypeDef::Struct(def) => def.has_lifetime(),
            TypeDef::Enum(def) => def.has_lifetime(),
        }
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
