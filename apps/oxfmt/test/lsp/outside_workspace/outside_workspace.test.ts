import { join, resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { formatMultipleFixtures } from "../utils";
import { pathToFileURL } from "node:url";

const FIXTURES_DIR = join(import.meta.dirname, "fixtures");

describe("LSP formatting outside current workspace", () => {
  describe("basic formatting", () => {
    it("should handle basic formatting", async () => {
      expect(
        await formatMultipleFixtures(FIXTURES_DIR, join(FIXTURES_DIR, "workspace"), [
          {
            uri: pathToFileURL(join(FIXTURES_DIR, "workspace", "test.ts")).href,
            content: "const x=1",
            languageId: "typescript",
          },
          {
            uri: pathToFileURL(resolve(FIXTURES_DIR, "test.ts")).href,
            content: "const x=1",
            languageId: "typescript",
          },
        ]),
      ).toMatchSnapshot();
    });
  });
});
