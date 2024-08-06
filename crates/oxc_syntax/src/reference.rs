use bitflags::bitflags;
use nonmax::NonMaxU32;
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer};

use oxc_index::Idx;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ReferenceId(NonMaxU32);

impl Idx for ReferenceId {
    #[allow(clippy::cast_possible_truncation)]
    fn from_usize(idx: usize) -> Self {
        assert!(idx < u32::MAX as usize);
        // SAFETY: We just checked `idx` is valid for `NonMaxU32`
        Self(unsafe { NonMaxU32::new_unchecked(idx as u32) })
    }

    fn index(self) -> usize {
        self.0.get() as usize
    }
}

#[cfg(feature = "serialize")]
impl Serialize for ReferenceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.0.get())
    }
}

#[cfg(feature = "serialize")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type ReferenceId = number;
export type ReferenceFlag = {
    None: 0,
    Read: 0b1,
    Write: 0b10,
    Type: 0b100,
    Value: 0b11
}
"#;

bitflags! {
    #[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
    #[cfg_attr(feature = "serialize", derive(Serialize))]
    pub struct ReferenceFlag: u8 {
        const None = 0;
        const Read = 1 << 0;
        const Write = 1 << 1;
        // Used in type definitions.
        const Type = 1 << 2;
        // Used in `typeof xx`
        const TSTypeQuery = 1 << 3;
        const Value = Self::Read.bits() | Self::Write.bits();
    }
}

impl ReferenceFlag {
    pub const fn read() -> Self {
        Self::Read
    }

    pub const fn write() -> Self {
        Self::Write
    }

    pub const fn read_write() -> Self {
        Self::Value
    }

    /// The identifier is read from. It may also be written to.
    pub const fn is_read(&self) -> bool {
        self.intersects(Self::Read)
    }

    /// The identifier is only read from.
    pub const fn is_read_only(&self) -> bool {
        self.contains(Self::Read)
    }

    /// The identifier is written to. It may also be read from.
    pub const fn is_write(&self) -> bool {
        self.intersects(Self::Write)
    }

    /// The identifier is only written to. It is not read from in this reference.
    pub const fn is_write_only(&self) -> bool {
        self.contains(Self::Write)
    }

    /// The identifier is both read from and written to, e.g `a += 1`.
    pub fn is_read_write(&self) -> bool {
        self.contains(Self::Read | Self::Write)
    }

    /// The identifier is used in a type referenced
    pub fn is_ts_type_query(&self) -> bool {
        self.contains(Self::TSTypeQuery)
    }

    /// The identifier is used in a type definition.
    pub const fn is_type(&self) -> bool {
        self.contains(Self::Type)
    }

    pub const fn is_type_only(self) -> bool {
        matches!(self, Self::Type)
    }

    pub const fn is_value(&self) -> bool {
        self.intersects(Self::Value)
    }
}
