/*
 * Methods related to globals.
 */

import { deepFreezeJsonValue } from "./json.js";
import { debugAssertIsNonNull } from "../utils/asserts.js";

/**
 * Globals for the file being linted.
 *
 * Globals are deserialized from JSON, so can only contain JSON-compatible values.
 * Each global variable maps to "readonly", "writable", "writeable", or "off".
 * Note: "writeable" is the misspelled version used by Oxlint (for compatibility),
 * which gets converted to "writable" for ESLint compatibility.
 */
export type Globals = Record<string, "readonly" | "writable" | "writeable" | "off">;

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
 */
export function initGlobals(): undefined {
  debugAssertIsNonNull(globalsJSON);
  globals = JSON.parse(globalsJSON);
  // Deep freeze the globals object, to prevent any mutation of the globals from plugins
  deepFreezeJsonValue(globals);
}

/**
 * Reset globals.
 */
export function resetGlobals(): undefined {
  globals = null;
  globalsJSON = null;
}
