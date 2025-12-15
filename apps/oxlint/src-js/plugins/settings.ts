/*
 * Methods related to settings.
 */

import { deepFreezeJsonValue } from "./json.ts";
import { debugAssertIsNonNull } from "../utils/asserts.ts";

import type { JsonObject } from "./json.ts";

/**
 * Settings for the file being linted.
 *
 * Settings are deserialized from JSON, so can only contain JSON-compatible values.
 */
export type Settings = JsonObject;

// Settings for current file.
// `settingsJSON` is set before linting a file by `setSettingsForFile`.
// `settings` is deserialized from `settingsJSON` lazily upon first access.
let settingsJSON: string | null = null;
export let settings: Readonly<Settings> | null = null;

/**
 * Updates the settings for the file.
 *
 * TODO(perf): Settings are deserialized once per file to accommodate folder level settings,
 * even if the settings haven't changed.
 *
 * @param settingsJSONInput - Settings for the file as JSON
 */
export function setSettingsForFile(settingsJSONInput: string): undefined {
  settingsJSON = settingsJSONInput;
}

/**
 * Deserialize settings from JSON.
 */
export function initSettings(): undefined {
  debugAssertIsNonNull(settingsJSON);
  settings = JSON.parse(settingsJSON);
  // Deep freeze the settings object, to prevent any mutation of the settings from plugins
  deepFreezeJsonValue(settings);
}

/**
 * Reset settings.
 */
export function resetSettings(): undefined {
  settings = null;
  settingsJSON = null;
}
