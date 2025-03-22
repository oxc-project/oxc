use super::super::TypeId;

/// Sentinel value for `alloc_bytes` indicating that it's not been calculated yet.
pub const NOT_CALCULATED: usize = usize::MAX;

/// Details for `TakeIn` derive on a struct.
#[derive(Clone, Copy, Debug)]
pub struct TakeInStruct {
    /// Number of bytes a dummy struct of this type allocates
    pub alloc_bytes: usize,
}

impl Default for TakeInStruct {
    fn default() -> Self {
        // Dummy sentinel value meaning "not calculated yet"
        Self { alloc_bytes: usize::MAX }
    }
}

/// Details for `TakeIn` derive on an enum.
#[derive(Clone, Copy, Debug)]
pub struct TakeInEnum {
    /// Number of bytes a dummy enum of this type allocates
    pub alloc_bytes: usize,
    /// Variant which allocates minimum number of bytes
    pub min_variant: MinVariant,
}

impl Default for TakeInEnum {
    fn default() -> Self {
        Self { alloc_bytes: NOT_CALCULATED, min_variant: MinVariant::default() }
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
