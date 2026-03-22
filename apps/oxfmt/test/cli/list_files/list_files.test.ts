import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("list_files", () => {
  it("should list selected target files in sorted order", async () => {
    const testCases = [
      ["--list-files"],
      ["--list-files", "z-last.js", "a-first.ts"],
      ["--list-files", "--ignore-path", "custom.ignore"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });

  it("should preserve no-file semantics", async () => {
    const testCases = [
      ["--list-files", "__nonexistent__/**/*.js"],
      ["--list-files", "--no-error-on-unmatched-pattern", "__nonexistent__/**/*.js"],
      ["--list-files", "--ignore-path", "custom.ignore", "ignored-by-ignore-path/skip.js"],
      [
        "--list-files",
        "--no-error-on-unmatched-pattern",
        "--ignore-path",
        "custom.ignore",
        "ignored-by-ignore-path/skip.js",
      ],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });

  it("should handle nested fixtures with config and ignore settings", async () => {
    const testCases = [
      ["--list-files", "list_files_nested"],
      ["--list-files", "--config", "list_files_nested/.oxfmtrc.json", "list_files_nested"],
      [
        "--list-files",
        "--config",
        "list_files_nested/.oxfmtrc.json",
        "--ignore-path",
        "list_files_nested/.customignore",
        "list_files_nested",
      ],
      [
        "--list-files",
        "--config",
        "list_files_nested/.oxfmtrc.json",
        "--with-node-modules",
        "list_files_nested",
      ],
      [
        "--list-files",
        "--config",
        "list_files_nested/.oxfmtrc.json",
        "list_files_nested/.hidden/hidden.js",
      ],
      [
        "--list-files",
        "--no-error-on-unmatched-pattern",
        "--config",
        "list_files_nested/.oxfmtrc.json",
        "list_files_nested/.hidden/hidden.js",
      ],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
