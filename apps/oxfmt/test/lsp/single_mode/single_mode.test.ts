import { join } from "node:path";
import { describe, expect, it } from "vite-plus/test";
import { formatSingleFileFixture } from "../utils";

const FIXTURES_DIR = join(import.meta.dirname, "fixtures");

describe("LSP formatting in single mode", () => {
  describe("basic formatting", () => {
    it.each([
      ["basic/test.ts", "typescript"],
      ["with-config-same-level/test.ts", "typescript"],
    ])("should handle %s", async (path, languageId) => {
      expect(await formatSingleFileFixture(FIXTURES_DIR, path, languageId)).toMatchSnapshot();
    });
  });
});
