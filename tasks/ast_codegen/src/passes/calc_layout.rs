use std::collections::HashMap;

use itertools::Itertools;
use lazy_static::lazy_static;
use quote::ToTokens;
use syn::Type;

use crate::{
    layout::{KnownLayout, Layout},
    rust_ast::{AstRef, AstType, Enum, Struct},
    util::{NormalizeError, TypeAnalysis, TypeExt, TypeWrapper},
    EarlyCtx, Result,
};

use super::{define_pass, Pass};

/// We use compiler to infer 64bit type layouts.
#[cfg(not(target_pointer_width = "64"))]
compile_error!("`oxc_ast_codegen::calc_layout` only supports 64 architectures.");

type WellKnown = HashMap<&'static str, PlatformLayout>;

define_pass! {
    pub struct CalcLayout;
}

impl Pass for CalcLayout {
    fn name(&self) -> &'static str {
        stringify!(CalcLayout)
    }

    fn each(&mut self, ty: &mut AstType, ctx: &EarlyCtx) -> crate::Result<bool> {
        calc_layout(ty, ctx)
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

/// Calculates the layout of `ty` by mutating it.
/// Returns `false` if the layout is unknown at this point.
pub fn calc_layout(ty: &mut AstType, ctx: &EarlyCtx) -> Result<bool> {
    let unknown_layout = ty
        .layout_32()
        .and_then(|x32| ty.layout_64().map(|x64| PlatformLayout(x64, x32)))
        .is_ok_and(|pl| pl.is_unknown());
    let layout = match ty {
        AstType::Enum(enum_) if unknown_layout => calc_enum_layout(enum_, ctx),
        AstType::Struct(struct_) if unknown_layout => calc_struct_layout(struct_, ctx),
        _ => return Ok(true),
    }?;
    if layout.is_unknown() {
        Ok(false)
    } else {
        let PlatformLayout(x64, x32) = layout;
        ty.set_layout(x64, x32)?;
        Ok(true)
    }
}

fn calc_enum_layout(ty: &mut Enum, ctx: &EarlyCtx) -> Result<PlatformLayout> {
    fn collect_variant_layouts(ty: &Enum, ctx: &EarlyCtx) -> Result<Vec<PlatformLayout>> {
        // all unit variants?
        if ty.item.variants.iter().all(|var| var.fields.is_empty()) {
            // all AST enums are `repr(u8)` so it would have a 1 byte layout/alignment,
            // if it holds no data
            let layout = KnownLayout::new(0, 1, 0);
            let layout = Layout::Layout(layout);
            Ok(vec![PlatformLayout(layout.clone(), layout)])
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

    #[allow(clippy::needless_pass_by_value)]
    fn fold_layout(mut acc: KnownLayout, layout: KnownLayout) -> KnownLayout {
        // SAFETY: we are folding valid layouts so it is safe.
        unsafe {
            // max alignment
            if layout.align() > acc.align() {
                acc.set_align_unchecked(layout.align());
            }
            // max size
            if layout.size() > acc.size() {
                acc.set_size_unchecked(layout.size());
            }
            // min niches
            if layout.niches() < acc.niches() {
                acc.set_niches_unchecked(layout.niches());
            }
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

fn calc_struct_layout(ty: &mut Struct, ctx: &EarlyCtx) -> Result<PlatformLayout> {
    fn collect_field_layouts(ty: &Struct, ctx: &EarlyCtx) -> Result<Vec<PlatformLayout>> {
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
        let layouts = layouts.iter().enumerate();
        let mut offsets = vec![0; layouts.len()];
        let mut output = std::alloc::Layout::from_size_align(0, 1)?;
        let mut niches = 0;
        for (ix, layout) in layouts {
            let (new_layout, offset) = output
                .extend(std::alloc::Layout::from_size_align(layout.size(), layout.align())?)?;
            output = new_layout;
            niches += layout.niches();
            offsets[ix] = offset;
        }
        let output = output.pad_to_align();
        Ok(KnownLayout::new(output.size(), output.align(), niches).with_offsets(offsets))
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

fn calc_type_layout(ty: &TypeAnalysis, ctx: &EarlyCtx) -> Result<PlatformLayout> {
    fn is_slice(ty: &TypeAnalysis) -> bool {
        if let Type::Reference(typ) = &ty.typ {
            // TODO: support for &[T] slices.
            typ.elem.get_ident().as_ident().is_some_and(|id| id == "str")
        } else {
            false
        }
    }

    fn try_fold_option(layout: Layout) -> Layout {
        let Layout::Layout(mut known) = layout else { return layout };
        // option needs only one niche, We allow resizing in case there isn't enough space.
        known.consume_niches(1, true);
        Layout::Layout(known)
    }

    let get_layout = |ast_ref: Option<&AstRef>| -> Result<PlatformLayout> {
        let result = if let Some(ast_ref) = &ast_ref {
            if calc_layout(&mut ast_ref.borrow_mut(), ctx)? {
                ast_ref.borrow().layouts().map(PlatformLayout::from)?
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
        TypeWrapper::Vec | TypeWrapper::VecBox | TypeWrapper::VecOpt => {
            WELL_KNOWN[stringify!(Vec)].clone()
        }
        TypeWrapper::OptVec => {
            let mut pl = WELL_KNOWN[stringify!(Vec)].clone();
            // preconsume one niche for option
            if let Layout::Layout(layout) = &mut pl.0 {
                layout.consume_niches(1, true);
            }
            if let Layout::Layout(layout) = &mut pl.1 {
                layout.consume_niches(1, true);
            }
            pl
        }
        TypeWrapper::Ref if is_slice(ty) => PlatformLayout::wide_ptr(),
        TypeWrapper::Ref | TypeWrapper::Box | TypeWrapper::OptBox => PlatformLayout::ptr(),
        TypeWrapper::None => get_layout(ty.type_id.map(|id| ctx.ast_ref(id)).as_ref())?,
        TypeWrapper::Opt => {
            let PlatformLayout(x64, x32) =
                get_layout(ty.type_id.map(|id| ctx.ast_ref(id)).as_ref())?;
            PlatformLayout(try_fold_option(x64), try_fold_option(x32))
        }
        TypeWrapper::Complex => {
            let PlatformLayout(x64, x32) =
                get_layout(ty.type_id.map(|id| ctx.ast_ref(id)).as_ref())?;
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
        Atom: {
            64 => Layout::wide_ptr_64(),
            32 => Layout::wide_ptr_32(),
        },
        // External Bumpalo type
        Vec: {
            64 => Layout::known(32, 8, 1),
            32 => Layout::known(16, 4, 1),
        },
        // Unsupported: we don't analyze `Cell` types
        Cell<Option<ScopeId>>: { _ => Layout::known(4, 4, 0), },
        Cell<Option<SymbolId>>: { _ => Layout::known(4, 4, 0), },
        Cell<Option<ReferenceId>>: { _ => Layout::known(4, 4, 0), },
        // Unsupported: this is a `bitflags` generated type, we don't expand macros
        ReferenceFlag: { _ => Layout::known(1, 1, 0), },
        // Unsupported: this is a `bitflags` generated type, we don't expand macros
        RegExpFlags: { _ => Layout::known(1, 1, 0), },
    };
}
