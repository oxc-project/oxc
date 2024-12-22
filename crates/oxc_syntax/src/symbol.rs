#![allow(missing_docs)] // fixme
use bitflags::bitflags;
use nonmax::NonMaxU32;
use oxc_index::Idx;
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SymbolId(NonMaxU32);

impl SymbolId {
    /// Create `SymbolId` from `u32`.
    ///
    /// # Panics
    /// Panics if `idx` is `u32::MAX`.
    pub const fn new(idx: u32) -> Self {
        if let Some(idx) = NonMaxU32::new(idx) {
            return Self(idx);
        }
        panic!();
    }

    /// Create `SymbolId` from `u32` unchecked.
    ///
    /// # SAFETY
    /// `idx` must not be `u32::MAX`.
    #[allow(clippy::missing_safety_doc, clippy::unnecessary_safety_comment)]
    pub const unsafe fn new_unchecked(idx: u32) -> Self {
        // SAFETY: Caller must ensure `idx` is not `u32::MAX`
        Self(NonMaxU32::new_unchecked(idx))
    }
}

impl Idx for SymbolId {
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
impl Serialize for SymbolId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.0.get())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RedeclarationId(NonMaxU32);

impl Idx for RedeclarationId {
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
impl Serialize for RedeclarationId {
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
export type SymbolId = number;
export type SymbolFlags = unknown;
export type RedeclarationId = unknown;
"#;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature = "serialize", derive(Serialize))]
    pub struct SymbolFlags: u32 {
        const None                    = 0;
        /// Variable (var) or parameter
        const FunctionScopedVariable  = 1 << 0;
        /// A block-scoped variable (let or const)
        const BlockScopedVariable     = 1 << 1;
        /// A const variable (const)
        const ConstVariable           = 1 << 2;
        const Class                   = 1 << 3;
        /// `try {} catch(catch_variable) {}`
        const CatchVariable           = 1 << 4;
        /// A function declaration or expression
        const Function                = 1 << 5;
        /// Imported ESM binding
        const Import                  = 1 << 6;
        /// Imported ESM type-only binding
        const TypeImport              = 1 << 7;
        // Type specific symbol flags
        const TypeAlias               = 1 << 8;
        const Interface               = 1 << 9;
        const RegularEnum             = 1 << 10;
        const ConstEnum               = 1 << 11;
        const EnumMember              = 1 << 12;
        const TypeParameter           = 1 << 13;
        const NameSpaceModule         = 1 << 14;
        const ValueModule             = 1 << 15;
        // In a dts file or there is a declare flag
        const Ambient                 = 1 << 16;

        const Enum = Self::ConstEnum.bits() | Self::RegularEnum.bits();
        const Variable = Self::FunctionScopedVariable.bits() | Self::BlockScopedVariable.bits();

        const BlockScoped = Self::BlockScopedVariable.bits() | Self::Enum.bits() | Self::Class.bits();

        const Value = Self::Variable.bits() | Self::Class.bits() | Self::Enum.bits() | Self::EnumMember.bits() | Self::ValueModule.bits();
        const Type =  Self::Class.bits() | Self::Interface.bits() | Self::Enum.bits() | Self::EnumMember.bits() | Self::TypeParameter.bits()  |  Self::TypeAlias.bits();

        /// Variables can be redeclared, but can not redeclare a block-scoped declaration with the
        /// same name, or any other value that is not a variable, e.g. ValueModule or Class
        const FunctionScopedVariableExcludes = Self::Value.bits() - Self::FunctionScopedVariable.bits();

        /// Block-scoped declarations are not allowed to be re-declared
        /// they can not merge with anything in the value space
        const BlockScopedVariableExcludes = Self::Value.bits();

        const ClassExcludes = (Self::Value.bits() | Self::TypeAlias.bits()) & !(Self::ValueModule.bits() | Self::Interface.bits() | Self::Function.bits());
        const ImportBindingExcludes = Self::Import.bits() | Self::TypeImport.bits();
        // Type specific excludes
        const TypeAliasExcludes = Self::Type.bits();
        const InterfaceExcludes = Self::Type.bits() & !(Self::Interface.bits() | Self::Class.bits());
        const TypeParameterExcludes = Self::Type.bits() & !Self::TypeParameter.bits();
        const ConstEnumExcludes = (Self::Type.bits() | Self::Value.bits()) & !Self::ConstEnum.bits();
        // TODO: include value module in regular enum excludes
        const RegularEnumExcludes = (Self::Value.bits() | Self::Type.bits()) & !(Self::RegularEnum.bits() | Self::ValueModule.bits() );
        const EnumMemberExcludes = Self::EnumMember.bits();

    }
}

impl SymbolFlags {
    #[inline]
    pub fn is_variable(&self) -> bool {
        self.intersects(Self::Variable)
    }

    #[inline]
    pub fn is_type_parameter(&self) -> bool {
        self.contains(Self::TypeParameter)
    }

    /// If true, then the symbol is a type, such as a TypeAlias, Interface, or Enum
    #[inline]
    pub fn is_type(&self) -> bool {
        self.intersects((Self::TypeImport | Self::Type) - Self::Value)
    }

    /// If true, then the symbol is a value, such as a Variable, Function, or Class
    #[inline]
    pub fn is_value(&self) -> bool {
        self.intersects(Self::Value | Self::Import | Self::Function)
    }

    #[inline]
    pub fn is_const_variable(&self) -> bool {
        self.contains(Self::ConstVariable)
    }

    /// Returns `true` if this symbol is a function declaration or expression.
    #[inline]
    pub fn is_function(&self) -> bool {
        self.contains(Self::Function)
    }

    #[inline]
    pub fn is_class(&self) -> bool {
        self.contains(Self::Class)
    }

    #[inline]
    pub fn is_interface(&self) -> bool {
        self.contains(Self::Interface)
    }

    #[inline]
    pub fn is_type_alias(&self) -> bool {
        self.contains(Self::TypeAlias)
    }

    #[inline]
    pub fn is_enum(&self) -> bool {
        self.intersects(Self::Enum)
    }

    #[inline]
    pub fn is_enum_member(&self) -> bool {
        self.contains(Self::EnumMember)
    }

    #[inline]
    pub fn is_catch_variable(&self) -> bool {
        self.contains(Self::CatchVariable)
    }

    #[inline]
    pub fn is_function_scoped_declaration(&self) -> bool {
        self.contains(Self::FunctionScopedVariable)
    }

    #[inline]
    pub fn is_import(&self) -> bool {
        self.intersects(Self::Import | Self::TypeImport)
    }

    #[inline]
    pub fn is_type_import(&self) -> bool {
        self.contains(Self::TypeImport)
    }

    /// If true, then the symbol can be referenced by a type reference
    #[inline]
    pub fn can_be_referenced_by_type(&self) -> bool {
        self.intersects(Self::Type | Self::TypeImport | Self::Import)
    }

    /// If true, then the symbol can be referenced by a value reference
    #[inline]
    pub fn can_be_referenced_by_value(&self) -> bool {
        self.intersects(Self::Value | Self::Import | Self::Function)
    }

    /// If true, then the symbol can be referenced by a value_as_type reference
    #[inline]
    pub fn can_be_referenced_by_value_as_type(&self) -> bool {
        self.intersects(Self::Value | Self::Import | Self::Function | Self::TypeImport)
    }
}
