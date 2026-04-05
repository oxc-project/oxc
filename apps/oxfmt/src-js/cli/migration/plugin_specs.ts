import { isAbsolute, relative } from "node:path";

const WINDOWS_ABSOLUTE_PATH_RE = /^[A-Za-z]:[\/]/u;
const RELATIVE_OR_ABSOLUTE_PATH_PREFIX_RE = /^(?:\.{1,2}(?:[\/]|$)|[\/])/u;
const PLUGINISH_PACKAGE_SEGMENT_RE = /(?:^|\/)(?:prettier-plugin-[A-Za-z0-9._-]+|plugin-[A-Za-z0-9._-]+)(?:$|\/)/u;

export function normalizePreservedPluginSpec(plugin: string, projectDir: string): string {
  if (!isAbsolute(plugin)) return plugin;

  const relativePlugin = relative(projectDir, plugin);
  if (relativePlugin === "" || relativePlugin === "." || relativePlugin === "..") {
    return plugin;
  }

  const normalizedRelativePlugin = relativePlugin.replaceAll("\\", "/");
  if (normalizedRelativePlugin.startsWith("../") || normalizedRelativePlugin === "..") {
    return plugin;
  }

  if (/^[A-Za-z]:\//.test(normalizedRelativePlugin)) {
    return plugin;
  }

  if (normalizedRelativePlugin === "node_modules" || normalizedRelativePlugin.startsWith("node_modules/")) {
    return plugin;
  }

  return normalizedRelativePlugin.startsWith(".")
    ? normalizedRelativePlugin
    : `./${normalizedRelativePlugin}`;
}

export function looksLikePreservablePluginSpec(plugin: string, source: "packageName" | "name" = "packageName"): boolean {
  if (plugin === "") {
    return false;
  }

  if (plugin.startsWith("file:") || RELATIVE_OR_ABSOLUTE_PATH_PREFIX_RE.test(plugin) || WINDOWS_ABSOLUTE_PATH_RE.test(plugin)) {
    return true;
  }

  if (!/^(?:@[^/]+\/)?[^/@][^/]*(?:\/[^/]+)*$/u.test(plugin)) {
    return false;
  }

  if (source === "packageName") {
    return true;
  }

  return PLUGINISH_PACKAGE_SEGMENT_RE.test(plugin);
}
