use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use rustc_hash::{FxHashMap, FxHashSet};
use syn::{parse_str, ItemUse};

use crate::{
    codegen::LateCtx,
    output::{output_path, Output},
    schema::TypeDef,
    Result,
};

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
    // Methods defined by implementer

    fn trait_name() -> &'static str;

    fn snake_name() -> String {
        Self::trait_name().to_case(Case::Snake)
    }

    fn derive(&mut self, def: &TypeDef, ctx: &LateCtx) -> TokenStream;

    fn prelude() -> TokenStream {
        TokenStream::default()
    }

    // Standard methods

    fn template(module_paths: Vec<&str>, impls: TokenStream) -> TokenStream {
        let prelude = Self::prelude();

        // from `x::y::z` to `crate::y::z::*`
        let use_modules = module_paths.into_iter().map(|it| {
            let local_path = ["crate"]
                .into_iter()
                .chain(it.strip_suffix("::mod").unwrap_or(it).split("::").skip(1))
                .chain(["*"])
                .join("::");
            let use_module: ItemUse = parse_str(format!("use {local_path};").as_str()).unwrap();
            quote! {
                ///@@line_break
                #use_module
            }
        });

        quote! {
            #prelude

            #(#use_modules)*

            ///@@line_break
            #impls
        }
    }

    fn output(&mut self, ctx: &LateCtx) -> Result<Vec<Output>> {
        let trait_name = Self::trait_name();
        let filename = format!("derive_{}.rs", Self::snake_name());
        let output = ctx
            .schema()
            .into_iter()
            .filter(|def| def.generates_derive(trait_name))
            .map(|def| (def, self.derive(def, ctx)))
            .fold(
                FxHashMap::<&str, (FxHashSet<&str>, Vec<TokenStream>)>::default(),
                |mut acc, (def, stream)| {
                    let module_path = def.module_path();
                    let krate = module_path.split("::").next().unwrap();
                    let streams = acc.entry(krate).or_default();
                    streams.0.insert(module_path);
                    streams.1.push(stream);
                    acc
                },
            )
            .into_iter()
            .sorted_by(|lhs, rhs| lhs.0.cmp(rhs.0))
            .fold(Vec::new(), |mut acc, (krate, (modules, streams))| {
                let mut modules = Vec::from_iter(modules);
                modules.sort_unstable();

                let output = Output::Rust {
                    path: output_path(&format!("crates/{krate}"), &filename),
                    tokens: Self::template(
                        modules,
                        streams.into_iter().fold(TokenStream::new(), |mut acc, it| {
                            acc.extend(quote! {
                                ///@@line_break
                            });
                            acc.extend(it);
                            acc
                        }),
                    ),
                };

                acc.push(output);
                acc
            });
        Ok(output)
    }
}

macro_rules! define_derive {
    ($ident:ident $($lifetime:lifetime)?) => {
        const _: () = {
            use $crate::{
                codegen::{LateCtx, Runner},
                output::Output,
                Result,
            };

            impl $($lifetime)? Runner for $ident $($lifetime)? {
                type Context = LateCtx;
                type Output = Vec<Output>;

                fn name(&self) -> &'static str {
                    stringify!($ident)
                }

                fn file_path(&self) -> &'static str {
                    file!()
                }

                fn run(&mut self, ctx: &LateCtx) -> Result<Vec<Output>> {
                    self.output(ctx)
                }
            }
        };
    };
}
pub(crate) use define_derive;
