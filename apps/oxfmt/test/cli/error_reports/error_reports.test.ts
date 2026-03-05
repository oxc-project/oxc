import { describe, expect, it } from "vite-plus/test";
import { join } from "node:path";
import { runAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("error_reports", () => {
  it("should report errors for invalid files", async () => {
    const testCases = [
      ["--check", "invalid.js"],
      ["--check", "invalid.yaml"],
    ];

    const snapshot = await runAndSnapshot(fixturesDir, testCases);
    expect(snapshot).toMatchSnapshot();
  });
});
