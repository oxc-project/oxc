use crate::{
    output::Output,
    parse::attr::{attr_positions, AttrLocation, AttrPart, AttrPositions},
    Codegen, Result, Runner, Schema,
};

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
pub use visit::VisitGenerator;

/// Trait to define a generator.
pub trait Generator: Runner {
    // Methods which can/must be defined by implementer.

    /// Attributes that this generator uses.
    ///
    /// If this [`Generator`] handles any attributes, override this method to return the details of where
    /// those attributes can legally be used.
    ///
    /// [`parse_attr`] will be called with any attributes on structs/enums matching these patterns.
    ///
    /// e.g.:
    ///
    /// ```ignore
    /// fn attrs(&self) -> &[(&'static str, AttrPositions)] {
    ///     &[("clone_in", AttrPositions::StructField)]
    /// }
    /// ```
    ///
    /// ```ignore
    /// fn attrs(&self) -> &[(&'static str, AttrPositions)] {
    ///     &[
    ///         ("visit", attr_positions!(AstAttr | StructField | EnumVariant)),
    ///         ("scope", attr_positions!(Struct | Enum | StructField)),
    ///     ]
    /// }
    /// ```
    ///
    /// [`parse_attr`]: Generator::parse_attr
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[]
    }

    /// Parse an attribute part and record information from it on type definition.
    ///
    /// `parse_attr` will only be called with attributes which this [`Generator`] has registered
    /// its ownership of by returning their details from [`attrs`] method.
    ///
    /// * `attr_name` is name of the attribute.
    /// * `location` is location attribute appears (e.g. on a struct field).
    /// * `part` contains the details of this part of the attribute.
    ///
    /// e.g.:
    ///
    /// ```
    /// #[ast(visit)]
    /// #[estree(rename = "FooFoo")]
    /// struct Foo {
    ///   #[estree(skip, rename = "Blah")]
    ///   #[span]
    ///   blip: Bar,
    /// }
    /// ```
    ///
    /// `parse_attr` will be called 5 times, with arguments:
    ///
    /// * `"visit", AttrLocation::StructAstAttr(struct_def), AttrPart::None`
    /// * `"estree", AttrLocation::Struct(struct_def), AttrPart::String("rename", "FooFoo")`
    /// * `"estree", AttrLocation::StructField(struct_def, 0), AttrPart::Tag("skip")`
    /// * `"estree", AttrLocation::StructField(struct_def, 0), AttrPart::String("rename", "Blah")`
    /// * `"span", AttrLocation::StructField(struct_def, 0), AttrPart::None`
    ///
    /// [`attrs`]: Generator::attrs
    #[expect(unused_variables)]
    fn parse_attr(
        &self,
        attr_name: &str,
        location: AttrLocation<'_>,
        part: AttrPart<'_>,
    ) -> Result<()> {
        Ok(())
    }

    /// Prepare for generatation, modifying schema.
    ///
    /// Runs before any `generate` or `derive` method runs.
    #[expect(unused_variables)]
    fn prepare(&self, schema: &mut Schema) {}

    /// Generate single output.
    #[expect(unused_variables, clippy::unimplemented)]
    fn generate(&self, schema: &Schema, codegen: &Codegen) -> Output {
        unimplemented!()
    }

    /// Generate multiple outputs.
    fn generate_many(&self, schema: &Schema, codegen: &Codegen) -> Vec<Output> {
        vec![self.generate(schema, codegen)]
    }
}

/// Macro to implement [`Runner`] for a [`Generator`].
///
/// Must be used on every [`Generator`].
///
/// # Example
/// ```
/// struct AssertLayouts;
/// define_generator!(AssertLayouts);
/// ```
macro_rules! define_generator {
    ($ident:ident $($lifetime:lifetime)?) => {
        const _: () = {
            use $crate::{
                codegen::{Codegen, Runner},
                output::Output,
                schema::Schema,
                Result,
            };

            impl $($lifetime)? Runner for $ident $($lifetime)? {
                fn name(&self) -> &'static str {
                    stringify!($ident)
                }

                fn file_path(&self) -> &'static str {
                    file!()
                }

                fn run(&self, schema: &Schema, codegen: &Codegen) -> Result<Vec<Output>> {
                    Ok(self.generate_many(schema, codegen))
                }
            }
        };
    };
}
pub(crate) use define_generator;
