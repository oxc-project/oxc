import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runWriteModeAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("sort_package_json", () => {
  it("should sort package.json by default", async () => {
    const snapshot = await runWriteModeAndSnapshot(fixturesDir, ["package.json"]);
    expect(snapshot).toMatchSnapshot();
  });

  it("should not sort package.json when disabled", async () => {
    const snapshot = await runWriteModeAndSnapshot(
      fixturesDir,
      ["package.json"],
      ["-c", "disabled.json"],
    );
    expect(snapshot).toMatchSnapshot();
  });

  it("should sort scripts when sortScripts is enabled", async () => {
    const snapshot = await runWriteModeAndSnapshot(
      fixturesDir,
      ["package.json"],
      ["-c", "sort_scripts.json"],
    );
    expect(snapshot).toMatchSnapshot();
  });
});
