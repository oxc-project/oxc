import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("single_file", () => {
  it("should handle single file with different flags", async () => {
    const testCases = [
      ["--check", "simple.js"],
      ["--list-different", "simple.js"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
