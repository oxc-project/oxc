//! Derive for `GetNodeId` trait.

use proc_macro2::TokenStream;
use quote::quote;

use crate::schema::{Def, Schema};

use super::{Derive, StructOrEnum, define_derive};

/// Derive for `GetNodeId` trait.
pub struct DeriveGetNodeId;

define_derive!(DeriveGetNodeId);

impl Derive for DeriveGetNodeId {
    fn trait_name(&self) -> &'static str {
        "GetNodeId"
    }

    fn crate_name(&self) -> &'static str {
        "oxc_syntax"
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            use oxc_syntax::{GetNodeId, node::NodeId};
        }
    }

    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        let ty = type_def.ty_anon(schema);

        quote! {
            impl GetNodeId for #ty {
                #[inline]
                fn node_id(&self) -> NodeId {
                    self.node_id()
                }
            }
        }
    }
}
