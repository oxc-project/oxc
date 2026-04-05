import type { Plugin } from "prettier";

const REGISTERED_PLUGIN_SPEC_PREFIX = "__OXFMT_REGISTERED_PLUGIN__";

const registeredPlugins = new Map<number, Plugin>();
const registeredPluginIds = new WeakMap<object, number>();

let nextRegisteredPluginId = 1;

function isObject(value: unknown): value is object {
  return typeof value === "object" && value !== null;
}

export function isRegisteredPluginSpec(spec: string): boolean {
  return spec.startsWith(REGISTERED_PLUGIN_SPEC_PREFIX);
}

export function registerPluginObject(plugin: Plugin): string {
  const pluginObject = plugin as object;
  const existingId = registeredPluginIds.get(pluginObject);
  if (existingId !== undefined) {
    return `${REGISTERED_PLUGIN_SPEC_PREFIX}${existingId}`;
  }

  const id = nextRegisteredPluginId++;
  registeredPlugins.set(id, plugin);
  registeredPluginIds.set(pluginObject, id);
  return `${REGISTERED_PLUGIN_SPEC_PREFIX}${id}`;
}

export function getRegisteredPlugin(spec: string): Plugin | null {
  if (!isRegisteredPluginSpec(spec)) return null;

  const id = Number(spec.slice(REGISTERED_PLUGIN_SPEC_PREFIX.length));
  if (!Number.isInteger(id) || id <= 0) {
    throw new Error(`Invalid registered formatter plugin spec: ${spec}`);
  }

  const plugin = registeredPlugins.get(id);
  if (plugin === undefined) {
    throw new Error(`Unknown registered formatter plugin ID: ${id}`);
  }

  return plugin;
}

function normalizePluginEntries(entries: unknown): unknown {
  if (Array.isArray(entries)) {
    return entries.map((entry) => (isObject(entry) ? registerPluginObject(entry as Plugin) : entry));
  }

  return isObject(entries) ? registerPluginObject(entries as Plugin) : entries;
}

export function normalizePluginObjectsForRust<T>(value: T): T {
  const normalizedObjects = new WeakMap<object, unknown>();

  const normalize = (current: unknown): unknown => {
    if (Array.isArray(current)) {
      return current.map((item) => normalize(item));
    }

    if (!isObject(current)) return current;

    const cached = normalizedObjects.get(current);
    if (cached !== undefined) return cached;

    const normalized: Record<string, unknown> = {};
    normalizedObjects.set(current, normalized);

    for (const [key, entryValue] of Object.entries(current as Record<string, unknown>)) {
      if (key === "plugins") {
        normalized[key] = normalizePluginEntries(entryValue);
      } else {
        normalized[key] = normalize(entryValue);
      }
    }

    return normalized;
  };

  return normalize(value) as T;
}
