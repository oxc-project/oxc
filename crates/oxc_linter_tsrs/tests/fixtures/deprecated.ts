/** @deprecated */
export class DeprecatedClass {
  /** @deprecated */
  foo: string = '';
}
/** @deprecated */
export const deprecatedVariable = 1;
/** @deprecated */
export function deprecatedFunction(): void {}
class NormalClass {}
const normalVariable = 1;
function normalFunction(): void;
function normalFunction(arg: string): void;
function normalFunction(arg?: string): void {}
function deprecatedFunctionWithOverloads(): void;
/** @deprecated */
function deprecatedFunctionWithOverloads(arg: string): void;
function deprecatedFunctionWithOverloads(arg?: string): void {}
export class ClassWithDeprecatedConstructor {
  constructor();
  /** @deprecated */
  constructor(arg: string);
  constructor(arg?: string) {}
}
export {
  /** @deprecated */
  NormalClass,
  /** @deprecated */
  normalVariable,
  /** @deprecated */
  normalFunction,
  deprecatedFunctionWithOverloads,
  /** @deprecated Reason */
  deprecatedFunctionWithOverloads as reexportedDeprecatedFunctionWithOverloads,
  /** @deprecated Reason */
  ClassWithDeprecatedConstructor as ReexportedClassWithDeprecatedConstructor,
};

/** @deprecated Reason */
export type T = { a: string };

export type U = { b: string };

/** @deprecated */
export default {
  foo: 1,
};
