use std::{
    cmp::max,
    mem::{align_of, size_of},
};

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

    /// Create [`Layout`] from a Rust type.
    pub fn from_type_with_niche_for_zero<T>() -> Self {
        let size = u32::try_from(size_of::<T>()).unwrap();
        Self::from_size_align_niche(
            size,
            u32::try_from(align_of::<T>()).unwrap(),
            Niche::new(0, size, 1, 0),
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
    pub size: u32,
    /// Number of values at start of range
    pub count_start: u32,
    /// Number of values at end of range
    pub count_end: u32,
}

impl Niche {
    /// Create new [`Niche`].
    pub fn new(offset: u32, size: u32, count_start: u32, count_end: u32) -> Self {
        Self { offset, size, count_start, count_end }
    }

    /// Get size of largest niche range (start or end)
    pub fn count_max(&self) -> u32 {
        max(self.count_start, self.count_end)
    }

    /// Get value of the [`Niche`].
    #[expect(unused)]
    pub fn value(&self) -> u128 {
        // Prefer to consume niches at start of range over end of range
        if self.count_start > 0 {
            u128::from(self.count_start - 1)
        } else {
            let max_value = match self.size {
                1 => u128::from(u8::MAX),
                2 => u128::from(u16::MAX),
                4 => u128::from(u32::MAX),
                8 => u128::from(u64::MAX),
                16 => u128::MAX,
                size => panic!("Invalid niche size: {size}"),
            };
            max_value - u128::from(self.count_end) + 1
        }
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
