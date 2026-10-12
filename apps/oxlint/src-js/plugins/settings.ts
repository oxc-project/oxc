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

// Settings deserialized for the last file which had any, and the JSON they came from.
// Retained across files, so unchanged settings are deserialized only once.
let lastSettingsJSON: string | null = null;
let lastSettings: Readonly<Settings> | null = null;

/**
 * Updates the settings for the file.
 *
 * @param settingsJSONInput - Settings for the file as JSON
 */
export function setSettingsForFile(settingsJSONInput: string): undefined {
  settingsJSON = settingsJSONInput;
}

/**
 * Deserialize settings from JSON.
 *
 * Settings are passed in as JSON per file, to accommodate folder level settings, but in practice
 * they're usually identical for every file. So reuse the object deserialized for the previous file
 * when the JSON is unchanged.
 *
 * As well as skipping the `JSON.parse` and deep freeze, this keeps `context.settings` referentially
 * stable across files. Plugins commonly normalize settings once and memoize the result in a
 * `WeakMap` keyed on the settings object, which a fresh object per file would defeat.
 *
 * Reuse is safe because the object is deeply frozen, so no plugin can have mutated it.
 */
export function initSettings(): undefined {
  debugAssertIsNonNull(settingsJSON);

  if (settingsJSON === lastSettingsJSON) {
    settings = lastSettings;
    return;
  }

  settings = JSON.parse(settingsJSON);
  // Deep freeze the settings object, to prevent any mutation of the settings from plugins
  deepFreezeJsonValue(settings);

  lastSettingsJSON = settingsJSON;
  lastSettings = settings;
}

/**
 * Reset settings.
 */
export function resetSettings(): undefined {
  settings = null;
  settingsJSON = null;
}

/**
 * Discard the settings object retained for reuse.
 *
 * Called when switching workspaces, so that a settings object is never shared between two
 * workspaces. Workspaces can have identical settings but different CWDs, and a plugin may derive
 * state from `context.cwd` and cache it keyed on the settings object.
 */
export function resetSettingsCache(): undefined {
  lastSettingsJSON = null;
  lastSettings = null;
}
