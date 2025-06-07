#![expect(missing_docs)] // fixme
use bitflags::bitflags;
use nonmax::NonMaxU32;
use oxc_allocator::{Allocator, CloneIn};
use oxc_index::Idx;
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer};

use oxc_ast_macros::ast;

#[ast]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[builder(default)]
#[clone_in(default)]
#[content_eq(skip)]
#[estree(skip)]
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
    pub const unsafe fn new_unchecked(idx: u32) -> Self {
        // SAFETY: Caller must ensure `idx` is not `u32::MAX`
        unsafe { Self(NonMaxU32::new_unchecked(idx)) }
    }
}

impl Idx for ScopeId {
    #[expect(clippy::cast_possible_truncation)]
    fn from_usize(idx: usize) -> Self {
        assert!(idx < u32::MAX as usize);
        // SAFETY: We just checked `idx` is a legal value for `NonMaxU32`
        Self(unsafe { NonMaxU32::new_unchecked(idx as u32) })
    }

    fn index(self) -> usize {
        self.0.get() as usize
    }
}

impl<'alloc> CloneIn<'alloc> for ScopeId {
    type Cloned = Self;

    fn clone_in(&self, _: &'alloc Allocator) -> Self {
        // `clone_in` should never reach this, because `CloneIn` skips scope_id field
        unreachable!();
    }

    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn clone_in_with_semantic_ids(&self, _: &'alloc Allocator) -> Self {
        *self
    }
}

#[cfg(feature = "serialize")]
impl Serialize for ScopeId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.0.get())
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        const DirectEval       = 1 << 10; // <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/eval#direct_and_indirect_eval>
        const TsConditional    = 1 << 11;
        const Var = Self::Top.bits() | Self::Function.bits() | Self::ClassStaticBlock.bits() | Self::TsModuleBlock.bits();
    }
}

impl ScopeFlags {
    #[must_use]
    #[inline]
    pub fn with_strict_mode(self, yes: bool) -> Self {
        if yes { self | Self::StrictMode } else { self }
    }

    #[inline]
    pub fn is_strict_mode(self) -> bool {
        self.contains(Self::StrictMode)
    }

    #[inline]
    pub fn is_block(self) -> bool {
        self.is_empty() || self == Self::StrictMode
    }

    #[inline]
    pub fn is_top(self) -> bool {
        self.contains(Self::Top)
    }

    #[inline]
    pub fn is_function(self) -> bool {
        self.contains(Self::Function)
    }

    #[inline]
    pub fn is_arrow(self) -> bool {
        self.contains(Self::Arrow)
    }

    #[inline]
    pub fn is_constructor(self) -> bool {
        self.contains(Self::Constructor)
    }

    #[inline]
    pub fn is_class_static_block(self) -> bool {
        self.contains(Self::ClassStaticBlock)
    }

    #[inline]
    pub fn is_ts_module_block(self) -> bool {
        self.contains(Self::TsModuleBlock)
    }

    #[inline]
    pub fn is_var(self) -> bool {
        self.intersects(Self::Var)
    }

    #[inline]
    pub fn is_set_accessor(self) -> bool {
        self.contains(Self::SetAccessor)
    }

    #[inline]
    pub fn is_set_or_get_accessor(self) -> bool {
        self.intersects(Self::SetAccessor | Self::GetAccessor)
    }

    #[inline]
    pub fn is_catch_clause(self) -> bool {
        self.contains(Self::CatchClause)
    }

    pub fn is_ts_conditional(self) -> bool {
        self.contains(Self::TsConditional)
    }

    #[inline]
    pub fn contains_direct_eval(self) -> bool {
        self.contains(Self::DirectEval)
    }
}
