import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { runAndSnapshot } from "./utils";

const fixturesDir = join(__dirname, "fixtures");

describe("single_file", () => {
  it("should handle single file with different flags", async () => {
    const cwd = join(fixturesDir, "single_file");
    const testCases = [
      ["--check", "simple.js"],
      ["--list-different", "simple.js"],
    ];

    const snapshot = await runAndSnapshot(cwd, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
