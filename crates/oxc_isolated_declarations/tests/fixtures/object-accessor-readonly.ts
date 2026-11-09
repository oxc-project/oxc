export const getter = { get value(): number { return 1; } };
export const setter = { set value(v: number) {} };
export const pair = { get value(): number { return 1; }, set value(v: number) {} };
export const reverse = { set value(v: number) {}, get value(): number { return 1; } };
export const computed = { get ["value"](): number { return 1; }, set ["value"](v: number) {} };
export const distinct = { get value(): number { return 1; }, set other(v: number) {} };
export const getterConst = { get value(): number { return 1; } } as const;
export const setterConst = { set value(v: number) {} } as const;
export const pairConst = { get value(): number { return 1; }, set value(v: number) {} } as const;
export const reverseConst = { set value(v: number) {}, get value(): number { return 1; } } as const;
export const computedConst = { get ["value"](): number { return 1; }, set ["value"](v: number) {} } as const;
export const distinctConst = { get value(): number { return 1; }, set other(v: number) {} } as const;
export const mixed = { get value(): number { return 1; }, set "value"(v: number) {} };
export const mixedReverse = { set "value"(v: number) {}, get value(): number { return 1; } };
export const numeric = { get 1(): number { return 1; }, set "1"(v: number) {} };
export const mixedConst = { get value(): number { return 1; }, set "value"(v: number) {} } as const;
export const mixedReverseConst = { set "value"(v: number) {}, get value(): number { return 1; } } as const;
export const numericConst = { get 1(): number { return 1; }, set "1"(v: number) {} } as const;

export const exponentialDistinct = {
  get 1e21(): number { return 1; },
  set "1000000000000000000000"(v: string) {},
};
export const exponentialPair = {
  get 1e21(): number { return 1; },
  set "1e+21"(v: number) {},
} as const;
export const smallDistinct = {
  set "0.0000001"(v: string) {},
  get 1e-7(): number { return 1; },
} as const;
export const smallPair = {
  set "1e-7"(v: number) {},
  get 1e-7(): number { return 1; },
};
export class NumericAccessors {
  get 1e21(): number { return 1; }
  set "1000000000000000000000"(v: string) {}
}

// Signed names follow TypeScript's binder, preserving the sign even for zero.
export const negative = { get [-1](): number { return 1; }, set "-1"(v: number) {} };
export const negativeReverse = { set "-1"(v: number) {}, get [-1](): number { return 1; } };
export const negativeConst = { get [-1](): number { return 1; }, set "-1"(v: number) {} } as const;
export const computedNumeric = { get [1](): number { return 1; }, set "1"(v: number) {} };
export const computedNumericReverse = { set "1"(v: number) {}, get [1](): number { return 1; } };
export const computedNumericConst = { get [1](): number { return 1; }, set "1"(v: number) {} } as const;
export const hexNumeric = { get 0x1(): number { return 1; }, set "1"(v: number) {} };
export const hexNumericReverse = { set "1"(v: number) {}, get 0x1(): number { return 1; } };
export const hexNumericConst = { get 0x1(): number { return 1; }, set "1"(v: number) {} } as const;
export const exponentNumeric = { get 1e0(): number { return 1; }, set "1"(v: number) {} };
export const exponentNumericReverse = { set "1"(v: number) {}, get 1e0(): number { return 1; } };
export const exponentNumericConst = { get 1e0(): number { return 1; }, set "1"(v: number) {} } as const;
export const negativeHex = { get [-0x1](): number { return 1; }, set "-1"(v: number) {} };
export const negativeHexReverse = { set "-1"(v: number) {}, get [-0x1](): number { return 1; } };
export const negativeHexConst = { get [-0x1](): number { return 1; }, set "-1"(v: number) {} } as const;
export const negativeExponent = { get [-1e0](): number { return 1; }, set "-1"(v: number) {} };
export const negativeExponentReverse = { set "-1"(v: number) {}, get [-1e0](): number { return 1; } };
export const negativeExponentConst = { get [-1e0](): number { return 1; }, set "-1"(v: number) {} } as const;
export const negativeZeroPair = { get [-0](): number { return 1; }, set "-0"(v: number) {} };
export const negativeZeroPairReverse = { set "-0"(v: number) {}, get [-0](): number { return 1; } };
export const negativeZeroPairConst = { get [-0](): number { return 1; }, set "-0"(v: number) {} } as const;
export const negativeZeroStringDistinct = { get [-0](): number { return 1; }, set "0"(v: number) {} };
export const negativeZeroStringDistinctReverse = { set "0"(v: number) {}, get [-0](): number { return 1; } };
export const negativeZeroStringDistinctConst = { get [-0](): number { return 1; }, set "0"(v: number) {} } as const;
export const negativeZeroNumericDistinct = { get [-0](): number { return 1; }, set 0(v: number) {} };
export const negativeZeroNumericDistinctReverse = { set 0(v: number) {}, get [-0](): number { return 1; } };
export const negativeZeroNumericDistinctConst = { get [-0](): number { return 1; }, set 0(v: number) {} } as const;
export const positiveSignedPair = { get [+1](): number { return 1; }, set "+1"(v: number) {} };
export const positiveSignedPairReverse = { set "+1"(v: number) {}, get [+1](): number { return 1; } };
export const positiveSignedPairConst = { get [+1](): number { return 1; }, set "+1"(v: number) {} } as const;
export const positiveSignedDistinct = { get [+1](): number { return 1; }, set "1"(v: number) {} };
export const positiveSignedDistinctReverse = { set "1"(v: number) {}, get [+1](): number { return 1; } };
export const positiveSignedDistinctConst = { get [+1](): number { return 1; }, set "1"(v: number) {} } as const;

export const signedComputedPair = { get [-0x1](): number { return 1; }, set [-1e0](v: number) {} };
export const negativeGetter = { get [-1](): number { return 1; } };
export const negativeDistinct = { get [-1](): number { return 1; }, set "-2"(v: number) {} };

// Either accessor can supply the type even when the other cannot be inferred.
export const negativeSetterAnnotation = { get [-1]() { return Math.abs(-1); }, set "-1"(v: number) {} };
export const negativeGetterAnnotation = { set [-1](v) {}, get "-1"(): number { return 1; } };

// The shared matcher must also find class accessor annotations across spellings.
export class SignedNumericAccessors {
  get [-1]() { return 1; }
  set "-1"(v: number) {}
  static set [-0x1](v: string) {}
  static get "-1"() { return ""; }
  get [-0]() { return 0; }
  set "-0"(v: number) {}
  get [+1]() { return 1; }
  set "+1"(v: number) {}
}
