import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { lintMultiWorkspaceFixture } from "../utils";

const FIXTURES_DIR = join(import.meta.dirname, "fixtures");

describe("LSP multi workspace linting", () => {
  describe("basic linting", () => {
    it.each([
      [
        { path: "default-both/workspace1/test.ts", languageId: "typescript" },
        { path: "default-both/workspace2/test.ts", languageId: "typescript" },
      ],
      [
        { path: "config-default-both/workspace1/test.ts", languageId: "typescript" },
        { path: "config-default-both/workspace2/test.ts", languageId: "typescript" },
      ],
      [
        { path: "different-severity/workspace1/test.ts", languageId: "typescript" },
        { path: "different-severity/workspace2/test.ts", languageId: "typescript" },
      ],
      [
        { path: "different-js-plugin/workspace1/test.js", languageId: "javascript" },
        { path: "different-js-plugin/workspace2/test.js", languageId: "javascript" },
      ],
    ])("should handle %s", async (...paths) => {
      expect(await lintMultiWorkspaceFixture(FIXTURES_DIR, paths)).toMatchSnapshot();
    });
  });
});
