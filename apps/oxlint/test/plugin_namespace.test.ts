import { afterEach, describe, expect, it } from "vitest";
import { registerPlugin, registeredPluginNames, registeredRules } from "../src-js/plugins/load.ts";

import type { Plugin } from "../src-js/plugins.ts";

function plugin(meta: Plugin["meta"]): Plugin {
  return { meta, rules: { rule: { create: () => ({}) } } };
}

afterEach(() => {
  registeredPluginNames.clear();
  registeredRules.length = 0;
});

describe("plugin namespace", () => {
  it("accepts a scoped namespace without a metadata or package name", () => {
    const result = registerPlugin(plugin({ namespace: "@scope/tools" }), null, false, null);
    expect(result.name).toBe("@scope/tools");
    expect(registeredRules[0].context.id).toBe("@scope/tools/rule");
  });

  it("uses an explicit alias without reading invalid metadata", () => {
    const invalidPlugin = plugin({ name: "package", namespace: 123 } as unknown as Plugin["meta"]);
    expect(registerPlugin(invalidPlugin, "alias", true, null).name).toBe("alias");
    expect(registeredRules[0].context.id).toBe("alias/rule");
  });

  it("falls back to the normalized metadata name for a null namespace", () => {
    const metadata = {
      name: "eslint-plugin-example",
      namespace: null,
    } as unknown as Plugin["meta"];
    expect(registerPlugin(plugin(metadata), "package", false, null).name).toBe("example");
  });

  it("rejects an invalid namespace before registering rules", () => {
    const invalidPlugin = plugin({ name: "package", namespace: 123 } as unknown as Plugin["meta"]);
    expect(() => registerPlugin(invalidPlugin, "fallback", false, null)).toThrow(
      new TypeError("`plugin.meta.namespace` must be a string if defined"),
    );
    expect(registeredRules).toHaveLength(0);
    expect(registeredPluginNames.size).toBe(0);
  });

  it("detects namespace collisions and allows an explicit alias", () => {
    registerPlugin(plugin({ name: "first", namespace: "shared" }), null, false, null);
    const second = plugin({ name: "second", namespace: "shared" });
    expect(() => registerPlugin(second, null, false, null)).toThrow(
      "Plugin name 'shared' is already registered",
    );
    expect(registerPlugin(second, "alias", true, null)).toEqual({
      name: "alias",
      offset: 1,
      ruleNames: ["rule"],
    });
  });
});
