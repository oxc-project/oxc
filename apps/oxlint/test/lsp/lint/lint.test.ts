import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { lintFixture } from "../utils";

const FIXTURES_DIR = join(import.meta.dirname, "fixtures");

describe("LSP linting", () => {
  describe("basic linting", () => {
    it.each([
      ["default/test.tsx", "typescriptreact"],
      ["default/test.ts", "typescript"],
    ])("should handle %s", async (path, languageId) => {
      expect(await lintFixture(FIXTURES_DIR, path, languageId)).toMatchSnapshot();
    });
  });

  describe("config options", () => {
    it.each([
      ["config-default/test.ts", "typescript"],
      ["config-disabled/test.ts", "typescript"],
      ["config-severity/test.ts", "typescript"],
      ["config-js-plugin/test.js", "javascript"],
      ["config-ts-config/test.js", "javascript"],
      ["config-ts-type-aware/test.ts", "typescript"],
      ["config-ts-nested-type-aware-invalid/nested/test.ts", "typescript"],
    ])("should apply config from %s", async (path, languageId) => {
      expect(await lintFixture(FIXTURES_DIR, path, languageId)).toMatchSnapshot();
    });

    it("should allow LSP typeAware option to override ts config", async () => {
      expect(
        await lintFixture(
          FIXTURES_DIR,
          "config-ts-type-aware/test-with-lsp-config.ts",
          "typescript",
          {
            typeAware: false,
          },
        ),
      ).toMatchSnapshot();
    });
  });

  describe("initializationOptions", () => {
    it("should use custom config path from configPath", async () => {
      expect(
        await lintFixture(FIXTURES_DIR, "custom-config-path/test.ts", "typescript", {
          configPath: "./lint.json",
        }),
      ).toMatchSnapshot();
    });
  });
});
