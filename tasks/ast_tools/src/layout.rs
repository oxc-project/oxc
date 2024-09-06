#[derive(Debug, Default, Clone)]
pub enum Layout {
    #[default]
    Unknown,
    Layout(KnownLayout),
}

impl Layout {
    pub const fn known(size: usize, align: usize, niches: u128) -> Self {
        Self::Layout(KnownLayout { size, align, niches, offsets: None })
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

#[derive(Debug, Default, Clone)]
pub struct KnownLayout {
    size: usize,
    align: usize,
    /// number of available niches
    niches: u128,
    offsets: Option<Vec<usize>>,
}

impl KnownLayout {
    pub const fn new(size: usize, align: usize, niches: u128) -> Self {
        Self { size, align, niches, offsets: None }
    }

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

    #[expect(unused)]
    #[inline]
    pub fn offsets(&self) -> Option<&Vec<usize>> {
        self.offsets.as_ref()
    }

    pub unsafe fn set_size_unchecked(&mut self, size: usize) {
        self.size = size;
    }

    pub unsafe fn set_align_unchecked(&mut self, align: usize) {
        self.align = align;
    }

    pub unsafe fn set_niches_unchecked(&mut self, niches: u128) {
        self.niches = niches;
    }

    pub fn with_offsets(mut self, offsets: Vec<usize>) -> Self {
        self.offsets = Some(offsets);
        self
    }

    /// Panics
    /// if doesn't have enough viable space and `can_resize` is false
    pub fn consume_niches(&mut self, n: u128, can_resize: bool) {
        if self.niches() >= n {
            self.niches -= n;
        } else if can_resize {
            let align = self.align();
            self.size += align;
            self.niches += max_val_of_bytes(align);
            self.consume_niches(n, can_resize);
        } else {
            panic!("`{}` called on a layout without enough space.", stringify!(consume_niches));
        }
    }

    pub fn unpack(self) -> (/* size */ usize, /* align */ usize, /* offsets */ Option<Vec<usize>>) {
        let Self { size, align, offsets, .. } = self;
        (size, align, offsets)
    }
}

impl Layout {
    /// # Panics
    /// If `T` has more than 8 niches.
    pub const fn of<T>() -> Self {
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
            panic!("`T` has more niches than what we can infer automatically");
        };

        Self::known(size, align, niches)
    }

    pub const fn zero() -> Self {
        #[repr(C)]
        struct Empty;
        Self::of::<Empty>()
    }

    pub const fn ptr_32() -> Self {
        Layout::known(4, 4, 0)
    }

    pub const fn ptr_64() -> Self {
        Layout::known(8, 8, 0)
    }

    pub const fn wide_ptr_32() -> Self {
        Layout::known(8, 4, 1)
    }

    pub const fn wide_ptr_64() -> Self {
        Layout::of::<&str>()
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

/// Returns the max valid number in a primitive with the size of `n` bytes.
/// Panics
/// For `n` bigger than `16`, Or if it's not a power of 2 number
fn max_val_of_bytes(n: usize) -> u128 {
    match n {
        1 => u8::MAX as u128,
        2 => u16::MAX as u128,
        4 => u32::MAX as u128,
        8 => u64::MAX as u128,
        16 => u128::MAX,
        _ => panic!("We do not support `n` bigger than 16 bytes."),
    }
}
