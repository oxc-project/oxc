use itertools::Itertools;
use syn::{parse_quote, Variant};

use crate::{schema::RType, CodegenCtx, Generator, GeneratorOutput};

pub struct AstKindGenerator;

impl Generator for AstKindGenerator {
    fn name(&self) -> &'static str {
        "AstKindGenerator"
    }

    fn generate(&mut self, ctx: &CodegenCtx) -> GeneratorOutput {
        let kinds: Vec<Variant> = ctx
            .ty_table
            .iter()
            .filter_map(|maybe_kind| match &*maybe_kind.borrow() {
                kind @ (RType::Enum(_) | RType::Struct(_)) => {
                    let ident = kind.ident();
                    let typ = kind.as_type();
                    Some(parse_quote!(#ident(#typ)))
                }
                _ => None,
            })
            .collect_vec();

        GeneratorOutput::One(parse_quote! {
            pub enum AstKind<'a> {
                #(#kinds),*
            }
        })
    }
}
