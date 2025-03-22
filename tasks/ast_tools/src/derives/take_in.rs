//! Derive for `TakeIn` trait.

use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    codegen::Codegen,
    schema::{
        Def, EnumDef, Schema, StructDef, TypeDef, TypeId,
        extensions::{
            layout::GetLayout,
            take_in::{MinVariant, NOT_CALCULATED},
        },
    },
    utils::format_cow,
};

use super::{Derive, StructOrEnum, define_derive};

// TODO: Prefer 1 single allocation to 2 or more.
// TODO: Comments

/// Sentinel value for `alloc_bytes` indicating that it's currently being calculated.
/// Is set on a type before starting calculation for that type, to prevent infinite loops.
const CALCULATING: usize = usize::MAX - 1;

/// Derive for `TakeIn` trait.
pub struct DeriveTakeIn;

define_derive!(DeriveTakeIn);

impl Derive for DeriveTakeIn {
    fn trait_name(&self) -> &'static str {
        "TakeIn"
    }

    fn trait_has_lifetime(&self) -> bool {
        true
    }

    fn crate_name(&self) -> &'static str {
        "oxc_allocator"
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(unused_imports, unused_variables)]

            ///@@line_break
            use std::cell::Cell;

            ///@@line_break
            use oxc_allocator::{Allocator, Box, TakeIn, Vec};
        }
    }

    /// Initialize `take_in.alloc_bytes` on structs and enums
    fn prepare(&self, schema: &mut Schema, _codegen: &Codegen) {
        for type_id in schema.types.indices() {
            calculate_alloc_bytes(type_id, schema);
        }
    }

    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        match type_def {
            StructOrEnum::Struct(struct_def) => derive_struct(struct_def, schema),
            StructOrEnum::Enum(enum_def) => derive_enum(enum_def, schema),
        }
    }
}

fn calculate_alloc_bytes(type_id: TypeId, schema: &mut Schema) -> usize {
    let type_def = &mut schema.types[type_id];
    #[expect(clippy::match_same_arms)]
    match type_def {
        TypeDef::Struct(struct_def) => match struct_def.take_in.alloc_bytes {
            NOT_CALCULATED => {
                struct_def.take_in.alloc_bytes = CALCULATING;
                calculate_alloc_bytes_for_struct(type_id, schema)
            }
            CALCULATING => NOT_CALCULATED,
            alloc_bytes => alloc_bytes,
        },
        TypeDef::Enum(enum_def) => match enum_def.take_in.alloc_bytes {
            NOT_CALCULATED => {
                enum_def.take_in.alloc_bytes = CALCULATING;
                calculate_alloc_bytes_for_enum(type_id, schema)
            }
            CALCULATING => NOT_CALCULATED,
            alloc_bytes => alloc_bytes,
        },
        // Primitives own no further allocation beyond themselves
        TypeDef::Primitive(_) => 0,
        // `Option`s are `None` in dummy nodes
        TypeDef::Option(_) => 0,
        TypeDef::Box(box_def) => {
            let inner_type_id = box_def.inner_type_id;
            let alloc_bytes = calculate_alloc_bytes(inner_type_id, schema);
            if alloc_bytes == NOT_CALCULATED {
                NOT_CALCULATED
            } else {
                let inner_type_bytes = schema.types[inner_type_id].layout_64().size as usize;
                alloc_bytes + inner_type_bytes
            }
        }
        // `Vec`s are empty in dummy nodes
        TypeDef::Vec(_) => 0,
        // `Cell`s only own allocations if their inner type does
        TypeDef::Cell(cell_def) => calculate_alloc_bytes(cell_def.inner_type_id, schema),
    }
}

fn calculate_alloc_bytes_for_struct(type_id: TypeId, schema: &mut Schema) -> usize {
    let mut bytes = 0;
    for field_index in schema.struct_def(type_id).field_indices() {
        let field_type_id = schema.struct_def(type_id).fields[field_index].type_id;
        let field_bytes = calculate_alloc_bytes(field_type_id, schema);
        if field_bytes == NOT_CALCULATED {
            bytes = NOT_CALCULATED;
            break;
        }
        bytes += field_bytes;
    }

    schema.struct_def_mut(type_id).take_in.alloc_bytes = bytes;

    bytes
}

fn calculate_alloc_bytes_for_enum(type_id: TypeId, schema: &mut Schema) -> usize {
    // All `#[ast]` enums are `#[repr(u8)]` or `#[repr(C, u8)]` so cannot have 0 variants
    let mut bytes = NOT_CALCULATED;
    let mut min_variant = MinVariant::default();

    // Own variants
    for variant_index in schema.enum_def(type_id).variant_indices() {
        let variant_type_id = schema.enum_def(type_id).variants[variant_index].field_type_id;
        if let Some(variant_type_id) = variant_type_id {
            let variant_bytes = calculate_alloc_bytes(variant_type_id, schema);
            if variant_bytes < bytes {
                bytes = variant_bytes;
                min_variant = MinVariant::Own(variant_index);
            }
        } else {
            bytes = 0;
            min_variant = MinVariant::Own(variant_index);
            break;
        }
    }

    // Inherited variants
    for inherits_index in schema.enum_def(type_id).inherits_indices() {
        let inherits_type_id = schema.enum_def(type_id).inherits[inherits_index];
        let inherits_bytes = calculate_alloc_bytes_for_enum(inherits_type_id, schema);
        if inherits_bytes < bytes {
            bytes = inherits_bytes;
            min_variant = schema.enum_def(inherits_type_id).take_in.min_variant;
            if let MinVariant::Own(variant_index) = min_variant {
                min_variant = MinVariant::Inherited(inherits_type_id, variant_index);
            }
        }
    }

    let take_in = &mut schema.enum_def_mut(type_id).take_in;
    take_in.alloc_bytes = bytes;
    take_in.min_variant = min_variant;

    bytes
}

fn derive_struct(struct_def: &StructDef, schema: &Schema) -> TokenStream {
    let fields = struct_def.fields.iter().map(|field| {
        let field_ident = field.ident();
        let value = generate_dummy_value(field.type_def(schema), schema);
        quote!(#field_ident: #value)
    });

    let value = quote! {
        Self {
            #(#fields),*
        }
    };

    generate_impl(struct_def.name(), &struct_def.ty(schema), &value, struct_def.take_in.alloc_bytes)
}

fn derive_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    let variant = match enum_def.take_in.min_variant {
        MinVariant::Own(variant_index) => &enum_def.variants[variant_index],
        MinVariant::Inherited(inherited_type_id, variant_index) => {
            &schema.enum_def(inherited_type_id).variants[variant_index]
        }
    };

    let variant_ident = variant.ident();
    let value = if let Some(variant_type) = variant.field_type(schema) {
        let value = generate_dummy_value(variant_type, schema);
        quote!( Self::#variant_ident(#value) )
    } else {
        quote!( Self::#variant_ident )
    };

    generate_impl(enum_def.name(), &enum_def.ty(schema), &value, enum_def.take_in.alloc_bytes)
}

fn generate_dummy_value(type_def: &TypeDef, schema: &Schema) -> TokenStream {
    match type_def {
        TypeDef::Struct(_) | TypeDef::Enum(_) => quote!(TakeIn::dummy_in(allocator)),
        TypeDef::Primitive(primitive_def) => {
            match primitive_def.name() {
                "bool" => quote!(false),
                "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64"
                | "i128" | "isize" => quote!(0),
                "f32" | "f64" => quote!(0.0),
                name if name.starts_with("NonZero") => {
                    let ident = primitive_def.ident();
                    quote! {{
                        ///@ SAFETY: 1 is a valid value for `NonZero` type
                        unsafe { #ident::new_unchecked(1) }
                    }}
                }
                "&str" => quote!(""),
                "Atom" => quote!(oxc_span::Atom::from("")),
                "PointerAlign" => quote!(PointerAlign),
                name => panic!("Unknown primitive type: {name}"),
            }
        }
        TypeDef::Option(_) => quote!(None),
        TypeDef::Box(box_def) => {
            let inner_value = generate_dummy_value(box_def.inner_type(schema), schema);
            quote!( Box::new_in(#inner_value, allocator) )
        }
        TypeDef::Vec(_) => quote!(Vec::new_in(allocator)),
        TypeDef::Cell(cell_def) => {
            let inner_value = generate_dummy_value(cell_def.inner_type(schema), schema);
            quote!( Cell::new(#inner_value) )
        }
    }
}

fn generate_impl(
    name: &str,
    ty: &TokenStream,
    value: &TokenStream,
    alloc_bytes: usize,
) -> TokenStream {
    let comment1 = format!(" Create a dummy [`{name}`].");
    let comment2 = if alloc_bytes == 0 {
        Cow::Borrowed(" Does not allocate any data into arena.")
    } else {
        format_cow!(" Has cost of allocating {alloc_bytes} bytes into arena.")
    };

    quote! {
        impl<'a> TakeIn<'a> for #ty {
            #[doc = #comment1]
            #[doc = ""]
            #[doc = #comment2]
            fn dummy_in(allocator: &'a Allocator) -> Self {
                #value
            }
        }
    }
}
