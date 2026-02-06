import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { writeFile, rm } from "node:fs/promises";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("glob_patterns", () => {
  it("should expand basic wildcard pattern", async () => {
    const testCases = [
      // src/*.js should match src/a.js and src/b.js, but not src/c.ts
      ["--check", "src/*.js"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });

  it("should expand recursive glob pattern and exclude", async () => {
    const testCases = [
      // Should match all .js files except util.js
      ["--check", "**/*.js", "!util.js"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });

  it("should expand brace expansion pattern", async () => {
    const testCases = [
      // {a,b}.js should match a.js and b.js
      ["--check", "src/{a,b}.js"],
      ["--check", "src/{a}.js"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });

  it("should handle mix of concrete paths and glob patterns", async () => {
    const testCases = [
      // Mix of concrete path and glob
      ["--check", "src/c.ts", "src/*.js"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });

  it("should deduplicate files specified both directly and via glob", async () => {
    const testCases = [
      // src/a.js is specified both directly and via glob - should only appear once
      ["--check", "src/a.js", "src/*.js"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });

  it("should treat glob-like filename as concrete path when it exists on disk", async () => {
    const testCases = [
      // `{d}.js` contains glob-like `{` character but actually exists on disk.
      // It should be treated as a concrete path, not as a brace expansion pattern.
      // Without the existence check, `{d}` would expand to `d` and try to match
      // `d.js` which does NOT exist, resulting in no files being found.
      ["--check", "src/{d}.js"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });

  it("should respect .gitignore for directories when expanding globs", async () => {
    const gitignorePath = join(fixturesDir, ".gitignore");

    try {
      // Create .gitignore that ignores the "lib/" directory
      await writeFile(gitignorePath, "lib/\n");

      const testCases = [
        // **/*.js should NOT include lib/util.js (directory is gitignored)
        ["--check", "**/*.js"],
      ];

      const snapshot = await runAndSnapshot(fixturesDir, testCases);
      expect(snapshot).toMatchSnapshot();
    } finally {
      await rm(gitignorePath, { force: true });
    }
  });
});
