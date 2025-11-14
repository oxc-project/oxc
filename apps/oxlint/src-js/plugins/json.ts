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
export type JsonObject = { [key: string]: JsonValue };

/**
 * Deep freeze a JSON value, recursively freezing all nested objects and arrays.
 *
 * Freezes the value in place, and returns `undefined`.
 *
 * @param value - The value to deep freeze
 */
export function deepFreezeJsonValue(value: JsonValue): undefined {
  if (value === null || typeof value !== 'object') return;

  // Circular references are not possible in JSON, so no need to handle them here
  if (isArray(value)) {
    for (let i = 0, len = value.length; i !== len; i++) {
      deepFreezeJsonValue(value[i]);
    }
  } else {
    // Symbol properties are not possible in JSON, so no need to handle them here.
    // All properties are enumerable own properties, so can use simple `for..in` loop.
    for (const key in value) {
      deepFreezeJsonValue(value[key]);
    }
  }

  freeze(value);
}
