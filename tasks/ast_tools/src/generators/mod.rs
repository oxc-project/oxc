use crate::{output::Output, Result, Schema};

mod assert_layouts;
mod ast_builder;
mod ast_kind;
mod get_id;
mod typescript;
mod visit;

pub use assert_layouts::AssertLayouts;
pub use ast_builder::AstBuilderGenerator;
pub use ast_kind::AstKindGenerator;
pub use get_id::GetIdGenerator;
pub use typescript::TypescriptGenerator;
pub use visit::{VisitGenerator, VisitMutGenerator};

pub trait Generator {
    // Methods which can/must be defined by implementer.

    /// Attributes on types that this derive uses.
    fn type_attrs(&self) -> &[&'static str] {
        &[]
    }

    /// Attributes on struct fields that this derive uses.
    fn field_attrs(&self) -> &[&'static str] {
        &[]
    }

    /// Attributes on enum variants that this derive uses.
    fn variant_attrs(&self) -> &[&'static str] {
        &[]
    }

    /// Generate output.
    fn generate(&mut self, schema: &Schema) -> Output;

    // Standard methods. Should not be overriden.

    fn output(&mut self, schema: &Schema) -> Result<Vec<Output>> {
        Ok(vec![self.generate(schema)])
    }
}

macro_rules! define_generator {
    ($ident:ident $($lifetime:lifetime)?) => {
        const _: () = {
            use $crate::{
                codegen::Runner,
                output::Output,
                schema::Schema,
                Result,
            };

            impl $($lifetime)? Runner for $ident $($lifetime)? {
                type Context = Schema;

                fn verb(&self) -> &'static str {
                    "Generate"
                }

                fn name(&self) -> &'static str {
                    stringify!($ident)
                }

                fn file_path(&self) -> &'static str {
                    file!()
                }

                fn run(&mut self, schema: &Schema) -> Result<Vec<Output>> {
                    self.output(schema)
                }
            }
        };
    };
}
pub(crate) use define_generator;
