use std::mem::{align_of, size_of};

/// The layout of a type.
#[derive(Clone, Default, Debug)]
pub struct Layout {
    /// Layout on 64-bit platforms
    pub layout_64: PlatformLayout,
    /// Layout on 32-bit platforms
    pub layout_32: PlatformLayout,
}

impl Layout {
    /// Create [`Layout`] from a Rust type.
    pub fn from_type<T>() -> Self {
        Self::from_size_align(
            u32::try_from(size_of::<T>()).unwrap(),
            u32::try_from(align_of::<T>()).unwrap(),
        )
    }

    /// Create [`Layout`] from `size` and `align` pair, with no niche.
    ///
    /// Layout is same for both 64-bit and 32-bit platforms.
    pub fn from_size_align(size: u32, align: u32) -> Self {
        Self {
            layout_64: PlatformLayout::from_size_align(size, align),
            layout_32: PlatformLayout::from_size_align(size, align),
        }
    }

    /// Create [`Layout`] from `size` and `align` pair, and [`Niche`].
    ///
    /// Layout is same for both 64-bit and 32-bit platforms.
    pub fn from_size_align_niche(size: u32, align: u32, niche: Niche) -> Self {
        Self {
            layout_64: PlatformLayout::from_size_align_niche(size, align, niche.clone()),
            layout_32: PlatformLayout::from_size_align_niche(size, align, niche),
        }
    }
}

/// The layout of a type on a specific platform type (64 bit or 32 bit).
#[derive(Clone, Default, Debug)]
pub struct PlatformLayout {
    pub size: u32,
    pub align: u32,
    pub niche: Option<Niche>,
}

impl PlatformLayout {
    /// Create [`PlatformLayout`] from `size` and `align` pair, with no niche.
    pub fn from_size_align(size: u32, align: u32) -> Self {
        Self { size, align, niche: None }
    }

    /// Create [`PlatformLayout`] from `size` and `align` pair, and [`Niche`].
    pub fn from_size_align_niche(size: u32, align: u32, niche: Niche) -> Self {
        Self { size, align, niche: Some(niche) }
    }
}

/// Niche that a type has.
#[derive(Clone, Debug)]
pub struct Niche {
    /// Byte offset of the niche from start of type
    pub offset: u32,
    /// Size of the niche in bytes
    #[expect(dead_code)]
    pub size: u32,
    /// `true` if niche is at start of range (e.g. 0..3).
    /// `false` if niche is at end of range (e.g. 2..255).
    #[expect(dead_code)]
    pub is_range_start: bool,
    /// Number of niche values in the niche (e.g. 1 for `&str`, 254 for `bool`)
    pub count: u32,
}

impl Niche {
    /// Create new [`Niche`].
    pub fn new(offset: u32, size: u32, is_range_start: bool, count: u32) -> Self {
        Self { offset, size, is_range_start, count }
    }
}

/// Offset of a struct field.
#[derive(Clone, Default, Debug)]
pub struct Offset {
    /// Offset in bytes on 64-bit platforms
    pub offset_64: u32,
    /// Offset in bytes on 32-bit platforms
    pub offset_32: u32,
}
