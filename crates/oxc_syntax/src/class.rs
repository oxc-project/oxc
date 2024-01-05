use bitflags::bitflags;
use oxc_index::define_index_type;

define_index_type! {
    pub struct ClassId = u32;
}
define_index_type! {
    pub struct ElementId = u32;
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct ElementKind: u8 {
        const Accessor = 0;
        const Method = 1;
        const Property = 2;
    }
}

impl ElementKind {
    pub fn is_property(self) -> bool {
        self.contains(Self::Property)
    }

    pub fn is_method(self) -> bool {
        self.contains(Self::Method)
    }

    pub fn is_accessor(self) -> bool {
        self.contains(Self::Accessor)
    }
}
