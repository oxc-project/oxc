use std::path::PathBuf;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;

use crate::{codegen::LateCtx, schema::TypeDef};

mod clone_in;
mod content_eq;
mod content_hash;
mod estree;
mod get_span;

pub use clone_in::DeriveCloneIn;
pub use content_eq::DeriveContentEq;
pub use content_hash::DeriveContentHash;
pub use estree::DeriveESTree;
pub use get_span::{DeriveGetSpan, DeriveGetSpanMut};

pub trait Derive {
    fn trait_name() -> &'static str;

    fn snake_name() -> String {
        Self::trait_name().to_case(Case::Snake)
    }

    fn derive(&mut self, def: &TypeDef, ctx: &LateCtx) -> TokenStream;

    fn prelude() -> TokenStream {
        TokenStream::default()
    }
}

pub trait DeriveTemplate: Derive {
    fn template(module_path: Vec<&str>, impls: TokenStream) -> TokenStream;
}

#[derive(Debug, Clone)]
pub struct DeriveOutput(pub Vec<(PathBuf, TokenStream)>);

macro_rules! define_derive {
    ($vis:vis struct $ident:ident $($lifetime:lifetime)? $($rest:tt)*) => {
        $vis struct $ident $($lifetime)? $($rest)*

        impl $($lifetime)? $crate::derives::DeriveTemplate for $ident $($lifetime)? {
            fn template(module_paths: Vec<&str>, impls: TokenStream) -> TokenStream {
                use itertools::Itertools;
                let header = $crate::codegen::generated_header!();
                let prelude = Self::prelude();

                // from `x::y::z` to `crate::y::z::*`
                let use_modules = module_paths.into_iter().map(|it| {
                    let local_path = ["crate"]
                        .into_iter()
                        .chain(it.strip_suffix("::mod").unwrap_or(it).split("::").skip(1))
                        .chain(["*"])
                        .join("::");
                    let use_module: syn::ItemUse =
                        syn::parse_str(format!("use {local_path};").as_str()).unwrap();
                    quote::quote! {
                        ///@@line_break
                        #[allow(clippy::wildcard_imports)]
                        #use_module
                    }
                });

                quote::quote! {
                    #header

                    #prelude

                    #(#use_modules)*

                    ///@@line_break
                    #impls
                }
            }
        }

        impl $($lifetime)? $crate::codegen::Runner for $ident $($lifetime)? {
            type Context = $crate::codegen::LateCtx;
            type Output = $crate::derives::DeriveOutput;

            fn name(&self) -> &'static str {
                stringify!($ident)
            }

            fn run(&mut self, ctx: &$crate::codegen::LateCtx) -> $crate::Result<Self::Output> {
                use std::vec::Vec;
                use itertools::Itertools;
                use rustc_hash::{FxHashMap, FxHashSet};

                use $crate::derives::DeriveTemplate;

                let trait_name = Self::trait_name();
                let filename = format!("derive_{}.rs", Self::snake_name());
                let output = ctx
                    .schema()
                    .into_iter()
                    .filter(|def| def.generates_derive(trait_name))
                    .map(|def| (def, self.derive(def, ctx)))
                    .fold(FxHashMap::<&str, (FxHashSet<&str>, Vec<TokenStream>)>::default(), |mut acc, (def, stream)| {
                        let module_path = def.module_path();
                        let krate = module_path.split("::").next().unwrap();
                        if !acc.contains_key(krate) {
                            acc.insert(krate, Default::default());
                        }
                        let streams = acc.get_mut(krate).expect("We checked this right above!");
                        streams.0.insert(module_path);
                        streams.1.push(stream);
                        acc
                    })
                    .into_iter()
                    .sorted_by(|lhs, rhs| lhs.0.cmp(rhs.0))
                    .fold(Vec::new(), |mut acc, (path, (modules, streams))| {
                        let mut modules = Vec::from_iter(modules);
                        modules.sort();

                        acc.push((
                            $crate::output(
                                format!("crates/{}", path.split("::").next().unwrap()).as_str(),
                                &filename,
                            ),
                            Self::template(
                                modules,
                                streams
                                .into_iter()
                                .fold(TokenStream::new(), |mut acc, it| {
                                    acc.extend(quote::quote!{
                                        ///@@line_break
                                    });
                                    acc.extend(it);
                                    acc
                                })
                            )
                        ));
                        acc
                    });
                Ok(DeriveOutput(output))
            }
        }
    };
}
pub(crate) use define_derive;
