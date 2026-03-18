use bitflags::bitflags;

bitflags! {
    /// Additional flags for types with `TypeFlags::Object`, `TypeFlags::Union`,
    /// `TypeFlags::Intersection`, or certain other type flags.
    ///
    /// Some flags are specific to certain type kinds and reuse the same bit position.
    /// Ported from TypeScript's `ObjectFlags`.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ObjectFlags: u32 {
        const None                                       = 0;
        const Class                                      = 1 << 0;
        const Interface                                  = 1 << 1;
        const Reference                                  = 1 << 2;
        const Tuple                                      = 1 << 3;
        const Anonymous                                  = 1 << 4;
        const Mapped                                     = 1 << 5;
        const Instantiated                               = 1 << 6;
        const ObjectLiteral                              = 1 << 7;
        const EvolvingArray                              = 1 << 8;
        const ObjectLiteralPatternWithComputedProperties = 1 << 9;
        const ReverseMapped                              = 1 << 10;
        const JsxAttributes                              = 1 << 11;
        const JSLiteral                                  = 1 << 12;
        const FreshLiteral                               = 1 << 13;
        const ArrayLiteral                               = 1 << 14;
        const PrimitiveUnion                             = 1 << 15;
        const ContainsWideningType                       = 1 << 16;
        const ContainsObjectOrArrayLiteral               = 1 << 17;
        const NonInferrableType                          = 1 << 18;
        const CouldContainTypeVariablesComputed          = 1 << 19;
        const CouldContainTypeVariables                  = 1 << 20;
        const MembersResolved                            = 1 << 21;
        const ContainsSpread                             = 1 << 22;
        const ObjectRestType                             = 1 << 23;
        const InstantiationExpressionType                = 1 << 24;
        const SingleSignatureType                        = 1 << 25;
        const IsClassInstanceClone                       = 1 << 26;

        const ClassOrInterface   = Self::Class.bits() | Self::Interface.bits();
        const InstantiatedMapped = Self::Mapped.bits() | Self::Instantiated.bits();
        const RequiresWidening   = Self::ContainsWideningType.bits() | Self::ContainsObjectOrArrayLiteral.bits();
        const PropagatingFlags   = Self::ContainsWideningType.bits() | Self::ContainsObjectOrArrayLiteral.bits() | Self::NonInferrableType.bits();
    }
}
