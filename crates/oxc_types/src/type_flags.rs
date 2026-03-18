use bitflags::bitflags;

bitflags! {
    /// Flags that classify what kind of type a `TypeId` represents.
    ///
    /// Ported from TypeScript's `TypeFlags`. The numeric ordering matters:
    /// for types of different kinds, these values determine the sort order of
    /// constituent types in union types.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TypeFlags: u32 {
        const None            = 0;
        const Any             = 1 << 0;
        const Unknown         = 1 << 1;
        const Undefined       = 1 << 2;
        const Null            = 1 << 3;
        const Void            = 1 << 4;
        const String          = 1 << 5;
        const Number          = 1 << 6;
        const BigInt          = 1 << 7;
        const Boolean         = 1 << 8;
        const ESSymbol        = 1 << 9;
        const StringLiteral   = 1 << 10;
        const NumberLiteral   = 1 << 11;
        const BigIntLiteral   = 1 << 12;
        const BooleanLiteral  = 1 << 13;
        const UniqueESSymbol  = 1 << 14;
        const EnumLiteral     = 1 << 15;
        const Enum            = 1 << 16;
        const NonPrimitive    = 1 << 17; // intrinsic object type
        const Never           = 1 << 18;
        const TypeParameter   = 1 << 19;
        const Object          = 1 << 20;
        const Index           = 1 << 21; // keyof T
        const TemplateLiteral = 1 << 22;
        const StringMapping   = 1 << 23; // Uppercase/Lowercase type
        const Substitution    = 1 << 24;
        const IndexedAccess   = 1 << 25; // T[K]
        const Conditional     = 1 << 26; // T extends U ? X : Y
        const Union           = 1 << 27;
        const Intersection    = 1 << 28;

        // Composite flags
        const AnyOrUnknown          = Self::Any.bits() | Self::Unknown.bits();
        const Nullable              = Self::Undefined.bits() | Self::Null.bits();
        const Literal               = Self::StringLiteral.bits() | Self::NumberLiteral.bits() | Self::BigIntLiteral.bits() | Self::BooleanLiteral.bits();
        const Unit                  = Self::Enum.bits() | Self::Literal.bits() | Self::UniqueESSymbol.bits() | Self::Nullable.bits();
        const Freshable             = Self::Enum.bits() | Self::Literal.bits();
        const StringOrNumberLiteral = Self::StringLiteral.bits() | Self::NumberLiteral.bits();

        const StringLike    = Self::String.bits() | Self::StringLiteral.bits() | Self::TemplateLiteral.bits() | Self::StringMapping.bits();
        const NumberLike    = Self::Number.bits() | Self::NumberLiteral.bits() | Self::Enum.bits();
        const BigIntLike    = Self::BigInt.bits() | Self::BigIntLiteral.bits();
        const BooleanLike   = Self::Boolean.bits() | Self::BooleanLiteral.bits();
        const EnumLike      = Self::Enum.bits() | Self::EnumLiteral.bits();
        const ESSymbolLike  = Self::ESSymbol.bits() | Self::UniqueESSymbol.bits();
        const VoidLike      = Self::Void.bits() | Self::Undefined.bits();

        const Intrinsic = Self::Any.bits() | Self::Unknown.bits() | Self::String.bits()
            | Self::Number.bits() | Self::BigInt.bits() | Self::ESSymbol.bits()
            | Self::Void.bits() | Self::Undefined.bits() | Self::Null.bits()
            | Self::Never.bits() | Self::NonPrimitive.bits();

        const Primitive = Self::StringLike.bits() | Self::NumberLike.bits() | Self::BigIntLike.bits()
            | Self::BooleanLike.bits() | Self::EnumLike.bits() | Self::ESSymbolLike.bits()
            | Self::VoidLike.bits() | Self::Null.bits();

        const DefinitelyNonNullable = Self::StringLike.bits() | Self::NumberLike.bits()
            | Self::BigIntLike.bits() | Self::BooleanLike.bits() | Self::EnumLike.bits()
            | Self::ESSymbolLike.bits() | Self::Object.bits() | Self::NonPrimitive.bits();

        const UnionOrIntersection    = Self::Union.bits() | Self::Intersection.bits();
        const StructuredType         = Self::Object.bits() | Self::Union.bits() | Self::Intersection.bits();
        const TypeVariable           = Self::TypeParameter.bits() | Self::IndexedAccess.bits();
        const InstantiableNonPrimitive = Self::TypeVariable.bits() | Self::Conditional.bits() | Self::Substitution.bits();
        const InstantiablePrimitive  = Self::Index.bits() | Self::TemplateLiteral.bits() | Self::StringMapping.bits();
        const Instantiable           = Self::InstantiableNonPrimitive.bits() | Self::InstantiablePrimitive.bits();
        const StructuredOrInstantiable = Self::StructuredType.bits() | Self::Instantiable.bits();

        const Narrowable = Self::Any.bits() | Self::Unknown.bits() | Self::StructuredOrInstantiable.bits()
            | Self::StringLike.bits() | Self::NumberLike.bits() | Self::BigIntLike.bits()
            | Self::BooleanLike.bits() | Self::ESSymbol.bits() | Self::UniqueESSymbol.bits()
            | Self::NonPrimitive.bits();
    }
}
