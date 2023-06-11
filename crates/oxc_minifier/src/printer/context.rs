#![allow(non_upper_case_globals)]

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct Context: u8 {
        /// [In]
        const In = 1 << 0;
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::In
    }
}

impl Context {
    #[inline]
    pub fn has_in(self) -> bool {
        self.contains(Self::In)
    }

    #[inline]
    pub fn and_in(self, include: bool) -> Self {
        self.and(Self::In, include)
    }

    #[inline]
    fn and(self, flag: Self, set: bool) -> Self {
        if set { self | flag } else { self - flag }
    }

    #[inline]
    pub(crate) fn union_in_if(self, include: bool) -> Self {
        self.union_if(Self::In, include)
    }

    #[inline]
    fn union_if(self, other: Self, include: bool) -> Self {
        if include { self.union(other) } else { self }
    }
}
