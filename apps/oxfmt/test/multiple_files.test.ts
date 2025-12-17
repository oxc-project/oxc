import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "./utils";

const fixturesDir = join(__dirname, "fixtures");

describe("multiple_files", () => {
  it("should handle multiple files with different patterns", async () => {
    const cwd = join(fixturesDir, "multiple_files");
    const testCases = [
      ["--check", "simple.js", "arrow.js"],
      ["--check"],
      ["--check", "."],
      ["--check", "./"],
      ["--check", "!*.{ts,tsx}"],
    ];

    const snapshot = await runAndSnapshot(cwd, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
