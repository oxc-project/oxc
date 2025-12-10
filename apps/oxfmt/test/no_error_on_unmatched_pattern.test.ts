import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "./utils";

const fixturesDir = join(__dirname, "fixtures");

describe("no_error_on_unmatched_pattern", () => {
  it("should handle --no-error-on-unmatched-pattern flag", async () => {
    const cwd = fixturesDir;
    const testCases = [
      ["--check", "--no-error-on-unmatched-pattern", "__non__existent__file.js"],
      ["--check", "__non__existent__file.js"],
    ];

    const snapshot = await runAndSnapshot(cwd, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
