import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { formatFixture } from "../utils";

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
