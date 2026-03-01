/**
 * Properties of global objects exported as variables.
 *
 * TSDown will replace e.g. `Object.keys` with import of `ObjectKeys` from this file.
 *
 * If you use any globals in code in `src-js` directory, you should add them to this file.
 *
 * See TSDown config file for more details.
 */

export const {
  prototype: ObjectPrototype,
  hasOwn: ObjectHasOwn,
  keys: ObjectKeys,
  values: ObjectValues,
  freeze: ObjectFreeze,
  defineProperty: ObjectDefineProperty,
  defineProperties: ObjectDefineProperties,
  create: ObjectCreate,
  assign: ObjectAssign,
  getPrototypeOf: ObjectGetPrototypeOf,
  setPrototypeOf: ObjectSetPrototypeOf,
  entries: ObjectEntries,
} = Object;

export const { prototype: ArrayPrototype, isArray: ArrayIsArray, from: ArrayFrom } = Array;

export const { min: MathMin, max: MathMax, floor: MathFloor } = Math;

export const { parse: JSONParse, stringify: JSONStringify } = JSON;

export const { ownKeys: ReflectOwnKeys } = Reflect;

export const { iterator: SymbolIterator } = Symbol;

export const { now: DateNow } = Date;
