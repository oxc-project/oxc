import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { fixFixture } from "../utils";

const FIXTURES_DIR = join(import.meta.dirname, "fixtures");

describe("LSP code actions", () => {
  describe("basic code actions", () => {
    it.each([
      ["fix/test.ts", "typescript"],
      ["suggestion/test.ts", "typescript"],
      ["js-plugin-fix/test.js", "javascript"],
      ["js-plugin-suggestion/test.js", "javascript"],
    ])("should handle %s", async (path, languageId) => {
      expect(
        await fixFixture(FIXTURES_DIR, path, languageId, {
          fixKind: "safe_fix_or_suggestion",
        }),
      ).toMatchSnapshot();
    });
  });

  describe("code actions hidden behind `fixKind` filter", () => {
    it.each([
      ["suggestion/test.ts", "typescript"],
      ["js-plugin-suggestion/test.js", "javascript"],
    ])("should handle %s", async (path, languageId) => {
      // because fixKind is default `safe_fix`, we do not expect any fix as code actions
      // the only code action valid for this test should be "ignore for this line" and "ignore for this file".
      expect(await fixFixture(FIXTURES_DIR, path, languageId)).toMatchSnapshot();
    });
  });
});
