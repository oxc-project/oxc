use std::path::PathBuf;

use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::TokenStream;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    codegen::{generate_header, CodegenBase, LateCtx},
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

#[derive(Debug, Clone)]
pub struct DeriveOutput(pub Vec<(PathBuf, TokenStream)>);

pub trait Derive: CodegenBase {
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
        let header = generate_header(Self::file_path());
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

    fn output(&mut self, ctx: &LateCtx) -> Result<DeriveOutput> {
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
                    if !acc.contains_key(krate) {
                        acc.insert(krate, Default::default());
                    }
                    let streams = acc.get_mut(krate).expect("We checked this right above!");
                    streams.0.insert(module_path);
                    streams.1.push(stream);
                    acc
                },
            )
            .into_iter()
            .sorted_by(|lhs, rhs| lhs.0.cmp(rhs.0))
            .fold(Vec::new(), |mut acc, (path, (modules, streams))| {
                let mut modules = Vec::from_iter(modules);
                modules.sort_unstable();

                acc.push((
                    crate::output(
                        format!("crates/{}", path.split("::").next().unwrap()).as_str(),
                        &filename,
                    ),
                    Self::template(
                        modules,
                        streams.into_iter().fold(TokenStream::new(), |mut acc, it| {
                            acc.extend(quote::quote! {
                                ///@@line_break
                            });
                            acc.extend(it);
                            acc
                        }),
                    ),
                ));
                acc
            });
        Ok(DeriveOutput(output))
    }
}

macro_rules! define_derive {
    ($ident:ident $($lifetime:lifetime)?) => {
        const _: () = {
            use $crate::{
                codegen::{CodegenBase, LateCtx, Runner},
                derives::DeriveOutput,
                Result,
            };

            impl $($lifetime)? CodegenBase for $ident $($lifetime)? {
                fn file_path() -> &'static str {
                    file!()
                }
            }

            impl $($lifetime)? Runner for $ident $($lifetime)? {
                type Context = LateCtx;
                type Output = DeriveOutput;

                fn name(&self) -> &'static str {
                    stringify!($ident)
                }

                fn run(&mut self, ctx: &LateCtx) -> Result<DeriveOutput> {
                    self.output(ctx)
                }
            }
        };
    };
}
pub(crate) use define_derive;
