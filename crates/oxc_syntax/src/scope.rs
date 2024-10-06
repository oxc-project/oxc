use bitflags::bitflags;
use nonmax::NonMaxU32;
use oxc_index::Idx;
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ScopeId(NonMaxU32);

impl ScopeId {
    /// Create `ScopeId` from `u32`.
    ///
    /// # Panics
    /// Panics if `idx` is `u32::MAX`.
    pub const fn new(idx: u32) -> Self {
        if let Some(idx) = NonMaxU32::new(idx) {
            return Self(idx);
        }
        panic!();
    }

    /// Create `ScopeId` from `u32` unchecked.
    ///
    /// # SAFETY
    /// `idx` must not be `u32::MAX`.
    #[allow(clippy::missing_safety_doc, clippy::unnecessary_safety_comment)]
    pub const unsafe fn new_unchecked(idx: u32) -> Self {
        // SAFETY: Caller must ensure `idx` is not `u32::MAX`
        Self(NonMaxU32::new_unchecked(idx))
    }
}

impl Idx for ScopeId {
    #[allow(clippy::cast_possible_truncation)]
    fn from_usize(idx: usize) -> Self {
        assert!(idx < u32::MAX as usize);
        // SAFETY: We just checked `idx` is a legal value for `NonMaxU32`
        Self(unsafe { NonMaxU32::new_unchecked(idx as u32) })
    }

    fn index(self) -> usize {
        self.0.get() as usize
    }
}

#[cfg(feature = "serialize")]
impl Serialize for ScopeId {
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
export type ScopeId = number;
"#;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct ScopeFlags: u16 {
        const StrictMode       = 1 << 0;
        const Top              = 1 << 1;
        const Function         = 1 << 2;
        const Arrow            = 1 << 3;
        const ClassStaticBlock = 1 << 4;
        const TsModuleBlock    = 1 << 5; // `declare namespace`
        const Constructor      = 1 << 6;
        const GetAccessor      = 1 << 7;
        const SetAccessor      = 1 << 8;
        const CatchClause      = 1 << 9;
        const Var = Self::Top.bits() | Self::Function.bits() | Self::ClassStaticBlock.bits() | Self::TsModuleBlock.bits();
        const Modifiers = Self::Constructor.bits() | Self::GetAccessor.bits() | Self::SetAccessor.bits();
    }
}

impl ScopeFlags {
    #[must_use]
    #[inline]
    pub fn with_strict_mode(self, yes: bool) -> Self {
        if yes {
            self | Self::StrictMode
        } else {
            self
        }
    }

    #[inline]
    pub fn is_strict_mode(&self) -> bool {
        self.contains(Self::StrictMode)
    }

    #[inline]
    pub fn is_block(&self) -> bool {
        self.is_empty() || *self == Self::StrictMode
    }

    #[inline]
    pub fn is_top(&self) -> bool {
        self.contains(Self::Top)
    }

    #[inline]
    pub fn is_function(&self) -> bool {
        self.contains(Self::Function)
    }

    #[inline]
    pub fn is_arrow(&self) -> bool {
        self.contains(Self::Arrow)
    }

    #[inline]
    pub fn is_constructor(&self) -> bool {
        self.contains(Self::Constructor)
    }

    #[inline]
    pub fn is_class_static_block(&self) -> bool {
        self.contains(Self::ClassStaticBlock)
    }

    #[inline]
    pub fn is_ts_module_block(&self) -> bool {
        self.contains(Self::TsModuleBlock)
    }

    #[inline]
    pub fn is_var(&self) -> bool {
        self.intersects(Self::Var)
    }

    #[inline]
    pub fn is_set_accessor(&self) -> bool {
        self.contains(Self::SetAccessor)
    }

    #[inline]
    pub fn is_set_or_get_accessor(&self) -> bool {
        self.intersects(Self::SetAccessor | Self::GetAccessor)
    }

    #[inline]
    pub fn is_catch_clause(&self) -> bool {
        self.contains(Self::CatchClause)
    }
}
