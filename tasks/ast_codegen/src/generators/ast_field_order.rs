use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemStruct;

use crate::{layout::Layout, output, schema::RType, CodegenCtx, Generator, GeneratorOutput};

use super::{define_generator, generated_header};

define_generator! {
    pub struct AstFieldOrder;
}

impl Generator for AstFieldOrder {
    fn name(&self) -> &'static str {
        stringify!(AstFieldOrder)
    }

    fn generate(&mut self, ctx: &CodegenCtx) -> GeneratorOutput {
        let orders_64 = ctx
            .ty_table
            .iter()
            .filter(|ty| matches!(&*ty.borrow(), RType::Struct(s) if !s.item.fields.is_empty()))
            .map(|ty| {
                let RType::Struct(ty) = &*ty.borrow() else { unreachable!() };
                generate_orders(&ty.item, &ty.meta.layout_64)
            });
        let orders_32 = ctx
            .ty_table
            .iter()
            .filter(|ty| matches!(&*ty.borrow(), RType::Struct(s) if !s.item.fields.is_empty()))
            .map(|ty| {
                let RType::Struct(ty) = &*ty.borrow() else { unreachable!() };
                generate_orders(&ty.item, &ty.meta.layout_64)
            });
        let header = generated_header!();
        GeneratorOutput::Stream((
            output(crate::AST_MACROS_CRATE, "ast_field_order_data.rs"),
            quote! {
                #header
                use lazy_static::lazy_static;
                use rustc_hash::FxHashMap;

                endl!();

                pub fn get(ident: &str) -> Option<&[u8]> {

                    #[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
                    std::compile_error!(
                        "Platforms with pointer width other than 64 or 32 bit are not supported"
                    );
                    #[cfg(target_pointer_width = "64")]
                    lazy_static! {
                        static ref DATA: FxHashMap<&'static str, &'static [u8]> =
                            FxHashMap::from_iter([#(#orders_64),*]);
                    }
                    #[cfg(target_pointer_width = "32")]
                    lazy_static! {
                        static ref DATA: FxHashMap<&'static str, &'static [u8]> =
                            FxHashMap::from_iter([#(#orders_32),*]);
                    }


                    DATA.get(ident).copied()
                }
            },
        ))
    }
}

fn generate_orders(ty: &ItemStruct, layout: &Layout) -> Option<TokenStream> {
    let ident = &ty.ident.to_string();
    let Layout::Layout(layout) = layout else { panic!("Layout isn't determined yet!") };
    let offsets = layout.offsets();
    if let Some(offsets) = offsets {
        let orders = offsets
            .iter()
            .zip(ty.fields.iter().enumerate())
            .sorted_by(|a, b| Ord::cmp(a.0, b.0))
            .map(|(_, fi)| fi)
            .enumerate()
            .sorted_by(|a, b| Ord::cmp(&a.1 .0, &b.1 .0))
            .map(|it| {
                u8::try_from(it.0).expect("We have no AST type with enough fields to exhaust `u8`.")
            });
        Some(quote!((#ident, &[#(#orders),*][..])))
    } else {
        None
    }
}
