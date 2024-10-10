use proc_macro2::TokenStream;
use quote::quote;

use super::{define_derive, Derive, DeriveOutput};
use crate::{
    codegen::LateCtx,
    schema::{GetGenerics, GetIdent, TypeDef},
};

define_derive! {
    pub struct DeriveESTree;
}

impl Derive for DeriveESTree {
    fn trait_name() -> &'static str {
        "Serialize"
    }

    fn derive(&mut self, def: &TypeDef, _: &LateCtx) -> TokenStream {
        let ident = def.ident();
        if let TypeDef::Struct(it) = def {
            for field in &it.fields {
                println!("{:?}: {:?}", field.name, field.markers.derive_attributes.estree);
            }
        }
        if def.has_lifetime() {
            quote! {
                impl<'a> Serialize for #ident<'a> {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where S: Serializer,
                    {
                        serializer.serialize_none()
                    }
                }
            }
        } else {
            quote! {
              impl Serialize for #ident {
                  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                  where S: Serializer,
                  {
                      serializer.serialize_none()
                  }
              }
            }
        }
    }

    fn prelude() -> TokenStream {
        quote! {
            use serde::{Serialize, Serializer};
        }
    }
}
