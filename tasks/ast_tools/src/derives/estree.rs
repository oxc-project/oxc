use proc_macro2::TokenStream;
use quote::quote;

use super::{define_derive, Derive, DeriveOutput};
use crate::{
    codegen::LateCtx,
    schema::{EnumDef, GetGenerics, GetIdent, TypeDef},
};

define_derive! {
    pub struct DeriveESTree;
}

impl Derive for DeriveESTree {
    fn trait_name() -> &'static str {
        "ESTree"
    }

    fn derive(&mut self, def: &TypeDef, _: &LateCtx) -> TokenStream {
        let ident = def.ident();
        if let TypeDef::Struct(it) = def {
            println!("{ident:?} {:?}", it.markers.estree);
            for field in &it.fields {
                println!("- {:?}: {:?}", field.name, field.markers.derive_attributes.estree);
            }
        }

        // let body = match def {
        //     TypeDef::Enum(def) => serialize_enum(def),
        //     _ => quote! { serializer.serialize_none() },
        // };
        let body = quote! { serializer.serialize_none() };

        if def.has_lifetime() {
            quote! {
                impl<'a> Serialize for #ident<'a> {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where S: Serializer,
                    {
                        #body
                    }
                }
            }
        } else {
            quote! {
              impl Serialize for #ident {
                  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                  where S: Serializer,
                  {
                      #body
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

fn serialize_enum(def: &EnumDef) -> TokenStream {
    let ident = def.ident();
    // if def.markers.estree.untagged {
    // 	let match_branches = def.variants.iter().map(|var| {
    // 		let var_ident= var.ident();
    // 		var.fields
    // 		quote! {
    // 			#ident::#var_ident()
    // 		}
    // 	})
    // }
    // def.markers.estree.
    todo!()
}
