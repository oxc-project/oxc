import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "./utils";

const fixturesDir = join(__dirname, "fixtures");

describe("ignore_and_override", () => {
  it("should handle ignore files with overrides", async () => {
    const cwd = join(fixturesDir, "ignore_and_override");
    const testCases = [
      ["--check", "!**/err.js"],
      ["--check", "--ignore-path", "ignore1"],
      ["--check", "--ignore-path", "ignore1", "should_format/ok.js"],
      ["--check", "--ignore-path", "ignore2"],
    ];

    const snapshot = await runAndSnapshot(cwd, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
