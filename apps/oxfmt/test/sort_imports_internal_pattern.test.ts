import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runWriteModeAndSnapshot } from "./utils";

const fixturesDir = join(__dirname, "fixtures");
const fixtureDir = join(fixturesDir, "sort_imports_internal_pattern");

describe("sort_imports_internal_pattern", () => {
  it("respects configured internal prefixes", async () => {
    const snapshot = await runWriteModeAndSnapshot(fixtureDir, ["monorepo.js"]);
    expect(snapshot).toMatchSnapshot();
  });

  it("treats literal asterisks as literal prefixes", async () => {
    const snapshot = await runWriteModeAndSnapshot(
      fixtureDir,
      ["literal.js"],
      ["--config", "./literal.jsonc"],
    );
    expect(snapshot).toMatchSnapshot();
  });
});
