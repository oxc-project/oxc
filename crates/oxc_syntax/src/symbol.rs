#![allow(non_upper_case_globals)]

use bitflags::bitflags;
use oxc_index::define_index_type;

define_index_type! {
    pub struct SymbolId = u32;
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SymbolFlags: u32 {
        const None                    = 0;
        /// Variable (var) or parameter
        const FunctionScopedVariable  = 1 << 0;
        /// A block-scoped variable (let or const)
        const BlockScopedVariable     = 1 << 1;
        /// A const variable (const)
        const ConstVariable           = 1 << 2;
        /// Is this symbol inside an import declaration
        const Import                  = 1 << 3;
        /// Is this symbol inside an export declaration
        const Export                  = 1 << 4;
        const Class                   = 1 << 5;
        const CatchVariable           = 1 << 6; // try {} catch(catch_variable) {}
        const Function                = 1 << 7;
        const ImportBinding           = 1 << 8; // Imported ESM binding
        // Type specific symbol flags
        const TypeAlias               = 1 << 9;
        const Interface               = 1 << 10;
        const RegularEnum             = 1 << 11;
        const ConstEnum               = 1 << 12;
        const EnumMember              = 1 << 13;
        const TypeLiteral             = 1 << 14;
        const TypeParameter           = 1 << 15;
        const NameSpaceModule         = 1 << 16;
        const ValueModule             = 1 << 17;
        // In a dts file or there is a declare flag
        const Ambient                 = 1 << 18;

        const Enum = Self::ConstEnum.bits() | Self::RegularEnum.bits();

        const Variable = Self::FunctionScopedVariable.bits() | Self::BlockScopedVariable.bits();
        const Value = Self::Variable.bits() | Self::Class.bits() | Self::Enum.bits() | Self::ValueModule.bits();
        const Type =  Self::Class.bits() | Self::Interface.bits() | Self::Enum.bits() | Self::TypeLiteral.bits() | Self::TypeParameter.bits()  |  Self::TypeAlias.bits();

        /// Variables can be redeclared, but can not redeclare a block-scoped declaration with the
        /// same name, or any other value that is not a variable, e.g. ValueModule or Class
        const FunctionScopedVariableExcludes = Self::Value.bits() - Self::FunctionScopedVariable.bits();

        /// Block-scoped declarations are not allowed to be re-declared
        /// they can not merge with anything in the value space
        const BlockScopedVariableExcludes = Self::Value.bits();

        const ClassExcludes = (Self::Value.bits() | Self::TypeAlias.bits()) & !Self::ValueModule.bits() ;
        const ImportBindingExcludes = Self::ImportBinding.bits();
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
    pub fn is_variable(&self) -> bool {
        self.intersects(Self::Variable)
    }

    pub fn is_type(&self) -> bool {
        !self.intersects(Self::Value)
    }

    pub fn is_const_variable(&self) -> bool {
        self.contains(Self::ConstVariable)
    }

    pub fn is_function(&self) -> bool {
        self.contains(Self::Function)
    }

    pub fn is_class(&self) -> bool {
        self.contains(Self::Class)
    }

    pub fn is_catch_variable(&self) -> bool {
        self.contains(Self::CatchVariable)
    }

    pub fn is_function_scoped_declaration(&self) -> bool {
        self.contains(Self::FunctionScopedVariable)
    }

    pub fn is_export(&self) -> bool {
        self.contains(Self::Export)
    }

    pub fn is_import_binding(&self) -> bool {
        self.contains(Self::ImportBinding)
    }
}
