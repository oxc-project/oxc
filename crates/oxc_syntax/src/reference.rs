use bitflags::bitflags;
use oxc_index::define_index_type;
#[cfg(feature = "serde")]
use serde::Serialize;

define_index_type! {
    pub struct ReferenceId = u32;
}

#[cfg_attr(
    all(feature = "serde", feature = "wasm"),
    wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)
)]
#[allow(dead_code)]
const TS_APPEND_CONTENT: &'static str = r#"
export type ReferenceId = number;
export type ReferenceFlag = {
    None: 0,
    Read: 0b1,
    Write: 0b10,
    Type: 0b100,
    ReadWrite: 0b11
}
"#;

bitflags! {
    #[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct ReferenceFlag: u8 {
        const None = 0;
        const Read = 1 << 0;
        const Write = 1 << 1;
        // Used in type definitions.
        const Type = 1 << 2;
        const ReadWrite = Self::Read.bits() | Self::Write.bits();
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
        Self::ReadWrite
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
    pub const fn is_read_write(&self) -> bool {
        self.contains(Self::ReadWrite)
    }

    /// The identifier is used in a type definition.
    pub const fn is_type(&self) -> bool {
        self.contains(Self::Type)
    }
}
