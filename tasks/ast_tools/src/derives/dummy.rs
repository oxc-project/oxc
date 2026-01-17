//! Derive for `Dummy` trait.

use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    codegen::Codegen,
    schema::{
        Def, EnumDef, Schema, StructDef, TypeDef, TypeId,
        extensions::{
            dummy::{Alloc, MinVariant},
            layout::GetLayout,
        },
    },
    utils::format_cow,
};

use super::{Derive, StructOrEnum, define_derive};

/// Derive for `Dummy` trait.
pub struct DeriveDummy;

define_derive!(DeriveDummy);

impl Derive for DeriveDummy {
    fn trait_name(&self) -> &'static str {
        "Dummy"
    }

    fn trait_has_lifetime(&self) -> bool {
        true
    }

    fn crate_name(&self) -> &'static str {
        "oxc_allocator"
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(unused_variables, clippy::inline_always)]

            ///@@line_break
            use oxc_allocator::{Allocator, Dummy};
        }
    }

    /// Initialize `dummy.alloc` on structs and enums
    fn prepare(&self, schema: &mut Schema, _codegen: &Codegen) {
        for type_id in schema.types.indices() {
            let alloc = calculate_alloc(type_id, schema);
            assert!(alloc != Alloc::NOT_CALCULATED);
        }
    }

    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        match type_def {
            StructOrEnum::Struct(struct_def) => generate_impl_for_struct(struct_def, schema),
            StructOrEnum::Enum(enum_def) => generate_impl_for_enum(enum_def, schema),
        }
    }
}

/// Calculate number and size of allocations required to construct a dummy for a type.
///
/// Before calculation, set `alloc` to [`Alloc::CALCULATING`].
/// If `calculate_alloc` is called again for that type, it returns [`Alloc::NOT_CALCULATED`],
/// which indicates the type has a circular dependency.
/// This should only happen while calculating the cost of an enum. That variant will not be chosen.
fn calculate_alloc(type_id: TypeId, schema: &mut Schema) -> Alloc {
    let type_def = &mut schema.types[type_id];
    #[expect(clippy::match_same_arms)]
    match type_def {
        TypeDef::Struct(struct_def) => {
            let alloc = &mut struct_def.dummy.alloc;
            if *alloc == Alloc::NOT_CALCULATED {
                *alloc = Alloc::CALCULATING;
                calculate_alloc_for_struct(type_id, schema)
            } else if *alloc == Alloc::CALCULATING {
                Alloc::NOT_CALCULATED
            } else {
                *alloc
            }
        }
        TypeDef::Enum(enum_def) => {
            let alloc = &mut enum_def.dummy.alloc;
            if *alloc == Alloc::NOT_CALCULATED {
                *alloc = Alloc::CALCULATING;
                calculate_alloc_for_enum(type_id, schema)
            } else if *alloc == Alloc::CALCULATING {
                Alloc::NOT_CALCULATED
            } else {
                *alloc
            }
        }
        // Primitives own no further allocation beyond themselves
        TypeDef::Primitive(_) => Alloc::ZERO,
        // `Option`s are `None` in dummy nodes
        TypeDef::Option(_) => Alloc::ZERO,
        // `Box`es have cost of the inner type plus an allocation of the type itself
        TypeDef::Box(box_def) => {
            let inner_type_id = box_def.inner_type_id;
            let mut alloc = calculate_alloc(inner_type_id, schema);
            if alloc != Alloc::NOT_CALCULATED {
                let inner_type = &schema.types[inner_type_id];
                alloc.bytes_64 += inner_type.layout_64().size;
                alloc.bytes_32 += inner_type.layout_32().size;
                alloc.count += 1;
            }
            alloc
        }
        // `Vec`s are empty in dummy nodes
        TypeDef::Vec(_) => Alloc::ZERO,
        // `Cell`s only own allocations if their inner type does
        TypeDef::Cell(cell_def) => calculate_alloc(cell_def.inner_type_id, schema),
        // Pointers cannot be created in dummies.
        // Pointers don't implement `Dummy`, so attempting to implement `Dummy` on a type containing
        // pointers will generate code which will not compile.
        TypeDef::Pointer(_) => Alloc::ZERO,
    }
}

/// Calculate number and size of allocations required to construct a dummy for a struct.
///
/// Equals the total of allocations for all the struct's fields.
fn calculate_alloc_for_struct(type_id: TypeId, schema: &mut Schema) -> Alloc {
    let mut alloc = Alloc::ZERO;
    for field_index in schema.struct_def(type_id).field_indices() {
        let field_type_id = schema.struct_def(type_id).fields[field_index].type_id;
        let field_alloc = calculate_alloc(field_type_id, schema);
        if field_alloc == Alloc::NOT_CALCULATED {
            alloc = field_alloc;
            break;
        }
        alloc.bytes_64 += field_alloc.bytes_64;
        alloc.bytes_32 += field_alloc.bytes_32;
        alloc.count += field_alloc.count;
    }

    schema.struct_def_mut(type_id).dummy.alloc = alloc;

    alloc
}

/// Calculate number and size of allocations required to construct a dummy for an enum.
///
/// Select the enum variant which has the lowest allocation cost.
///
/// Choice is made on these criteria, in order:
/// * Smallest number of bytes allocated on 64-bit systems.
/// * Smallest number of bytes allocated on 32-bit systems.
/// * Smallest number of individual allocations.
///
/// Record both the allocation cost, and which variant has the smallest cost.
fn calculate_alloc_for_enum(type_id: TypeId, schema: &mut Schema) -> Alloc {
    // All `#[ast]` enums are `#[repr(u8)]` or `#[repr(C, u8)]` so cannot have 0 variants
    let mut alloc = Alloc::NOT_CALCULATED;
    let mut min_variant = MinVariant::default();

    // Own variants
    for variant_index in schema.enum_def(type_id).variant_indices() {
        let variant_type_id = schema.enum_def(type_id).variants[variant_index].field_type_id;
        if let Some(variant_type_id) = variant_type_id {
            let variant_alloc = calculate_alloc(variant_type_id, schema);
            if variant_alloc < alloc {
                alloc = variant_alloc;
                min_variant = MinVariant::Own(variant_index);
            }
        } else {
            alloc = Alloc::ZERO;
            min_variant = MinVariant::Own(variant_index);
            break;
        }
    }

    // Inherited variants
    for inherits_index in schema.enum_def(type_id).inherits_indices() {
        let inherits_type_id = schema.enum_def(type_id).inherits[inherits_index];
        let inherits_alloc = calculate_alloc_for_enum(inherits_type_id, schema);
        if inherits_alloc < alloc {
            alloc = inherits_alloc;
            min_variant = schema.enum_def(inherits_type_id).dummy.min_variant;
            if let MinVariant::Own(variant_index) = min_variant {
                min_variant = MinVariant::Inherited(inherits_type_id, variant_index);
            }
        }
    }

    let dummy = &mut schema.enum_def_mut(type_id).dummy;
    dummy.alloc = alloc;
    dummy.min_variant = min_variant;

    alloc
}

/// Generate `Dummy` impl for struct.
fn generate_impl_for_struct(struct_def: &StructDef, schema: &Schema) -> TokenStream {
    let fields = struct_def.fields.iter().map(|field| {
        let field_ident = field.ident();
        // Special case: node_id uses NodeId::DUMMY instead of Dummy::dummy
        if field.name() == "node_id" {
            quote!(#field_ident: oxc_syntax::node::NodeId::DUMMY)
        } else {
            quote!(#field_ident: Dummy::dummy(allocator))
        }
    });

    let value = quote! {
        Self {
            #(#fields),*
        }
    };

    generate_impl(struct_def.name(), &struct_def.ty(schema), &value, struct_def.dummy.alloc, false)
}

/// Generate `Dummy` impl for enum.
fn generate_impl_for_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    let variant = match enum_def.dummy.min_variant {
        MinVariant::Own(variant_index) => &enum_def.variants[variant_index],
        MinVariant::Inherited(inherited_type_id, variant_index) => {
            &schema.enum_def(inherited_type_id).variants[variant_index]
        }
    };

    let variant_ident = variant.ident();
    let (value, should_inline) = if variant.field_type(schema).is_some() {
        (quote!( Self::#variant_ident(Dummy::dummy(allocator)) ), false)
    } else {
        (quote!( Self::#variant_ident ), true)
    };

    generate_impl(
        enum_def.name(),
        &enum_def.ty(schema),
        &value,
        enum_def.dummy.alloc,
        should_inline,
    )
}

/// Generate `Dummy` impl for a type.
fn generate_impl(
    name: &str,
    ty: &TokenStream,
    value: &TokenStream,
    alloc: Alloc,
    should_inline: bool,
) -> TokenStream {
    let comment1 = format!(" Create a dummy [`{name}`].");
    let comment2 = if alloc.bytes_64 == 0 {
        Cow::Borrowed(" Does not allocate any data into arena.")
    } else {
        let s = if alloc.count > 1 { "s" } else { "" };
        format_cow!(" Has cost of making {} allocation{s} ({} bytes).", alloc.count, alloc.bytes_64)
    };

    let inline = if should_inline { quote!( #[inline(always)] ) } else { quote!() };

    quote! {
        impl<'a> Dummy<'a> for #ty {
            #[doc = #comment1]
            #[doc = ""]
            #[doc = #comment2]
            #inline
            fn dummy(allocator: &'a Allocator) -> Self {
                #value
            }
        }
    }
}
