use std::borrow::Cow;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use rustc_hash::{FxHashMap, FxHashSet};
use syn::{Path, parse_str};

use crate::{
    Codegen, Result, Runner,
    output::{Output, output_path},
    parse::attr::{AttrLocation, AttrPart, AttrPositions, attr_positions},
    schema::{Def, FileId, Schema, StructOrEnum},
    utils::format_cow,
};

mod clone_in;
mod content_eq;
mod dummy;
pub mod estree;
mod get_address;
mod get_span;
mod take_in;
mod unstable_address;

pub use clone_in::DeriveCloneIn;
pub use content_eq::DeriveContentEq;
pub use dummy::DeriveDummy;
pub use estree::DeriveESTree;
pub use get_address::DeriveGetAddress;
pub use get_span::{DeriveGetSpan, DeriveGetSpanMut};
pub use take_in::DeriveTakeIn;
pub use unstable_address::DeriveUnstableAddress;

/// Trait to define a derive.
pub trait Derive: Runner {
    // Methods which can/must be defined by implementer.

    /// Get trait name.
    fn trait_name(&self) -> &'static str;

    /// Get if trait has lifetime.
    ///
    /// Default to `false`, but can be overridden.
    fn trait_has_lifetime(&self) -> bool {
        false
    }

    /// Get crate trait is defined in.
    fn crate_name(&self) -> &'static str;

    /// Get snake case trait name.
    ///
    /// Defaults to `trait_name()` converted to snake case.
    /// Can be overridden.
    fn snake_name(&self) -> String {
        self.trait_name().to_case(Case::Snake)
    }

    /// Attributes that this derive uses.
    ///
    /// If this [`Derive`] handles any attributes, override this method to return the details of where
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
    /// [`parse_attr`]: Derive::parse_attr
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[]
    }

    /// Parse an attribute part and record information from it on type definition.
    ///
    /// `parse_attr` will only be called with attributes which this [`Derive`] has registered
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
    /// [`attrs`]: Derive::attrs
    #[expect(unused_variables)]
    fn parse_attr(
        &self,
        attr_name: &str,
        location: AttrLocation<'_>,
        part: AttrPart<'_>,
    ) -> Result<()> {
        Ok(())
    }

    /// Generate prelude to be output at top of generated files.
    ///
    /// Defaults to no prelude.
    /// Can be overridden.
    fn prelude(&self) -> TokenStream {
        quote!()
    }

    /// Prepare for generatation, modifying schema.
    ///
    /// Runs before any `generate` or `derive` method runs.
    #[expect(unused_variables)]
    fn prepare(&self, schema: &mut Schema, codegen: &Codegen) {}

    /// Generate trait implementation for a type.
    fn derive(&self, type_def: StructOrEnum<'_>, schema: &Schema) -> TokenStream;

    // Standard methods. Should not be overridden.

    /// Run derive on all types which derive the trait, and compile into 1 file per crate.
    fn output(&self, schema: &Schema, codegen: &Codegen) -> Vec<Output> {
        #[derive(Default)]
        struct CrateContent {
            import_file_ids: FxHashSet<FileId>,
            output: TokenStream,
        }

        // Run derive on all types which has `#[generate_derive]` attr for this trait.
        // Store results in a hash map indexed by crate name.
        let derive_id = codegen.get_derive_id_by_name(self.trait_name());

        let mut crate_contents = FxHashMap::<&str, CrateContent>::default();
        for type_def in schema.structs_and_enums() {
            let (derived, file_id) = match type_def {
                StructOrEnum::Struct(struct_def) if struct_def.generates_derive(derive_id) => {
                    let derived = self.derive(type_def, schema);
                    (derived, struct_def.file_id)
                }
                StructOrEnum::Enum(enum_def) if enum_def.generates_derive(derive_id) => {
                    let derived = self.derive(type_def, schema);
                    (derived, enum_def.file_id)
                }
                _ => continue,
            };

            let content = crate_contents.entry(schema.files[file_id].krate()).or_default();
            content.import_file_ids.insert(file_id);

            content.output.extend(quote! {
                ///@@line_break
                #derived
            });
        }

        // Generate an output for each crate.
        // Wrap each output in template with `use` statements to import types which were derived.
        let filename = format!("derive_{}.rs", self.snake_name());
        crate_contents
            .into_iter()
            .map(|(krate, content)| {
                let mut import_paths = content
                    .import_file_ids
                    .into_iter()
                    .map(|file_id| schema.files[file_id].import_path())
                    .collect::<Vec<_>>();
                import_paths.sort_unstable();

                let crate_path = if krate.starts_with("napi/") {
                    Cow::Borrowed(krate)
                } else {
                    format_cow!("crates/{krate}")
                };

                Output::Rust {
                    path: output_path(&crate_path, &filename),
                    tokens: self.template(&import_paths, content.output),
                }
            })
            .collect()
    }

    /// Wrap derived output for a crate in template.
    /// Add prelude, and `use` statements to import types which were derived.
    fn template(&self, import_paths: &[&str], impls: TokenStream) -> TokenStream {
        let prelude = self.prelude();

        let use_modules = import_paths.iter().map(|import_path| {
            if import_path.is_empty() {
                quote! {
                    use crate::*;
                }
            } else {
                // `::ast::js` -> `use crate::ast::js::*;`
                let import_path: Path = parse_str(import_path).unwrap();
                quote!( use crate #import_path ::*; )
            }
        });

        quote! {
            #prelude

            ///@@line_break
            #(#use_modules)*

            ///@@line_break
            #impls
        }
    }
}

/// Macro to implement [`Runner`] for a [`Derive`].
///
/// Must be used on every [`Derive`].
///
/// # Example
/// ```
/// struct DeriveCloneIn;
/// define_derive!(DeriveCloneIn);
/// ```
macro_rules! define_derive {
    ($ident:ident $($lifetime:lifetime)?) => {
        const _: () = {
            use $crate::{Output, Runner, Schema, Result, Codegen};

            impl $($lifetime)? Runner for $ident $($lifetime)? {
                fn name(&self) -> &'static str {
                    stringify!($ident)
                }

                fn file_path(&self) -> &'static str {
                    file!()
                }

                fn run(&self, schema: &Schema, codegen: &Codegen) -> Result<Vec<Output>> {
                    Ok(self.output(schema, codegen))
                }
            }
        };
    };
}
pub(crate) use define_derive;
