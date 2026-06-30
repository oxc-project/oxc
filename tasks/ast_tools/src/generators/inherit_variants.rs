//! Generator for conversion methods between enums which inherit variants from other enums.
//!
//! Some AST enums "inherit" all the variants of another enum. e.g. `Expression` is inherited by
//! `ArrayExpressionElement`, `Argument`, and others. `Statement` inherits the variants of
//! `Declaration` and `ModuleDeclaration`.
//!
//! Where enum `Parent` inherits all the variants of enum `Child`, the 2 enums are laid out in memory
//! such that the shared variants have identical discriminants and field types. This allows zero-cost
//! conversion between the 2 types.
//!
//! This generator produces, for each such `Parent` / `Child` pair:
//!
//! * Methods on `Parent`: `is_child`, `into_child`, `as_child`, `as_child_mut`, `to_child`, `to_child_mut`
//! * `impl TryFrom<Parent> for Child`
//! * `impl From<Child> for Parent`
//! * Compile-time assertions that the discriminants of shared variants match between the 2 enums
//!
//! It also generates a `match_child!` macro for each enum which is inherited by another enum.
//! e.g. `match_expression!(ty)` expands to a match pattern covering all of `Expression`'s variants
//! (`ty::BooleanLiteral(_) | ty::NullLiteral(_) | ...`), for use in `match` arms and `matches!`.
//!
//! Note: The actual insertion of inherited variants into enum definitions, and calculation of variant
//! discriminants, is (currently) still handled by the `inherit_variants!` declarative macro.
//! This generator is intended to grow to take over those responsibilities too.

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH, AST_MACROS_CRATE_PATH, Codegen, Generator,
    output::{Output, output_path},
    schema::{Def, EnumDef, Schema, VariantDef},
    utils::{article_for, generate_phf_map},
};

use super::define_generator;

/// Generator for conversion methods between enums which inherit variants from other enums.
pub struct InheritVariantsGenerator;

define_generator!(InheritVariantsGenerator);

impl Generator for InheritVariantsGenerator {
    fn generate_many(&self, schema: &Schema, _codegen: &Codegen) -> Vec<Output> {
        vec![generate_inherit_variants(schema), generate_enum_details(schema)]
    }
}

/// Generate `inherit_variants.rs` in `oxc_ast` - conversion methods, trait impls and `match_*!` macros.
fn generate_inherit_variants(schema: &Schema) -> Output {
    let impls = generate_impls(schema);
    let match_macros = generate_match_macros(schema);

    let output = quote! {
        //!@ Some `TryFrom` impls have a single non-shared variant left for the catch-all arm
        #![expect(clippy::match_wildcard_for_single_variants)]

        ///@@line_break
        use std::{mem::ManuallyDrop, ptr::addr_of};

        ///@@line_break
        use oxc_allocator::ArenaBox;

        ///@@line_break
        use crate::ast::*;

        ///@@line_break
        #impls

        #match_macros
    };

    Output::Rust { path: output_path(AST_CRATE_PATH, "inherit_variants.rs"), tokens: output }
}

/// Generate `enums.rs` in `oxc_ast_macros` - the `ENUMS` data table the `#[ast]` macro reads
/// to decide how to modify each enum, without having to derive it from the enum on every compilation.
fn generate_enum_details(schema: &Schema) -> Output {
    let map = generate_phf_map(schema.enums().map(|enum_def| {
        let is_fieldless = enum_def.is_fieldless();
        let details = quote!( EnumDetails { is_fieldless: #is_fieldless } );
        (enum_def.name(), details)
    }));

    let code = quote! {
        use crate::ast::EnumDetails;

        ///@@line_break
        /// Details of how `#[ast]` macro should modify enums.
        #[expect(clippy::unreadable_literal)]
        pub static ENUMS: phf::Map<&'static str, EnumDetails> = #map;
    };

    Output::Rust { path: output_path(AST_MACROS_CRATE_PATH, "enums.rs"), tokens: code }
}

/// Generate conversion methods and trait impls for all enums which inherit from other enums.
fn generate_impls(schema: &Schema) -> TokenStream {
    let impls = schema.enums().flat_map(|enum_def| {
        enum_def.all_inherits(schema).map(|child| generate_conversions(enum_def, child, schema))
    });

    quote! {
        /// Macro to get discriminant of an enum.
        ///
        /// # SAFETY
        /// Enum must be `#[repr(C, u8)]` or using this macro is unsound.
        /// <https://doc.rust-lang.org/std/mem/fn.discriminant.html>
        macro_rules! discriminant {
            ($ty:ident :: $variant:ident) => {{
                #[expect(clippy::undocumented_unsafe_blocks)]
                unsafe {
                    let t = ManuallyDrop::new($ty::$variant(ArenaBox::dangling()));
                    *(addr_of!(t).cast::<u8>())
                }
            }};
        }

        #(#impls)*
    }
}

/// Generate a `match_child!` macro for every enum which is inherited by another enum.
fn generate_match_macros(schema: &Schema) -> TokenStream {
    let mut output = TokenStream::new();

    for enum_def in schema.enums() {
        if !enum_def.inherited_by.is_empty() {
            output.extend(generate_match_macro(enum_def, schema));
        }
    }

    output
}

/// Generate a `match_child!` macro for an enum.
///
/// e.g. for `Expression`:
///
/// ```ignore
/// macro_rules! match_expression {
///     ($ty:ident) => {
///         $ty::BooleanLiteral(_) | $ty::NullLiteral(_) | /* ...all other variants... */
///     };
/// }
/// ```
fn generate_match_macro(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    let macro_ident = format_ident!("match_{}", enum_def.snake_name());

    let patterns = enum_def.all_variants(schema).map(|variant| {
        let variant_ident = variant.ident();
        quote!( $ty::#variant_ident(_) )
    });

    let doc = format!(" Macro for matching [`{}`]'s variants.", enum_def.name());
    let mut inherited = enum_def.all_inherits(schema).map(EnumDef::name).peekable();
    let inherited_doc = inherited.peek().is_some().then(|| {
        let line = format!(" Includes variants inherited from [`{}`].", inherited.join("`], [`"));
        quote! {
            ///
            #[doc = #line]
        }
    });

    // `#[macro_export]` exports the macro at the crate root.
    // The `pub use` makes it an item of this module too, so `crate::ast` can glob re-export it too.
    quote! {
        ///@@line_break
        #[doc = #doc]
        #inherited_doc
        #[macro_export]
        macro_rules! #macro_ident {
            ($ty:ident) => {
                #(#patterns)|*
            };
        }
        pub use #macro_ident;
    }
}

/// Generate conversion methods and trait impls for converting between `parent` and `child`,
/// where `parent` inherits all of `child`'s variants.
fn generate_conversions(parent: &EnumDef, child: &EnumDef, schema: &Schema) -> TokenStream {
    let parent_ident = parent.ident();
    let child_ident = child.ident();
    let child_snake = child.snake_name();

    // The shared variants are all of `child`'s variants (including those `child` itself inherits)
    let variant_idents = child.all_variants(schema).map(VariantDef::ident).collect::<Vec<_>>();

    // Compile-time assertions that discriminants match for all shared variants between the 2 enums.
    // This guarantees the transmutes in `as_child` / `as_child_mut` are sound.
    let assertions = variant_idents.iter().map(|variant_ident| {
        let message = format!(
            "Non-matching discriminants for `{variant_ident}` between `{}` and `{}`",
            parent.name(),
            child.name(),
        );
        quote! {
            assert!(
                discriminant!(#parent_ident::#variant_ident) == discriminant!(#child_ident::#variant_ident),
                #message
            );
        }
    });

    let is_fn = format_ident!("is_{child_snake}");
    let into_fn = format_ident!("into_{child_snake}");
    let as_fn = format_ident!("as_{child_snake}");
    let as_mut_fn = format_ident!("as_{child_snake}_mut");
    let to_fn = format_ident!("to_{child_snake}");
    let to_mut_fn = format_ident!("to_{child_snake}_mut");

    let parent_name = parent.name();
    let child_name = child.name();
    let parent_article = article_for(parent_name);
    let child_article = article_for(child_name);

    let is_doc = format!(
        " Return if {parent_article} [`{parent_name}`] is {child_article} [`{child_name}`].",
    );
    let into_doc =
        format!(" Convert {parent_article} [`{parent_name}`] to {child_article} [`{child_name}`].");
    let from_doc =
        format!(" Convert {child_article} [`{child_name}`] to {parent_article} [`{parent_name}`].");

    let as_doc1 = format!(
        " Convert {parent_article} [`&{parent_name}`] to {child_article} [`&{child_name}`]."
    );
    let as_doc2 = format!(" [`&{parent_name}`]: {parent_name}");
    let as_doc3 = format!(" [`&{child_name}`]: {child_name}");

    let as_mut_doc1 = format!(
        " Convert {parent_article} [`&mut {parent_name}`] to {child_article} [`&mut {child_name}`]."
    );
    let as_mut_doc2 = format!(" [`&mut {parent_name}`]: {parent_name}");
    let as_mut_doc3 = format!(" [`&mut {child_name}`]: {child_name}");

    quote! {
        ///@@line_break
        const _: () = {
            #(#assertions)*
        };

        ///@@line_break
        impl<'a> #parent_ident<'a> {
            #[doc = #is_doc]
            #[inline]
            pub fn #is_fn(&self) -> bool {
                matches!(self, #(Self::#variant_idents(_))|*)
            }

            ///@@line_break
            #[doc = #into_doc]
            ///
            /// # Panics
            /// Panics if not convertible.
            #[inline]
            pub fn #into_fn(self) -> #child_ident<'a> {
                #child_ident::try_from(self).unwrap()
            }

            ///@@line_break
            #[doc = #as_doc1]
            ///
            #[doc = #as_doc2]
            #[doc = #as_doc3]
            #[inline]
            pub fn #as_fn(&self) -> Option<&#child_ident<'a>> {
                if self.#is_fn() {
                    ///@ SAFETY: Transmute is safe because discriminants + types are identical between
                    ///@ `parent` and `child` for the shared variants
                    Some(unsafe { &*std::ptr::from_ref(self).cast::<#child_ident>() })
                } else {
                    None
                }
            }

            ///@@line_break
            #[doc = #as_mut_doc1]
            ///
            #[doc = #as_mut_doc2]
            #[doc = #as_mut_doc3]
            #[inline]
            pub fn #as_mut_fn(&mut self) -> Option<&mut #child_ident<'a>> {
                if self.#is_fn() {
                    ///@ SAFETY: Transmute is safe because discriminants + types are identical between
                    ///@ `parent` and `child` for the shared variants
                    Some(unsafe { &mut *std::ptr::from_mut(self).cast::<#child_ident>() })
                } else {
                    None
                }
            }

            ///@@line_break
            #[doc = #as_doc1]
            ///
            /// # Panics
            /// Panics if not convertible.
            ///
            #[doc = #as_doc2]
            #[doc = #as_doc3]
            #[inline]
            pub fn #to_fn(&self) -> &#child_ident<'a> {
                self.#as_fn().unwrap()
            }

            ///@@line_break
            #[doc = #as_mut_doc1]
            ///
            /// # Panics
            /// Panics if not convertible.
            ///
            #[doc = #as_mut_doc2]
            #[doc = #as_mut_doc3]
            #[inline]
            pub fn #to_mut_fn(&mut self) -> &mut #child_ident<'a> {
                self.#as_mut_fn().unwrap()
            }
        }

        ///@@line_break
        impl<'a> TryFrom<#parent_ident<'a>> for #child_ident<'a> {
            type Error = ();

            ///@@line_break
            #[doc = #into_doc]
            ///
            /// # Errors
            /// Returns `Err` if not convertible.
            #[inline]
            fn try_from(value: #parent_ident<'a>) -> Result<Self, Self::Error> {
                ///@ Compiler should implement this as a check of discriminant and then zero-cost transmute,
                ///@ as discriminants for `parent` and `child` are aligned
                match value {
                    #(#parent_ident::#variant_idents(o) => Ok(#child_ident::#variant_idents(o)),)*
                    _ => Err(()),
                }
            }
        }

        ///@@line_break
        impl<'a> From<#child_ident<'a>> for #parent_ident<'a> {
            #[doc = #from_doc]
            #[inline]
            fn from(value: #child_ident<'a>) -> Self {
                ///@ Compiler should implement this as zero-cost transmute as discriminants
                ///@ for `child` and `parent` are aligned
                match value {
                    #(#child_ident::#variant_idents(o) => #parent_ident::#variant_idents(o),)*
                }
            }
        }
    }
}
