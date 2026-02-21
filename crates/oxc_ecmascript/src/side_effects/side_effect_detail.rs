use bitflags::bitflags;

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
    /// Detailed side-effect information for tree-shaking decisions.
    pub struct SideEffectDetail: u8 {
        const GlobalVarAccess = 1;
        const PureCjs = 1 << 1;
        const Unknown = 1 << 2;
        const PureAnnotation = 1 << 3;
    }
}

impl SideEffectDetail {
    #[inline]
    pub fn has_side_effect(self) -> bool {
        self.intersects(SideEffectDetail::PureCjs | SideEffectDetail::Unknown)
    }
}

impl From<bool> for SideEffectDetail {
    fn from(value: bool) -> Self {
        if value { SideEffectDetail::Unknown } else { SideEffectDetail::empty() }
    }
}
