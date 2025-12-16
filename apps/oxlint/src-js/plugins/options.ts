/*
 * Options for rules.
 */

import assert from "node:assert";
import Ajv from "ajv";
import ajvPackageJson from "ajv/package.json" with { type: "json" };
import metaSchema from "ajv/lib/refs/json-schema-draft-04.json" with { type: "json" };
import { registeredRules } from "./load.ts";
import { deepCloneJsonValue, deepFreezeJsonArray } from "./json.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

import type { JSONSchema4 } from "json-schema";
import type { Writable } from "type-fest";
import type { JsonValue } from "./json.ts";
import type { RuleDetails } from "./load.ts";

export type SchemaValidator = Ajv.ValidateFunction;

// =================================================================================================
// Options types and constants
// =================================================================================================

/**
 * Options for a rule on a file.
 */
export type Options = JsonValue[];

/**
 * Schema describing valid options for a rule.
 * `schema` property of `RuleMeta`.
 *
 * Can be one of:
 * - `JSONSchema4` - Full JSON Schema object (must have `type: "array"` at root).
 * - `JSONSchema4[]` - Array shorthand where each element describes corresponding options element.
 * - `false` - Opts out of schema validation (not recommended).
 */
export type RuleOptionsSchema = JSONSchema4 | JSONSchema4[] | false;

// Default rule options
export const DEFAULT_OPTIONS: Readonly<Options> = Object.freeze([]);

// All rule options.
// `lintFile` is called with an array of options IDs, which are indices into this array.
// First element is irrelevant - never accessed - because 0 index is a sentinel meaning default options.
export let allOptions: Readonly<Options>[] | null = null;

// Index into `allOptions` for default options
export const DEFAULT_OPTIONS_ID = 0;

// =================================================================================================
// Schema compilation
// =================================================================================================

// ESLint uses AJV v6.
// AJV v7 removed support for JSON Schema draft-04, which ESLint rule schemas use.
// We must stay on v6 to match ESLint's behavior.
debugAssert(
  ajvPackageJson.version.startsWith("6."),
  `AJV must be v6.x for JSON Schema draft-04 support, got ${ajvPackageJson.version}`,
);

// AJV instance configured to match ESLint's behavior.
// `useDefaults: true` applies schema `default` values to options during validation.
//
// Based on ESLint's AJV configuration:
// https://github.com/eslint/eslint/blob/v9.39.2/lib/config/config.js#L15-L16
// https://github.com/eslint/eslint/blob/v9.39.2/lib/shared/ajv.js#L18-L34
const AJV = new Ajv({
  meta: false,
  useDefaults: true,
  validateSchema: false,
  missingRefs: "ignore",
  verbose: true,
  schemaId: "auto",
});
AJV.addMetaSchema(metaSchema);
// Ajv internal API
(AJV._opts as { defaultMeta: string }).defaultMeta = metaSchema.id;

// Schema for rules with no options.
// Currently this is unused because we don't support options validation, but leaving it here for when we do.
// oxlint-disable-next-line no-unused-vars
const NO_OPTIONS_SCHEMA: JSONSchema4 = {
  type: "array",
  minItems: 0,
  maxItems: 0,
};

/**
 * Compile a rule's schema into a validator function.
 *
 * This should be called once when loading a rule, and the returned validator stored in `RuleDetails`.
 *
 * ESLint allows array shorthand: `schema: [item1, item2]` which means options[0] must match `item1`, etc.
 * This function converts that to a proper JSON Schema, before compiling it.
 *
 * Based on ESLint's `getRuleOptionsSchema`:
 * https://github.com/eslint/eslint/blob/v9.39.2/lib/config/config.js#L177-L210
 *
 * @param schema - Rule's schema from `meta.schema`
 * @returns Compiled AJV validator, or `null` if no schema
 */
export function compileSchema(
  schema: RuleOptionsSchema | null | undefined,
): SchemaValidator | null {
  // `null`, `undefined`, or `false` means no schema validation
  if (schema == null || schema === false) return null;

  // TODO: Does AJV already do this validation?
  if (typeof schema !== "object") {
    throw new TypeError("`rule.meta.schema` must be an array, object, or `false` if provided");
  }

  if (Array.isArray(schema)) {
    // TODO: Once we support options validation, we should return `NO_OPTIONS_SCHEMA` instead of `null`
    if (schema.length === 0) return null;

    schema = {
      type: "array",
      items: schema,
      minItems: 0,
      maxItems: schema.length,
    };
  }

  return AJV.compile(schema);
}

// =================================================================================================
// Options processing
// =================================================================================================

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
    assert(Array.isArray(allOptions), `options must be an array, got ${typeof allOptions}`);
    assert(Array.isArray(ruleIds), `ruleIds must be an array, got ${typeof allOptions}`);
    assert.strictEqual(
      allOptions.length,
      ruleIds.length,
      "ruleIds and options arrays must be the same length",
    );

    for (const options of allOptions) {
      assert(Array.isArray(options), `Elements of options must be arrays, got ${typeof options}`);
    }

    for (const ruleId of ruleIds) {
      assert(
        typeof ruleId === "number" && ruleId >= 0 && ruleId === Math.floor(ruleId),
        `Elements of ruleIds must be non-negative integers, got ${ruleId}`,
      );
    }
  }

  // Process each options array.
  // For each options array, merge with default options and apply schema defaults for the corresponding rule.
  // Skip the first, as index 0 is a sentinel value meaning default options. First element is never accessed.
  // `processOptions` also deep-freezes the options.
  for (let i = 1, len = allOptions.length; i < len; i++) {
    allOptions[i] = processOptions(
      // `allOptions`' type is `Readonly`, but the array is mutable at present
      allOptions[i] as Writable<(typeof allOptions)[number]>,
      registeredRules[ruleIds[i]],
    );
  }
}

/**
 * Process user-provided options for a rule by applying the rule's defaults
 * - merging with default options and applying schema defaults for the rule.
 *
 * Order of operations (matching ESLint's behavior):
 * 1. Merge with `defaultOptions` (config options take precedence).
 * 2. Apply schema defaults via AJV (fills in remaining gaps).
 *
 * This order ensures precedence: config > defaultOptions > schema defaults.
 *
 * ESLint calls `#normalizeRulesConfig()` first (merges `defaultOptions`), then `validateRulesConfig()` (AJV):
 * https://github.com/eslint/eslint/blob/v9.39.2/lib/config/config.js#L483-L484
 * https://github.com/eslint/eslint/blob/v9.39.2/lib/config/config.js#L532-L637
 *
 * Returned options are deep frozen.
 * `defaultOptions` must already be deep frozen before calling this function.
 * `configOption` may be mutated.
 *
 * Default options merging follows the same logic as ESLint's `getRuleOptions`:
 * https://github.com/eslint/eslint/blob/0f5a94a84beee19f376025c74f703f275d52c94b/lib/linter/linter.js#L443-L454
 * https://github.com/eslint/eslint/blob/0f5a94a84beee19f376025c74f703f275d52c94b/lib/shared/deep-merge-arrays.js
 *
 * Notably, nested arrays are not merged - config options wins. e.g.:
 * - Config options:  [ [1] ]
 * - Default options: [ [2, 3], 4 ]
 * - Merged options:  [ [1], 4 ]
 *
 * @param configOptions - Options from config (may be mutated by AJV)
 * @param ruleDetails - Rule details
 * @returns Processed options (deep frozen)
 */
function processOptions(configOptions: Options, ruleDetails: RuleDetails): Readonly<Options> {
  // Merge with `defaultOptions` first
  const { defaultOptions } = ruleDetails;

  const options =
    defaultOptions === DEFAULT_OPTIONS
      ? configOptions
      : mergeOptions(configOptions, defaultOptions);

  // Apply schema defaults (mutates `options` in place).
  //
  // AJV validation with `useDefaults: true` fills in default values from the schema.
  // `mergeOptions` cloned `defaultOptions`, so mutations made by AJV validation won't affect `defaultOptions`
  // (and `defaultOptions` is frozen anyway, so it can't be mutated).
  // `configOptions` may be mutated, but that's OK, because we only use it once.
  //
  // We ignore validation errors - we only care about applying defaults.
  // TODO: Pass validation errors back to Rust.
  const validator = ruleDetails.optionsSchemaValidator;
  if (validator !== null) validator(options);

  deepFreezeJsonArray(options);
  return options;
}

/**
 * Merge user-provided options from config with rule's default options.
 *
 * Config options take precedence over default options.
 *
 * Options returned are entirely mutable. No parts are frozen, even parts which come from default options.
 * Any parts of `defaultOptions` which are included are deep cloned.
 * Any parts of `configOptions` which are included in return value are *not* cloned.
 *
 * @param configOptions - Options from config
 * @param defaultOptions - Default options from `rule.meta.defaultOptions`
 * @returns Merged options (mutable, not frozen)
 */
function mergeOptions(configOptions: Options, defaultOptions: Readonly<Options>): Options {
  const merged: Options = [];

  const defaultOptionsLength = defaultOptions.length,
    configOptionsLength = configOptions.length,
    bothLength = Math.min(defaultOptionsLength, configOptionsLength);

  // Merge elements shared by both arrays
  let i = 0;
  for (; i < bothLength; i++) {
    merged.push(mergeValues(configOptions[i], defaultOptions[i]));
  }

  // Take remaining elements from whichever array is longer
  if (defaultOptionsLength > configOptionsLength) {
    // `defaultOptions` has more elements - deep clone remaining elements
    for (; i < defaultOptionsLength; i++) {
      merged.push(deepCloneJsonValue(defaultOptions[i]));
    }
  } else {
    // `configOptions` has more elements - just copy references (will be frozen later)
    for (; i < configOptionsLength; i++) {
      merged.push(configOptions[i]);
    }
  }

  return merged;
}

/**
 * Merge value from user-provided options with value from default options.
 *
 * Config value takes precedence over default value.
 * Returns a mutable value (not frozen) - caller is responsible for freezing.
 * `configValue` is mutated in place.
 *
 * @param configValue - Value from config
 * @param defaultValue - Value from default options
 * @returns Merged value (mutable)
 */
function mergeValues(configValue: JsonValue, defaultValue: JsonValue): JsonValue {
  // If config value is a primitive or array, it wins
  if (configValue === null || typeof configValue !== "object" || Array.isArray(configValue)) {
    return configValue;
  }

  // If default value is a primitive or an array, config value wins (it's an object)
  if (defaultValue === null || typeof defaultValue !== "object" || Array.isArray(defaultValue)) {
    return configValue;
  }

  // Both are objects (not arrays) - merge `defaultValue` into `configValue`.
  //
  // Symbol properties and circular references are not possible in JSON, so no need to handle them here.
  // `defaultValue` is from JSON, so we can use a simple `for..in` loop over `defaultValue`.
  for (const key in defaultValue) {
    // `Object.hasOwn` not `in`, in case `key` is `"__proto__"`
    if (Object.hasOwn(configValue, key)) {
      // Both have this key - recursively merge.
      // `key` is an own property of both `configValue` and `defaultValue`.
      // Therefore, we don't need special handling for if `key` is `"__proto__"`.
      // All the property reads and writes here will affect only the owned properties of these objects,
      // (including if those properties are named `"__proto__"`).
      configValue[key] = mergeValues(configValue[key], defaultValue[key]);
    } else {
      // Only `defaultValue` has this key - deep clone and add to `configValue`.
      // `Object.defineProperty` in case `key` is `"__proto__"`.
      Object.defineProperty(configValue, key, {
        value: deepCloneJsonValue(defaultValue[key]),
        writable: true,
        enumerable: true,
        configurable: true,
      });
    }
  }

  return configValue;
}
