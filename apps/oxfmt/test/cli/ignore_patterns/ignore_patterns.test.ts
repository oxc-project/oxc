import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("ignore_patterns", () => {
  it("should handle ignore files correctly", async () => {
    const testCases = [
      ["--check"],
      ["--check", "--ignore-path", "gitignore.txt"],
      ["--check", "--ignore-path", "gitignore.txt", "--ignore-path", ".prettierignore"],
      ["--check", "--ignore-path", "nonexistent.ignore"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
