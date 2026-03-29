import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("exclude_nested", () => {
  it("should correctly exclude nested paths", async () => {
    const testCases = [
      ["--check", "!foo/bar/error.js"],
      ["--check", "!foo/bar"],
      ["--check", "!foo"],
      ["--check", "!**/error.js"],
      ["--check", "foo", "!foo/bar/error.js"],
      ["--check", "foo", "!foo/bar"],
      ["--check", "foo", "!**/bar/error.js"],
      ["--check", "foo", "!**/bar/*"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });

  it("should correctly exclude nested paths with dot - set 1", async () => {
    const testCases = [
      ["--check", ".", "!foo/bar/error.js"],
      ["--check", ".", "!foo/bar"],
      ["--check", ".", "!foo"],
      ["--check", ".", "!**/error.js"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });

  it("should correctly exclude nested paths with dot - set 2", async () => {
    const testCases = [
      ["--check", "./foo", "!**/bar/error.js"],
      ["--check", "./foo", "!**/error.js"],
      ["--check", "./foo", "!**/bar/*"],
      ["--check", "./foo", "!foo/bar/error.js"],
      ["--check", "./foo", "!foo/bar"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });

  it("should handle glob include with glob exclude", async () => {
    const testCases = [
      // Glob include all .js, glob exclude error.js
      ["--check", "*", "!**/error.js"],
      // Glob include foo/**/*.js, glob exclude bar directory
      ["--check", "foo/**/*.js", "!**/bar/*"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
