// Test various namespace reference scenarios

// Simple namespace reference
export type SimpleRef = NS1.Type1;

namespace NS1 {
  export type Type1 = string;
}

// Nested namespace reference
export type NestedRef = Outer.Inner.Type2;

namespace Outer {
  export namespace Inner {
    export type Type2 = number;
  }
}

// Multiple references to same namespace
export type Ref1 = NS2.TypeA;
export type Ref2 = NS2.TypeB;

namespace NS2 {
  export type TypeA = boolean;
  export type TypeB = symbol;
}

// Namespace used in union type
export type UnionRef = NS3.Type3 | string;

namespace NS3 {
  export type Type3 = "literal";
}

// Namespace used in interface
export interface InterfaceWithNS {
  field: NS4.Type4;
}

namespace NS4 {
  export type Type4 = bigint;
}

// Unreferenced namespace should not be included
namespace UnusedNS {
  export type UnusedType = never;
}
