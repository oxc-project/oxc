//! Class and class element syntax items
use bitflags::bitflags;
use oxc_index::define_index_type;

define_index_type! {
    pub struct ClassId = u32;
    MAX_INDEX = u32::MAX as usize;
}
define_index_type! {
    pub struct ElementId = u32;
    MAX_INDEX = u32::MAX as usize;
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
    pub struct ElementKind: u8 {
        const Accessor = 1 << 0;
        const Method = 1 << 1;
        const Property = 1 << 2;
        const Setter = 1 << 3;
        const Getter = 1 << 4;
    }
}

impl ElementKind {
    #[inline]
    pub fn is_property(self) -> bool {
        self.contains(Self::Property)
    }

    #[inline]
    pub fn is_method(self) -> bool {
        self.contains(Self::Method)
    }

    #[inline]
    pub fn is_accessor(self) -> bool {
        self.contains(Self::Accessor)
    }

    #[inline]
    pub fn is_setter_or_getter(self) -> bool {
        self.intersects(Self::Setter | Self::Getter)
    }
}
