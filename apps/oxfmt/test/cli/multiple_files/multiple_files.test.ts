import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("multiple_files", () => {
  it("should handle multiple files with different patterns", async () => {
    const testCases = [
      ["--check", "simple.js", "arrow.js"],
      ["--check"],
      ["--check", "."],
      ["--check", "./"],
      ["--check", "!*.{ts,tsx}"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
