import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("config_file", () => {
  it("auto discovery", async () => {
    const testCases = [
      { name: "root", cwd: fixturesDir },
      { name: "nested", cwd: join(fixturesDir, "nested") },
      { name: "nested_deep", cwd: join(fixturesDir, "nested", "deep") },
    ];
    for (const { name, cwd } of testCases) {
      // oxlint-disable no-await-in-loop
      const snapshot = await runAndSnapshot(cwd, [["--check", "!*.{json,jsonc,ts}"]]);
      expect(snapshot).toMatchSnapshot(name);
    }
  });

  it("explicit config", async () => {
    const testCases = [
      ["--check", "!*.{json,jsonc,ts}", "--config", "./fmt.json"],
      ["--check", "!*.{json,jsonc,ts}", "--config", "./fmt.jsonc"],
      ["--check", "!*.{json,jsonc,ts}", "--config", "./fmt.config.ts"],
      ["--check", "--config", "NOT_EXISTS.json"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
