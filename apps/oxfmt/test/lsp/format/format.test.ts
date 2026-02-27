import { dirname, join } from "node:path";
import { describe, expect, it } from "vitest";
import { createLspConnection, formatFixture, formatFixtureContent } from "../utils";
import { pathToFileURL } from "node:url";

const FIXTURES_DIR = join(import.meta.dirname, "fixtures");

describe("LSP formatting", () => {
  describe("basic formatting", () => {
    it.each([
      ["format/test.tsx", "typescriptreact"],
      ["format/test.json", "json"],
      ["format/test.vue", "vue"],
      ["format/test.toml", "toml"],
      ["format/formatted.ts", "typescript"],
      ["format/test.txt", "plaintext"],
    ])("should handle %s", async (path, languageId) => {
      expect(await formatFixture(FIXTURES_DIR, path, languageId)).toMatchSnapshot();
    });
  });

  describe("config options", () => {
    it.each([
      ["config-semi/test.ts", "typescript"],
      ["config-no-sort-package-json/package.json", "json"],
      ["config-vue-indent/test.vue", "vue"],
      ["config-sort-imports/test.js", "javascript"],
      ["config-sort-tailwindcss/test.tsx", "typescriptreact"],
      ["config-sort-tailwindcss/test.vue", "vue"],
      ["config-sort-both/test.jsx", "javascriptreact"],
      ["editorconfig/test.ts", "typescript"],
    ])("should apply config from %s", async (path, languageId) => {
      expect(await formatFixture(FIXTURES_DIR, path, languageId)).toMatchSnapshot();
    });
  });

  describe("config options in nested workspace folders", () => {
    it.each([
      ["nested-workspaces/test.ts", "nested-workspaces/second/test.ts"],
      ["nested-workspaces-with-config/test.ts", "nested-workspaces-with-config/second/test.ts"],
    ])("should respect nested oxfmt config with nested workspace folders %s", async (...paths) => {
      await using client = createLspConnection();
      const dirUris = paths.map((path) => pathToFileURL(dirname(join(FIXTURES_DIR, path))).href);
      await client.initialize(
        [
          { uri: dirUris[0], name: "test" },
          { uri: dirUris[1], name: "test-2" },
        ],
        {},
        [
          {
            workspaceUri: dirUris[0],
            options: null,
          },
          {
            workspaceUri: dirUris[1],
            options: null,
          },
        ],
      );
      for (const path of paths) {
        // oxlint-disable-next-line no-await-in-loop
        expect(await formatFixture(FIXTURES_DIR, path, "typescript", client)).toMatchSnapshot();
      }
    });
  });

  describe("unsaved document", () => {
    it.each([
      ["format/test.tsx", "typescriptreact"],
      ["format/test.json", "json"],
      ["format/test.vue", "vue"],
      ["format/test.toml", "toml"],
      ["format/formatted.ts", "typescript"],
      ["format/test.txt", "plaintext"],
    ])("should format unsaved file %s", async (path, languageId) => {
      expect(
        await formatFixtureContent(
          FIXTURES_DIR,
          path,
          "untitled://Untitled-" + languageId,
          languageId,
        ),
      ).toMatchSnapshot();
    });
  });

  describe("ignore patterns", () => {
    it.each([
      ["ignore-prettierignore/ignored.ts", "typescript"],
      ["ignore-config/file.generated.ts", "typescript"],
    ])("should handle %s", async (path, languageId) => {
      expect(await formatFixture(FIXTURES_DIR, path, languageId)).toMatchSnapshot();
    });
  });

  describe("initializationOptions", () => {
    it("should use custom config path from fmt.configPath", async () => {
      expect(
        await formatFixture(
          FIXTURES_DIR,
          "custom_config_path/semicolons-as-needed.ts",
          "typescript",
          {
            "fmt.configPath": "./format.json",
          },
        ),
      ).toMatchSnapshot();
    });
  });
});
