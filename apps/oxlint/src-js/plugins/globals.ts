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

// Globals for current file.
// `globalsJSON` is set before linting a file by `setGlobalsForFile`.
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

  ({ globals } = JSON.parse(globalsJSON));

  debugAssert(
    typeof globals === "object" && globals !== null && !Array.isArray(globals),
    "`globals` should be an object",
  );

  Object.freeze(globals);
}

/**
 * Reset globals.
 */
export function resetGlobals(): undefined {
  globals = null;
  globalsJSON = null;
}
