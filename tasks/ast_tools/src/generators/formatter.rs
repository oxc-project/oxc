//! Generator for `oxc_formatter`.
//!

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    Codegen, Generator,
    output::{Output, output_path},
    schema::{Def, Schema, TypeDef},
};

use super::define_generator;

const FORMATTER_CRATE_PATH: &str = "crates/oxc_formatter";

pub struct FormatterFormatGenerator;

define_generator!(FormatterFormatGenerator);

impl Generator for FormatterFormatGenerator {
    fn generate(&self, schema: &Schema, _codegen: &Codegen) -> Output {
        let impls = schema
            .types
            .iter()
            .filter(|type_def| match type_def {
                TypeDef::Struct(struct_def) => {
                    struct_def.visit.has_visitor() && !struct_def.builder.skip
                }
                TypeDef::Enum(enum_def) => enum_def.visit.has_visitor(),
                _ => false,
            })
            .map(|type_def| {
                let type_ident = type_def.ident();
                let type_ty = type_def.ty(schema);

                let has_kind = match type_def {
                    TypeDef::Struct(struct_def) => struct_def.kind.has_kind,
                    TypeDef::Enum(enum_def) => enum_def.kind.has_kind,
                    _ => unreachable!(),
                };

                if has_kind {
                    quote! {
                        ///@@line_break
                        impl<'a> Format<'a> for #type_ty {
                            fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
                                f.state_mut().stack.push(AstKind::#type_ident(hack(self)));
                                let result = self.write(f);
                                unsafe { f.state_mut().stack.pop_unchecked() };
                                result
                            }
                        }
                    }
                } else {
                    quote! {
                        ///@@line_break
                        impl<'a> Format<'a> for #type_ty {
                            fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
                                self.write(f)
                            }
                        }
                    }
                }
            })
            .collect::<TokenStream>();

        let output = quote! {
            #![allow(clippy::undocumented_unsafe_blocks)]

            use oxc_ast::{AstKind, ast::*};

            ///@@line_break
            use crate::{
                formatter::{Buffer, Format, FormatResult, Formatter},
                write,
                write::FormatWrite,
            };

            ///@@line_break
            /// A hack for erasing the lifetime requirement.
            pub fn hack<'ast, T>(t: &T) -> &'ast T {
                // SAFETY: This is not safe :-)
                unsafe { std::mem::transmute(t) }
            }

            #impls
        };

        Output::Rust { path: output_path(FORMATTER_CRATE_PATH, "format.rs"), tokens: output }
    }
}
