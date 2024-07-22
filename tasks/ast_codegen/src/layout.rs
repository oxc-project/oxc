/// We use compiler to infer 64bit type layouts.
#[cfg(not(target_pointer_width = "64"))]
compile_error!("`oxc_ast_codegen::layout` only supports 64 architectures.");
use std::collections::HashMap;

use itertools::Itertools;
use lazy_static::lazy_static;
use quote::ToTokens;
use syn::Type;

use crate::{
    schema::{REnum, RStruct, RType},
    util::{NormalizeError, TypeAnalyzeResult, TypeExt, TypeWrapper},
    CodegenCtx, Result, TypeRef,
};

/// Calculates the layout of `ty` by mutating it.
/// Returns `false` if the layout is unknown at this point.
pub fn calc_layout(ty: &mut RType, ctx: &CodegenCtx) -> Result<bool> {
    let unknown_layout = ty
        .layout_32()
        .and_then(|x32| ty.layout_64().map(|x64| PlatformLayout(x64, x32)))
        .is_ok_and(|pl| pl.is_unknown());
    let layout = match ty {
        RType::Enum(enum_) if unknown_layout => calc_enum_layout(enum_, ctx),
        RType::Struct(struct_) if unknown_layout => calc_struct_layout(struct_, ctx),
        _ => return Ok(true),
    }?;
    if layout.is_unknown() {
        Ok(false)
    } else {
        ty.set_layout(layout.x64(), layout.x32())?;
        Ok(true)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Layout {
    #[default]
    Unknown,
    Layout(KnownLayout),
}

impl Layout {
    const fn known(size: usize, align: usize, niches: u128) -> Self {
        Self::Layout(KnownLayout { size, align, niches })
    }

    pub fn layout(self) -> Option<KnownLayout> {
        if let Self::Layout(layout) = self {
            Some(layout)
        } else {
            None
        }
    }
}

impl From<KnownLayout> for Layout {
    fn from(layout: KnownLayout) -> Self {
        Self::Layout(layout)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct KnownLayout {
    size: usize,
    align: usize,
    /// number of available niches
    niches: u128,
}

impl KnownLayout {
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn align(&self) -> usize {
        self.align
    }

    /// number of available niches
    #[inline]
    pub fn niches(&self) -> u128 {
        self.niches
    }

    /// Panics
    /// if doesn't have enough viable space and `can_resize` is false
    pub fn consume_niches(&mut self, n: u128, can_resize: bool) {
        if self.niches() >= n {
            self.niches -= 1;
        } else if can_resize {
            let align = self.align();
            self.size += align;
            self.niches += match align {
                1 => u8::MAX as u128,
                2 => u16::MAX as u128,
                4 => u32::MAX as u128,
                8 => u64::MAX as u128,
                16 => u128::MAX,
                _ => unreachable!("We do not support paddings bigger than 16 bytes."),
            };
            self.consume_niches(n, can_resize);
        } else {
            panic!("`{}` called on a layout without enough space.", stringify!(consume_niches));
        }
    }

    pub fn unpack(self) -> (/* size */ usize, /* align */ usize) {
        let Self { size, align, .. } = self;
        (size, align)
    }
}

impl Layout {
    /// # Panics
    /// If alignment of `T` is higher than one byte.
    const fn of<T>() -> Self {
        // TODO: find a better way of calculating this.
        struct N1<T>(Option<T>);
        struct N2<T>(N1<N1<T>>);
        struct N3<T>(N1<N2<T>>);
        struct N4<T>(N1<N3<T>>);
        struct N5<T>(N1<N4<T>>);
        struct N6<T>(N1<N5<T>>);
        struct N7<T>(N1<N6<T>>);
        struct N8<T>(N1<N7<T>>);

        let size = size_of::<T>();
        let align = align_of::<T>();
        let niches = if size_of::<N1<T>>() > size {
            0
        } else if size_of::<N2<T>>() > size {
            1
        } else if size_of::<N3<T>>() > size {
            2
        } else if size_of::<N4<T>>() > size {
            3
        } else if size_of::<N5<T>>() > size {
            4
        } else if size_of::<N6<T>>() > size {
            5
        } else if size_of::<N7<T>>() > size {
            6
        } else if size_of::<N8<T>>() > size {
            7
        } else if size_of::<N8<T>>() == size {
            8
        } else {
            panic!(
                "Alignment of `T` is bigger than what this method can calculate the headroom for."
            );
        };
        // NOTE: some or all of `niches` might be `trailing_pad` but we don't need to
        // distinguish between them. This method is only used to get layout info of simple types.
        // most of them are builtin primitives.
        Self::known(size, align, niches)
    }

    const fn zero() -> Self {
        #[repr(C)]
        struct Empty;
        Self::of::<Empty>()
    }

    const fn ptr_32() -> Self {
        Layout::known(4, 4, 0)
    }

    const fn ptr_64() -> Self {
        Layout::known(8, 8, 0)
    }

    const fn wide_ptr_32() -> Self {
        Layout::known(8, 4, 1)
    }

    const fn wide_ptr_64() -> Self {
        Layout::of::<&str>()
    }

    fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

#[derive(Debug, Clone)]
struct PlatformLayout(/* 64 */ Layout, /* 32 */ Layout);

impl PlatformLayout {
    const UNKNOWN: Self = Self(Layout::Unknown, Layout::Unknown);

    const fn zero() -> Self {
        Self(Layout::zero(), Layout::zero())
    }

    const fn ptr() -> Self {
        Self(Layout::ptr_64(), Layout::ptr_32())
    }

    const fn wide_ptr() -> Self {
        Self(Layout::wide_ptr_64(), Layout::wide_ptr_32())
    }

    fn x64(&self) -> Layout {
        self.0
    }

    fn x32(&self) -> Layout {
        self.1
    }

    /// Return `true` if either of platform layouts is unknown.
    fn is_unknown(&self) -> bool {
        self.0.is_unknown() || self.1.is_unknown()
    }
}

impl From<(Layout, Layout)> for PlatformLayout {
    fn from((x64, x32): (Layout, Layout)) -> Self {
        Self(x64, x32)
    }
}

type WellKnown = HashMap<&'static str, PlatformLayout>;

fn calc_enum_layout(ty: &mut REnum, ctx: &CodegenCtx) -> Result<PlatformLayout> {
    fn collect_variant_layouts(ty: &REnum, ctx: &CodegenCtx) -> Result<Vec<PlatformLayout>> {
        // all unit variants?
        if ty.item.variants.iter().all(|var| var.fields.is_empty()) {
            // all AST enums are `repr(C)` so it would have a 4 byte layout/alignment,
            let layout = Layout::known(0, 4, 0);
            Ok(vec![PlatformLayout(layout, layout)])
        } else {
            ty.item
                .variants
                .iter()
                .map(|var| {
                    let typ =
                        var.fields.iter().exactly_one().map(|f| f.ty.analyze(ctx)).normalize()?;
                    calc_type_layout(&typ, ctx)
                })
                .collect()
        }
    }

    fn fold_layout(mut acc: KnownLayout, layout: KnownLayout) -> KnownLayout {
        // max alignment
        if layout.align > acc.align {
            acc.align = layout.align;
        }
        // max size
        if layout.size > acc.size {
            acc.size = layout.size;
        }
        // min niches
        if layout.niches() < acc.niches() {
            acc.niches = layout.niches();
        }
        acc
    }

    let with_tag = |mut acc: KnownLayout| -> KnownLayout {
        acc.consume_niches(ty.item.variants.len() as u128, true);
        acc
    };

    let layouts = collect_variant_layouts(ty, ctx)?;
    let (layouts_x64, layouts_x32): (Vec<KnownLayout>, Vec<KnownLayout>) = layouts
        .into_iter()
        .map(|PlatformLayout(x64, x32)| {
            x64.layout().and_then(|x64| x32.layout().map(|x32| (x64, x32)))
        })
        .collect::<Option<_>>()
        .expect("already checked.");

    let x32 = with_tag(layouts_x32.into_iter().fold(KnownLayout::default(), fold_layout));
    let x64 = with_tag(layouts_x64.into_iter().fold(KnownLayout::default(), fold_layout));
    Ok(PlatformLayout(Layout::from(x64), Layout::from(x32)))
}

fn calc_struct_layout(ty: &mut RStruct, ctx: &CodegenCtx) -> Result<PlatformLayout> {
    fn collect_field_layouts(ty: &RStruct, ctx: &CodegenCtx) -> Result<Vec<PlatformLayout>> {
        if ty.item.fields.is_empty() {
            Ok(vec![PlatformLayout::zero()])
        } else {
            ty.item
                .fields
                .iter()
                .map(|field| {
                    let typ = field.ty.analyze(ctx);
                    calc_type_layout(&typ, ctx)
                })
                .collect()
        }
    }

    fn with_padding(
        layouts: &[KnownLayout],
    ) -> std::result::Result<KnownLayout, std::alloc::LayoutError> {
        // TODO: store `offsets` in the layout
        let mut offsets = Vec::new();
        let mut output = std::alloc::Layout::from_size_align(0, 1)?;
        let mut niches = 0;
        for layout in layouts {
            let (new_layout, offset) =
                output.extend(std::alloc::Layout::from_size_align(layout.size, layout.align)?)?;
            output = new_layout;
            offsets.push(offset);
            niches += layout.niches();
        }
        let output = output.pad_to_align();
        Ok(KnownLayout { size: output.size(), align: output.align(), niches })
    }

    let layouts = collect_field_layouts(ty, ctx)?;

    if layouts.iter().any(PlatformLayout::is_unknown) {
        return Ok(PlatformLayout::UNKNOWN);
    }

    let (layouts_x64, layouts_x32): (Vec<KnownLayout>, Vec<KnownLayout>) = layouts
        .into_iter()
        .map(|PlatformLayout(x64, x32)| {
            x64.layout().and_then(|x64| x32.layout().map(|x32| (x64, x32)))
        })
        .collect::<Option<_>>()
        .expect("already checked.");

    let x32 = with_padding(&layouts_x32).normalize()?;
    let x64 = with_padding(&layouts_x64).normalize()?;

    Ok(PlatformLayout(Layout::from(x64), Layout::from(x32)))
}

fn calc_type_layout(ty: &TypeAnalyzeResult, ctx: &CodegenCtx) -> Result<PlatformLayout> {
    fn is_slice(ty: &TypeAnalyzeResult) -> bool {
        if let Type::Reference(typ) = &ty.typ {
            // TODO: support for &[T] slices.
            typ.elem.get_ident().as_ident().is_some_and(|id| id == "str")
        } else {
            false
        }
    }

    fn try_fold_option(layout: Layout) -> Layout {
        let Layout::Layout(mut known) = layout else { return layout };
        // option needs only one bit to store its tag and it can be in a fragmented offset
        known.consume_niches(1, true);
        Layout::Layout(known)
    }

    let get_layout = |type_ref: Option<&TypeRef>| -> Result<PlatformLayout> {
        let result = if let Some(type_ref) = &type_ref {
            if calc_layout(&mut type_ref.borrow_mut(), ctx)? {
                type_ref.borrow().layouts().map(PlatformLayout::from)?
            } else {
                PlatformLayout::UNKNOWN
            }
        } else if let Some(well_known) =
            WELL_KNOWN.get(ty.typ.get_ident().inner_ident().to_string().as_str())
        {
            well_known.clone()
        } else {
            let Type::Path(typ) = &ty.typ else {
                panic!();
            };

            let typ = typ
                .path
                .segments
                .first()
                .map(|it| it.to_token_stream().to_string().replace(' ', ""))
                .expect("We only accept single segment types.");

            if let Some(typ) = WELL_KNOWN.get(typ.as_str()) {
                typ.clone()
            } else {
                panic!("Unsupported type: {:#?}", ty.typ.to_token_stream().to_string())
            }
        };
        Ok(result)
    };

    let layout = match ty.wrapper {
        TypeWrapper::Vec | TypeWrapper::VecBox | TypeWrapper::VecOpt | TypeWrapper::OptVec => {
            WELL_KNOWN[stringify!(Vec)].clone()
        }
        TypeWrapper::Ref if is_slice(ty) => PlatformLayout::wide_ptr(),
        TypeWrapper::Ref | TypeWrapper::Box | TypeWrapper::OptBox => PlatformLayout::ptr(),
        TypeWrapper::None => get_layout(ty.type_ref.as_ref())?,
        TypeWrapper::Opt => {
            let PlatformLayout(x64, x32) = get_layout(ty.type_ref.as_ref())?;
            PlatformLayout(try_fold_option(x64), try_fold_option(x32))
        }
        TypeWrapper::Complex => {
            let PlatformLayout(x64, x32) = get_layout(ty.type_ref.as_ref())?;
            PlatformLayout(x64, x32)
        }
    };
    Ok(layout)
}

macro_rules! well_known {
    ($($typ:ty: { $($platform:tt => $layout:expr,)*},)*) => {
        WellKnown::from([
            $((
                stringify!($typ),
                well_known!(@ $( $platform => $layout,)*)
            )),*
        ])
    };

    // entries
    (@ _ => $layout:expr,) => {
        PlatformLayout($layout, $layout)
    };
    (@ 64 => $layout_64:expr, 32 => $layout_32:expr,) => {
        PlatformLayout($layout_64, $layout_32)
    };
    (@ 32 => $layout_32:expr, 64 => $layout_64:expr,) => {
        well_known!(@ 64 => $layout_64, 32 => $layout_32)
    };

    // compile errors
    (@ 32 => $layout:expr,) => {
        ::std::compile_error!("non_exhaustive well known type, `64` target isn't covered.")
    };
    (@ 64 => $layout:expr,) => {
        ::std::compile_error!("non_exhaustive well known type, `32` target isn't covered.")
    };
}

lazy_static! {
    static ref WELL_KNOWN: WellKnown = well_known! {
        // builtins
        // types smaller or equal to 4bytes have the same layout on most platforms.
        char: { _ => Layout::of::<char>(), },
        bool: { _ => Layout::of::<bool>(), },
        u8: { _ => Layout::of::<u8>(), },
        i8: { _ => Layout::of::<i8>(), },
        u16: { _ => Layout::of::<u16>(), },
        i16: { _ => Layout::of::<i16>(), },
        u32: { _ => Layout::of::<u32>(), },
        i32: { _ => Layout::of::<i32>(), },
        f32: { _ => Layout::of::<f32>(), },
        // 32bit layouts are based on WASM
        u64: {
            64 => Layout::of::<u64>(),
            32 => Layout::known(8, 8, 0),
        },
        i64: {
            64 => Layout::of::<i64>(),
            32 => Layout::known(8, 8, 0),
        },
        f64: {
            64 => Layout::of::<f64>(),
            32 => Layout::known(8, 8, 0),
        },
        usize: {
            64 => Layout::of::<usize>(),
            32 => Layout::known(4, 4, 0),
        },
        isize: {
            64 => Layout::of::<isize>(),
            32 => Layout::known(4, 4, 0),
        },
        // well known types
        // TODO: generate const assertions for these in the ast crate
        Span: { _ => Layout::known(8, 4, 0), },
        Atom: {
            64 => Layout::wide_ptr_64(),
            32 => Layout::wide_ptr_32(),
        },
        Vec: {
            64 => Layout::known(32, 8, 1),
            32 => Layout::known(16, 4, 1),
        },
        Cell<Option<ScopeId>>: { _ => Layout::known(4, 4, 1), },
        Cell<Option<SymbolId>>: { _ => Layout::known(4, 4, 1), },
        Cell<Option<ReferenceId>>: { _ => Layout::known(4, 4, 1), },
        ReferenceFlag: { _ => Layout::known(1, 1, 0), },
        AssignmentOperator: { _ => Layout::known(1, 1, 1), },
        LogicalOperator: { _ => Layout::known(1, 1, 1), },
        UnaryOperator: { _ => Layout::known(1, 1, 1), },
        BinaryOperator: { _ => Layout::known(1, 1, 1), },
        UpdateOperator: { _ => Layout::known(1, 1, 1), },
        SourceType: { _ => Layout::known(4, 1, 1), },
        RegExpFlags: { _ => Layout::known(1, 1, 0), },
        BigintBase: { _ => Layout::known(1, 1, 1), },
        NumberBase: { _ => Layout::known(1, 1, 1), },
    };
}
