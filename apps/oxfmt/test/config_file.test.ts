import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "./utils";

const fixturesDir = join(__dirname, "fixtures");

describe("config_file", () => {
  it("auto discovery", async () => {
    const testCases = [
      { name: "root", cwd: join(fixturesDir, "config_file") },
      { name: "nested", cwd: join(fixturesDir, "config_file", "nested") },
      { name: "nested_deep", cwd: join(fixturesDir, "config_file", "nested", "deep") },
    ];

    for (const { name, cwd } of testCases) {
      // oxlint-disable no-await-in-loop
      const snapshot = await runAndSnapshot(cwd, [["--check"]]);
      expect(snapshot).toMatchSnapshot(name);
    }
  });

  it("explicit config", async () => {
    const cwd = join(fixturesDir, "config_file");
    const testCases = [
      ["--check", "--config", "./fmt.json"],
      ["--check", "--config", "./fmt.jsonc"],
      ["--check", "--config", "NOT_EXISTS.json"],
    ];

    const snapshot = await runAndSnapshot(cwd, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
