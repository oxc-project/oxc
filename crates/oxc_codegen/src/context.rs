#![allow(missing_docs)] // FIXME
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Context: u8 {
        /// [In]
        const FORBID_IN   = 1 << 0;
        const FORBID_CALL = 1 << 1;
        const TYPESCRIPT  = 1 << 2;
    }
}

impl Context {
    #[inline]
    pub fn forbid_in(self) -> bool {
        self.contains(Self::FORBID_IN)
    }

    #[inline]
    pub fn forbid_call(self) -> bool {
        self.contains(Self::FORBID_CALL)
    }

    #[inline]
    #[must_use]
    pub fn with_typescript(mut self) -> Self {
        self |= Self::TYPESCRIPT;
        self
    }

    #[inline]
    #[must_use]
    pub fn and_forbid_in(self, include: bool) -> Self {
        self.and(Self::FORBID_IN, include)
    }

    #[inline]
    #[must_use]
    pub fn and_forbid_call(self, include: bool) -> Self {
        self.and(Self::FORBID_CALL, include)
    }

    #[inline]
    fn and(self, flag: Self, set: bool) -> Self {
        if set {
            self | flag
        } else {
            self - flag
        }
    }
}
