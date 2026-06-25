import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { lintSingleFileFixture } from "../utils";

const FIXTURES_DIR = join(import.meta.dirname, "fixtures");

describe("LSP single file mode linting", () => {
  describe("basic linting", () => {
    it.each([
      { path: "basic/test.ts", languageId: "typescript" },
      { path: "with-config-same-level/test.ts", languageId: "typescript" },
    ])("should handle %s", async ({ path, languageId }) => {
      expect(await lintSingleFileFixture(FIXTURES_DIR, path, languageId)).toMatchSnapshot();
    });
  });
});
