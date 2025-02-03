//! Calculate memory layout of all types.
//! Generate const assertions for the correctness of those calculations.
//!
//! Memory layouts are different on 64-bit and 32-bit platforms.
//! Calculate each separately, and generate assertions for each.

use std::cmp::{max, min};

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{
    output::{output_path, Output},
    schema::{
        extensions::layout::{Layout, Niche, Offset, PlatformLayout},
        Def, Discriminant, EnumDef, PrimitiveDef, Schema, StructDef, TypeDef, TypeId, Visibility,
    },
    Codegen, Generator,
};

use super::define_generator;

/// Generator for memory layout assertions.
pub struct AssertLayouts;

define_generator!(AssertLayouts);

impl Generator for AssertLayouts {
    /// Calculate layouts of all types.
    fn prepare(&self, schema: &mut Schema) {
        for type_id in schema.types.indices() {
            calculate_layout(type_id, schema);
        }
    }

    /// Generate assertions that calculated layouts are correct.
    fn generate(&self, schema: &Schema, _codegen: &Codegen) -> Output {
        let (assertions_64, assertions_32): (TokenStream, TokenStream) =
            schema.types.iter().map(generate_layout_assertions).unzip();

        let output = quote! {
            use std::mem::{align_of, offset_of, size_of};

            ///@@line_break
            use oxc_regular_expression::ast::*;

            ///@@line_break
            use crate::ast::*;

            ///@@line_break
            #[cfg(target_pointer_width = "64")]
            const _: () = { #assertions_64 };

            ///@@line_break
            #[cfg(target_pointer_width = "32")]
            const _: () = { #assertions_32 };

            ///@@line_break
            #[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
            const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
        };

        Output::Rust { path: output_path(crate::AST_CRATE, "assert_layouts.rs"), tokens: output }
    }
}

/// Calculate layout for a type.
///
/// If layout was calculated already, just return the existing `Layout`.
fn calculate_layout(type_id: TypeId, schema: &mut Schema) -> &Layout {
    fn is_not_calculated(layout: &Layout) -> bool {
        // `align` field is set to 0 initially, but that's an illegal value
        layout.layout_64.align == 0
    }

    let type_def = &schema.types[type_id];
    match type_def {
        TypeDef::Struct(struct_def) => {
            if is_not_calculated(&struct_def.layout) {
                schema.struct_def_mut(type_id).layout =
                    calculate_layout_for_struct(type_id, schema);
            }
            &schema.struct_def(type_id).layout
        }
        TypeDef::Enum(enum_def) => {
            if is_not_calculated(&enum_def.layout) {
                schema.enum_def_mut(type_id).layout = calculate_layout_for_enum(type_id, schema);
            }
            &schema.enum_def(type_id).layout
        }
        TypeDef::Primitive(primitive_def) => {
            if is_not_calculated(&primitive_def.layout) {
                schema.primitive_def_mut(type_id).layout =
                    calculate_layout_for_primitive(primitive_def);
            }
            &schema.primitive_def(type_id).layout
        }
        TypeDef::Option(option_def) => {
            if is_not_calculated(&option_def.layout) {
                schema.option_def_mut(type_id).layout =
                    calculate_layout_for_option(type_id, schema);
            }
            &schema.option_def(type_id).layout
        }
        TypeDef::Box(box_def) => {
            if is_not_calculated(&box_def.layout) {
                schema.box_def_mut(type_id).layout = calculate_layout_for_box();
            }
            &schema.box_def(type_id).layout
        }
        TypeDef::Vec(vec_def) => {
            if is_not_calculated(&vec_def.layout) {
                schema.vec_def_mut(type_id).layout = calculate_layout_for_vec();
            }
            &schema.vec_def(type_id).layout
        }
        TypeDef::Cell(cell_def) => {
            if is_not_calculated(&cell_def.layout) {
                schema.cell_def_mut(type_id).layout = calculate_layout_for_cell(type_id, schema);
            }
            &schema.cell_def(type_id).layout
        }
    }
}

/// Calculate layout for a struct.
///
/// All structs in AST are `#[repr(C)]`. In a `#[repr(C)]` struct, compiler does not re-order the fields,
/// so they are stored in memory in same order as they're defined.
///
/// Each field is aligned to the alignment of the field type. Padding bytes are added between fields
/// as necessary to ensure this.
///
/// Alignment of the struct is the highest alignment of its fields (or 1 if no fields).
/// Size of struct is a multiple of its alignment.
///
/// A struct has a niche if any of its fields has a niche. The niche will be the largest niche
/// in any of its fields. Padding bytes are not used as niches.
fn calculate_layout_for_struct(type_id: TypeId, schema: &mut Schema) -> Layout {
    let mut layout_64 = PlatformLayout::from_size_align(0, 1);
    let mut layout_32 = PlatformLayout::from_size_align(0, 1);

    for field_index in schema.struct_def(type_id).field_indices() {
        let field_type_id = schema.struct_def(type_id).fields[field_index].type_id;
        let field_layout = calculate_layout(field_type_id, schema);

        #[expect(clippy::items_after_statements)]
        fn update(layout: &mut PlatformLayout, field_layout: &PlatformLayout) -> u32 {
            // Field needs to be aligned
            let offset = layout.size.next_multiple_of(field_layout.align);

            // Update alignment
            layout.align = max(layout.align, field_layout.align);

            // Update niche.
            // Take the largest niche. Preference for earlier niche if 2 fields have niches of same size.
            if let Some(field_niche) = &field_layout.niche {
                if layout.niche.as_ref().is_none_or(|niche| field_niche.count > niche.count) {
                    let mut niche = field_niche.clone();
                    niche.offset += offset;
                    layout.niche = Some(niche);
                }
            }

            // Next field starts after this one
            layout.size = offset + field_layout.size;

            // Return offset of this field
            offset
        }

        let offset_64 = update(&mut layout_64, &field_layout.layout_64);
        let offset_32 = update(&mut layout_32, &field_layout.layout_32);

        // Store offset on `field`
        let field = &mut schema.struct_def_mut(type_id).fields[field_index];
        field.offset = Offset { offset_64, offset_32 };
    }

    // Round up size to alignment
    layout_64.size = layout_64.size.next_multiple_of(layout_64.align);
    layout_32.size = layout_32.size.next_multiple_of(layout_32.align);

    Layout { layout_64, layout_32 }
}

/// Calculate layout for an enum.
///
/// All enums in AST are `#[repr(C, u8)]` (if has fields) or `#[repr(u8)]` if fieldless.
///
/// `#[repr(C, u8)]` enums have alignment of highest-aligned variant.
/// Size is size of largest variant + alignment of highest-aligned variant.
///
/// Fieldless `#[repr(u8)]` enums obey the same rules. Fieldless variants act as size 0, align 1.
///
/// `#[repr(C, u8)]` and `#[repr(u8)]` enums must always have at least one variant.
///
/// Any unused discriminant values at start of end of the range form a niche.
fn calculate_layout_for_enum(type_id: TypeId, schema: &mut Schema) -> Layout {
    struct State {
        min_discriminant: Discriminant,
        max_discriminant: Discriminant,
        layout_64: PlatformLayout,
        layout_32: PlatformLayout,
    }

    fn process_variants(type_id: TypeId, state: &mut State, schema: &mut Schema) {
        let State { min_discriminant, max_discriminant, layout_64, layout_32 } = state;

        for variant_index in schema.enum_def(type_id).variant_indices() {
            let variant = &schema.enum_def(type_id).variants[variant_index];

            *min_discriminant = min(*min_discriminant, variant.discriminant);
            *max_discriminant = max(*max_discriminant, variant.discriminant);

            if let Some(variant_type_id) = variant.field_type_id {
                let variant_layout = calculate_layout(variant_type_id, schema);

                layout_64.size = max(layout_64.size, variant_layout.layout_64.size);
                layout_64.align = max(layout_64.align, variant_layout.layout_64.align);
                layout_32.size = max(layout_32.size, variant_layout.layout_32.size);
                layout_32.align = max(layout_32.align, variant_layout.layout_32.align);
            }
        }

        for inherits_index in schema.enum_def(type_id).inherits_indices() {
            let inherits_type_id = schema.enum_def(type_id).inherits[inherits_index];
            process_variants(inherits_type_id, state, schema);
        }
    }

    let mut state = State {
        min_discriminant: Discriminant::MAX,
        max_discriminant: 0,
        layout_64: PlatformLayout::from_size_align(0, 1),
        layout_32: PlatformLayout::from_size_align(0, 1),
    };
    process_variants(type_id, &mut state, schema);
    let State { min_discriminant, max_discriminant, mut layout_64, mut layout_32 } = state;

    layout_64.size += layout_64.align;
    layout_32.size += layout_32.align;

    // Any unused discriminant values at start of end of the range form a niche.
    // Note: The unused discriminants must be at start or end of range, *not* in the middle.
    // `#[repr(u8)] enum Foo { A = 0, B = 255 }` has no niche.
    // The largest available range (from start or from end) is used for the niche.
    let niches_start = min_discriminant;
    let niches_end = Discriminant::MAX - max_discriminant;

    if niches_start != 0 || niches_end != 0 {
        let is_range_start = niches_start >= niches_end;
        let count = u32::from(if is_range_start { niches_start } else { niches_end });
        let niche = Niche::new(0, 1, is_range_start, count);
        layout_64.niche = Some(niche.clone());
        layout_32.niche = Some(niche);
    }

    Layout { layout_64, layout_32 }
}

/// Calculate layout for an `Option`.
///
/// * If inner type has a niche:
///   `Option` uses that niche to represent `None`.
///   The `Option` is same size and alignment as the inner type.
/// * If inner type has no niche:
///   The `Option`'s size = inner type size + inner type alignment.
///   `Some` / `None` discriminant is stored as a `bool` in first byte.
///   This introduces a new niche, identical to a struct with `bool` as first field.
fn calculate_layout_for_option(type_id: TypeId, schema: &mut Schema) -> Layout {
    let option_def = schema.option_def(type_id);
    let inner_layout = calculate_layout(option_def.inner_type_id, schema);

    #[expect(clippy::items_after_statements)]
    fn consume_niche(layout: &mut PlatformLayout) {
        if let Some(niche) = &mut layout.niche {
            if niche.count == 1 {
                layout.niche = None;
            } else {
                niche.count -= 1;
            }
        } else {
            layout.size += layout.align;
            layout.niche = Some(Niche::new(0, 1, false, 254));
        }
    }

    let mut layout = inner_layout.clone();
    consume_niche(&mut layout.layout_64);
    consume_niche(&mut layout.layout_32);
    layout
}

/// Calculate layout for a `Box`.
///
/// All `Box`es have same layout, regardless of the inner type.
/// `Box`es are pointer-sized, with a single niche (like `NonNull`).
fn calculate_layout_for_box() -> Layout {
    Layout {
        layout_64: PlatformLayout::from_size_align_niche(8, 8, Niche::new(0, 8, true, 1)),
        layout_32: PlatformLayout::from_size_align_niche(4, 4, Niche::new(0, 4, true, 1)),
    }
}

/// Calculate layout for a `Vec`.
///
/// All `Vec`s have same layout, regardless of the inner type.
/// `Vec`s contain 4 x pointer-sized fields.
/// They have a single niche on the first field - the pointer which is `NonNull`.
fn calculate_layout_for_vec() -> Layout {
    Layout {
        layout_64: PlatformLayout::from_size_align_niche(32, 8, Niche::new(0, 8, true, 1)),
        layout_32: PlatformLayout::from_size_align_niche(16, 4, Niche::new(0, 4, true, 1)),
    }
}

/// Calculate layout for a `Cell`.
///
/// `Cell`s have same layout as their inner type, but with no niche.
fn calculate_layout_for_cell(type_id: TypeId, schema: &mut Schema) -> Layout {
    let cell_def = schema.cell_def(type_id);
    let inner_layout = calculate_layout(cell_def.inner_type_id, schema);

    let mut layout = inner_layout.clone();
    layout.layout_64.niche = None;
    layout.layout_32.niche = None;
    layout
}

/// Calculate layout for a primitive.
///
/// Primitives have varying layouts. Some have niches, most don't.
fn calculate_layout_for_primitive(primitive_def: &PrimitiveDef) -> Layout {
    // `ScopeId`, `SymbolId` and `ReferenceId` are a `NonZeroU32`, with a niche for 0
    let semantic_id_layout = Layout::from_size_align_niche(4, 4, Niche::new(0, 4, true, 1));
    // `&str` and `Atom` are a `NonNull` pointer + `usize` pair. Niche for 0 on the pointer field
    let str_layout = Layout {
        layout_64: PlatformLayout::from_size_align_niche(16, 8, Niche::new(0, 8, true, 1)),
        layout_32: PlatformLayout::from_size_align_niche(8, 4, Niche::new(0, 4, true, 1)),
    };
    // `usize` and `isize` are pointer-sized, but with no niche
    let usize_layout = Layout {
        layout_64: PlatformLayout::from_size_align(8, 8),
        layout_32: PlatformLayout::from_size_align(4, 4),
    };

    #[expect(clippy::match_same_arms)]
    match primitive_def.name() {
        "bool" => Layout::from_size_align_niche(1, 1, Niche::new(0, 1, false, 254)),
        "u8" => Layout::from_type::<u8>(),
        "u16" => Layout::from_type::<u16>(),
        "u32" => Layout::from_type::<u32>(),
        "u64" => Layout::from_type::<u64>(),
        "u128" => {
            panic!("Cannot calculate alignment for `u128`. It differs depending on Rust version.")
        }
        "usize" => usize_layout.clone(),
        "i8" => Layout::from_type::<i8>(),
        "i16" => Layout::from_type::<i16>(),
        "i32" => Layout::from_type::<i32>(),
        "i64" => Layout::from_type::<i64>(),
        "i128" => {
            panic!("Cannot calculate alignment for `i128`. It differs depending on Rust version.")
        }
        "isize" => usize_layout.clone(),
        "f32" => Layout::from_type::<f32>(),
        "f64" => Layout::from_type::<f64>(),
        "&str" => str_layout.clone(),
        "Atom" => str_layout,
        "ScopeId" => semantic_id_layout.clone(),
        "SymbolId" => semantic_id_layout.clone(),
        "ReferenceId" => semantic_id_layout,
        "PointerAlign" => Layout {
            layout_64: PlatformLayout::from_size_align(0, 8),
            layout_32: PlatformLayout::from_size_align(0, 4),
        },
        name => panic!("Unknown primitive type: {name}"),
    }
}

/// Generate layout assertions for a type
fn generate_layout_assertions(
    type_def: &TypeDef,
) -> (/* 64 bit */ TokenStream, /* 32 bit */ TokenStream) {
    match type_def {
        TypeDef::Struct(struct_def) => generate_layout_assertions_for_struct(struct_def),
        TypeDef::Enum(enum_def) => generate_layout_assertions_for_enum(enum_def),
        _ => (quote!(), quote!()),
    }
}

/// Generate layout assertions for a struct.
/// This includes size and alignment assertions, plus assertions about offset of fields.
fn generate_layout_assertions_for_struct(struct_def: &StructDef) -> (TokenStream, TokenStream) {
    fn gen(struct_def: &StructDef, is_64: bool, struct_ident: &Ident) -> TokenStream {
        let layout =
            if is_64 { &struct_def.layout.layout_64 } else { &struct_def.layout.layout_32 };

        let size_align_assertions = generate_size_align_assertions(layout, struct_ident);

        let offset_asserts = struct_def.fields.iter().filter_map(|field| {
            if field.visibility != Visibility::Public {
                // Cannot create assertions for fields which are not public, as assertions
                // are generated in `oxc_ast` crate, and those types are in other crates
                return None;
            }

            let field_ident = field.ident();
            let offset =
                if is_64 { field.offset.offset_64 } else { field.offset.offset_32 } as usize;
            // TODO: Don't print numbers as `4usize` - just `4` would be fine
            Some(quote! {
                assert!(offset_of!(#struct_ident, #field_ident) == #offset);
            })
        });

        quote! {
            #size_align_assertions
            #(#offset_asserts)*
        }
    }

    let ident = struct_def.ident();
    (gen(struct_def, true, &ident), gen(struct_def, false, &ident))
}

/// Generate layout assertions for an enum.
/// This is just size and alignment assertions.
fn generate_layout_assertions_for_enum(enum_def: &EnumDef) -> (TokenStream, TokenStream) {
    let ident = enum_def.ident();
    (
        generate_size_align_assertions(&enum_def.layout.layout_64, &ident),
        generate_size_align_assertions(&enum_def.layout.layout_32, &ident),
    )
}

/// Generate size and alignment assertions for a type.
fn generate_size_align_assertions(layout: &PlatformLayout, ident: &Ident) -> TokenStream {
    let size = layout.size as usize;
    let align = layout.align as usize;
    // TODO: Don't print numbers as `4usize` - just `4` would be fine
    quote! {
        ///@@line_break
        assert!(size_of::<#ident>() == #size);
        assert!(align_of::<#ident>() == #align);
    }
}
