import { describe, expect, it } from "vitest";
import { defineLanguagePlugin } from "../src-js/language-plugins.ts";

describe("defineLanguagePlugin", () => {
  it("returns the plugin unchanged when valid", () => {
    const plugin = defineLanguagePlugin({
      meta: { name: "vue-language-plugin" },
      defaultFiles: [".vue"],
      visitorKeys: { nodes: { Program: { body: ["Statement"] } } },
      parse() {
        return { ast: { type: "Program", body: [] } };
      },
      load(_filePath, parseResult) {
        return {
          languageId: "vue",
          ast: parseResult.ast,
          transform: null,
          isESTree: false,
        };
      },
    });
    expect(plugin.meta.name).toBe("vue-language-plugin");
    expect(plugin.defaultFiles).toEqual([".vue"]);
  });

  it("rejects glob defaultFiles", () => {
    expect(() =>
      defineLanguagePlugin({
        meta: { name: "bad" },
        defaultFiles: ["**/*.vue"],
        visitorKeys: { nodes: {} },
        parse() {
          return { ast: { type: "Program" } };
        },
        load(_f, parseResult) {
          return {
            languageId: "bad",
            ast: parseResult.ast,
            transform: null,
            isESTree: false,
          };
        },
      }),
    ).toThrow(/extensions or filenames/);
  });

  it("requires parse and load", () => {
    expect(() =>
      // @ts-expect-error intentional invalid plugin
      defineLanguagePlugin({ meta: { name: "x" }, visitorKeys: { nodes: {} } }),
    ).toThrow(/parse/);
  });
});
