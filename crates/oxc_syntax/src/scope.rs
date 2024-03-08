use bitflags::bitflags;
use oxc_index::define_index_type;

define_index_type! {
    pub struct ScopeId = u32;
}

#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type ScopeId = number;
"#;

bitflags! {
    #[derive(Debug, Clone, Copy)]
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
        const Var = Self::Top.bits() | Self::Function.bits() | Self::ClassStaticBlock.bits() | Self::TsModuleBlock.bits();
        const Modifiers = Self::Constructor.bits() | Self::GetAccessor.bits() | Self::SetAccessor.bits();
    }
}

impl ScopeFlags {
    #[must_use]
    pub fn with_strict_mode(self, yes: bool) -> Self {
        if yes {
            self | Self::StrictMode
        } else {
            self
        }
    }

    pub fn is_strict_mode(&self) -> bool {
        self.contains(Self::StrictMode)
    }

    pub fn is_top(&self) -> bool {
        self.contains(Self::Top)
    }

    pub fn is_function(&self) -> bool {
        self.contains(Self::Function)
    }

    pub fn is_arrow(&self) -> bool {
        self.contains(Self::Arrow)
    }

    pub fn is_constructor(&self) -> bool {
        self.contains(Self::Constructor)
    }

    pub fn is_class_static_block(&self) -> bool {
        self.contains(Self::ClassStaticBlock)
    }

    pub fn is_ts_module_block(&self) -> bool {
        self.contains(Self::TsModuleBlock)
    }

    pub fn is_var(&self) -> bool {
        self.intersects(Self::Var)
    }

    pub fn is_set_accessor(&self) -> bool {
        self.contains(Self::SetAccessor)
    }

    pub fn is_set_or_get_accessor(&self) -> bool {
        self.intersects(Self::SetAccessor | Self::GetAccessor)
    }
}
