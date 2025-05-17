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
    pub const unsafe fn new_unchecked(idx: u32) -> Self {
        // SAFETY: Caller must ensure `idx` is not `u32::MAX`
        unsafe { Self(NonMaxU32::new_unchecked(idx)) }
    }
}

impl Idx for SymbolId {
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

impl<'alloc> CloneIn<'alloc> for SymbolId {
    type Cloned = Self;

    fn clone_in(&self, _: &'alloc Allocator) -> Self {
        // `clone_in` should never reach this, because `CloneIn` skips symbol_id field
        unreachable!();
    }

    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn clone_in_with_semantic_ids(&self, _: &'alloc Allocator) -> Self {
        *self
    }
}

#[cfg(feature = "serialize")]
impl Serialize for SymbolId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.0.get())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RedeclarationId(NonMaxU32);

impl Idx for RedeclarationId {
    #[expect(clippy::cast_possible_truncation)]
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
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.0.get())
    }
}

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
        /// Uninstantiated module
        const NamespaceModule         = 1 << 14;
        /// Instantiated module
        const ValueModule             = 1 << 15;
        /// Declared with `declare` modifier, like `declare function x() {}`.
        //
        // This flag is not part of TypeScript's `SymbolFlags`, it comes from TypeScript's `NodeFlags`. We introduced it into
        // here because `NodeFlags` is incomplete and we only can access to `NodeFlags` in the Semantic, but we also need to
        // access it in the Transformer.
        // https://github.com/microsoft/TypeScript/blob/15392346d05045742e653eab5c87538ff2a3c863/src/compiler/types.ts#L819-L820
        const Ambient                 = 1 << 16;

        const Enum = Self::ConstEnum.bits() | Self::RegularEnum.bits();
        const Variable = Self::FunctionScopedVariable.bits() | Self::BlockScopedVariable.bits();

        const BlockScoped = Self::BlockScopedVariable.bits() | Self::Enum.bits() | Self::Class.bits();

        const Value = Self::Variable.bits() | Self::Class.bits() | Self::Function.bits() | Self::Enum.bits() | Self::EnumMember.bits() | Self::ValueModule.bits();
        const Type = Self::Class.bits() | Self::Interface.bits() | Self::Enum.bits() | Self::EnumMember.bits() | Self::TypeParameter.bits()  |  Self::TypeAlias.bits();
        const Namespace = Self::ValueModule.bits() | Self::NamespaceModule.bits() | Self::Enum.bits();


        /// Variables can be redeclared, but can not redeclare a block-scoped declaration with the
        /// same name, or any other value that is not a variable, e.g. ValueModule or Class
        const FunctionScopedVariableExcludes = Self::Value.bits() - Self::FunctionScopedVariable.bits() - Self::Function.bits();

        /// Block-scoped declarations are not allowed to be re-declared
        /// they can not merge with anything in the value space
        const BlockScopedVariableExcludes = Self::Value.bits();
        const FunctionExcludes = Self::Value.bits() & !(Self::Function.bits() | Self::ValueModule.bits() | Self::Class.bits());
        const ClassExcludes = (Self::Value.bits() | Self::Type.bits()) & !(Self::ValueModule.bits() | Self::Function.bits() | Self::Interface.bits());

        const ImportBindingExcludes = Self::Import.bits() | Self::TypeImport.bits();
        // Type specific excludes
        const TypeAliasExcludes = Self::Type.bits();
        const InterfaceExcludes = Self::Type.bits() & !(Self::Interface.bits() | Self::Class.bits());
        const TypeParameterExcludes = Self::Type.bits() & !Self::TypeParameter.bits();
        const ConstEnumExcludes = (Self::Type.bits() | Self::Value.bits()) & !Self::ConstEnum.bits();
        const ValueModuleExcludes = Self::Value.bits() & !(Self::Function.bits() | Self::Class.bits() | Self::RegularEnum.bits() | Self::ValueModule.bits());
        const NamespaceModuleExcludes = 0;
        // TODO: include value module in regular enum excludes
        const RegularEnumExcludes = (Self::Value.bits() | Self::Type.bits()) & !(Self::RegularEnum.bits() | Self::ValueModule.bits() );
        const EnumMemberExcludes = Self::EnumMember.bits();

    }
}

impl SymbolFlags {
    #[inline]
    pub fn is_variable(self) -> bool {
        self.intersects(Self::Variable)
    }

    #[inline]
    pub fn is_type_parameter(self) -> bool {
        self.contains(Self::TypeParameter)
    }

    /// If true, then the symbol is a type, such as a TypeAlias, Interface, or Enum
    #[inline]
    pub fn is_type(self) -> bool {
        self.intersects((Self::TypeImport | Self::Type) - Self::Value)
    }

    /// If true, then the symbol is a value, such as a Variable, Function, or Class
    #[inline]
    pub fn is_value(self) -> bool {
        self.intersects(Self::Value | Self::Import)
    }

    #[inline]
    pub fn is_const_variable(self) -> bool {
        self.contains(Self::ConstVariable)
    }

    /// Returns `true` if this symbol is a function declaration or expression.
    #[inline]
    pub fn is_function(self) -> bool {
        self.contains(Self::Function)
    }

    #[inline]
    pub fn is_class(self) -> bool {
        self.contains(Self::Class)
    }

    #[inline]
    pub fn is_interface(self) -> bool {
        self.contains(Self::Interface)
    }

    #[inline]
    pub fn is_type_alias(self) -> bool {
        self.contains(Self::TypeAlias)
    }

    #[inline]
    pub fn is_enum(self) -> bool {
        self.intersects(Self::Enum)
    }

    #[inline]
    pub fn is_const_enum(self) -> bool {
        self.intersects(Self::ConstEnum)
    }

    #[inline]
    pub fn is_enum_member(self) -> bool {
        self.contains(Self::EnumMember)
    }

    #[inline]
    pub fn is_catch_variable(self) -> bool {
        self.contains(Self::CatchVariable)
    }

    #[inline]
    pub fn is_function_scoped_declaration(self) -> bool {
        self.contains(Self::FunctionScopedVariable)
    }

    #[inline]
    pub fn is_import(self) -> bool {
        self.intersects(Self::Import | Self::TypeImport)
    }

    #[inline]
    pub fn is_type_import(self) -> bool {
        self.contains(Self::TypeImport)
    }

    #[inline]
    pub fn is_ambient(self) -> bool {
        self.contains(Self::Ambient)
    }

    #[inline]
    pub fn is_namespace_module(self) -> bool {
        self.contains(Self::NamespaceModule)
    }

    #[inline]
    pub fn is_value_module(self) -> bool {
        self.contains(Self::ValueModule)
    }

    /// If true, then the symbol can be referenced by a type reference
    #[inline]
    pub fn can_be_referenced_by_type(self) -> bool {
        self.intersects(Self::Type | Self::TypeImport | Self::Import | Self::Namespace)
    }

    /// If true, then the symbol can be referenced by a value reference
    #[inline]
    pub fn can_be_referenced_by_value(self) -> bool {
        self.is_value()
    }

    /// If true, then the symbol can be referenced by a value_as_type reference
    #[inline]
    pub fn can_be_referenced_by_value_as_type(self) -> bool {
        self.intersects(Self::Value | Self::Import | Self::Function | Self::TypeImport)
    }
}
