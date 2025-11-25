import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "./utils";

const fixturesDir = join(__dirname, "fixtures");

describe("config_ignore_patterns", () => {
  it("should respect ignorePatterns in config", async () => {
    const cwd = join(fixturesDir, "config_ignore_patterns");
    const testCases = [["--check"], ["--check", "--config", "fmtrc.jsonc"]];

    const snapshot = await runAndSnapshot(cwd, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
