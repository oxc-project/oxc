import { describe, expect, it } from "vitest";
import { join } from "node:path";
import {
  looksLikePreservablePluginSpec,
  normalizePreservedPluginSpec,
} from "../src-js/cli/migration/plugin_specs.ts";

describe("plugin spec normalization", () => {
  it("keeps package specs and relative file specs unchanged", () => {
    expect(normalizePreservedPluginSpec("prettier-plugin-svelte", "/repo")).toBe("prettier-plugin-svelte");
    expect(normalizePreservedPluginSpec("./plugins/prettier-plugin-custom.mjs", "/repo")).toBe(
      "./plugins/prettier-plugin-custom.mjs",
    );
  });

  it("re-relativizes project-local absolute plugin file specs", () => {
    const projectDir = "/repo";
    const pluginPath = join(projectDir, "plugins", "prettier-plugin-custom.mjs");

    expect(normalizePreservedPluginSpec(pluginPath, projectDir)).toBe(
      "./plugins/prettier-plugin-custom.mjs",
    );
  });

  it("does not rewrite node_modules plugin paths", () => {
    const projectDir = "/repo";
    const pluginPath = join(projectDir, "node_modules", "prettier-plugin-svelte", "dist", "index.js");

    expect(normalizePreservedPluginSpec(pluginPath, projectDir)).toBe(pluginPath);
  });

  it("does not rewrite plugin paths outside the project", () => {
    const projectDir = "/repo";
    const pluginPath = "/other/plugins/prettier-plugin-custom.mjs";

    expect(normalizePreservedPluginSpec(pluginPath, projectDir)).toBe(pluginPath);
  });

  it("accepts pluginish metadata names but rejects arbitrary names", () => {
    expect(looksLikePreservablePluginSpec("prettier-plugin-custom", "name")).toBe(true);
    expect(looksLikePreservablePluginSpec("@scope/prettier-plugin-custom/subpath", "name")).toBe(true);
    expect(looksLikePreservablePluginSpec("just-a-random-object-name", "name")).toBe(false);
    expect(looksLikePreservablePluginSpec("@scope/custom-package", "packageName")).toBe(true);
    expect(looksLikePreservablePluginSpec("./plugins/plugin.mjs", "name")).toBe(true);
  });
});
