import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures", "basic");

describe("config_ignore_patterns", () => {
  it("should respect ignorePatterns in config", async () => {
    const testCases = [["--check"], ["--check", "--config", "fmtrc.jsonc"]];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });

  // Structure:
  //   config_ignore_patterns_nested_cwd/
  //     .oxfmtrc.json   # ignorePatterns: ["sub/generated/**"]
  //     sub/            # <- cwd (nested directory)
  //       generated/
  //         ignored.js  # Should be ignored
  //       src/
  //         test.js     # Should be formatted
  //
  // When running from `sub/` directory with target `.`:
  // - .oxfmtrc.json is found in parent directory
  // - ignorePatterns should resolve relative to .oxfmtrc.json location
  // - Pattern "sub/generated/**" should match "sub/generated/ignored.js"
  //   relative to .oxfmtrc.json location, NOT "sub/sub/generated/**" from cwd
  //
  // Expected: generated/ignored.js should be ignored, src/test.js should be listed
  it("nested cwd - ignorePatterns resolved relative to .oxfmtrc.json location", async () => {
    // Run from nested `sub/` directory - check mode doesn't need temp copy
    const nestedCwd = join(import.meta.dirname, "fixtures", "nested_cwd", "sub");
    const snapshot = await runAndSnapshot(nestedCwd, [["--check", "."]]);
    expect(snapshot).toMatchSnapshot();
  });

  // Same structure as above, but explicitly specifying `--config ../.oxfmtrc.json`
  // instead of relying on automatic upward config search.
  //
  // When `--config` contains `..`, `normalize_relative_path` joins
  // cwd + `../.oxfmtrc.json` without resolving `..`, leaving the path as
  // `.../sub/../.oxfmtrc.json`. This causes `GitignoreBuilder`'s root
  // to be `.../sub/..` which doesn't match file paths via `strip_prefix`,
  // making `ignorePatterns` ineffective.
  it("explicit --config with parent-relative path resolves ignorePatterns correctly", async () => {
    const nestedCwd = join(import.meta.dirname, "fixtures", "nested_cwd", "sub");
    const snapshot = await runAndSnapshot(nestedCwd, [
      ["--check", "--config", "../.oxfmtrc.json", "."],
    ]);
    expect(snapshot).toMatchSnapshot();
  });
});
