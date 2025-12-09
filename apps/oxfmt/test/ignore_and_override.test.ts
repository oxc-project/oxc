import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "./utils";

const fixturesDir = join(__dirname, "fixtures");

describe("ignore_and_override", () => {
  it("should handle ignore files with overrides", async () => {
    const cwd = join(fixturesDir, "ignore_and_override");
    const testCases = [
      // Exclude `err.js` via `!` pattern, format `ok.js` only
      ["--check", "!**/err.js"],
      // `ignore1` excludes all files -> no files found error
      ["--check", "--ignore-path", "ignore1"],
      // Explicitly specified file is also excluded by `ignore1` -> no files found error
      ["--check", "--ignore-path", "ignore1", "should_format/ok.js"],
      // Same as above, but suppress error with `--no-error-on-unmatched-pattern`
      [
        "--check",
        "--ignore-path",
        "ignore1",
        "should_format/ok.js",
        "--no-error-on-unmatched-pattern",
      ],
      // `ignore2` has `!should_format/ok.js` (whitelist), so `ok.js` is formatted
      ["--check", "--ignore-path", "ignore2"],
      // Whitelist + explicit file: `ok.js` is whitelisted in `ignore2` and explicitly specified
      ["--check", "--ignore-path", "ignore2", "should_format/ok.js"],
    ];

    const snapshot = await runAndSnapshot(cwd, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
