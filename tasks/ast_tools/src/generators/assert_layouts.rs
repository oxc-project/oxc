//! Calculate memory layout of all types.
//! Generate const assertions for the correctness of those calculations.
//!
//! Memory layouts are different on 64-bit and 32-bit platforms.
//! Calculate each separately, and generate assertions for each.

use std::{
    borrow::Cow,
    cmp::{Ordering, max, min},
    num,
    sync::atomic,
};

use phf_codegen::Map as PhfMapGen;
use proc_macro2::TokenStream;
use quote::quote;
use rustc_hash::FxHashMap;
use syn::{Expr, Ident, parse_str};

use crate::{
    AST_MACROS_CRATE_PATH, Codegen, Generator,
    output::{Output, output_path},
    schema::{
        Def, Discriminant, EnumDef, PointerKind, PrimitiveDef, Schema, StructDef, TypeDef, TypeId,
        Visibility,
        extensions::layout::{GetLayout, GetOffset, Layout, Niche, Offset, PlatformLayout},
    },
    utils::{format_cow, number_lit},
};

use super::define_generator;

/// Generator for memory layout assertions.
pub struct AssertLayouts;

define_generator!(AssertLayouts);

impl Generator for AssertLayouts {
    /// Calculate layouts of all types.
    fn prepare(&self, schema: &mut Schema, _codegen: &Codegen) {
        for type_id in schema.types.indices() {
            calculate_layout(type_id, schema);
        }
    }

    /// Generate assertions that calculated layouts are correct,
    /// and struct layout data for `oxc_ast_macros` crate.
    fn generate_many(&self, schema: &Schema, _codegen: &Codegen) -> Vec<Output> {
        let mut outputs = generate_assertions(schema);
        outputs.push(generate_struct_details(schema));
        outputs
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

    let span_type_id = schema.type_names["Span"];

    let type_def = &schema.types[type_id];
    match type_def {
        TypeDef::Struct(struct_def) => {
            if is_not_calculated(&struct_def.layout) {
                schema.struct_def_mut(type_id).layout =
                    calculate_layout_for_struct(type_id, span_type_id, schema);
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
        TypeDef::Pointer(pointer_def) => {
            if is_not_calculated(&pointer_def.layout) {
                schema.pointer_def_mut(type_id).layout =
                    calculate_layout_for_pointer(type_id, schema);
            }
            &schema.pointer_def(type_id).layout
        }
    }
}

/// Calculate layout for a struct.
///
/// All structs in AST are `#[repr(C)]`. In a `#[repr(C)]` struct, compiler does not re-order the fields,
/// so they are stored in memory in same order as they're defined.
///
/// So we determine a field order here which avoids excess padding. [`generate_struct_details`] generates
/// code describing this field order, and `#[ast]` macro re-orders the fields.
///
/// This gives us the stability guarantees of `#[repr(C)]` without the downside of structs which are
/// larger than they need to be due to excess padding.
///
/// Alignment of the struct is the highest alignment of its fields (or 1 if no fields).
/// Size of struct is a multiple of its alignment.
///
/// A struct has a niche if any of its fields has a niche. The niche will be the largest niche
/// in any of its fields. Padding bytes are not used as niches.
///
/// # Field order
///
/// Fields are ordered according to the following rules, in order:
///
/// 1. If field is `span: Span` it goes first.
/// 2. If field is ZST, it goes last.
/// 3. Fields with higher alignment on 64-bit systems go first.
/// 4. Fields with higher alignment on 32-bit systems go first.
/// 5. Otherwise, retain original field order.
///
/// This ordering scheme does not match `#[repr(Rust)]`, but is equally efficient in terms of packing
/// structs into as few bytes as possible.
///
/// `#[repr(Rust)]` would move fields with niches (e.g. `Expression`) earlier than fields without
/// niches (e.g. `Option<Box<T>>`), but we don't do that. AST is generally visited in source order,
/// so keeping fields in original order as much as possible is preferable for CPU caching.
///
/// `span: Span` is always first to make `Expression::get_span` etc branchless, because the `span` field
/// is in the same position for all of `Expression`'s variants.
///
/// Ordering by 64-bit alignment first, then 32-alignment creates a layout that is optimally packed
/// on both platforms. Fields will be ordered `u64`, `usize`, `u32`, so `usize` will have same alignment
/// as `u64` fields that come before it on 64-bit systems, and will have same alignment as
/// `u32`s that come after it on 32-bit systems. So it never results in padding on either platform.
///
/// Note: "usize" here also includes pointer-aligned types e.g. `Box`, `Vec`, `Atom`, `&str`.
/// "u64" includes other 8-byte aligned types e.g. `f64`, `Span`.
///
/// ZSTs need to go last so that they don't have same offset as another sized field.
/// If they did, this would screw up sorting the fields in `generate_struct_details`.
fn calculate_layout_for_struct(
    type_id: TypeId,
    span_type_id: TypeId,
    schema: &mut Schema,
) -> Layout {
    // Get layout of fields' types and calculate optimal field order
    struct FieldData {
        index: usize,
        layout: Layout,
        is_span: bool,
        is_zst: bool,
    }

    let mut field_order = schema
        .struct_def(type_id)
        .field_indices()
        .map(|index| {
            let field = &schema.struct_def(type_id).fields[index];
            let is_span = field.type_id == span_type_id && field.name() == "span";
            let layout = calculate_layout(field.type_id, schema);
            let is_zst = layout.layout_64.size == 0 && layout.layout_32.size == 0;
            FieldData { index, layout: layout.clone(), is_span, is_zst }
        })
        .collect::<Vec<_>>();

    field_order.sort_unstable_by(|f1, f2| {
        let mut order = f1.is_span.cmp(&f2.is_span).reverse();
        if order == Ordering::Equal {
            order = f1.is_zst.cmp(&f2.is_zst);
            if order == Ordering::Equal {
                order = f1.layout.layout_64.align.cmp(&f2.layout.layout_64.align).reverse();
                if order == Ordering::Equal {
                    order = f1.layout.layout_32.align.cmp(&f2.layout.layout_32.align).reverse();
                    if order == Ordering::Equal {
                        order = f1.index.cmp(&f2.index);
                    }
                }
            }
        }
        order
    });

    // Calculate offset of each field, and size + alignment of struct
    let mut layout_64 = PlatformLayout::from_size_align(0, 1);
    let mut layout_32 = PlatformLayout::from_size_align(0, 1);

    let struct_def = schema.struct_def_mut(type_id);
    for field_data in &field_order {
        fn update(
            layout: &mut PlatformLayout,
            field_layout: &PlatformLayout,
            struct_name: &str,
        ) -> u32 {
            // Field should already be aligned as we've re-ordered fields to ensure they're tightly packed.
            // This shouldn't break as long as all fields are aligned on `< 16`.
            // If we introduce a `u128` into AST, we might need to change the field ordering algorithm
            // to fill in the gap between `span` and the `u128` field.
            let offset = layout.size;
            assert!(
                offset.is_multiple_of(field_layout.align),
                "Incorrect alignment for struct fields in `{struct_name}`"
            );

            // Update alignment
            layout.align = max(layout.align, field_layout.align);

            // Update niche.
            // Take the largest niche. Preference for (in order):
            // * Largest single range of niche values.
            // * Largest number of niche values at start of range.
            // * Earlier field (earlier after re-ordering).
            if let Some(field_niche) = &field_layout.niche
                && layout.niche.as_ref().is_none_or(|niche| {
                    field_niche.count_max() > niche.count_max()
                        || (field_niche.count_max() == niche.count_max()
                            && field_niche.count_start > niche.count_start)
                })
            {
                let mut niche = field_niche.clone();
                niche.offset += offset;
                layout.niche = Some(niche);
            }

            // Next field starts after this one
            layout.size = offset + field_layout.size;

            // Return offset of this field
            offset
        }

        let offset_64 = update(&mut layout_64, &field_data.layout.layout_64, struct_def.name());
        let offset_32 = update(&mut layout_32, &field_data.layout.layout_32, struct_def.name());

        // Store offset on `field`
        let field = &mut struct_def.fields[field_data.index];
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
        let niche = Niche::new(0, 1, u32::from(niches_start), u32::from(niches_end));
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
            if niche.count_start == 0 {
                niche.count_end -= 1;
            } else {
                niche.count_start -= 1;
            }

            if niche.count_start == 0 && niche.count_end == 0 {
                layout.niche = None;
            }
        } else {
            layout.size += layout.align;
            layout.niche = Some(Niche::new(0, 1, 0, 254));
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
        layout_64: PlatformLayout::from_size_align_niche(8, 8, Niche::new(0, 8, 1, 0)),
        layout_32: PlatformLayout::from_size_align_niche(4, 4, Niche::new(0, 4, 1, 0)),
    }
}

/// Calculate layout for a `Vec`.
///
/// All `Vec`s have same layout, regardless of the inner type.
/// `Vec`s contain 4 x pointer-sized fields.
/// They have a single niche on the first field - the pointer which is `NonNull`.
fn calculate_layout_for_vec() -> Layout {
    Layout {
        layout_64: PlatformLayout::from_size_align_niche(24, 8, Niche::new(0, 8, 1, 0)),
        layout_32: PlatformLayout::from_size_align_niche(16, 4, Niche::new(0, 4, 1, 0)),
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

/// Calculate layout for a pointer.
///
/// `NonNull` pointers have a niche, `*const` and `*mut` have no niche.
fn calculate_layout_for_pointer(type_id: TypeId, schema: &Schema) -> Layout {
    let pointer_def = schema.pointer_def(type_id);
    if pointer_def.kind == PointerKind::NonNull {
        Layout {
            layout_64: PlatformLayout::from_size_align_niche(8, 8, Niche::new(0, 8, 1, 0)),
            layout_32: PlatformLayout::from_size_align_niche(4, 4, Niche::new(0, 4, 1, 0)),
        }
    } else {
        Layout {
            layout_64: PlatformLayout::from_size_align(8, 8),
            layout_32: PlatformLayout::from_size_align(4, 4),
        }
    }
}

/// Calculate layout for a primitive.
///
/// Primitives have varying layouts. Some have niches, most don't.
fn calculate_layout_for_primitive(primitive_def: &PrimitiveDef) -> Layout {
    // `&str` and `Atom` are a `NonNull` pointer + `usize` pair. Niche for 0 on the pointer field
    let str_layout = Layout {
        layout_64: PlatformLayout::from_size_align_niche(16, 8, Niche::new(0, 8, 1, 0)),
        layout_32: PlatformLayout::from_size_align_niche(8, 4, Niche::new(0, 4, 1, 0)),
    };
    // `usize` and `isize` are pointer-sized, but with no niche
    let usize_layout = Layout {
        layout_64: PlatformLayout::from_size_align(8, 8),
        layout_32: PlatformLayout::from_size_align(4, 4),
    };
    // `NonZeroUsize` and `NonZeroIsize` are pointer-sized, with a single niche
    let non_zero_usize_layout = Layout {
        layout_64: PlatformLayout::from_size_align_niche(8, 8, Niche::new(0, 8, 1, 0)),
        layout_32: PlatformLayout::from_size_align_niche(4, 4, Niche::new(0, 4, 1, 0)),
    };

    #[expect(clippy::match_same_arms)]
    match primitive_def.name() {
        "bool" => Layout::from_size_align_niche(1, 1, Niche::new(0, 1, 0, 254)),
        "u8" => Layout::from_type::<u8>(),
        "u16" => Layout::from_type::<u16>(),
        "u32" => Layout::from_type::<u32>(),
        "u64" => Layout::from_type::<u64>(),
        "u128" => Layout::from_type::<u128>(),
        "usize" => usize_layout,
        "i8" => Layout::from_type::<i8>(),
        "i16" => Layout::from_type::<i16>(),
        "i32" => Layout::from_type::<i32>(),
        "i64" => Layout::from_type::<i64>(),
        "i128" => Layout::from_type::<i128>(),
        "isize" => usize_layout,
        "f32" => Layout::from_type::<f32>(),
        "f64" => Layout::from_type::<f64>(),
        "&str" => str_layout,
        "Atom" => str_layout,
        "NonZeroU8" => Layout::from_type_with_niche_for_zero::<num::NonZeroU8>(),
        "NonZeroU16" => Layout::from_type_with_niche_for_zero::<num::NonZeroU16>(),
        "NonZeroU32" => Layout::from_type_with_niche_for_zero::<num::NonZeroU32>(),
        "NonZeroU64" => Layout::from_type_with_niche_for_zero::<num::NonZeroU64>(),
        "NonZeroU128" => Layout::from_type_with_niche_for_zero::<num::NonZeroU128>(),
        "NonZeroUsize" => non_zero_usize_layout,
        "NonZeroI8" => Layout::from_type_with_niche_for_zero::<num::NonZeroI8>(),
        "NonZeroI16" => Layout::from_type_with_niche_for_zero::<num::NonZeroI16>(),
        "NonZeroI32" => Layout::from_type_with_niche_for_zero::<num::NonZeroI32>(),
        "NonZeroI64" => Layout::from_type_with_niche_for_zero::<num::NonZeroI64>(),
        "NonZeroI128" => Layout::from_type_with_niche_for_zero::<num::NonZeroI128>(),
        "NonZeroIsize" => non_zero_usize_layout,
        // Unlike `bool`, `AtomicBool` does not have any niches
        "AtomicBool" => Layout::from_type::<atomic::AtomicBool>(),
        "AtomicU8" => Layout::from_type::<atomic::AtomicU8>(),
        "AtomicU16" => Layout::from_type::<atomic::AtomicU16>(),
        "AtomicU32" => Layout::from_type::<atomic::AtomicU32>(),
        "AtomicU64" => Layout::from_type::<atomic::AtomicU64>(),
        "AtomicUsize" => usize_layout,
        "AtomicI8" => Layout::from_type::<atomic::AtomicI8>(),
        "AtomicI16" => Layout::from_type::<atomic::AtomicI16>(),
        "AtomicI32" => Layout::from_type::<atomic::AtomicI32>(),
        "AtomicI64" => Layout::from_type::<atomic::AtomicI64>(),
        "AtomicIsize" => usize_layout,
        // `AtomicPtr` has no niche - like `*mut T`, not `NonNull<T>`
        "AtomicPtr" => usize_layout,
        "PointerAlign" => Layout {
            layout_64: PlatformLayout::from_size_align(0, 8),
            layout_32: PlatformLayout::from_size_align(0, 4),
        },
        // `NodeId` is a `NonMaxU32` wrapper with a niche for max value
        "NodeId" => Layout::from_size_align_niche(4, 4, Niche::new(0, 4, 1, 0)),
        name => panic!("Unknown primitive type: {name}"),
    }
}

/// Generate layout assertions for all types.
fn generate_assertions(schema: &Schema) -> Vec<Output> {
    let mut assertions = FxHashMap::default();

    for type_def in &schema.types {
        generate_layout_assertions(type_def, &mut assertions, schema);
    }

    assertions
        .into_iter()
        .map(|(krate, (assertions_64, assertions_32))| {
            let output = template(krate, &assertions_64, &assertions_32);

            let crate_path = if krate.starts_with("napi/") {
                Cow::Borrowed(krate)
            } else {
                format_cow!("crates/{krate}")
            };
            Output::Rust { path: output_path(&crate_path, "assert_layouts.rs"), tokens: output }
        })
        .collect()
}

/// Generate layout assertions for a type.
fn generate_layout_assertions<'s>(
    type_def: &TypeDef,
    assertions: &mut FxHashMap<&'s str, (/* 64 bit */ TokenStream, /* 32 bit */ TokenStream)>,
    schema: &'s Schema,
) {
    match type_def {
        TypeDef::Struct(struct_def) => {
            generate_layout_assertions_for_struct(struct_def, assertions, schema);
        }
        TypeDef::Enum(enum_def) => {
            generate_layout_assertions_for_enum(enum_def, assertions, schema);
        }
        _ => {}
    }
}

/// Generate layout assertions for a struct.
/// This includes size and alignment assertions, plus assertions about offset of fields.
fn generate_layout_assertions_for_struct<'s>(
    struct_def: &StructDef,
    assertions: &mut FxHashMap<&'s str, (/* 64 bit */ TokenStream, /* 32 bit */ TokenStream)>,
    schema: &'s Schema,
) {
    fn r#gen(
        struct_def: &StructDef,
        is_64: bool,
        struct_ident: &Ident,
        schema: &Schema,
    ) -> TokenStream {
        let layout = struct_def.platform_layout(is_64);

        let fields_total_bytes: u32 = struct_def
            .fields
            .iter()
            .map(|field| field.type_def(schema).platform_layout(is_64).size)
            .sum();
        let padding_bytes = layout.size - fields_total_bytes;
        let padding_comment = format!("@ Padding: {padding_bytes} bytes");
        let padding_comment = quote!( #[doc = #padding_comment] );

        let size_align_assertions = generate_size_align_assertions(layout, struct_ident);

        let offset_asserts = struct_def.fields.iter().filter_map(|field| {
            if struct_def.is_foreign || field.visibility == Visibility::Private {
                // Cannot create assertions for private fields (cant access them)
                // or foreign types (we don't know what fields they have)
                return None;
            }

            let field_ident = field.ident();
            let offset = number_lit(field.platform_offset(is_64));
            Some(quote! {
                assert!(offset_of!(#struct_ident, #field_ident) == #offset);
            })
        });

        quote! {
            ///@@line_break
            #padding_comment
            #size_align_assertions
            #(#offset_asserts)*
        }
    }

    let (assertions_64, assertions_32) =
        assertions.entry(struct_def.file(schema).krate()).or_default();

    let ident = struct_def.ident();
    assertions_64.extend(r#gen(struct_def, true, &ident, schema));
    assertions_32.extend(r#gen(struct_def, false, &ident, schema));
}

/// Generate layout assertions for an enum.
/// This is just size and alignment assertions.
fn generate_layout_assertions_for_enum<'s>(
    enum_def: &EnumDef,
    assertions: &mut FxHashMap<&'s str, (/* 64 bit */ TokenStream, /* 32 bit */ TokenStream)>,
    schema: &'s Schema,
) {
    let (assertions_64, assertions_32) =
        assertions.entry(enum_def.file(schema).krate()).or_default();

    let ident = enum_def.ident();
    add_line_break(assertions_64);
    assertions_64.extend(generate_size_align_assertions(enum_def.layout_64(), &ident));
    add_line_break(assertions_32);
    assertions_32.extend(generate_size_align_assertions(enum_def.layout_32(), &ident));
}

/// Generate size and alignment assertions for a type.
fn generate_size_align_assertions(layout: &PlatformLayout, ident: &Ident) -> TokenStream {
    let size = number_lit(layout.size);
    let align = number_lit(layout.align);
    quote! {
        assert!(size_of::<#ident>() == #size);
        assert!(align_of::<#ident>() == #align);
    }
}

/// Generate output for a crate.
fn template(krate: &str, assertions_64: &TokenStream, assertions_32: &TokenStream) -> TokenStream {
    #[expect(clippy::match_same_arms)]
    let imports = match krate {
        "oxc_ast" => quote! {
            use crate::ast::*;
        },
        "oxc_regular_expression" => quote! {
            use crate::ast::*;
        },
        "oxc_span" => quote! {
            use crate::*;
        },
        "oxc_syntax" => quote! {
            use nonmax::NonMaxU32;

            ///@@line_break
            use crate::{comment_node::*, module_record::*, number::*, operator::*, reference::*, scope::*, symbol::*};
        },
        "napi/parser" => quote! {
            use crate::raw_transfer_types::*;
        },
        _ => quote! {
            use crate::*;
        },
    };

    quote! {
        #![allow(unused_imports)]

        ///@@line_break
        use std::mem::{align_of, offset_of, size_of};

        ///@@line_break
        #imports

        ///@@line_break
        #[cfg(target_pointer_width = "64")]
        const _: () = { #assertions_64 };

        // Some 32-bit platforms have 8-byte alignment for `u64` and `f64`, while others have 4-byte alignment.
        //
        // Skip these assertions on 32-bit platforms where `u64` / `f64` have 4-byte alignment, because
        // some layout calculations may be incorrect. https://github.com/oxc-project/oxc/issues/13694
        //
        // At present 32-bit layouts aren't relied on by any code, so it's fine if they're incorrect for now.
        // However, raw transfer will be supported on WASM32 in future, and layout calculations are correct
        // for WASM32 at present. So keep these assertions for WASM, to ensure changes to AST or this codegen
        // don't break anything.
        ///@@line_break
        #[cfg(target_pointer_width = "32")]
        const _: () = if cfg!(target_family = "wasm") || align_of::<u64>() == 8 {
            #assertions_32
        };

        ///@@line_break
        #[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
        const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
    }
}

/// Generate struct field orders for `oxc_ast_macros` crate.
///
/// `#[ast]` macro will re-order struct fields in order we provide here.
fn generate_struct_details(schema: &Schema) -> Output {
    let mut map = PhfMapGen::new();
    for type_def in &schema.types {
        let TypeDef::Struct(struct_def) = type_def else { continue };

        // Get indexes of fields in ascending order of offset.
        // Field index is included in sorting so any ZST fields remain in same order as in source
        // (multiple ZST fields will have same offset as each other).
        //
        // If struct as written already has all fields in offset order then no-reordering is required,
        // in which case output `None`.
        let mut field_offsets_and_source_indexes = struct_def
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| (field.offset.offset_64, index))
            .collect::<Vec<_>>();
        field_offsets_and_source_indexes.sort_unstable();

        let field_order = if field_offsets_and_source_indexes
            .iter()
            .enumerate()
            .any(|(new_index, &(_, source_index))| source_index != new_index)
        {
            // Field order needs to change from source order.
            // Remap to `Vec` indexed by source field index,
            // with each entry containing the new index after re-ordering
            let mut source_and_new_indexes = field_offsets_and_source_indexes
                .into_iter()
                .enumerate()
                .map(|(new_index, (_, source_index))| (source_index, new_index))
                .collect::<Vec<_>>();
            source_and_new_indexes.sort_unstable_by_key(|&(source_index, _)| source_index);
            let new_indexes = source_and_new_indexes
                .into_iter()
                .map(|(_, new_index)| number_lit(u8::try_from(new_index).unwrap()));
            quote!(Some(&[#(#new_indexes),*]))
        } else {
            // Field order stays as it is in source
            quote!(None)
        };

        let details = quote!( StructDetails { field_order: #field_order } );

        map.entry(struct_def.name(), details.to_string());
    }
    let map = parse_str::<Expr>(&map.build().to_string()).unwrap();

    let code = quote! {
        use crate::ast::StructDetails;

        ///@@line_break
        /// Details of how `#[ast]` macro should modify structs.
        #[expect(clippy::unreadable_literal)]
        pub static STRUCTS: phf::Map<&'static str, StructDetails> = #map;
    };

    Output::Rust { path: output_path(AST_MACROS_CRATE_PATH, "structs.rs"), tokens: code }
}

/// Add a line break to [`TokenStream`].
fn add_line_break(tokens: &mut TokenStream) {
    tokens.extend(quote! {
        ///@@line_break
    });
}
