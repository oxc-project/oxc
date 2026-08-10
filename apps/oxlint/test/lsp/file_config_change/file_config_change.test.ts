import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { lintFixtureWithFileContentChange } from "../utils";

const FIXTURES_DIR = join(import.meta.dirname, "fixtures");

describe("diagnostics after config change", () => {
  it.each([
    ["severity-change/test.ts", "typescript", ".oxlintrc.json", ".oxlintrc-new.json"],
    ["ts-config/test.ts", "typescript", "oxlint.config.ts", "oxlint.config-new.ts"],
  ])(
    "should get refreshed diagnostics config from %s",
    async (path, languageId, oldConfig, newConfig) => {
      expect(
        await lintFixtureWithFileContentChange(
          FIXTURES_DIR,
          path,
          languageId,
          oldConfig,
          newConfig,
        ),
      ).toMatchSnapshot();
    },
  );

  it("reloads the suppression baseline after a watched file changes", async () => {
    const diagnostics = await lintFixtureWithFileContentChange(
      FIXTURES_DIR,
      "suppression-refresh/test.js",
      "javascript",
      "oxlint-suppressions.json",
      "without-suppressions.json",
    );
    const [beforeChange, afterChange] = diagnostics.split("=== After Config Change ===");

    expect(beforeChange).toContain("Warning: Unexpected console statement");
    expect(afterChange).toContain("Error: Unexpected console statement");
  });
});
