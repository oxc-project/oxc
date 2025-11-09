/*
 * Methods related to settings.
 */

const { isArray } = Array;

// Settings for current file.
// `settingsJSON` is set before linting a file by `setSettingsForFile`.
// `settings` is deserialized from `settingsJSON` lazily upon first access.
let settingsJSON: string | null = null;
export let settings: Record<string, unknown> | null = null;

/**
 * Updates the settings for the file.
 *
 * TODO(perf): Settings are deserialized once per file to accommodate folder level settings,
 * even if the settings haven't changed.
 *
 * @param settingsJSONInput - Settings for the file as JSON
 */
export function setSettingsForFile(settingsJSONInput: string) {
  settingsJSON = settingsJSONInput;
}

/**
 * Deserialize settings from JSON.
 */
export function initSettings() {
  settings = JSON.parse(settingsJSON);
  deepFreezeSettings(settings);
}

/**
 * Reset settings.
 */
export function resetSettings() {
  settings = null;
  settingsJSON = null;
}

/**
 * Deep freeze the settings object, recursively freezing all nested objects and arrays.
 * This prevents any mutation of the settings from plugins.
 * @param obj - The object to deep freeze
 */
function deepFreezeSettings(obj: unknown): undefined {
  if (obj === null || typeof obj !== 'object') return;

  if (isArray(obj)) {
    for (let i = 0, len = obj.length; i < len; i++) {
      deepFreezeSettings(obj[i]);
    }
  } else {
    // We don't need to handle symbol properties or circular references because settings are deserialized from JSON
    for (const key in obj) {
      deepFreezeSettings((obj as unknown as Record<string, unknown>)[key]);
    }
  }

  Object.freeze(obj);
}
