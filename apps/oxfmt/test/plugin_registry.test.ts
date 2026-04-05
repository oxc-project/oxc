import { describe, expect, it } from "vitest";
import { getRegisteredPlugin, isRegisteredPluginSpec, normalizePluginObjectsForRust } from "../src-js/plugin_registry.ts";

const pluginObject = {
  languages: [{ name: "Svelte", parsers: ["svelte"], extensions: [".svelte"] }],
  parsers: {},
  printers: {},
};

describe("plugin_registry", () => {
  it("normalizes plugin objects in root config and overrides", () => {
    const config = {
      plugins: [pluginObject],
      overrides: [
        {
          files: ["*.svelte"],
          options: {
            plugins: [pluginObject, "prettier-plugin-svelte"],
          },
        },
      ],
    };

    const normalized = normalizePluginObjectsForRust(config) as {
      plugins: string[];
      overrides: { options: { plugins: string[] } }[];
    };

    expect(normalized.plugins).toHaveLength(1);
    expect(isRegisteredPluginSpec(normalized.plugins[0])).toBe(true);
    expect(normalized.overrides[0].options.plugins[0]).toBe(normalized.plugins[0]);
    expect(normalized.overrides[0].options.plugins[1]).toBe("prettier-plugin-svelte");
    expect(getRegisteredPlugin(normalized.plugins[0])).toBe(pluginObject as any);
  });
});
