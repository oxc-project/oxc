use super::super::TypeId;

/// Details for `Dummy` derive on a struct.
#[derive(Clone, Copy, Default, Debug)]
pub struct DummyStruct {
    /// Details of allocations a dummy enum of this type requires
    pub alloc: Alloc,
}

/// Details for `Dummy` derive on an enum.
#[derive(Clone, Copy, Default, Debug)]
pub struct DummyEnum {
    /// Details of allocations a dummy enum of this type requires
    pub alloc: Alloc,
    /// Variant which allocates minimum number of bytes
    pub min_variant: MinVariant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Alloc {
    /// Number of bytes a dummy of this type allocates on 64-bit system
    pub bytes_64: u32,
    /// Number of bytes a dummy of this type allocates on 32-bit system
    pub bytes_32: u32,
    /// Number of allocations a dummy of this type requires to construct
    pub count: u32,
}

impl Alloc {
    /// [`Alloc`] representing zero cost.
    pub const ZERO: Self = Self { bytes_64: 0, bytes_32: 0, count: 0 };

    /// Sentinel value for [`Alloc`], indicating that it's not been calculated yet.
    pub const NOT_CALCULATED: Self = Self { bytes_64: u32::MAX, bytes_32: 0, count: 0 };

    /// Sentinel value for [`Alloc`], indicating that it currently being calculated.
    /// Used for preventing infinite cycles.
    pub const CALCULATING: Self = Self { bytes_64: 0, bytes_32: u32::MAX, count: 0 };
}

impl Default for Alloc {
    fn default() -> Self {
        Self::NOT_CALCULATED
    }
}

/// Which variant of an enum is the cheapest to generate a dummy for.
#[derive(Clone, Copy, Debug)]
pub enum MinVariant {
    /// Own variant index
    Own(usize),
    /// Inherited variant - `TypeId` of the inherited enum and variant index
    Inherited(TypeId, usize),
}

impl Default for MinVariant {
    fn default() -> Self {
        // Dummy value
        MinVariant::Own(0)
    }
}
