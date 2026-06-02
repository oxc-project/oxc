//! Calculate memory layout of all types.
//! Generate const assertions for the correctness of those calculations.
//!
//! Memory layouts are different on 64-bit and 32-bit platforms.
//! Calculate each separately, and generate assertions for each.

use std::{
    borrow::Cow,
    cmp::{max, min},
    num,
    sync::atomic,
};

use itertools::Itertools;
use phf_codegen::Map as PhfMapGen;
use proc_macro2::TokenStream;
use quote::quote;
use rustc_hash::FxHashMap;
use syn::{Expr, Ident, parse_str};

use crate::{
    AST_MACROS_CRATE_PATH, Codegen, Generator,
    output::{Output, output_path},
    schema::{
        Def, Discriminant, EnumDef, FieldDef, PointerKind, PrimitiveDef, Schema, StructDef,
        StructOrEnum, TypeDef, TypeId, Visibility,
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
        LayoutCalculator::calculate(schema);
    }

    /// Generate assertions that calculated layouts are correct,
    /// and struct layout data for `oxc_ast_macros` crate.
    fn generate_many(&self, schema: &Schema, _codegen: &Codegen) -> Vec<Output> {
        let mut outputs = generate_assertions(schema);
        outputs.push(generate_struct_details(schema));
        outputs
    }
}

/// Layout calculator.
struct LayoutCalculator<'s> {
    schema: &'s mut Schema,
    span_type_id: TypeId,
    node_id_cell_type_id: TypeId,
}

impl LayoutCalculator<'_> {
    /// Calculate layouts of all types.
    fn calculate(schema: &mut Schema) {
        let span_type_id = schema.type_names["Span"];
        let node_id_cell_type_id =
            schema.type_by_name("NodeId").as_struct().unwrap().containers.cell_id.unwrap();

        let mut calculator = LayoutCalculator { schema, span_type_id, node_id_cell_type_id };

        for type_id in calculator.schema.types.indices() {
            calculator.calculate_type(type_id);
        }
    }

    /// Calculate layout for a type.
    ///
    /// If layout was calculated already, just return the existing `Layout`.
    fn calculate_type(&mut self, type_id: TypeId) -> &Layout {
        fn is_not_calculated(layout: &Layout) -> bool {
            // `align` field is set to 0 initially, but that's an illegal value
            layout.layout_64.align == 0
        }

        let type_def = &self.schema.types[type_id];
        match type_def {
            TypeDef::Struct(struct_def) => {
                if is_not_calculated(&struct_def.layout) {
                    self.schema.struct_def_mut(type_id).layout = self.calculate_struct(type_id);
                }
                &self.schema.struct_def(type_id).layout
            }
            TypeDef::Enum(enum_def) => {
                if is_not_calculated(&enum_def.layout) {
                    self.schema.enum_def_mut(type_id).layout = self.calculate_enum(type_id);
                }
                &self.schema.enum_def(type_id).layout
            }
            TypeDef::Primitive(primitive_def) => {
                if is_not_calculated(&primitive_def.layout) {
                    self.schema.primitive_def_mut(type_id).layout =
                        Self::calculate_primitive(primitive_def);
                }
                &self.schema.primitive_def(type_id).layout
            }
            TypeDef::Option(option_def) => {
                if is_not_calculated(&option_def.layout) {
                    self.schema.option_def_mut(type_id).layout = self.calculate_option(type_id);
                }
                &self.schema.option_def(type_id).layout
            }
            TypeDef::Box(box_def) => {
                if is_not_calculated(&box_def.layout) {
                    self.schema.box_def_mut(type_id).layout = Self::calculate_box();
                }
                &self.schema.box_def(type_id).layout
            }
            TypeDef::Vec(vec_def) => {
                if is_not_calculated(&vec_def.layout) {
                    self.schema.vec_def_mut(type_id).layout = Self::calculate_vec();
                }
                &self.schema.vec_def(type_id).layout
            }
            TypeDef::Cell(cell_def) => {
                if is_not_calculated(&cell_def.layout) {
                    self.schema.cell_def_mut(type_id).layout = self.calculate_cell(type_id);
                }
                &self.schema.cell_def(type_id).layout
            }
            TypeDef::Pointer(pointer_def) => {
                if is_not_calculated(&pointer_def.layout) {
                    self.schema.pointer_def_mut(type_id).layout = self.calculate_pointer(type_id);
                }
                &self.schema.pointer_def(type_id).layout
            }
        }
    }
}

struct StructState<'s> {
    layout_64: PlatformLayout,
    layout_32: PlatformLayout,
    struct_def: &'s mut StructDef,
    field_count: u32,
    current_align_64: u32,
    current_align_32: u32,
}

struct FieldData {
    index: usize,
    layout: Layout,
    priority: u64,
}

const MAX_ALIGN: u32 = 1 << 31;

impl<'s> StructState<'s> {
    fn new(struct_def: &'s mut StructDef) -> Self {
        Self {
            layout_64: PlatformLayout::from_size_align(0, 1),
            layout_32: PlatformLayout::from_size_align(0, 1),
            struct_def,
            field_count: 0,
            current_align_64: MAX_ALIGN,
            current_align_32: MAX_ALIGN,
        }
    }

    fn add_field(&mut self, field_data: &FieldData) {
        let struct_name = self.struct_def.name();
        let offset_64 = Self::update(
            &mut self.layout_64,
            &field_data.layout.layout_64,
            &mut self.current_align_64,
            struct_name,
        );
        let offset_32 = Self::update(
            &mut self.layout_32,
            &field_data.layout.layout_32,
            &mut self.current_align_32,
            struct_name,
        );

        // Store offset on `field`
        let field = &mut self.struct_def.fields[field_data.index];
        field.offset = Offset { offset_64, offset_32, layout_index: self.field_count };

        self.field_count += 1;
    }

    fn update(
        layout: &mut PlatformLayout,
        field_layout: &PlatformLayout,
        current_align: &mut u32,
        struct_name: &str,
    ) -> u32 {
        // Check alignment is correct for this field
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
        if field_layout.size > 0 {
            let next_offset = offset + field_layout.size;
            layout.size = next_offset;

            // Get highest alignment next field can have
            *current_align = next_offset & next_offset.wrapping_neg();
        }

        // Return offset of this field
        offset
    }
}

impl LayoutCalculator<'_> {
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
    /// 1. `span: Span` goes first.
    /// 2. `node_id: Cell<NodeId>` goes next.
    /// 3. Fields with higher alignment on 64-bit systems go first.
    /// 4. Fields with higher alignment on 32-bit systems go first.
    /// 5. If the highest-aligned field cannot go next, take the next field which has suitable alignment.
    /// 6. Otherwise, retain original field order.
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
    /// `node_id: Cell<NodeId>` goes after `span: Span` for the same reason - so it has consistent position
    /// in all AST structs, which makes `AstKind::node_id` and `AstKind::set_node_id` branchless.
    ///
    /// Ordering by 64-bit alignment first, then 32-alignment creates a layout that is optimally packed
    /// on both platforms. Fields will be ordered `u64`, `usize`, `u32`, so `usize` will have same alignment
    /// as `u64` fields that come before it on 64-bit systems, and will have same alignment as
    /// `u32`s that come after it on 32-bit systems. So it never results in padding on either platform.
    ///
    /// Note: "usize" here also includes pointer-aligned types e.g. `Box`, `Vec`, `Str`, `&str`.
    /// "u64" includes other 8-byte aligned types e.g. `f64`, `Span`.
    fn calculate_struct(&mut self, type_id: TypeId) -> Layout {
        // Get layout of fields' types and calculate optimal field order
        let mut fields = self
            .schema
            .struct_def(type_id)
            .field_indices()
            .map(|index| {
                let field = &self.schema.struct_def(type_id).fields[index];
                let layout = self.calculate_type(field.type_id).clone();

                let field = &self.schema.struct_def(type_id).fields[index];
                let priority = if field.type_id == self.span_type_id && field.name() == "span" {
                    // Highest priority
                    u64::MAX
                } else if field.type_id == self.node_id_cell_type_id && field.name() == "node_id" {
                    // 2nd highest priority
                    u64::MAX - 1
                } else {
                    // Prioritize fields with higher alignment on 64-bit systems, then 32-bit systems
                    (u64::from(layout.layout_64.align) << 32) | u64::from(layout.layout_32.align)
                };

                FieldData { index, layout, priority }
            })
            .collect::<Vec<_>>();

        fields.sort_by(|f1, f2| f1.priority.cmp(&f2.priority).reverse());

        // Add fields
        let mut state = StructState::new(self.schema.struct_def_mut(type_id));

        while !fields.is_empty() {
            // Find next field with alignment <= current alignment for both 32 bit and 64 bit.
            // `other_fields` is sorted by alignment in descending order,
            // so this will pick the next field with largest alignment which can be placed at current offset.
            let index = fields.iter().position(|f| {
                f.layout.layout_64.align <= state.current_align_64
                    && f.layout.layout_32.align <= state.current_align_32
            });

            if let Some(index) = index {
                // Found field with suitable alignment - add it to struct
                let field_data = fields.remove(index);
                state.add_field(&field_data);
            } else {
                // No field with suitable alignment found.
                // Double desired alignment, and add padding so it's aligned.
                // Go round loop again to find next field which satisfies new looser alignment requirements.
                let smallest_align_64 = fields.last().unwrap().layout.layout_64.align;
                if state.current_align_64 < smallest_align_64 {
                    // No field meets alignment requirement for 64 bit
                    state.current_align_64 *= 2;
                    state.layout_64.size =
                        state.layout_64.size.next_multiple_of(state.current_align_64);
                } else {
                    // No field meets alignment requirement for 32 bit
                    state.current_align_32 *= 2;
                    state.layout_32.size =
                        state.layout_32.size.next_multiple_of(state.current_align_32);
                }
            }
        }

        // Round up size to alignment
        let StructState { mut layout_64, mut layout_32, .. } = state;
        layout_64.size = layout_64.size.next_multiple_of(layout_64.align);
        layout_32.size = layout_32.size.next_multiple_of(layout_32.align);

        Layout { layout_64, layout_32 }
    }
}

struct EnumState {
    min_discriminant: Discriminant,
    max_discriminant: Discriminant,
    layout_64: PlatformLayout,
    layout_32: PlatformLayout,
}

impl LayoutCalculator<'_> {
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
    fn calculate_enum(&mut self, type_id: TypeId) -> Layout {
        let mut state = EnumState {
            min_discriminant: Discriminant::MAX,
            max_discriminant: 0,
            layout_64: PlatformLayout::from_size_align(0, 1),
            layout_32: PlatformLayout::from_size_align(0, 1),
        };
        self.process_enum_variants(type_id, &mut state);
        let EnumState { min_discriminant, max_discriminant, mut layout_64, mut layout_32 } = state;

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

    fn process_enum_variants(&mut self, type_id: TypeId, state: &mut EnumState) {
        let EnumState { min_discriminant, max_discriminant, layout_64, layout_32 } = state;

        for variant_index in self.schema.enum_def(type_id).variant_indices() {
            let variant = &self.schema.enum_def(type_id).variants[variant_index];

            *min_discriminant = min(*min_discriminant, variant.discriminant);
            *max_discriminant = max(*max_discriminant, variant.discriminant);

            if let Some(variant_type_id) = variant.field_type_id {
                let variant_layout = self.calculate_type(variant_type_id);

                layout_64.size = max(layout_64.size, variant_layout.layout_64.size);
                layout_64.align = max(layout_64.align, variant_layout.layout_64.align);
                layout_32.size = max(layout_32.size, variant_layout.layout_32.size);
                layout_32.align = max(layout_32.align, variant_layout.layout_32.align);
            }
        }

        for inherits_index in self.schema.enum_def(type_id).inherits_indices() {
            let inherits_type_id = self.schema.enum_def(type_id).inherits[inherits_index];
            self.process_enum_variants(inherits_type_id, state);
        }
    }
}

impl LayoutCalculator<'_> {
    /// Calculate layout for an `Option`.
    ///
    /// * If inner type has a niche:
    ///   `Option` uses that niche to represent `None`.
    ///   The `Option` is same size and alignment as the inner type.
    /// * If inner type has no niche:
    ///   The `Option`'s size = inner type size + inner type alignment.
    ///   `Some` / `None` discriminant is stored as a `bool` in first byte.
    ///   This introduces a new niche, identical to a struct with `bool` as first field.
    fn calculate_option(&mut self, type_id: TypeId) -> Layout {
        let option_def = self.schema.option_def(type_id);
        let inner_layout = self.calculate_type(option_def.inner_type_id);

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
    fn calculate_box() -> Layout {
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
    fn calculate_vec() -> Layout {
        Layout {
            layout_64: PlatformLayout::from_size_align_niche(24, 8, Niche::new(0, 8, 1, 0)),
            layout_32: PlatformLayout::from_size_align_niche(16, 4, Niche::new(0, 4, 1, 0)),
        }
    }

    /// Calculate layout for a `Cell`.
    ///
    /// `Cell`s have same layout as their inner type, but with no niche.
    fn calculate_cell(&mut self, type_id: TypeId) -> Layout {
        let cell_def = self.schema.cell_def(type_id);
        let inner_layout = self.calculate_type(cell_def.inner_type_id);

        let mut layout = inner_layout.clone();
        layout.layout_64.niche = None;
        layout.layout_32.niche = None;
        layout
    }

    /// Calculate layout for a pointer.
    ///
    /// `NonNull` pointers have a niche, `*const` and `*mut` have no niche.
    fn calculate_pointer(&self, type_id: TypeId) -> Layout {
        let pointer_def = self.schema.pointer_def(type_id);
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
    fn calculate_primitive(primitive_def: &PrimitiveDef) -> Layout {
        // `&str` and `Str` are a `NonNull` pointer + `usize` pair. Niche for 0 on the pointer field
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
            "Str" => str_layout,
            // `Ident` is `NonNull<u8>` + `u64` on 64-bit, `NonNull<u8>` + `u32` + `u32` on 32-bit.
            // Niche for 0 on the pointer field.
            "Ident" => Layout {
                layout_64: PlatformLayout::from_size_align_niche(16, 8, Niche::new(0, 8, 1, 0)),
                layout_32: PlatformLayout::from_size_align_niche(12, 4, Niche::new(0, 4, 1, 0)),
            },
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
            name => panic!("Unknown primitive type: {name}"),
        }
    }
}

/// Generate layout assertions for all types.
fn generate_assertions(schema: &Schema) -> Vec<Output> {
    let mut assertions = FxHashMap::default();

    for type_def in schema.structs_and_enums() {
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
    type_def: StructOrEnum<'s>,
    assertions: &mut FxHashMap<&'s str, (/* 64 bit */ TokenStream, /* 32 bit */ TokenStream)>,
    schema: &'s Schema,
) {
    match type_def {
        StructOrEnum::Struct(struct_def) => {
            generate_layout_assertions_for_struct(struct_def, assertions, schema);
        }
        StructOrEnum::Enum(enum_def) => {
            generate_layout_assertions_for_enum(enum_def, assertions, schema);
        }
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
        fields: &[&FieldDef],
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

        let offset_asserts = fields.iter().map(|&field| {
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

    // Sort fields in memory layout order
    let mut fields = struct_def.fields.iter().collect_vec();
    fields.sort_by_key(|f1| f1.offset.layout_index);

    let (assertions_64, assertions_32) =
        assertions.entry(struct_def.file(schema).krate()).or_default();

    let ident = struct_def.ident();
    assertions_64.extend(r#gen(struct_def, &fields, true, &ident, schema));
    assertions_32.extend(r#gen(struct_def, &fields, false, &ident, schema));
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
            use crate::{module_record::*, node::*, number::*, operator::*, reference::*, scope::*, symbol::*};
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
    for struct_def in schema.structs() {
        // Get layout indexes of fields in source order.
        // If struct as written already has fields in layout order, then no-reordering is required,
        // in which case output `None`.
        let mut new_indexes = vec![0; struct_def.fields.len()];
        let mut reordering_needed = false;
        for (index, field) in struct_def.fields.iter().enumerate() {
            if index != field.offset.layout_index as usize {
                reordering_needed = true;
            }
            new_indexes[index] = field.offset.layout_index;
        }

        let field_order = if reordering_needed {
            let new_indexes =
                new_indexes.into_iter().map(|index| number_lit(u8::try_from(index).unwrap()));
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
