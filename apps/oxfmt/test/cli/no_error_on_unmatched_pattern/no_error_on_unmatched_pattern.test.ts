import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("no_error_on_unmatched_pattern", () => {
  it("should handle --no-error-on-unmatched-pattern flag", async () => {
    const testCases = [
      ["--check", "--no-error-on-unmatched-pattern", "__non__existent__file.js"],
      ["--check", "__non__existent__file.js"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });

  it("should handle unmatched glob patterns", async () => {
    const testCases = [
      // Glob pattern that matches nothing - should error without flag
      ["--check", "__nonexistent__/**/*.js"],
      // With flag - should not error
      ["--check", "--no-error-on-unmatched-pattern", "__nonexistent__/**/*.js"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });

  // When a file path inside an ignored directory is passed directly via CLI, it should still be ignored.
  // This is important for tools like lint-staged that pass explicit file paths.
  it("should ignore ignored paths even when explicitly passed", async () => {
    const testCases = [
      // Explicit file path - should still be ignored
      ["--check", "--no-error-on-unmatched-pattern", "ignored-by-config/bad.js"],
      // Explicit directory path - should still be ignored
      ["--check", "--no-error-on-unmatched-pattern", "ignored-by-config"],
      // Explicit file path - should be ignored
      ["--check", "--no-error-on-unmatched-pattern", "ignored-by-prettierignore/bad.js"],
      // Explicit file path with custom ignore file - should be ignored
      [
        "--check",
        "--no-error-on-unmatched-pattern",
        "--ignore-path",
        "custom.ignore",
        "ignored-by-ignore-path/bad.js",
      ],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
