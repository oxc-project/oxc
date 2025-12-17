/*
 * Methods related to globals.
 */

import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

/**
 * Globals for the file being linted.
 *
 * Globals are deserialized from JSON, so can only contain JSON-compatible values.
 * Each global variable maps to "readonly", "writable", or "off".
 */
export type Globals = Record<string, "readonly" | "writable" | "off">;

// Empty globals object.
// When globals are empty, we use this singleton object to avoid allocating a new object each time.
const EMPTY_GLOBALS: Globals = Object.freeze({});

// Globals for current file.
// `globalsJSON` is set before linting a file by `setGlobalsForFile`.
// `globals` is deserialized from `globalsJSON` lazily upon first access.
let globalsJSON: string | null = null;
export let globals: Readonly<Globals> | null = null;

/**
 * Updates the globals for the file.
 *
 * TODO(perf): Globals are deserialized once per file to accommodate folder level settings,
 * even if the globals haven't changed.
 *
 * @param globalsJSONInput - Globals for the file as JSON
 */
export function setGlobalsForFile(globalsJSONInput: string): undefined {
  globalsJSON = globalsJSONInput;
}

/**
 * Deserialize globals from JSON.
 *
 * Caller must ensure that `globalsJSON` has been initialized before calling this function.
 */
export function initGlobals(): void {
  debugAssertIsNonNull(globalsJSON);

  if (globalsJSON === "{}") {
    // Re-use a single object for empty globals as an optimization
    globals = EMPTY_GLOBALS;
  } else {
    globals = JSON.parse(globalsJSON);

    // Freeze the globals object, to prevent any mutation of `globals` by plugins.
    // No need to deep freeze since all keys are just strings.
    Object.freeze(globals);
  }

  debugAssertIsNonNull(globals);
  debugAssert(typeof globals === "object" && !Array.isArray(globals));
}

/**
 * Reset globals.
 */
export function resetGlobals(): undefined {
  globals = null;
  globalsJSON = null;
}
