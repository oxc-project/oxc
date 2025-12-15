/*
 * Options for rules.
 */

import assert from "node:assert";
import { registeredRules } from "./load.ts";
import {
  deepFreezeJsonValue as deepFreezeValue,
  deepFreezeJsonArray as deepFreezeArray,
  deepFreezeJsonObject as deepFreezeObject,
} from "./json.ts";
import { debugAssertIsNonNull } from "../utils/asserts.ts";

import type { Writable } from "type-fest";
import type { JsonValue } from "./json.ts";

const { freeze } = Object,
  { isArray } = Array,
  { min } = Math;

/**
 * Options for a rule on a file.
 */
export type Options = JsonValue[];

/**
 * Schema describing valid options for a rule.
 * `schema` property of `RuleMeta`.
 *
 * `false` opts out of schema validation. This is not recommended, as it increases the chance of bugs and mistakes.
 */
// TODO: Make this more precise.
// TODO: Use this to validate options in configs.
export type RuleOptionsSchema = Record<string, unknown> | unknown[] | false;

// Default rule options
export const DEFAULT_OPTIONS: Readonly<Options> = Object.freeze([]);

// All rule options.
// `lintFile` is called with an array of options IDs, which are indices into this array.
// First element is irrelevant - never accessed - because 0 index is a sentinel meaning default options.
export let allOptions: Readonly<Options>[] | null = null;

// Index into `allOptions` for default options
export const DEFAULT_OPTIONS_ID = 0;

/**
 * Set all external rule options.
 * Called once from Rust after config building, before any linting occurs.
 * @param optionsJSON - Array of all rule options across all configurations, serialized as JSON
 */
export function setOptions(optionsJson: string): void {
  const details = JSON.parse(optionsJson);
  allOptions = details.options;
  debugAssertIsNonNull(allOptions);

  const { ruleIds } = details;

  // Validate
  if (DEBUG) {
    assert(isArray(allOptions), `options must be an array, got ${typeof allOptions}`);
    assert(isArray(ruleIds), `ruleIds must be an array, got ${typeof allOptions}`);
    assert.strictEqual(
      allOptions.length,
      ruleIds.length,
      "ruleIds and options arrays must be the same length",
    );

    for (const options of allOptions) {
      assert(isArray(options), `Elements of options must be arrays, got ${typeof options}`);
    }

    for (const ruleId of ruleIds) {
      assert(
        typeof ruleId === "number" && ruleId >= 0 && ruleId === Math.floor(ruleId),
        `Elements of ruleIds must be non-negative integers, got ${ruleId}`,
      );
    }
  }

  // Merge each options array with default options for their corresponding rule.
  // Skip the first, as index 0 is a sentinel value meaning default options. First element is never accessed.
  // `mergeOptions` also deep-freezes the options.
  for (let i = 1, len = allOptions.length; i < len; i++) {
    allOptions[i] = mergeOptions(
      // `allOptions`' type is `Readonly`, but the array is mutable at present
      allOptions[i] as Writable<(typeof allOptions)[number]>,
      registeredRules[ruleIds[i]].defaultOptions,
    );
  }
}

/**
 * Initialize `allOptions` to 1-element array.
 * The first element is irrelevant and never accessed.
 *
 * This function is only used in `RuleTester`.
 * Main linter process uses `setOptions` instead.
 */
export function initAllOptions(): void {
  allOptions = [DEFAULT_OPTIONS];
}

/**
 * Merge user-provided options from config with rule's default options.
 *
 * Config options take precedence over default options.
 *
 * Returned options are deep frozen.
 * `configOptions` may be frozen in place (or partially frozen) too.
 * `defaultOptions` must already be deep frozen before calling this function.
 *
 * Follows the same merging logic as ESLint's `getRuleOptions`.
 * https://github.com/eslint/eslint/blob/0f5a94a84beee19f376025c74f703f275d52c94b/lib/linter/linter.js#L443-L454
 * https://github.com/eslint/eslint/blob/0f5a94a84beee19f376025c74f703f275d52c94b/lib/shared/deep-merge-arrays.js
 *
 * Notably, nested arrays are not merged - config options wins. e.g.:
 * - Config options:  [ [1] ]
 * - Default options: [ [2, 3], 4 ]
 * - Merged options:  [ [1], 4 ]
 *
 * @param configOptions - Options from config
 * @param defaultOptions - Default options from `rule.meta.defaultOptions`
 * @returns Merged options
 */
export function mergeOptions(
  configOptions: Options,
  defaultOptions: Readonly<Options>,
): Readonly<Options> {
  if (defaultOptions === DEFAULT_OPTIONS) {
    deepFreezeArray(configOptions);
    return configOptions;
  }

  // Both are defined - merge them
  const merged = [];

  const defaultOptionsLength = defaultOptions.length,
    ruleOptionsLength = configOptions.length,
    bothLength = min(defaultOptionsLength, ruleOptionsLength);

  let i = 0;
  for (; i < bothLength; i++) {
    merged.push(mergeValues(configOptions[i], defaultOptions[i]));
  }

  if (defaultOptionsLength > ruleOptionsLength) {
    for (; i < defaultOptionsLength; i++) {
      merged.push(defaultOptions[i]);
    }
  } else {
    for (; i < ruleOptionsLength; i++) {
      const prop = configOptions[i];
      deepFreezeValue(prop);
      merged.push(prop);
    }
  }

  return freeze(merged);
}

/**
 * Merge value from user-provided options with value from default options.
 *
 * @param configValue - Value from config
 * @param defaultValue - Value from default options
 * @returns Merged value
 */
function mergeValues(configValue: JsonValue, defaultValue: JsonValue): JsonValue {
  // If config value is a primitive, it wins
  if (configValue === null || typeof configValue !== "object") return configValue;

  // If config value is an array, it wins
  if (isArray(configValue)) {
    deepFreezeArray(configValue);
    return configValue;
  }

  // If default value is a primitive or an array, config value wins (it's an object)
  if (defaultValue === null || typeof defaultValue !== "object" || isArray(defaultValue)) {
    deepFreezeObject(configValue);
    return configValue;
  }

  // Both are objects (not arrays)
  const merged = { ...defaultValue, ...configValue };

  // Symbol properties are not possible in JSON, so no need to handle them here.
  //
  // `defaultValue` is not from JSON, so we can't use a simple `for..in` loop over `defaultValue`.
  // That would also pick up enumerable properties from prototype of `defaultValue`.
  // `configValue` *is* from JSON, so simple `key in configValue` check is fine.
  //
  // A malicious plugin could potentially get up to mischief here (prototype pollution?) if `defaultValue` is a `Proxy`.
  // But plugins are executable code, so they have far easier ways to do that. No point in defending against it here.
  for (const key of Object.keys(defaultValue)) {
    if (key in configValue) {
      // `key` is an own property of both `configValue` and `defaultValue`, so must be an own property of `merged` too.
      // Therefore, we don't need special handling for if `key` is `"__proto__"`.
      // All the property reads and writes here will affect only the owned properties of these objects,
      // (including if those properties are named `"__proto__"`).
      merged[key] = mergeValues(configValue[key], defaultValue[key]);
    } else {
      deepFreezeValue(configValue[key]);
    }
  }

  return freeze(merged);
}
