//! Generator for `oxc_formatter`.
//!

use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH, Codegen, Generator,
    output::{self, Output, output_path},
    schema::{Def, Schema, TypeDef},
    utils::number_lit,
};

use super::define_generator;

const FORMATTER_CRATE_PATH: &str = "crates/oxc_formatter";

pub struct FormatterGenerator;

define_generator!(FormatterGenerator);

impl Generator for FormatterGenerator {
    fn prepare(&self, schema: &mut Schema, _codegen: &Codegen) {}

    fn generate(&self, schema: &Schema, _codegen: &Codegen) -> Output {
        let output = quote! {};

        Output::Rust { path: output_path(FORMATTER_CRATE_PATH, "format.rs"), tokens: output }
    }
}
