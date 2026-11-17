import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { describe, expect, it } from "vitest";
import { createLspConnection, lintMultiWorkspaceFixture } from "../utils";

const FIXTURES_DIR = join(import.meta.dirname, "fixtures");

describe("LSP multi workspace linting", () => {
  describe("basic linting", () => {
    it.each([
      {
        name: "default-both",
        paths: [
          { path: "default-both/workspace1/test.ts", languageId: "typescript" },
          { path: "default-both/workspace2/test.ts", languageId: "typescript" },
        ],
      },
      {
        name: "config-default-both",
        paths: [
          { path: "config-default-both/workspace1/test.ts", languageId: "typescript" },
          { path: "config-default-both/workspace2/test.ts", languageId: "typescript" },
        ],
      },
      {
        name: "different-severity",
        paths: [
          { path: "different-severity/workspace1/test.ts", languageId: "typescript" },
          { path: "different-severity/workspace2/test.ts", languageId: "typescript" },
        ],
      },
      {
        name: "different-js-plugin",
        paths: [
          { path: "different-js-plugin/workspace1/test.js", languageId: "javascript" },
          { path: "different-js-plugin/workspace2/test.js", languageId: "javascript" },
        ],
      },
    ])("should handle $name", async ({ paths }) => {
      expect(await lintMultiWorkspaceFixture(FIXTURES_DIR, paths)).toMatchSnapshot();
    });
  });

  describe("nested workspace folders", () => {
    it("should honour the ignore files of the outer workspace folder", async () => {
      expect(
        await lintMultiWorkspaceFixture(FIXTURES_DIR, [
          { path: "nested/test.ts", languageId: "typescript" },
          { path: "nested/packages/a/test.ts", languageId: "typescript" },
          { path: "nested/packages/b/ignored.ts", languageId: "typescript" },
        ]),
      ).toMatchSnapshot();
    });

    it("should report an invalid config of a nested workspace folder once", async () => {
      const outerUri = pathToFileURL(join(FIXTURES_DIR, "nested-invalid-config")).href;
      const nestedUri = pathToFileURL(
        join(FIXTURES_DIR, "nested-invalid-config", "packages", "a"),
      ).href;

      await using client = createLspConnection();
      const messages = client.collectShowMessages();
      // the server registers its watchers after it sent every message
      const registration = client.getDynamicRegistration();
      await client.initialize(
        [
          { uri: outerUri, name: "outer" },
          { uri: nestedUri, name: "nested" },
        ],
        { workspace: { didChangeWatchedFiles: { dynamicRegistration: true } } },
        [
          { workspaceUri: outerUri, options: null },
          { workspaceUri: nestedUri, options: null },
        ],
      );
      await registration;

      expect(messages).toHaveLength(1);
      expect(messages[0].message.replaceAll("\\", "/")).toContain("packages/a/src/.oxlintrc.json");
    });
  });
});
