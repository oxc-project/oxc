/*
 * Methods and types related to JSON.
 */

/**
 * A JSON value.
 */
export type JsonValue = JsonObject | JsonValue[] | string | number | boolean | null;

/**
 * A JSON object.
 */
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

  if (Array.isArray(value)) {
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
  Object.freeze(obj);
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
  Object.freeze(arr);
}

// Benchmarking shows that this clone implementation is much faster than `structuredClone`
// and `JSON.parse(JSON.stringify(value))`.
// https://benchmarklab.azurewebsites.net/Benchmarks/ShowResult/622558

/**
 * Deep clone a JSON value, recursively cloning all nested objects and arrays.
 * @param value - The value to deep clone
 * @returns Cloned value
 */
export function deepCloneJsonValue(value: JsonValue): JsonValue {
  if (value === null || typeof value !== "object") return value;

  if (Array.isArray(value)) return deepCloneJsonArray(value);
  return deepCloneJsonObject(value);
}

/**
 * Deep clone a JSON object, recursively cloning all nested objects and arrays.
 * @param obj - The object to deep clone
 * @returns Cloned object
 */
export function deepCloneJsonObject(obj: JsonObject): JsonObject {
  // Circular references are not possible in JSON, so no need to handle them here.
  // Symbol properties are not possible in JSON, so no need to handle them here.
  // All properties are enumerable own properties, so can use simple `for..in` loop.
  // Clone `obj` into `cloned` first, so then don't need special handling for if object has a key called `__proto__`.
  const cloned = { ...obj };
  for (const key in cloned) {
    const value = cloned[key];
    if (typeof value !== "object" || value === null) continue;
    cloned[key] = Array.isArray(value) ? deepCloneJsonArray(value) : deepCloneJsonObject(value);
  }
  return cloned;
}

/**
 * Deep clone a JSON array, recursively cloning all nested objects and arrays.
 * @param arr - The array to deep clone
 * @returns Cloned array
 */
export function deepCloneJsonArray(arr: JsonValue[]): JsonValue[] {
  // Circular references are not possible in JSON, so no need to handle them here
  const cloned = [];
  for (let i = 0, len = arr.length; i !== len; i++) {
    cloned.push(deepCloneJsonValue(arr[i]));
  }
  return cloned;
}
