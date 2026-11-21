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
export const negativeConst = { get [-1](): number { return 1; }, set "-1"(v: number) {} } as const;
export const computedNumeric = { get [1](): number { return 1; }, set "1"(v: number) {} };
export const hexNumeric = { get 0x1(): number { return 1; }, set "1"(v: number) {} };
export const exponentNumeric = { get 1e0(): number { return 1; }, set "1"(v: number) {} };
export const negativeZeroPair = { get [-0](): number { return 1; }, set "-0"(v: number) {} };
export const negativeZeroStringDistinct = { get [-0](): number { return 1; }, set "0"(v: number) {} };
export const negativeZeroNumericDistinct = { get [-0](): number { return 1; }, set 0(v: number) {} };
export const positiveSignedPair = { get [+1](): number { return 1; }, set "+1"(v: number) {} };
export const positiveSignedDistinct = { get [+1](): number { return 1; }, set "1"(v: number) {} };
export const signedComputedPair = { get [-0x1](): number { return 1; }, set [-1e0](v: number) {} };
export const negativeSetterAnnotation = { get [-1]() { return Math.abs(-1); }, set "-1"(v: number) {} };
export const negativeGetterAnnotation = { set [-1](v) {}, get "-1"(): number { return 1; } };

// The shared matcher also supplies class accessor annotations across spellings.
export class SignedNumericAccessors {
  get [-1]() { return 1; }
  set "-1"(v: number) {}
}
