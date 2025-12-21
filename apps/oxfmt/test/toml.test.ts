import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot, runWriteModeAndSnapshot } from "./utils";

const fixturesDir = join(__dirname, "fixtures");

describe("toml", () => {
  it("should support toml files", async () => {
    const cwd = join(fixturesDir, "toml");
    const testCases = [
      // Finish on 3 toml + 1 json, 3 toml have diff, lock files are ignored
      ["--check"],
    ];

    const snapshot = await runAndSnapshot(cwd, testCases);
    expect(snapshot).toMatchSnapshot();
  });

  it("should apply format options", async () => {
    const cwd = join(fixturesDir, "toml");
    const snapshot = await runWriteModeAndSnapshot(
      cwd,
      ["config.toml"],
      ["--config", "use-tab.json"],
    );
    expect(snapshot).toMatchSnapshot();
  });
});
