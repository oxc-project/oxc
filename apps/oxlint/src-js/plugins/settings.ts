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

let pluginSettingsAliases = new Map<string, string>();

/**
 * Set plugin settings aliases. Used when switching workspaces.
 * @param aliases - Map from configured plugin alias to canonical plugin settings key
 */
export function setPluginSettingsAliases(aliases: Map<string, string>): undefined {
  pluginSettingsAliases = aliases;
}

/**
 * Add a plugin settings alias.
 * @param alias - Configured plugin alias
 * @param pluginName - Canonical plugin settings key
 */
export function addPluginSettingsAlias(alias: string, pluginName: string): undefined {
  pluginSettingsAliases.set(alias, pluginName);
}

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
  const nextSettings = JSON.parse(settingsJSON) as Settings;
  applyPluginSettingsAliases(nextSettings);
  settings = nextSettings;
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

function applyPluginSettingsAliases(settings: Settings): undefined {
  for (const [alias, pluginName] of pluginSettingsAliases) {
    if (Object.hasOwn(settings, alias) && !Object.hasOwn(settings, pluginName)) {
      settings[pluginName] = settings[alias]!;
    }
  }
}
