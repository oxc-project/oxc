/*
 * Methods and types related to JSON.
 */

const { isArray } = Array,
  { freeze } = Object;

/**
 * A JSON value.
 */
export type JsonValue = JsonObject | JsonValue[] | string | number | boolean | null;

/**
 * A JSON object.
 */
// Can't use `Record<string, JsonValue>` here, because of circular reference between `JsonObject` and `JsonValue`
// oxlint-disable-next-line typescript/consistent-indexed-object-style
export type JsonObject = { [key: string]: JsonValue };

/**
 * Deep freeze a JSON value, recursively freezing all nested objects and arrays.
 *
 * Freezes the value in place, and returns `undefined`.
 *
 * @param value - The value to deep freeze
 */
export function deepFreezeJsonValue(value: JsonValue): undefined {
  if (value === null || typeof value !== "object") return;

  if (isArray(value)) {
    deepFreezeJsonArray(value);
  } else {
    deepFreezeJsonObject(value);
  }
}

/**
 * Deep freeze a JSON object, recursively freezing all nested objects and arrays.
 *
 * Freezes the object in place, and returns `undefined`.
 *
 * @param obj - The value to deep freeze
 */
export function deepFreezeJsonObject(obj: JsonObject): undefined {
  // Circular references are not possible in JSON, so no need to handle them here.
  // Symbol properties are not possible in JSON, so no need to handle them here.
  // All properties are enumerable own properties, so can use simple `for..in` loop.
  for (const key in obj) {
    deepFreezeJsonValue(obj[key]);
  }
  freeze(obj);
}

/**
 * Deep freeze a JSON array, recursively freezing all nested objects and arrays.
 *
 * Freezes the array in place, and returns `undefined`.
 *
 * @param arr - The value to deep freeze
 */
export function deepFreezeJsonArray(arr: JsonValue[]): undefined {
  // Circular references are not possible in JSON, so no need to handle them here
  for (let i = 0, len = arr.length; i !== len; i++) {
    deepFreezeJsonValue(arr[i]);
  }
  freeze(arr);
}
